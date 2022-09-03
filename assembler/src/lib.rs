use std::fs::File;
use std::io::prelude::*;

pub static IM_IMPLIED: u32 = 1 << 0;
pub static IM_IMMEDIATE: u32 = 1 << 1;
pub static IM_CONSTANT: u32 = 1 << 2;
pub static IM_ABSOLUTE: u32 = 1 << 3;
pub static IM_INDIRECT: u32 = 1 << 4;
pub static IM_ZEROPAGE: u32 = 1 << 5;
pub static IM_REGA: u32 = 1 << 6;
pub static IM_REGB: u32 = 1 << 7;

pub type Instruction = (u32, &'static str, u32);
pub static INSTRUCTIONS: &[Instruction] = &[
    (0x00, "NOP", IM_IMPLIED),
    (0x01, "LDA", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (0x02, "STA", IM_ABSOLUTE | IM_ZEROPAGE),
    (0x03, "TBA", IM_IMPLIED),
    (0x04, "TAB", IM_IMPLIED),
    (0x05, "TFA", IM_IMPLIED),
    (0x06, "TAF", IM_IMPLIED),
    (0x07, "JMP", IM_CONSTANT),
    (0x08, "JZ", IM_CONSTANT),
    (0x09, "JC", IM_CONSTANT),
    (0x0a, "ADD", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (0x0b, "ADC", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (0x0c, "SUB", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (
        0x0d,
        "NOT",
        IM_IMPLIED | IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE,
    ),
    (0x0e, "NAND", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (
        0x0f,
        "SRA",
        IM_IMPLIED | IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE,
    ),
    (
        0x10,
        "SLA",
        IM_IMPLIED | IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE,
    ),
    (0x11, "RB", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (0x12, "WB", IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE),
    (0x1f, "HLT", IM_IMPLIED),
];

pub fn get_instruction(name: &str) -> Option<&(u32, &str, u32)> {
    INSTRUCTIONS
        .iter()
        .find(|&i| i.1.to_lowercase() == name.to_lowercase())
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
