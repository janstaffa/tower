use std::fs::File;
use std::io::Write;

use regex::Regex;

use crate::{
    get_argument_size_by_im, get_available_im_names, get_im_name, get_instruction_by_name,
    microasm::COMMENT_IDENT, read_file, AssemblerError, SyntaxError, IM_CONSTANT, IM_IMPLIED,
};

use super::{analyze_arg, parse_arg, Argument, Instruction, Label, MacroDef, Token, TokenizedLine};

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
                String::from("Assembly failed at parsing step"),
                Some(e),
            ))
        }
    };
    // println!("parsed {:?}", parsed);

    // assemble
    let output = assemble(parsed);

    // write to output file
    let mut file = File::create(file_out).unwrap();
    file.write_all(&output).unwrap();
    Ok(())
}

fn tokenize(code: String) -> Result<Vec<TokenizedLine>, SyntaxError> {
    let mut tokenized_lines: Vec<TokenizedLine> = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
        let real_line = line_idx as u32 + 1;
        let line = line.trim();

        // check for comments and remove if
        let comment_idx = line.chars().position(|c| c == COMMENT_IDENT);
        let line = if let Some(idx) = comment_idx {
            line[0..idx].trim().to_string()
        } else {
            line.trim().to_string()
        };

        // split by whitespace
        let words: Vec<String> = line
            .split_whitespace()
            .map(|s| s.trim().to_string())
            .collect();

        // skip empty lines
        if words.len() == 0 {
            continue;
        }

        // the line is a marker
        let tokenized = if let Some('#') = line.chars().nth(0) {
            if line.chars().count() == 1 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("No keyword was specified."),
                ));
            }

            TokenizedLine(
                real_line,
                Token::Marker(words[0][1..].to_lowercase(), words[1..].to_vec()),
            )
        }
        // the line is a label
        else if let Some(':') = words.last().unwrap().chars().last() {
            if words.len() > 1 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("Invalid label definition, a label can only be one word."),
                ));
            }

            let mut label_name = words[0].clone();
            // remove the colon
            label_name.pop();

            if label_name.chars().count() == 0 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("Invalid label name, name cannot be empty."),
                ));
            }

            let re = Regex::new(r"[^a-zA-Z0-9_]").unwrap();
            if re.is_match(&label_name) {
                return Err(SyntaxError::new(
                    real_line,
                    format!("Invalid label name '{}'. Label name can only contain characters a-Z, numbers or the '_' symbol.", label_name),
                ));
            }

            // check if first char is not a number
            let first_char = label_name.chars().next().unwrap();
            if first_char.is_digit(10) {
                return Err(SyntaxError::new(
                    real_line,
                    format!(
                        "Invalid label name '{}', label name has to start with a letter.",
                        label_name
                    ),
                ));
            }

            TokenizedLine(real_line, Token::Label(label_name))
        }
        // the line is an instruction
        else {
            let args = if words.len() > 1 {
                let arg_str = words[1..].join(" ");
                let re = Regex::new(r"\s*,\s*").unwrap();
                let args: Vec<String> = re
                    .split(&arg_str)
                    .map(|s| s.trim().to_lowercase())
                    .collect();

                for a in &args {
                    if a.chars().count() == 0 {
                        return Err(SyntaxError::new(
                            real_line,
                            String::from("Invalid argument, argument cannot be emty."),
                        ));
                    }
                    if a.contains(" ") {
                        return Err(SyntaxError::new(
								real_line,
								format!("Invalid argument '{}', arguments can only be one word. If you want to specify multiple arguments separate them by a comma.", a),
							));
                    }
                }
                args
            } else {
                Vec::new()
            };

            TokenizedLine(real_line, Token::Instruction(words[0].to_owned(), args))
        };

        tokenized_lines.push(tokenized);
    }

    if tokenized_lines.len() == 0 {
        return Err(SyntaxError::new(0, String::from("No code was found.")));
    }

    Ok(tokenized_lines)
}

