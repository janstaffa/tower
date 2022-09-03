use chrono::Utc;
use clap::Parser;
use regex::Regex;
use std::collections::VecDeque;
use std::io::Write;
use std::{fs::File, mem::MaybeUninit};
use tower_assembler::{
    get_instruction, read_file, AssemblerError, SyntaxError, IM_ABSOLUTE, IM_CONSTANT,
    IM_IMMEDIATE, IM_IMPLIED, IM_INDIRECT, IM_REGA, IM_REGB, IM_ZEROPAGE, INSTRUCTIONS,
};

const COMMENT_IDENT: char = ';';

const CONTROL_SIGNALS: &[&'static str] = &[
    "IEND", "HLT", "PCE", "PCO", "PCJ", "AI", "AO", "BI", "BO", "RSO", "OPADD", "OPSUB", "OPNOT",
    "OPNAND", "OPSR", "FI", "FO", "MI", "MO", "INI", "HI", "HO", "LI", "LO", "HLO", "DVE", "DVW",
];

// constants
const STEP_COUNTER_BIT_SIZE: u32 = 4;
const INSTRUCTION_MODE_BIT_SIZE: u32 = 3;
const FLAGS_BIT_SIZE: u32 = 2;

const INSTRUCTION_MODE_COUNT: usize = 8;
const FLAG_COMBINATIONS: usize = 4;
const TOTAL_DEF_COMBINATIONS: usize = INSTRUCTION_MODE_COUNT * FLAG_COMBINATIONS;

const MAX_MICRO_STEP_COUNT: usize = 16;

const FLAGS: [&'static str; FLAGS_BIT_SIZE as usize] = ["CARRY", "ZERO"];

// the values are exponents
type MicroStep = Vec<u32>;

struct MacroDef {
    name: String,
    steps: Vec<MicroStep>,
}

#[derive(Debug, Clone, PartialEq)]
struct InstructionDef {
    name: String,
    instruction_mode: u32,
    flags: u32,
    steps: Vec<MicroStep>,
}

#[derive(Debug, PartialEq)]
struct TokenizedLine(u32, LineType);

#[derive(Debug, PartialEq)]
enum LineType {
    // name, args
    KeyLine(String, Vec<String>),
    // words
    StepLine(Vec<String>),
    // name of the label
    LabelLine(String),
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Source file to be assembled
    #[clap(short, long, value_parser)]
    r#in: String,
}

#[derive(Debug, Clone)]
struct Conditional {
    flag: u32,
    is_inverted: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format!("❌ Error: {}", e.message));
        if let Some(serr) = e.syntax_error {
            eprintln!("{}", format!("{} (line {})", serr.message, serr.line));
        }
    }
}

fn run() -> Result<(), AssemblerError> {
    // let args = Args::parse();
    let input_file_path = "examples/test.asm"; // &args.r#in;
    let input = read_file(input_file_path)?;

    let start_time = Utc::now();

    println!("Assembling... {}", input_file_path);

    // tokenize
    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at tokenization step"),
                Some(e),
            ))
        }
    };

    // parse
    let parsed = match parse(tokens) {
        Ok(idf) => idf,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at parsing step"),
                Some(e),
            ))
        }
    };

    // assemble
    let output = assemble(parsed);

    // write to output file
    let mut file = File::create("microcode.bin").unwrap();
    file.write_all(&output).unwrap();

    let now = Utc::now();
    let delta_time = now - start_time;
    println!("✔️  Finished (after {}ms)", delta_time.num_milliseconds());

    Ok(())
}

