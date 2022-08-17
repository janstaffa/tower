use clap::Parser;
use tower_assembler::{INSTRUCTIONS, read_file, CompilerError, CompilationError};
use std::fs::File;
use std::io::Error;
use std::path::PathBuf;


const COMMENT_IDENT: char = ';';

enum CONTROL_SIGNALS {
    HLT    = 0,
    IEND   = 1,
    PCE    = 2,
    PCO    = 3,
    PCJ    = 4,
    AI     = 5,
    AO     = 6,
    BI     = 7,
    BO     = 8,
    RSO    = 9,
    OPADD  = 10,
    OPSUB  = 11,
    OPNOT  = 12,
    OPNAND = 13,
    OPSR   = 14,
    FI     = 15,
    FO     = 16,
    MI     = 17,
    MO     = 18,
    INI    = 19,
    HI     = 20,
    HO     = 21,
    LI     = 22,
    LO     = 23,
    HLO    = 24,
    DVE    = 25,
    DVW    = 26,
}

struct MicroStep(Vec<CONTROL_SIGNALS>);

struct MacroDef {
    name: String,
    steps: Vec<MicroStep>
}

struct FlagStatus {
    carry: bool,
    zero: bool
}

struct InstructionDef {
    name: String,
    args: u8,
    flags: FlagStatus,
    steps: Vec<MicroStep>
}


enum TokenizedLine {
    /// real line, name, args
    KeyLine(u32, String, Vec<String>),
    /// real line, words
    StepLine(u32, Vec<String>)
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// Source file to be assembled
   #[clap(short, long, value_parser)]
   r#in: String,
}


fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e.message);
        for se in e.syntax_errors {
            eprintln!("{} On line {}", se.message, se.line);
        }
    }
}


fn run() -> Result<(), AssemblerError> { let args = Args::parse();
    let input = read_file(&args.r#in)?;

    let tokens = match tokenize(input) {
        Some(t) => t,
        Err(e) => return e
    };

    Ok(())
}


fn tokenize(code: String) -> Result<Vec<TokenizedLine>, AssemblerError> {
    let mut tokenized_lines: Vec<TokenizedLine> = Vec::new();
    let mut errors: Vec<SyntaxError> = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
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


        let words = line.split_whitescpace();

        let tokenized = if line.chars().nth(0) == '#' {
            if line.chars().len() == 1 {
                erors.push(SyntaxError::new(line_idx + 1, "No keyword was specified"));
                continue;
            }

            TokenizedLine::KeyLine(line_idx + 1, words[0][1..].to_string(), words[1..].collect())
        } else {
            TokenizedLine::StepLine(line_idx + 1, words)
        }

        tokenized_lines.push(tokenized);
    }

    if tokenized_lines.len() == 0 {
        return Err(AssemblerError::new("There is no code to be assembled", Vec::new()));
    }
    if errors.len() > 0 {
        return Err(AssemblerError::new(format!("Failed to assemble the code. There are {} errors", errors.len()), errors));
    }
}


fn parse(tokens: Vec<TokenizedLine>) -> Result<Vec<InstructionDef>, AssemblerError> {
    let mut errors: Vec<SyntaxError> = Vec::new();

    let mut macros: Vec<MacroDef> = Vec::new();
    let mut instructions: Vec<InstructionDef> = Vec::new();

    let mut is_macro = false;
    let mut is_def = false;

    let mut current_macro: Option<MacroDef> = None;
    let mut current_instruction: Option<InstructionDef> = None;

    let mut micro_steps: Vec<MicroStep> = Vec::new();

    for token in tokens {
        match token {
            TokenizedLine::KeyLine(keyword, args) => {
                match &keyword {
                    "def" => {
                        is_def = true;
                        InstructionDef {
                            name: args[0],

                        }
                        args[]
                    },
                    "macro" => {
                        is_macro = true;
                    },
                    _ => {
                        is_error = true;
                        errors.push(syntaxerror::new(line_idx + 1, format!("invalid keyword '{}'", words[0])));
                    }
                }
            },
            tokenizedline::stepline(words) => {

            }
        }

    }
}


