use crate::{
    get_instruction_by_name, read_file, AssemblerError, SyntaxError, IM_ABSOLUTE, IM_ACCUMULATOR,
    IM_CONSTANT, IM_IMMEDIATE, IM_IMPLIED, IM_INDIRECT, IM_ZEROPAGE,
};
use regex::Regex;
use std::io::Write;
use std::mem::MaybeUninit;
use std::{collections::VecDeque, fs::File};

use crate::microasm::{
    Conditional, InstructionDef, LineType, MacroDef, MicroStep, TokenizedLine, COMMENT_IDENT,
    CONTROL_SIGNALS, FLAGS, FLAGS_BIT_SIZE, FLAG_COMBINATIONS, INSTRUCTION_MODE_BIT_SIZE,
    INSTRUCTION_MODE_COUNT, MAX_MICRO_STEP_COUNT, OPCODE_BIT_SIZE, STEP_COUNTER_BIT_SIZE,
    TOTAL_DEF_COMBINATIONS,
};

use super::ConditionalStep;

pub fn assembler(file_in: &str, file_out: &str) -> Result<(), AssemblerError> {
    let input = read_file(file_in)?;
    // tokenize
    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at tokenization step."),
                Some(e),
            ))
        }
    };

    // parse
    let parsed = match parse(tokens) {
        Ok(idf) => idf,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at parsing step."),
                Some(e),
            ))
        }
    };

    // assemble
    let output = assemble(parsed);

    // write to output file
    let mut file = File::create(file_out).unwrap();
    file.write_all(&output).unwrap();
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
                    String::from("No keyword was specified."),
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
        return Err(SyntaxError::new(0, String::from("No code was found.")));
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
    let mut current_pref: Option<Vec<ConditionalStep>> = None;
    let mut is_defining_suf = false;
    let mut current_suf: Option<Vec<ConditionalStep>> = None;

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

        // this is a new definition
        if is_new_def {
            // add a suffix if it is defined
            let extra_steps = current_suf.clone().unwrap_or(Vec::new());

            // complete the current definition and save it to its corresponding vector
            if is_defining_instruction {
                let mut current_instruction = current_instruction.clone().unwrap();
                let available_ims =
                    get_instruction_by_name(&current_instruction.first().unwrap().name)
                        .unwrap()
                        .2;

                for ins in &mut current_instruction {
                    // skip if this mode is not available
                    if (available_ims & ins.instruction_mode) == 0 {
                        continue;
                    }

                    // remove all steps which dont match the current flags
                    let steps: Vec<Vec<u64>> = extra_steps
                        .iter()
                        .filter(|&xs| {
                            if xs.conditions.len() > 0 {
                                for c in &xs.conditions {
                                    let is_flag_set = (ins.flags & c.flag) != 0;

                                    if is_flag_set == c.is_inverted {
                                        return false;
                                    }
                                }
                            }
                            return true;
                        })
                        .map(|xs| xs.step.clone())
                        .collect();

                    ins.steps.extend(steps);

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
            conditional_stack = VecDeque::new();
        }
        match line {
            LineType::KeyLine(keyword, args) => match &keyword[..] {
                "def" => {
                    if args.len() == 0 {
                        return Err(SyntaxError::new(
                            *real_line,
                            String::from("Instruction name not provided."),
                        ));
                    }

                    let inst_name = args[0].to_lowercase().trim().to_string();

                    // check if an instruction with this name is available
                    let exists = get_instruction_by_name(&inst_name).is_some();

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
                    let prefix_steps = current_pref.clone().unwrap_or(Vec::new());

                    let modes: [u32; INSTRUCTION_MODE_COUNT] = [
                        IM_IMPLIED,
                        IM_IMMEDIATE,
                        IM_CONSTANT,
                        IM_ABSOLUTE,
                        IM_INDIRECT,
                        IM_ZEROPAGE,
                        IM_ACCUMULATOR,
                    ];

                    // initialize a version of the instruction for every Instruction Mode and flag combination
                    let instruction_versions = {
                        let mut array: [MaybeUninit<InstructionDef>; TOTAL_DEF_COMBINATIONS] =
                            unsafe { MaybeUninit::uninit().assume_init() };

                        for im_idx in 0..INSTRUCTION_MODE_COUNT {
                            let m = modes[im_idx].clone();

                            for flg_idx in 0..FLAG_COMBINATIONS {
                                let flg_val = flg_idx as u32;

                                // remove all steps from the prefix which dont match the current flags
                                let steps = prefix_steps
                                    .iter()
                                    .filter(|&xs| {
                                        if xs.conditions.len() > 0 {
                                            for c in &xs.conditions {
                                                let is_flag_set = (flg_val & c.flag) != 0;

                                                if is_flag_set == c.is_inverted {
                                                    return false;
                                                }
                                            }
                                        }
                                        return true;
                                    })
                                    .map(|xs| xs.step.clone())
                                    .collect();

                                let ins_v = InstructionDef {
                                    name: inst_name.clone(),
                                    instruction_mode: m,
                                    flags: flg_val,
                                    steps,
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
                        || get_instruction_by_name(&macro_name).is_some();

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
                    if args.len() != 1 || args[0].len() == 0 {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Condition not provided."),
                        ));
                    }

                    let flag_name = args[0].trim().to_lowercase();

                    let is_inverted = flag_name.chars().next().unwrap() == '!';

                    let flag_name = if is_inverted {
                        flag_name[1..].to_string()
                    } else {
                        flag_name
                    };

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
                    let conditional = Conditional { flag, is_inverted };
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
                    last_conditional.is_inverted = !last_conditional.is_inverted;
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
                let mut control_signals: Vec<u64> = Vec::new();

                // store the additional steps added by a macro
                let mut macro_steps = Vec::new();

                // get the control signals from words
                for word in words {
                    let signal_idx = CONTROL_SIGNALS
                        .iter()
                        .position(|&s| &s.to_lowercase() == word);
                    let signal_exists = signal_idx.is_some();

                    let macro_def = macros.iter().find(|&m| m.name == *word);
                    let macro_exists = macro_def.is_some();

                    if signal_exists {
                        control_signals.push(signal_idx.unwrap() as u64);
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
                                control_signals
                                    .extend(macro_def.steps.first().unwrap().step.clone());
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
                for m in &mut macro_steps {
                    m.conditions.extend(conditional_stack.clone());
                }
                // add all the steps together
                let mut steps = if control_signals.len() > 0 {
                    let conditional = ConditionalStep {
                        conditions: conditional_stack.clone().into(),
                        step: control_signals,
                    };
                    [conditional].to_vec()
                } else {
                    Vec::new()
                };
                steps.extend(macro_steps);

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
                    continue;
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
                    continue;
                }

                if is_defining_instruction {
                    let current_instruction = current_instruction.as_mut().unwrap();

                    let available_ims =
                        get_instruction_by_name(&current_instruction.first().unwrap().name)
                            .unwrap()
                            .2;

                    let matches_conditions = |inm: &u32, fgs: u32| -> bool {
                        if (available_ims & inm) == 0 {
                            return false;
                        }

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

                    // apply changes
                    for ins in current_instruction {
                        // skip the non matching intruction definitions
                        if !matches_conditions(&ins.instruction_mode, ins.flags) {
                            continue;
                        }

                        // add all of the appropriate steps
                        'step_loop: for s in &steps {
                            if s.conditions.len() > 0 {
                                for c in &s.conditions {
                                    let is_flag_set = (ins.flags & c.flag) != 0;

                                    if is_flag_set == c.is_inverted {
                                        continue 'step_loop;
                                    }
                                }
                            }
                            ins.steps.push(s.step.clone());
                        }

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
                    continue;
                }
                if is_defining_macro {
                    let macro_def = current_macro.as_mut().unwrap();

                    macro_def.steps.extend(steps);

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
                    "accumulator" => IM_ACCUMULATOR,
                    _ => {
                        return Err(SyntaxError::new(
                            *real_line,
                            format!("Invalid Instruction Mode label '{}'", label),
                        ));
                    }
                };

                let current_instruction_name = &current_instruction.as_ref().unwrap()[0].name;
                let inst = get_instruction_by_name(current_instruction_name).unwrap();
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
            // remove all steps which dont match the current flags
            let steps: Vec<MicroStep> = extra_steps
                .iter()
                .filter(|&xs| {
                    if xs.conditions.len() > 0 {
                        for c in &xs.conditions {
                            let is_flag_set = (ins.flags & c.flag) != 0;

                            if is_flag_set == c.is_inverted {
                                return false;
                            }
                        }
                    }
                    return true;
                })
                .map(|xs| xs.step.clone())
                .collect();

            ins.steps.extend(steps);
            if ins.steps.len() > MAX_MICRO_STEP_COUNT {
                return Err(SyntaxError::new(
                    tokens.last().unwrap().0,
                    format!(
                        "Invalid instruction definition, maximum step count is {}. The added suffix has brought the step count over the limit.", MAX_MICRO_STEP_COUNT
					),
                ));
            }
        }

        instructions.extend(current_instruction);
    }

    // remove all empty or undefined instruction definitions
    let mut final_instructions = Vec::new();
    for ins in instructions {
        let available_ims = get_instruction_by_name(&ins.name).unwrap().2;
        if ins.steps.len() == 0 || (available_ims & ins.instruction_mode) == 0 {
            continue;
        }
        final_instructions.push(ins);
    }

    Ok(final_instructions)
}

/// Takes the defined instructions and converts them to a binary file that is to be used inside the microcode ROM.
fn assemble(instruction_defs: Vec<InstructionDef>) -> Vec<u8> {
    let combs = 2_u32.pow(OPCODE_BIT_SIZE)
        * 2_u32.pow(STEP_COUNTER_BIT_SIZE)
        * 2_u32.pow(FLAGS_BIT_SIZE)
        * 2_u32.pow(INSTRUCTION_MODE_BIT_SIZE);

    let mut raw_bytes: Vec<u8> = vec![0; (combs * 5) as usize];

    for idf in &instruction_defs {
        let inst = get_instruction_by_name(&idf.name).unwrap();

        let opcode = inst.0 << (STEP_COUNTER_BIT_SIZE + FLAGS_BIT_SIZE + INSTRUCTION_MODE_BIT_SIZE);
        let instruction_mode = ((idf.instruction_mode as f32).log2() as u32)
            << (STEP_COUNTER_BIT_SIZE + FLAGS_BIT_SIZE);
        let flags = idf.flags << STEP_COUNTER_BIT_SIZE;
        let instruction_raw_start_idx = (opcode | instruction_mode | flags) * 5;

        for si in 0..MAX_MICRO_STEP_COUNT {
            let step = idf.steps.get(si);
            let real_byte_idx = instruction_raw_start_idx as usize + si * 5;

            if let Some(control_signals) = step {
                // construct the control word
                let mut control_word: u64 = 0;
                for cl in control_signals {
                    control_word |= 2_u64.pow(*cl as u32);
                }
                // split the control word into five bytes
                let control_bytes = &[
                    (control_word >> 32) as u8,
                    ((control_word >> 24) & 0xff) as u8,
                    ((control_word >> 16) & 0xff) as u8,
                    ((control_word >> 8) & 0xff) as u8,
                    (control_word & 0xff) as u8,
                ];

                raw_bytes.splice(real_byte_idx..real_byte_idx + 5, *control_bytes);
                continue;
            }
            break;
        }
    }
    raw_bytes
}

// Archived VERY slow code

// /// Takes the defined instructions and converts them to a binary file that is to be used inside the microcode ROM.
// fn assemble(instruction_defs: Vec<InstructionDef>) -> Vec<u8> {
//     let mut raw_bytes: Vec<u8> = Vec::new();

// 	for ins in instruction_defs {

// 	}
//     // calculate all possible combinations
//     let combs = (INSTRUCTIONS.len() as u32)
//         * 2_u32.pow(STEP_COUNTER_BIT_SIZE)
//         * 2_u32.pow(FLAGS_BIT_SIZE)
//         * 2_u32.pow(INSTRUCTION_MODE_BIT_SIZE);

//     'addr_loop: for addr in 0..combs {
//         // get individual components of the address
//         let opcode = addr >> (STEP_COUNTER_BIT_SIZE + FLAGS_BIT_SIZE + INSTRUCTION_MODE_BIT_SIZE);
//         let instruction_mode = (addr >> (STEP_COUNTER_BIT_SIZE + FLAGS_BIT_SIZE)) & 0b111;
//         let flags = (addr >> STEP_COUNTER_BIT_SIZE) & 0b111;
//         let micro_step = addr & 0b1111;

//         for idf in &instruction_defs {
//             let flags_match = flags == idf.flags;

//             let instruction_modes_match =
//                 (idf.instruction_mode as f32).log2() as u32 == instruction_mode;

//             let inst = get_instruction_by_name(&idf.name).unwrap();
//             let opcodes_match = inst.0 as u32 == opcode;

//             // check if this is the correct definition
//             if flags_match && instruction_modes_match && opcodes_match {
//                 // fill remaining steps
//                 if micro_step >= idf.steps.len() as u32 {
//                     raw_bytes.extend(&[0x00, 0x00, 0x00, 0x00, 0x00]);
//                     continue 'addr_loop;
//                 }
//                 let micro_step_csignals = idf.steps.get(micro_step as usize).unwrap();

//                 // construct the control word
//                 let mut control_word: u64 = 0;
//                 for cl in micro_step_csignals {
//                     control_word |= 2_u64.pow(*cl as u32);
//                 }

//                 // split the control word into four bytes
//                 let control_bytes = &[
//                     (control_word >> 32) as u8,
//                     ((control_word >> 24) & 0xff) as u8,
//                     ((control_word >> 16) & 0xff) as u8,
//                     ((control_word >> 8) & 0xff) as u8,
//                     (control_word & 0xff) as u8,
//                 ];

//                 raw_bytes.extend(control_bytes);
//                 continue 'addr_loop;
//             }
//         }
//         raw_bytes.extend(&[0x00, 0x00, 0x00, 0x00, 0x00]);
//     }

//     raw_bytes
// }