/// Takes the raw input data as String and returns a vector of tokens. Tokens are individual lines identified by their contents.
fn tokenize(code: String) -> Result<Vec<TokenizedLine>, SyntaxError> {
    let mut tokenized_lines: Vec<TokenizedLine> = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
        let real_line = line_idx as u32 + 1;
        let line = line.trim().to_lowercase();

        // check for comments and remove if found
        let comment_idx = line.chars().position(|c| c == COMMENT_IDENT);
        let line = if let Some(idx) = comment_idx {
            line[0..idx].trim().to_string()
        } else {
            line.trim().to_string()
        };

        // skip empty lines
        if line.len() == 0 {
            continue;
        }

        // split by whitespace
        let words: Vec<String> = line
            .split_whitespace()
            .map(|s| s.trim().to_lowercase())
            .collect();

        // the line is a key line (#def, #macro,...)
        let tokenized = if let Some('#') = line.chars().nth(0) {
            if line.chars().count() == 1 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("No keyword was specified"),
                ));
            }

            TokenizedLine(
                real_line,
                LineType::KeyLine(words[0][1..].to_string(), words[1..].to_vec()),
            )
        }
        // the line is a label line
        else if let Some(':') = line.chars().last() {
            if words.len() > 1 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("Invalid label definition, a label can only be one word."),
                ));
            }
            let mut label = line.clone();

            // remove the colon
            label.pop();

            TokenizedLine(real_line, LineType::LabelLine(label))
        } else {
            // split by whitespace or commas
            let re = Regex::new(r"\s*,\s*|\s+").unwrap();
            let words: Vec<String> = re.split(&line).map(|s| s.trim().to_lowercase()).collect();

            TokenizedLine(real_line, LineType::StepLine(words))
        };

        tokenized_lines.push(tokenized);
    }

    if tokenized_lines.len() == 0 {
        return Err(SyntaxError::new(0, String::from("No code was found")));
    }

    Ok(tokenized_lines)
}