pub fn parse(tokens: Vec<TokenizedLine>) -> Result<Vec<Instruction>, SyntaxError> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut macros: Vec<MacroDef> = Vec::new();
    let mut is_defining_macro = false;
    let mut current_macro: Option<MacroDef> = None;

    let mut labels: Vec<Label> = Vec::new();
    let mut instructions_using_labels: Vec<(usize, u32)> = Vec::new();

    let mut current_address = 0;

    // TODO: move argument parsing into tokenizer
    for t in tokens {
        let (real_line, token) = (t.0, t.1);

        match token {
            Token::Instruction(name, args) => {
                // parse raw args to nice structures
                let mut parsed_args = Vec::new();

                for arg in &args {
                    let parsed_arg = match parse_arg(arg) {
                        Ok(arg) => arg,
                        Err(e) => return Err(SyntaxError::new(real_line, e)),
                    };

                    let im = if let Some(Argument::Label(name)) = &parsed_arg {
                        instructions_using_labels.push((instructions.len(), real_line));

                        let re = Regex::new(r"[^a-zA-Z0-9_]").unwrap();
                        if re.is_match(&name) {
                            return Err(SyntaxError::new(
								real_line,
								format!("Invalid label name '{}'. Label name can only contain characters a-Z, numbers or the '_' symbol.", name),
							));
                        }
                        IM_CONSTANT
                    } else {
                        match analyze_arg(arg) {
                            Ok(im) => im,
                            Err(e) => return Err(SyntaxError::new(real_line, e)),
                        }
                    };

                    parsed_args.push((im, parsed_arg));
                }

                if is_defining_macro {
                    let current_macro = current_macro.as_mut().unwrap();

                    for arg in &parsed_args {
                        if let Some(Argument::Implicit(arg_idx)) = &arg.1 {
                            let exists = current_macro
                                .args
                                .iter()
                                .find(|&ma| *ma == *arg_idx)
                                .is_some();

                            if !exists {
                                current_macro.args.push(*arg_idx);
                            }
                        }
                    }
                } else {
                    let found_placeholder = parsed_args
                        .iter()
                        .find(|&a| matches!(a, (_, Some(Argument::Implicit(arg_idx)))));

                    if found_placeholder.is_some() {
                        if let Argument::Implicit(arg_idx) =
                            found_placeholder.as_ref().unwrap().1.as_ref().unwrap()
                        {
                            return Err(SyntaxError::new(
                                real_line,
                                format!("Wrong usage of argument placeholder '${}'. Argument placeholders can only be used inside a macro.", arg_idx),
                            ));
                        }
                    }
                }

                let ins = get_instruction_by_name(&name);

                // check if this instruction exists
                if ins.is_some() {
                    if args.len() > 1 {
                        return Err(SyntaxError::new(
                            real_line,
                            String::from("Instructions can only have one argument."),
                        ));
                    }

                    let (instruction_mode, argument) = if args.len() == 1 {
                        parsed_args[0].clone()
                    } else {
                        (IM_IMPLIED, None)
                    };

                    let new_instruction = Instruction {
                        name: name.clone(),
                        argument: argument.clone(),
                        instruction_mode: instruction_mode.clone(),
                    };

                    if is_defining_macro {
                        let current_macro = current_macro.as_mut().unwrap();

                        if let Some(Argument::Implicit(arg_idx)) = &argument {
                            let exists = current_macro
                                .args
                                .iter()
                                .find(|&ma| *ma == *arg_idx)
                                .is_some();

                            if !exists {
                                current_macro.args.push(*arg_idx);
                            }
                        }

                        current_macro
                            .instructions
                            .push((new_instruction, real_line, Vec::new()));
                    } else {
                        if instruction_mode == 0 {
                            return Err(SyntaxError::new(
								real_line,
								format!("No mode identifier specified for argument '{}' of instruction '{}'.", args[0], name),
							));
                        } else {
                            let available_modes_val = ins.unwrap().2;
                            if (available_modes_val & instruction_mode) == 0 {
                                let available_modes = get_available_im_names(available_modes_val);
                                let this_mode =
                                    get_im_name((instruction_mode as f32).log2() as u32).unwrap();
                                return Err(SyntaxError::new(
									real_line,
									format!("Instruction '{}' cannot take an argument in '{}' instruction mode. Available modes are: {}", name, this_mode, available_modes.join(","))
								));
                            }
                        }

                        instructions.push(new_instruction);
                        current_address += 1 + get_argument_size_by_im(instruction_mode);
                    }
                } else {
                    let macro_def = macros.iter_mut().find(|m| m.name == name);

                    if let Some(macro_def) = macro_def {
                        if args.len() != macro_def.args.len() {
                            return Err(SyntaxError::new(
							    real_line,
							    format!("Wrong number of arguments for macro '{}'. This macro requires {} arguments.", macro_def.name, macro_def.args.len()),
							));
                        }

                        let mut new_instructions = Vec::new();
                        for ins in &macro_def.instructions {
                            let analyzed = if let Some(arg) = ins.0.argument.clone() {
                                let analyzed = if let Argument::Implicit(idx) = arg {
                                    let upstream_arg = &args[idx as usize - 1];
                                    let im = match analyze_arg(upstream_arg) {
                                        Ok(im) => im,
                                        Err(e) => return Err(SyntaxError::new(real_line, e)),
                                    };

                                    let argument = match parse_arg(upstream_arg) {
                                        Ok(arg) => arg,
                                        Err(e) => return Err(SyntaxError::new(real_line, e)),
                                    };

                                    (im, argument)
                                } else {
                                    (ins.0.instruction_mode, Some(arg))
                                };

                                analyzed
                            } else {
                                (IM_IMPLIED, None)
                            };

                            let mut new_instruction = ins.clone();

                            new_instruction.2.push(macro_def.name.clone());
                            new_instruction.0.argument = analyzed.1;

                            if !is_defining_macro {
                                if new_instruction.0.instruction_mode == 0 && analyzed.0 == 0 {
                                    let mut trace = new_instruction.2.clone();
                                    trace.reverse();
                                    return Err(SyntaxError::new(
                                        real_line,
                                        format!(
											"No mode identifier specified for '{}' on line {}. (macro trace: {})",
											new_instruction.0.name,
											new_instruction.1,
											trace.join("->")
										),
                                    ));
                                }

                                current_address += 1 + get_argument_size_by_im(analyzed.0);
                            }
                            if new_instruction.0.instruction_mode == 0 {
                                new_instruction.0.instruction_mode = analyzed.0;
                            }

                            new_instructions.push(new_instruction);
                        }

                        if is_defining_macro {
                            let current_macro = current_macro.as_mut().unwrap();
                            current_macro.instructions.extend(new_instructions);

                            for arg in &parsed_args {
                                if let Some(Argument::Implicit(arg_idx)) = &arg.1 {
                                    let exists =
                                        macro_def.args.iter().find(|&ma| *ma == *arg_idx).is_some();

                                    if !exists {
                                        macro_def.args.push(*arg_idx);
                                    }
                                }
                            }
                        } else {
                            let only_instructions: Vec<Instruction> =
                                new_instructions.iter().map(|i| i.0.clone()).collect();
                            instructions.extend(only_instructions);
                        }
                    } else {
                        return Err(SyntaxError::new(
                            real_line,
                            format!("Unknown instruction '{}'.", name),
                        ));
                    }
                }
            }

            Token::Label(name) => {
                let exists = labels.iter().find(|&l| l.name == name);

                if exists.is_some() {
                    return Err(SyntaxError::new(
                        real_line,
                        format!("Label with name '{}' already exists.", name),
                    ));
                }

                let new_label = Label {
                    name,
                    address: current_address,
                };
                labels.push(new_label);
            }
            Token::Marker(name, args) => match name.as_ref() {
                "macro" => {
                    if is_defining_macro {
                        return Err(SyntaxError::new(
                            real_line,
                            String::from("Invalid placement of macro marker."),
                        ));
                    }

                    if args.len() == 0 || args[0].chars().count() == 0 {
                        return Err(SyntaxError::new(
                            real_line,
                            String::from("Missing macro name."),
                        ));
                    }
                    if args.len() > 1 {
                        return Err(SyntaxError::new(
                            real_line,
                            format!(
                                "Invalid macro name '{}', it has to be one word.",
                                args[0..].join(" ")
                            ),
                        ));
                    }

                    let name = args[0].to_owned();

                    let new_macro_def = MacroDef {
                        name,
                        args: Vec::new(),
                        instructions: Vec::new(),
                    };
                    current_macro = Some(new_macro_def);
                    is_defining_macro = true;
                }
                "end" => {
                    if let Some(current_macro) = &current_macro {
                        let mut ordered = current_macro.args.clone();
                        ordered.sort_by(|a, b| a.cmp(&b));

                        let mut prev_idx = 0;
                        for o in ordered {
                            if o != prev_idx + 1 {
                                return Err(SyntaxError::new(
                                    real_line,
                                    format!(
                                        "Invalid argument index '{}'. Argument indexes have to be in order.",
                                        o
                                    ),
                                ));
                            }
                            prev_idx += 1;
                        }

                        macros.push(current_macro.clone());
                        is_defining_macro = false;
                    } else {
                        return Err(SyntaxError::new(
                            real_line,
                            String::from("Invalid usage of '#end', there is no scope to be ended."),
                        ));
                    }
                }
                "include" => {}
                _ => {
                    return Err(SyntaxError::new(
                        real_line,
                        format!("Invalid keyword '{}'.", name),
                    ));
                }
            },
        }
    }

    for (idx, line) in instructions_using_labels {
        let ins = instructions.get_mut(idx).unwrap();

        if let Some(Argument::Label(name)) = &ins.argument {
            let label = labels.iter().find(|&l| l.name == *name);

            if label.is_none() {
                return Err(SyntaxError::new(
                    line,
                    format!("Label '{}' is not defined.", name),
                ));
            }

            ins.argument = Some(Argument::Explicit(label.unwrap().address));
        }
    }
    Ok(instructions)
}

/// Takes a vector of instructions and converts them to a vector of bytes which can be executed by the Tower architecture
fn assemble(instructions: Vec<Instruction>) -> Vec<u8> {
    let mut raw_bytes: Vec<u8> = Vec::new();

    for ins in instructions {
        let instruction = get_instruction_by_name(&ins.name).unwrap();
        let opcode = instruction.0;
        // convert to 0-7
        let im = (ins.instruction_mode as f32).log2() as u32;

        let instruction_byte = ((opcode << 3) | im) as u8;
        raw_bytes.push(instruction_byte);

        if let Some(arg) = ins.argument {
            if let Argument::Explicit(arg_val) = arg {
                let size = get_argument_size_by_im(ins.instruction_mode);
                if size == 0 {
                    continue;
                }

                if size == 2 {
                    raw_bytes.push(((arg_val >> 8) & 0xFF) as u8);
                }
                raw_bytes.push((arg_val & 0xFF) as u8);
            }
        }
    }
    raw_bytes
}
