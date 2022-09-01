use clap::Parser;
use std::collections::VecDeque;
use std::io::Write;
use std::{fs::File, mem::MaybeUninit};
use tower_assembler::{read_file, AssemblerError, SyntaxError, IM, INSTRUCTIONS};

const COMMENT_IDENT: char = ';';

const CONTROL_SIGNALS: &[&'static str] = &[
    "IEND", "HLT", "PCE", "PCO", "PCJ", "AI", "AO", "BI", "BO", "RSO", "OPADD", "OPSUB", "OPNOT",
    "OPNAND", "OPSR", "FI", "FO", "MI", "MO", "INI", "HI", "HO", "LI", "LO", "HLO", "DVE", "DVW",
];

const FLAGS: &[&'static str] = &["CARRY", "ZERO"];

// constants
const STEP_COUNTER_BIT_SIZE: u32 = 4;
const INSTRUCTION_MODE_BIT_SIZE: u32 = 3;
const FLAGS_BIT_SIZE: u32 = 2;

const INSTRUCTION_MODE_COUNT: usize = 8;
const FLAG_COMBINATIONS: usize = 4;
const TOTAL_DEF_COMBINATIONS: usize = INSTRUCTION_MODE_COUNT * FLAG_COMBINATIONS;

// the values are exponents
type MicroStep = Vec<u32>;

struct MacroDef {
    name: String,
    steps: Vec<MicroStep>,
}

#[derive(Debug, PartialEq, Clone)]
struct FlagStatus {
    carry: bool,
    zero: bool,
}

#[derive(Debug, Clone)]
struct InstructionDef {
    name: String,
    instruction_mode: IM,
    flags: FlagStatus,
    steps: Vec<MicroStep>,
}

#[derive(Debug)]
struct TokenizedLine(u32, LineType);

#[derive(Debug)]
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
    flag: String,
    is_inverted: bool,
}

fn main() {
    // This is temporary test code
    let dummy_code = "
    ; this is a comment
    #macro TEST
    PCE PCO

    #pref
    HLT

    #suf
    PCJ IEND
    
    ; a comment in the middle of nowhere
    #def ADD
    imm:
        OPSUB
        #if carry
            HLT IEND TEST
            #if zero
                DVE OPADD
            #end
        #else
            DVE DVW
        #end
        OPSR

    const:
        DVE DVW
    IMP:
        MI mo
	"
    .into();

    let tokens = tokenize(dummy_code).unwrap();
    println!("Tokenized: {:?}", tokens);
    let parsed = parse(tokens).unwrap();
    println!("Parsed: {:?}", parsed);
    let output = assemble(parsed);
    // println!("Result: {:?}", output);

    let mut file = File::create("microcode.bin").unwrap();
    file.write_all(&output).unwrap();

    if let Err(e) = run() {
        eprintln!("Error: {}", e.message);
        if let Some(serr) = e.syntax_error {
            eprintln!("Err: {} On line {}", serr.message, serr.line);
        }
    }
}

fn run() -> Result<(), AssemblerError> {
    let args = Args::parse();
    let input = read_file(&args.r#in)?;

    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at tokenization step"),
                Some(e),
            ))
        }
    };

    let instruction_defs = match parse(tokens) {
        Ok(idf) => idf,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at parsing step"),
                Some(e),
            ))
        }
    };

    Ok(())
}

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

        if line.len() == 0 {
            continue;
        }

        // TODO: split by commas or spaces
        let words: Vec<String> = line
            .split_whitespace()
            .map(|s| s.trim().to_lowercase())
            .collect();

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
        } else if let Some(':') = line.chars().last() {
            if words.len() > 1 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("Invalid label definition, a label can only be one word."),
                ));
            }
            let mut label = line.clone();
            label.pop();

            TokenizedLine(real_line, LineType::LabelLine(label))
        } else {
            TokenizedLine(real_line, LineType::StepLine(words))
        };

        tokenized_lines.push(tokenized);
    }

    if tokenized_lines.len() == 0 {
        return Err(SyntaxError::new(0, String::from("No code was found")));
    }

    Ok(tokenized_lines)
}