/// Takes the tokens produced by the tokenizer and parses them, pasting macro code, adding prefixes and suffixes and different conditional definitions.
/// The result is a vector of instruction definitions defined for every combination of instruction modes and flags.
fn parse(tokens: Vec<TokenizedLine>) -> Result<Vec<InstructionDef>, SyntaxError> {
    let mut instructions: Vec<InstructionDef> = Vec::new();

    // keeps track of the currently defined macros
    let mut macros: Vec<MacroDef> = Vec::new();

    // currently defined macro
    let mut is_defining_macro = false;
    let mut current_macro: Option<MacroDef> = None;

    // currently defined instruction
    let mut is_defining_instruction = false;
    let mut current_instruction: Option<[InstructionDef; TOTAL_DEF_COMBINATIONS]> = None;
    let mut currently_defined_im: Option<u32> = None;

    // currently defined prefix/suffix
    let mut is_defining_pref = false;
    let mut current_pref: Option<Vec<MicroStep>> = None;
    let mut is_defining_suf = false;
    let mut current_suf: Option<Vec<MicroStep>> = None;

    // keeps track of nested conditionals
    let mut conditional_stack: VecDeque<Conditional> = VecDeque::new();

    for token in &tokens {
        let (real_line, line) = (&token.0, &token.1);

        let mut is_new_def = false;
        if let LineType::KeyLine(keyword, _) = line {
            if let "def" | "pref" | "suf" | "macro" = &keyword[..] {
                is_new_def = true;
            }
        }
        if is_new_def {
            // add a suffix if it is defined
            let extra_steps = current_suf.clone().unwrap_or(Vec::new());

            // complete the current definition and save it to its corresponding vector
            if is_defining_instruction {
                let mut current_instruction = current_instruction.clone().unwrap();
                for ins in &mut current_instruction {
                    ins.steps.extend(extra_steps.clone());
                    if ins.steps.len() > MAX_MICRO_STEP_COUNT {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!(
                                "Invalid instruction definition, maximum step count is {}. The added suffix has brought the step count over the limit.",
                                MAX_MICRO_STEP_COUNT
                            ),
                        ));
                    }
                }
                instructions.extend(current_instruction);
                currently_defined_im = None;
                is_defining_instruction = false;
            }
            if is_defining_macro {
                macros.push(current_macro.unwrap());
                current_macro = None;
                is_defining_macro = false;
            }
            is_defining_pref = false;
            is_defining_suf = false;
        }
        match line {
            LineType::KeyLine(keyword, args) => match &keyword[..] {
                "def" => {
                    if args.len() == 0 {
                        return Err(SyntaxError::new(
                            *real_line,
                            String::from("Instruction name not provided"),
                        ));
                    }

                    let inst_name = args[0].to_lowercase().trim().to_string();

                    // check if an instruction with this name is available
                    let exists = get_instruction(&inst_name).is_some();

                    if !exists {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Unknown instruction '{}'", inst_name),
                        ));
                    }

                    let is_already_defined =
                        instructions.iter().find(|&i| i.name == inst_name).is_some();

                    if is_already_defined {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Instruction '{}' is already defined.", inst_name),
                        ));
                    }

                    is_defining_instruction = true;

                    // add a prefix if it is defined
                    let steps = current_pref.clone().unwrap_or(Vec::new());

                    let modes: [u32; INSTRUCTION_MODE_COUNT] = [
                        IM_IMPLIED,
                        IM_IMMEDIATE,
                        IM_CONSTANT,
                        IM_ABSOLUTE,
                        IM_INDIRECT,
                        IM_ZEROPAGE,
                        IM_REGA,
                        IM_REGB,
                    ];

                    // initialize a version of the instruction for every Instruction Mode and flag combination
                    let instruction_versions = {
                        let mut array: [MaybeUninit<InstructionDef>; TOTAL_DEF_COMBINATIONS] =
                            unsafe { MaybeUninit::uninit().assume_init() };

                        for im_idx in 0..INSTRUCTION_MODE_COUNT {
                            let m = modes[im_idx].clone();

                            for flg_idx in 0..FLAG_COMBINATIONS {
                                let flg_val = flg_idx as u32;

                                let ins_v = InstructionDef {
                                    name: inst_name.clone(),
                                    instruction_mode: m,
                                    flags: flg_val,
                                    steps: steps.clone(),
                                };

                                let abs_idx = (im_idx * FLAG_COMBINATIONS) + flg_idx;
                                let element = array.get_mut(abs_idx).unwrap();
                                *element = MaybeUninit::new(ins_v);
                            }
                        }

                        unsafe {
                            std::mem::transmute::<_, [InstructionDef; TOTAL_DEF_COMBINATIONS]>(
                                array,
                            )
                        }
                    };

                    current_instruction = Some(instruction_versions);
                }
                "macro" => {
                    let macro_name = args[0].to_lowercase().trim().to_string();

                    // check if this name is used by a control signal, macro or an instruction
                    let is_used = CONTROL_SIGNALS
                        .iter()
                        .find(|&s| s.to_lowercase() == macro_name)
                        .is_some()
                        || macros.iter().find(|&m| m.name == macro_name).is_some()
                        || get_instruction(&macro_name).is_some();

                    if is_used {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("The name '{}' is already used.", macro_name),
                        ));
                    }

                    let new_macro_def = MacroDef {
                        name: macro_name,
                        steps: Vec::new(),
                    };

                    current_macro = Some(new_macro_def);
                    is_defining_macro = true;
                }
                "if" => {
                    let flag_name = args[0].trim().to_lowercase();

                    // check if a flag with this name exists
                    let flg_idx = FLAGS.iter().position(|&f| f.to_lowercase() == flag_name);

                    if !flg_idx.is_some() {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Unknown flag '{}'.", flag_name),
                        ));
                    }

                    // convert the flag name to interger value
                    let flag = 2_u32.pow(flg_idx.unwrap() as u32);

                    // push this conditional to the top of the conditional stack
                    let conditional = Conditional {
                        flag,
                        is_inverted: false,
                    };
                    conditional_stack.push_back(conditional);
                }
                "end" => {
                    if conditional_stack.len() == 0 {
                        return Err(SyntaxError::new(
                            *real_line,
                            String::from(
                                "Invalid use of 'end', there is no conditional to be closed.",
                            ),
                        ));
                    }

                    // remove the last conditional
                    conditional_stack.pop_back().unwrap();
                }
                "else" => {
                    if conditional_stack.len() == 0 {
                        return Err(SyntaxError::new(
                            *real_line,
                            String::from("Invalid use of 'else', there is no if block."),
                        ));
                    }

                    // invert the last conditional
                    let mut last_conditional = conditional_stack.back_mut().unwrap();
                    last_conditional.is_inverted = true;
                }
                "pref" => {
                    is_defining_pref = true;
                    current_pref = Some(Vec::new());
                }
                "suf" => {
                    is_defining_suf = true;
                    current_suf = Some(Vec::new());
                }
                _ => {
                    return Err(SyntaxError::new(
                        *real_line,
                        format!("Invalid keyword '{}'", keyword),
                    ));
                }
            },
            LineType::StepLine(words) => {
                // store all the control signals for this step
                let mut control_signals = Vec::new();

                // store the additional steps added by a macro
                let mut macro_steps = Vec::new();

                for word in words {
                    let signal_idx = CONTROL_SIGNALS
                        .iter()
                        .position(|&s| &s.to_lowercase() == word);
                    let signal_exists = signal_idx.is_some();

                    let macro_def = macros.iter().find(|&m| m.name == *word);
                    let macro_exists = macro_def.is_some();

                    if signal_exists {
                        control_signals.push(signal_idx.unwrap() as u32);
                    } else {
                        if macro_exists {
                            let macro_def = macro_def.unwrap();
                            if words.len() > 1 {
                                if macro_def.steps.len() > 1 {
                                    return Err(SyntaxError::new(
                                        *real_line,
                                        format!("Invalid macro usage. Multi step macro '{}' cannot be used inline.", macro_def.name),
                                    ));
                                }
                            }

                            if macro_def.steps.len() == 1 {
                                control_signals.extend(macro_def.steps.first().unwrap());
                            } else {
                                macro_steps.extend(macro_def.steps.to_vec());
                            }
                        } else {
                            return Err(SyntaxError::new(
                                *real_line,
                                format!("Unknown identifier '{}'.", word),
                            ));
                        }
                    }
                }

                // add all the steps together
                let mut steps = if control_signals.len() > 0 {
                    [control_signals].to_vec()
                } else {
                    Vec::new()
                };
                steps.extend(macro_steps.clone());

                if is_defining_pref {
                    let current_pref = current_pref.as_mut().unwrap();
                    current_pref.extend(steps.clone());
                    if current_pref.len() > MAX_MICRO_STEP_COUNT {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!(
                                "Invalid prefix definition, maximum step count is {}.",
                                MAX_MICRO_STEP_COUNT
                            ),
                        ));
                    }
                }
                if is_defining_suf {
                    let current_suf = current_suf.as_mut().unwrap();
                    current_suf.extend(steps.clone());
                    if current_suf.len() > MAX_MICRO_STEP_COUNT {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!(
                                "Invalid suffix definition, maximum step count is {}.",
                                MAX_MICRO_STEP_COUNT
                            ),
                        ));
                    }
                }

                if is_defining_instruction {
                    let current_instruction = current_instruction.as_mut().unwrap();

                    // filter out, what definitions should be affected
                    let matches_conditions = |inm: &u32, fgs: u32| -> bool {
                        if let Some(im) = currently_defined_im.clone() {
                            if im != *inm {
                                return false;
                            }
                        }

                        if conditional_stack.len() > 0 {
                            for c in &conditional_stack {
                                let is_flag_set = (fgs & c.flag) != 0;

                                if is_flag_set == c.is_inverted {
                                    return false;
                                }
                            }
                        }
                        true
                    };
                    for ins in current_instruction {
                        if matches_conditions(&ins.instruction_mode, ins.flags) {
                            ins.steps.extend(steps.clone());
                            if ins.steps.len() > MAX_MICRO_STEP_COUNT {
                                return Err(SyntaxError::new(
                                    *real_line,
                                    format!(
                                        "Invalid instruction definition, maximum step count is {}.",
                                        MAX_MICRO_STEP_COUNT
                                    ),
                                ));
                            }
                        }
                    }
                }
                if is_defining_macro {
                    let macro_def = current_macro.as_mut().unwrap();
                    macro_def.steps.extend(steps.clone());
                    if macro_def.steps.len() > MAX_MICRO_STEP_COUNT {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!(
                                "Invalid macro definition, maximum step count is {}.",
                                MAX_MICRO_STEP_COUNT
                            ),
                        ));
                    }
                }
            }
            LineType::LabelLine(label) => {
                let formated_label = label.trim().to_lowercase();

                // get the instruction mode value
                let instruction_mode_val = match &formated_label[..] {
                    "imp" => IM_IMPLIED,
                    "imm" => IM_IMMEDIATE,
                    "const" => IM_CONSTANT,
                    "abs" => IM_ABSOLUTE,
                    "ind" => IM_INDIRECT,
                    "zpage" => IM_ZEROPAGE,
                    "rega" => IM_REGA,
                    "regb" => IM_REGB,
                    _ => {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Invalid Instruction Mode label '{}'", label),
                        ));
                    }
                };

                let current_instruction_name = &current_instruction.as_ref().unwrap()[0].name;
                let inst = get_instruction(current_instruction_name).unwrap();
                if (inst.2 & instruction_mode_val) == 0 {
                    return Err(SyntaxError::new(
                        *real_line,
                        format!(
                            "Cannot define instruction mode '{}' for '{}'.",
                            formated_label, current_instruction_name
                        ),
                    ));
                }

                currently_defined_im = Some(instruction_mode_val);
            }
        }
    }

    // add a suffix if it is defined
    let extra_steps = current_suf.clone().unwrap_or(Vec::new());

    // finish the last definition
    if is_defining_instruction {
        let mut current_instruction = current_instruction.unwrap();
        for ins in &mut current_instruction {
            ins.steps.extend(extra_steps.clone());
            if ins.steps.len() > MAX_MICRO_STEP_COUNT {
                return Err(SyntaxError::new(
                    tokens.last().unwrap().0,
                    format!(
                        "Invalid instruction definition, maximum step count is {}. The added suffix has brought the step count over the limit.",                        MAX_MICRO_STEP_COUNT
                    ),
                ));
            }
        }
        instructions.extend(current_instruction);
    }

    Ok(instructions)
}

