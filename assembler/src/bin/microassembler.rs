use std::fs::File;

use chrono::Utc;
use clap::Parser;
use tower_assembler::{
    microasm::{asm::assembler, disasm::disassembler},
    AssemblerError,
};

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