fn parse(tokens: Vec<TokenizedLine>) -> Result<Vec<InstructionDef>, SyntaxError> {
    let mut macros: Vec<MacroDef> = Vec::new();
    let mut instructions: Vec<InstructionDef> = Vec::new();

    let mut current_macro: Option<MacroDef> = None;

    let mut is_defining_instruction = false;
    let mut current_instruction: Option<[InstructionDef; TOTAL_DEF_COMBINATIONS]> = None;
    let mut currently_defined_im: Option<IM> = None;

    let mut is_defining_pref = false;
    let mut current_pref: Option<Vec<MicroStep>> = None;
    let mut is_defining_suf = false;
    let mut current_suf: Option<Vec<MicroStep>> = None;

    let mut conditional_stack: VecDeque<Conditional> = VecDeque::new();

    for token in tokens {
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
            if is_defining_instruction {
                let mut current_instruction = current_instruction.clone().unwrap();
                for ins in &mut current_instruction {
                    ins.steps.extend(extra_steps.clone());
                }
                instructions.extend(current_instruction);
                currently_defined_im = None;
                is_defining_instruction = false;
            }
            if let Some(macro_def) = current_macro {
                macros.push(macro_def);
                current_macro = None;
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

                    let mut exists = false;

                    let inst_name = args[0].to_lowercase().trim().to_string();
                    for inst in INSTRUCTIONS {
                        if inst.1.to_lowercase() == inst_name {
                            exists = true;
                        }
                    }
                    if !exists {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Unknown instruction name '{}'", inst_name),
                        ));
                    }

                    is_defining_instruction = true;

                    // add a prefix if it is defined
                    let steps = current_pref.clone().unwrap_or(Vec::new());

                    let modes: [IM; INSTRUCTION_MODE_COUNT] = [
                        IM::Implied,
                        IM::Immediate,
                        IM::Constant,
                        IM::Absolute,
                        IM::Indirect,
                        IM::ZeroPage,
                        IM::RegA,
                        IM::RegB,
                    ];

                    // initialize a version of the instruction for every Instruction Mode
                    let instruction_versions = {
                        let mut array: [MaybeUninit<InstructionDef>; TOTAL_DEF_COMBINATIONS] =
                            unsafe { MaybeUninit::uninit().assume_init() };

                        for im_idx in 0..INSTRUCTION_MODE_COUNT {
                            let m = modes[im_idx].clone();

                            for flg_idx in 0..FLAG_COMBINATIONS {
                                let carry = (flg_idx & 0b1) != 0;
                                let zero = (flg_idx & 0b10) != 0;

                                let ins_v = InstructionDef {
                                    name: inst_name.clone(),
                                    instruction_mode: m,
                                    flags: FlagStatus { carry, zero },
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

                    let exists = CONTROL_SIGNALS
                        .iter()
                        .find(|&s| s.to_lowercase() == macro_name)
                        .is_some()
                        || macros.iter().find(|&m| m.name == macro_name).is_some();

                    if exists {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("This name is already used '{}'", macro_name),
                        ));
                    }

                    let new_macro_def = MacroDef {
                        name: macro_name,
                        steps: Vec::new(),
                    };

                    current_macro = Some(new_macro_def);
                }

                "if" => {
                    let flag = args[0].trim().to_lowercase();
                    let exists = FLAGS.iter().find(|&f| f.to_lowercase() == flag).is_some();

                    if !exists {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Unknown flag '{}'", flag),
                        ));
                    }

                    conditional_stack.push_back(Conditional {
                        flag,
                        is_inverted: false,
                    });
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
                    conditional_stack.pop_back().unwrap();
                }
                "else" => {
                    if conditional_stack.len() == 0 {
                        return Err(SyntaxError::new(
                            *real_line,
                            String::from("Invalid use of 'else', there is no if block."),
                        ));
                    }

                    let mut conditional = conditional_stack.pop_back().unwrap();
                    conditional.is_inverted = true;

                    conditional_stack.push_back(conditional);
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
                let mut control_signals = Vec::new();

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
                            control_signals.extend(macro_def.steps.first().unwrap());

                            if macro_def.steps.len() > 1 {
                                macro_steps.extend(macro_def.steps[1..].to_vec());
                            }
                        } else {
                            return Err(SyntaxError::new(
                                *real_line,
                                format!("Unknown identifier '{}'", word),
                            ));
                        }
                    }
                }

                let this_step = control_signals as MicroStep;

                if is_defining_pref {
                    current_pref.as_mut().unwrap().push(this_step.clone());
                    current_pref.as_mut().unwrap().extend(macro_steps.clone());
                }
                if is_defining_suf {
                    current_suf.as_mut().unwrap().push(this_step.clone());
                    current_suf.as_mut().unwrap().extend(macro_steps.clone());
                }

                if is_defining_instruction {
                    let current_instruction = current_instruction.as_mut().unwrap();

                    let matches_conditions = |inm: &IM, fgs: &FlagStatus| -> bool {
                        if let Some(im) = currently_defined_im.clone() {
                            if im != *inm {
                                return false;
                            }
                        }

                        if conditional_stack.len() > 0 {
                            for c in &conditional_stack {
                                if (c.flag == "carry" && fgs.carry == c.is_inverted)
                                    || (c.flag == "zero" && fgs.zero == c.is_inverted)
                                {
                                    return false;
                                }
                            }
                        }
                        true
                    };
                    for ins in current_instruction {
                        if matches_conditions(&ins.instruction_mode, &ins.flags) {
                            ins.steps.push(this_step.clone());
                            ins.steps.extend(macro_steps.clone());
                        }
                    }
                }
                if let Some(macro_def) = &mut current_macro {
                    macro_def.steps.push(this_step);
                    macro_def.steps.extend(macro_steps.clone());
                }
            }
            LineType::LabelLine(label) => {
                let formated = label.trim().to_lowercase();
                let instruction_mode = match &formated[..] {
                    "imp" => IM::Implied,
                    "imm" => IM::Immediate,
                    "const" => IM::Constant,
                    "abs" => IM::Absolute,
                    "ind" => IM::Indirect,
                    "zpage" => IM::ZeroPage,
                    "rega" => IM::RegA,
                    "regb" => IM::RegB,
                    _ => {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Invalid Instruction Mode label '{}'", label),
                        ));
                    }
                };

                currently_defined_im = Some(instruction_mode);
            }
        }
    }

    // add a suffix if it is defined
    let extra_steps = current_suf.clone().unwrap_or(Vec::new());

    if is_defining_instruction {
        let mut current_instruction = current_instruction.unwrap();
        for ins in &mut current_instruction {
            ins.steps.extend(extra_steps.clone());
        }
        instructions.extend(current_instruction);
    }
    if let Some(macro_def) = current_macro {
        macros.push(macro_def);
    }

    Ok(instructions)
}