/// Takes the defined instructions and converts them to a binary file that is to be used inside the microcode ROM.
fn assemble(instruction_defs: Vec<InstructionDef>) -> Vec<u8> {
    let mut raw_bytes: Vec<u8> = Vec::new();

    // calculate all possible combinations
    let combs = (INSTRUCTIONS.len() as u32)
        * 2_u32.pow(STEP_COUNTER_BIT_SIZE)
        * 2_u32.pow(FLAGS_BIT_SIZE)
        * 2_u32.pow(INSTRUCTION_MODE_BIT_SIZE);

    'addr_loop: for addr in 0..combs {
        // get individual components of the address
        let opcode = addr >> 9;
        let instruction_mode = (addr >> 6) & 0b111;
        let flags = (addr >> 4) & 0b11;
        let micro_step = addr & 0b1111;

        for idf in &instruction_defs {
            // get the flags value

            let flags_match = flags == idf.flags;
            let instruction_modes_match = idf.instruction_mode == instruction_mode;

            let inst = get_instruction(&idf.name).unwrap();
            let opcodes_match = inst.0 as u32 == opcode;

            // check if this is the correct definition
            if flags_match && instruction_modes_match && opcodes_match {
                if micro_step > (idf.steps.len() - 1) as u32 {
                    raw_bytes.extend(&[0x00, 0x00, 0x00, 0x00]);
                    continue 'addr_loop;
                }
                let micro_step_csignals = idf.steps.get(micro_step as usize).unwrap();

                // construct the control word
                let mut control_word = 0;
                for cl in micro_step_csignals {
                    control_word |= 2_u32.pow(*cl);
                }

                // split the control word into four bytes
                let control_bytes = &[
                    (control_word >> 24) as u8,
                    ((control_word >> 16) & 0xff) as u8,
                    ((control_word >> 8) & 0xff) as u8,
                    (control_word & 0xff) as u8,
                ];
                println!("{} {:?}", addr, control_bytes);

                raw_bytes.extend(control_bytes);
                continue;
            }
            // if nothing is defined add zeroes
            raw_bytes.extend(&[0x00, 0x00, 0x00, 0x00]);
        }
    }

    raw_bytes
}
