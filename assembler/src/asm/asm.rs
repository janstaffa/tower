use std::{fs::File};

use regex::Regex;

use crate::{
    get_instruction_by_name, microasm::COMMENT_IDENT, read_file, AssemblerError, SyntaxError,
    IM_ABSOLUTE, IM_CONSTANT, IM_IMMEDIATE, IM_IMPLIED, IM_INDIRECT, IM_REGA, IM_REGB,
};

use super::{
    analyze_arg, parse_arg, Argument, Instruction, Label, MacroDef, Token,
    TokenizedLine,
};

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
    // println!("tokens {:?}", tokens);

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
    println!("parsed {:?}", parsed);

    // assemble
    // let output = assemble(parsed);

    // write to output file
    let mut file = File::create(file_out).unwrap();
    // file.write_all(&output).unwrap();
    Ok(())
}

fn tokenize(code: String) -> Result<Vec<TokenizedLine>, SyntaxError> {
    let mut tokenized_lines: Vec<TokenizedLine> = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
        let real_line = line_idx as u32 + 1;
        let line = line.trim();

        // check for comments and remove if found
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
        else if let Some(':') = line.chars().last() {
            if words.len() > 1 {
                return Err(SyntaxError::new(
                    real_line,
                    String::from("Invalid label definition, a label can only be one word."),
                ));
            }
            let mut label = words[0].to_owned();

            let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
            if re.is_match(&label) {
                return Err(SyntaxError::new(
                    real_line,
                    format!("Invalid label name '{}'. Label name can only contain characters a-z or numbers.", label),
                ));
            }

            // remove the colon
            label.pop();

            TokenizedLine(real_line, Token::Label(label))
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
                    if a.contains(" ") {
                        return Err(SyntaxError::new(
								real_line,
								format!("Invalid argument '{}', arguments can only be one word. If you want to specify two arguments separate them by a comma.", a),
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
    let mut current_macro_args: Vec<u32> = Vec::new();

    let mut labels: Vec<Label> = Vec::new();

    let mut current_address = 0;

    for t in tokens {
        let (line_idx, token) = (t.0, t.1);
        let real_line = line_idx + 1;

        match token {
            Token::Instruction(name, args) => {
                let mut new_args = Vec::new();

                // parse raw args to nice structures
                for a in &args {
                    match parse_arg(&a) {
                        Ok(a) => new_args.push(a),
                        Err(e) => return Err(SyntaxError::new(real_line, e)),
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
                        let arg = &args[0];
                        let im = match analyze_arg(arg) {
                            Ok(im) => im,
                            Err(e) => return Err(SyntaxError::new(real_line, e)),
                        };
                        let arg = match parse_arg(arg) {
                            Ok(arg) => arg,
                            Err(e) => return Err(SyntaxError::new(real_line, e)),
                        };
                        (im, arg)
                    } else {
                        (IM_IMPLIED, None)
                    };

                    let new_instruction = Instruction {
                        name: name.to_lowercase(),
                        argument,
                        instruction_mode,
                    };

                    if is_defining_macro {
                        let current_macro = current_macro.as_mut().unwrap();

                        if args.len() == 1 {
                            let arg = &args[0];
                            let in_place_argument_idx = arg.find('$');
                            if let Some(ipa_idx) = in_place_argument_idx {
                                let argument_index_str = &arg[(ipa_idx + 1)..];
                                if let Ok(argument_index) = argument_index_str.parse() {
                                    let exists = current_macro
                                        .args
                                        .iter()
                                        .find(|&ma| *ma == argument_index)
                                        .is_some();

                                    if !exists {
                                        current_macro.args.push(argument_index);
                                    }
                                } else {
                                    return Err(SyntaxError::new(
                                        real_line,
                                        format!(
                                            "Invalid macro argument index '{}'.",
                                            argument_index_str
                                        ),
                                    ));
                                }
                            }
                        }
                        current_macro.instructions.push(new_instruction);
                    } else {
                        instructions.push(new_instruction);
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
                            let argument = if let Some(arg) = ins.argument.clone() {
                                let argument = if let Argument::Implicit(idx) = arg {
                                    match parse_arg(&args[idx as usize - 1]) {
                                        Ok(arg) => arg,
                                        Err(e) => return Err(SyntaxError::new(real_line, e)),
                                    }
                                } else {
                                    Some(arg)
                                };
                                argument
                            } else {
                                None
                            };

                            let mut new_instruction = ins.clone();
                            new_instruction.argument = argument;

                            new_instructions.push(new_instruction);
                        }

                        if is_defining_macro {
                            let current_macro = current_macro.as_mut().unwrap();
                            current_macro.instructions.extend(new_instructions);

                            for arg in args {
                                let in_place_argument_idx = arg.find('$');
                                if let Some(ipa_idx) = in_place_argument_idx {
                                    let argument_index_str = &arg[(ipa_idx + 1)..];
                                    if let Ok(argument_index) = argument_index_str.parse() {
                                        let exists = macro_def
                                            .args
                                            .iter()
                                            .find(|&ma| *ma == argument_index)
                                            .is_some();

                                        if !exists {
                                            macro_def.args.push(argument_index);
                                        }
                                    } else {
                                        return Err(SyntaxError::new(
                                            real_line,
                                            format!(
                                                "Invalid macro argument index '{}'.",
                                                argument_index_str
                                            ),
                                        ));
                                    }
                                }
                            }
                        } else {
                            instructions.extend(new_instructions);
                        }
                    } else {
                        return Err(SyntaxError::new(
                            real_line,
                            format!("Unknown instruction '{}'.", name),
                        ));
                    }
                }
            }

            Token::Label(name) => {}
            Token::Marker(name, args) => match name.as_ref() {
                "macro" => {
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

    Ok(instructions)
}