fn assemble(instruction_defs: Vec<InstructionDef>) -> Vec<u8> {
    let mut raw_bytes: Vec<u8> = Vec::new();

    let combs = (INSTRUCTIONS.len() as u32)
        * 2_u32.pow(STEP_COUNTER_BIT_SIZE)
        * 2_u32.pow(FLAGS_BIT_SIZE)
        * 2_u32.pow(INSTRUCTION_MODE_BIT_SIZE);

    'addr_loop: for addr in 0..combs {
        let opcode = addr >> 9;
        let instruction_mode = (addr >> 6) & 0b111;
        let flags = (addr >> 4) & 0b11;
        let micro_step = addr & 0b1111;

        for idf in &instruction_defs {
            let mut f = 0;
            if idf.flags.carry {
                f += 1;
            }
            if idf.flags.zero {
                f += 2;
            }

            let flags_match = flags == f;
            let instruction_modes_match = idf.instruction_mode.get_value() == instruction_mode;

            let inst = INSTRUCTIONS
                .iter()
                .find(|&i| i.1.to_lowercase() == idf.name)
                .unwrap();
            let opcodes_match = inst.0 as u32 == opcode;

            if flags_match && instruction_modes_match && opcodes_match {
                if micro_step > (idf.steps.len() - 1) as u32 {
                    raw_bytes.extend(&[0x00, 0x00, 0x00, 0x00]);
                    continue 'addr_loop;
                }
                let micro_step = idf.steps.get(micro_step as usize).unwrap();
                let mut control_word = 0;
                for cl in micro_step {
                    control_word |= 2_u32.pow(*cl);
                }

                let control_bytes = &[
                    (control_word >> 24) as u8,
                    ((control_word >> 16) & 0xff) as u8,
                    ((control_word >> 8) & 0xff) as u8,
                    (control_word & 0xff) as u8,
                ];

                raw_bytes.extend(control_bytes);
                continue;
            }
            raw_bytes.extend(&[0x00, 0x00, 0x00, 0x00]);
        }
    }

    raw_bytes
}

