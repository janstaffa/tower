use clap::Parser;
use std::io::{Write};
use std::{fs::File};
use tower_assembler::{read_file, AssemblerError, SyntaxError, IM, INSTRUCTIONS};

const COMMENT_IDENT: char = ';';

const CONTROL_SIGNALS: &[&'static str] = &[
    "HLT", "IEND", "PCE", "PCO", "PCJ", "AI", "AO", "BI", "BO", "RSO", "OPADD", "OPSUB", "OPNOT",
    "OPNAND", "OPSR", "FI", "FO", "MI", "MO", "INI", "HI", "HO", "LI", "LO", "HLO", "DVE", "DVW",
];

const STEP_COUNTER_BIT_SIZE: u32 = 4;
const INSTRUCTION_MODE_BIT_SIZE: u32 = 3;
const FLAGS_BIT_SIZE: u32 = 2;

// the values are exponents
type MicroStep = Vec<u32>;

struct MacroDef {
    name: String,
    steps: Vec<MicroStep>,
}

#[derive(Debug, PartialEq)]
struct FlagStatus {
    carry: bool,
    zero: bool,
}

#[derive(Debug)]
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

fn main() {
    let dummy_code = "
	 #macro TEST
	 PCE PCO

	 #macro TEST2
	 PCE PCO
     PCJ

	 #def ADD
	 HLT IEND TEST
  
	 #def SUB
	 HLT IEND
     TEST2
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
            line[0..idx].to_string()
        } else {
            line
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
    let mut current_instruction: Option<InstructionDef> = None;

    for (i, token) in tokens.iter().enumerate() {
        let (real_line, line) = (&token.0, &token.1);

        let mut is_new_def = false;
        if let LineType::KeyLine(keyword, _) = line {
            if let "def" | "pref" | "suf" | "macro" = &keyword[..] {
                is_new_def = true;
            }
        }
        if is_new_def {
            if let Some(instruction) = current_instruction {
                instructions.push(instruction);
                current_instruction = None;
            }
            if let Some(macro_def) = current_macro {
                macros.push(macro_def);
                current_macro = None;
            }
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
                    let new_inst_def = InstructionDef {
                        name: inst_name,
                        instruction_mode: IM::Implied,
                        flags: FlagStatus {
                            carry: false,
                            zero: false,
                        },
                        steps: Vec::new(),
                    };

                    current_instruction = Some(new_inst_def);
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

                "if" => {}
                "end" => {}
                "else" => {}
                "pref" => {}
                "suf" => {}
                _ => {
                    return Err(SyntaxError::new(
                        *real_line,
                        format!("Invalid keyword '{}'", keyword),
                    ));
                }
            },
            LineType::StepLine(words) => {
                let mut control_signals = Vec::new();

                let mut added_steps = Vec::new();

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
                                added_steps.extend(macro_def.steps[1..].to_vec());
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
                if let Some(instruction) = &mut current_instruction {
                    instruction.steps.push(this_step.clone());
                    instruction.steps.extend(added_steps.clone());
                }
                if let Some(macro_def) = &mut current_macro {
                    macro_def.steps.push(this_step);
                    macro_def.steps.extend(added_steps.clone());
                }
            }
            LineType::LabelLine(label) => {}
        }
    }
    if let Some(instruction) = current_instruction {
        instructions.push(instruction);
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
        let flags = (addr >> 8) & 0b11;
        let instruction_mode = (addr >> 5) & 0b111;
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
        #macro TEST
        PCE PCO

        #macro TEST2
        PCE PCO
        PCJ

        #def ADD
        HLT IEND TEST
    
        #def SUB
        HLT IEND
        TEST2
	"
    .into();

    let tokens = tokenize(dummy_code).unwrap();
    let correctly_tokenized = Vec::from([
        TokenizedLine(
            2,
            LineType::KeyLine("macro".into(), Vec::from(["test".into()])),
        ),
        TokenizedLine(
            3,
            LineType::StepLine(Vec::from(["pce".into(), "pco".into()])),
        ),
        TokenizedLine(
            5,
            LineType::KeyLine("macro".into(), Vec::from(["test2".into()])),
        ),
        TokenizedLine(
            6,
            LineType::StepLine(Vec::from(["pce".into(), "pco".into()])),
        ),
        TokenizedLine(7, LineType::StepLine(Vec::from(["pcj".into()]))),
        TokenizedLine(
            9,
            LineType::KeyLine("def".into(), Vec::from(["add".into()])),
        ),
        TokenizedLine(
            10,
            LineType::StepLine(Vec::from(["hlt".into(), "iend".into(), "test".into()])),
        ),
        TokenizedLine(
            12,
            LineType::KeyLine("def".into(), Vec::from(["sub".into()])),
        ),
        TokenizedLine(
            13,
            LineType::StepLine(Vec::from(["hlt".into(), "iend".into()])),
        ),
        TokenizedLine(14, LineType::StepLine(Vec::from(["test2".into()]))),
    ]);

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

    let correctly_parsed = Vec::from([
        InstructionDef {
            name: "add".into(),
            instruction_mode: IM::Implied,
            flags: FlagStatus {
                carry: false,
                zero: false,
            },
            steps: Vec::from([Vec::from([0, 1, 2, 3])]),
        },
        InstructionDef {
            name: "sub".into(),
            instruction_mode: IM::Implied,
            flags: FlagStatus {
                carry: false,
                zero: false,
            },
            steps: Vec::from([Vec::from([0, 1]), Vec::from([2, 3]), Vec::from([4])]),
        },
    ]);

    for (i, p) in parsed.iter().enumerate() {
        assert_eq!(p.name, correctly_parsed[i].name);
        assert_eq!(p.instruction_mode, correctly_parsed[i].instruction_mode);
        assert_eq!(p.flags, correctly_parsed[i].flags);
        assert_eq!(p.steps, correctly_parsed[i].steps);
    }
}
