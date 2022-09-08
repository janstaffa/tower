use std::fs::File;
use std::io::prelude::*;

pub mod microasm;
pub mod asm;

pub const IM_IMPLIED: u32 = 1 << 0;
pub const IM_IMMEDIATE: u32 = 1 << 1;
pub const IM_CONSTANT: u32 = 1 << 2;
pub const IM_ABSOLUTE: u32 = 1 << 3;
pub const IM_INDIRECT: u32 = 1 << 4;
pub const IM_ZEROPAGE: u32 = 1 << 5;
pub const IM_REGA: u32 = 1 << 6;
pub const IM_REGB: u32 = 1 << 7;

pub fn get_im_name(im: u32) -> Result<&'static str, ()> {
    let im_v = 1 << im;
    let im = match im_v {
        IM_IMPLIED => "Implied",
        IM_IMMEDIATE => "Immediate",
        IM_CONSTANT => "Constant",
        IM_ABSOLUTE => "Absolute",
        IM_INDIRECT => "Indirect",
        IM_ZEROPAGE => "Zeropage",
        IM_REGA => "RegA",
        IM_REGB => "RegB",
        _ => return Err(()),
    };
    Ok(im)
}

pub type Instruction = (&'static str, u32);
pub static INSTRUCTIONS: &[Instruction] = &[
    ("NOP", IM_IMPLIED),
    ("LDA", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("STA", IM_CONSTANT),
    ("TBA", IM_IMPLIED),
    ("TAB", IM_IMPLIED),
    ("TFA", IM_IMPLIED),
    ("TAF", IM_IMPLIED),
    ("JMP", IM_CONSTANT),
    ("JZ", IM_CONSTANT),
    ("JC", IM_CONSTANT),
    ("ADD", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("ADC", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("SUB", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("NOT", IM_IMPLIED | IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("NAND", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("SRA", IM_IMPLIED | IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("SLA", IM_IMPLIED | IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("RB", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("WB", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    ("HLT", IM_IMPLIED),
];

pub fn get_instruction_by_name(name: &str) -> Option<(u32, &'static str, u32)> {
    let sig_idx = INSTRUCTIONS
        .iter()
        .position(|&i| i.0.to_lowercase() == name.to_lowercase());

    if let None = sig_idx {
        return None;
    }

    let sig_idx = sig_idx.unwrap();
    let sig = INSTRUCTIONS[sig_idx];

    return Some((sig_idx as u32, sig.0, sig.1));
}

pub fn read_file(path: &str) -> Result<String, AssemblerError> {
    let mut file = File::open(path).or(Err(AssemblerError::new(
        format!("File '{}' was not found", path),
        None,
    )))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .or(Err(AssemblerError::new(
            "Failed to read the input file".to_string(),
            None,
        )))?;

    Ok(contents)
}

pub fn read_file_binary(path: &str) -> Result<Vec<u8>, AssemblerError> {
    let mut file = File::open(path).or(Err(AssemblerError::new(
        format!("File '{}' was not found", path),
        None,
    )))?;

    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents).or(Err(AssemblerError::new(
        "Failed to read the input file".to_string(),
        None,
    )))?;

    Ok(contents)
}

#[derive(Debug)]
pub struct AssemblerError {
    pub message: String,
    pub syntax_error: Option<SyntaxError>,
}

impl AssemblerError {
    pub fn new(message: String, syntax_error: Option<SyntaxError>) -> Self {
        AssemblerError {
            message,
            syntax_error,
        }
    }
}

#[derive(Debug)]
pub struct SyntaxError {
    pub line: u32,
    pub message: String,
}

impl SyntaxError {
    pub fn new(line: u32, message: String) -> Self {
        SyntaxError { line, message }
    }
}