#[test]
fn test_tokenizer_and_parser() {
    let dummy_code = "
    ; this is a comment
    #macro TEST
    PCE PCO

    #pref
    TEST
    HLT

    #suf
    IEND

    #macro TEST2
    PCE PCO
    PCJ
    ; a comment in the middle of nowhere
    #def ADD
    HLT IEND TEST

    #pref
    DVW
    
    #def SUB
    HLT IEND
    TEST2
    ; does this work?
	"
    .into();

    let tokens = tokenize(dummy_code).unwrap();

    let correctly_tokenized = [
        TokenizedLine(
            3,
            LineType::KeyLine("macro".into(), Vec::from(["test".into()])),
        ),
        TokenizedLine(
            4,
            LineType::StepLine(Vec::from(["pce".into(), "pco".into()])),
        ),
        TokenizedLine(6, LineType::KeyLine("pref".into(), Vec::from([]))),
        TokenizedLine(7, LineType::StepLine(Vec::from(["test".into()]))),
        TokenizedLine(8, LineType::StepLine(Vec::from(["hlt".into()]))),
        TokenizedLine(10, LineType::KeyLine("suf".into(), Vec::from([]))),
        TokenizedLine(11, LineType::StepLine(Vec::from(["iend".into()]))),
        TokenizedLine(
            13,
            LineType::KeyLine("macro".into(), Vec::from(["test2".into()])),
        ),
        TokenizedLine(
            14,
            LineType::StepLine(Vec::from(["pce".into(), "pco".into()])),
        ),
        TokenizedLine(15, LineType::StepLine(Vec::from(["pcj".into()]))),
        TokenizedLine(
            17,
            LineType::KeyLine("def".into(), Vec::from(["add".into()])),
        ),
        TokenizedLine(
            18,
            LineType::StepLine(Vec::from(["hlt".into(), "iend".into(), "test".into()])),
        ),
        TokenizedLine(20, LineType::KeyLine("pref".into(), Vec::from([]))),
        TokenizedLine(21, LineType::StepLine(Vec::from(["dvw".into()]))),
        TokenizedLine(
            23,
            LineType::KeyLine("def".into(), Vec::from(["sub".into()])),
        ),
        TokenizedLine(
            24,
            LineType::StepLine(Vec::from(["hlt".into(), "iend".into()])),
        ),
        TokenizedLine(25, LineType::StepLine(Vec::from(["test2".into()]))),
    ];

    for (i, t) in tokens.iter().enumerate() {
        assert_eq!(t.0, correctly_tokenized[i].0);

        match (&t.1, &correctly_tokenized[i].1) {
            (LineType::KeyLine(name, args), LineType::KeyLine(name2, args2)) => {
                assert_eq!(name, name2);
                assert_eq!(args, args2);
            }
            (LineType::StepLine(words), LineType::StepLine(words2)) => {
                assert_eq!(words, words2);
            }
            _ => panic!("Invalid LineType value"),
        }
    }

    let parsed = parse(tokens).unwrap();

    let correctly_parsed = [
        InstructionDef {
            name: "add".into(),
            instruction_mode: IM::Implied,
            flags: FlagStatus {
                carry: false,
                zero: false,
            },
            steps: Vec::from([
                Vec::from([2, 3]),
                Vec::from([1]),
                Vec::from([1, 0, 2, 3]),
                Vec::from([0]),
            ]),
        },
        InstructionDef {
            name: "sub".into(),
            instruction_mode: IM::Implied,
            flags: FlagStatus {
                carry: false,
                zero: false,
            },
            steps: Vec::from([
                Vec::from([26]),
                Vec::from([1, 0]),
                Vec::from([2, 3]),
                Vec::from([4]),
                Vec::from([0]),
            ]),
        },
    ];

    for (i, p) in parsed.iter().enumerate() {
        assert_eq!(p.name, correctly_parsed[i].name);
        assert_eq!(p.instruction_mode, correctly_parsed[i].instruction_mode);
        assert_eq!(p.flags, correctly_parsed[i].flags);
        assert_eq!(p.steps, correctly_parsed[i].steps);
    }
}
