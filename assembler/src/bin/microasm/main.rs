use std::fs::File;

use chrono::Utc;
use clap::Parser;
use tower_assembler::AssemblerError;

mod asm;
use crate::asm::assembler;

mod disasm;
use crate::disasm::disassembler;

// ==============================================
// =             SHARED DEFINITIONS             =
// ==============================================

pub const COMMENT_IDENT: char = ';';

pub const CONTROL_SIGNALS: &[&'static str] = &[
    "IEND", "HLT", "PCE", "PCO", "PCJ", "AI", "AO", "BI", "BO", "RSO", "OPADD", "OPSUB", "OPNOT",
    "OPNAND", "OPSR", "FI", "FO", "MI", "MO", "INI", "ARHI", "ARHO", "ARLI", "ARLO", "ARHLO", "HI",
    "HO", "LI", "LO", "HLO", "HLI", "DVE", "DVW",
];

// constants
pub const STEP_COUNTER_BIT_SIZE: u32 = 4;
pub const INSTRUCTION_MODE_BIT_SIZE: u32 = 3;
pub const FLAGS_BIT_SIZE: u32 = 2;

pub const INSTRUCTION_MODE_COUNT: usize = 8;
pub const FLAG_COMBINATIONS: usize = 4;
pub const TOTAL_DEF_COMBINATIONS: usize = INSTRUCTION_MODE_COUNT * FLAG_COMBINATIONS;

pub const MAX_MICRO_STEP_COUNT: usize = 16;

pub const FLAGS: [&'static str; FLAGS_BIT_SIZE as usize] = ["CARRY", "ZERO"];

// the values are exponents
pub type MicroStep = Vec<u32>;

pub struct MacroDef {
    name: String,
    steps: Vec<MicroStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstructionDef {
    name: String,
    instruction_mode: u32,
    flags: u32,
    steps: Vec<MicroStep>,
}

#[derive(Debug, PartialEq)]
pub struct TokenizedLine(u32, LineType);

#[derive(Debug, PartialEq)]
pub enum LineType {
    // name, args
    KeyLine(String, Vec<String>),
    // words
    StepLine(Vec<String>),
    // name of the label
    LabelLine(String),
}

#[derive(Debug, Clone)]
pub struct Conditional {
    flag: u32,
    is_inverted: bool,
}

// ==============================================

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Source file to be assembled
    #[clap(short, long)]
    r#in: String,

    /// File to be written the output to.
    #[clap(short, long)]
    out: Option<String>,

    #[clap(subcommand)]
    cmd: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Assemble,
    Disassemble,
}

const ASSEMBLER_DEFAULT_OUT_FILE: &'static str = "microcode.bin";
const DISASSEMBLER_DEFAULT_OUT_FILE: &'static str = "out.txt";

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", format!("❌ Error: {}", e.message));
        if let Some(serr) = e.syntax_error {
            eprintln!("{}", format!("{} (line {})", serr.message, serr.line));
        }
    }
}

fn run() -> Result<(), AssemblerError> {
    let args = Args::parse();

    let input_file_path = &args.r#in;
    let output_file_path = &args.out;

    if matches!(File::open(input_file_path), Err(_)) {
        return Err(AssemblerError::new(
            String::from("Failed to read the input file."),
            None,
        ));
    }

    let start_time = Utc::now();

    match args.cmd {
        Action::Assemble => {
            let output_file_path = output_file_path
                .clone()
                .unwrap_or(String::from(ASSEMBLER_DEFAULT_OUT_FILE));

            println!("Assembling... '{}'", input_file_path);
            assembler(input_file_path, &output_file_path)?;

            let now = Utc::now();
            let delta_time = now - start_time;
            println!(
                "✔️  Finished and written to '{}' (after {}ms)",
                output_file_path,
                delta_time.num_milliseconds()
            );
        }
        Action::Disassemble => {
            let output_file_path = output_file_path
                .clone()
                .unwrap_or(String::from(DISASSEMBLER_DEFAULT_OUT_FILE));
            println!("Disassembling... '{}'", input_file_path);
            disassembler(input_file_path, &output_file_path)?;

            let now = Utc::now();
            let delta_time = now - start_time;
            println!(
                "✔️  Finished and written to '{}' (after {}ms)",
                output_file_path,
                delta_time.num_milliseconds()
            );
        }
    }

    Ok(())
}
