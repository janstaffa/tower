use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Instruction(pub u8, pub &'static str, pub &'static [IM]);

#[derive(Debug, PartialEq, Clone)]
pub enum IM {
    Implied = 0,
    Immediate = 1,
    Constant = 2,
    Absolute = 3,
    Indirect = 4,
    ZeroPage = 5,
    RegA = 6,
    RegB = 7,
}

impl IM {
    pub fn get_value(&self) -> u32 {
        match &self {
            IM::Implied => IM::Implied as u32,
            IM::Immediate => IM::Immediate as u32,
            IM::Constant => IM::Constant as u32,
            IM::Absolute => IM::Absolute as u32,
            IM::Indirect => IM::Indirect as u32,
            IM::ZeroPage => IM::ZeroPage as u32,
            IM::RegA => IM::RegA as u32,
            IM::RegB => IM::RegB as u32,
        }
    }
}
#[macro_export]
macro_rules! Inst {
    ( $op_code:literal, $name:ident, $($arg_type:expr),+ ) => {
            Instruction($op_code, stringify!($name), &[$($arg_type,)+])
    };
}

pub static INSTRUCTIONS: &[Instruction] = &[
    Inst!(0x00, NOP, IM::Implied),
    Inst!(0x01, LDA, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(0x02, STA, IM::Absolute, IM::ZeroPage),
    Inst!(0x03, TBA, IM::Implied),
    Inst!(0x04, TAB, IM::Implied),
    Inst!(0x05, TFA, IM::Implied),
    Inst!(0x06, TAF, IM::Implied),
    Inst!(0x07, JMP, IM::Absolute),
    Inst!(0x08, JZ, IM::Absolute),
    Inst!(0x09, JC, IM::Absolute),
    Inst!(0x0a, ADD, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(0x0b, ADC, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(0x0c, SUB, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(
        0x0d,
        NOT,
        IM::Implied,
        IM::Immediate,
        IM::Absolute,
        IM::ZeroPage
    ),
    Inst!(0x0e, NAND, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(
        0x0f,
        SRA,
        IM::Implied,
        IM::Immediate,
        IM::Absolute,
        IM::ZeroPage
    ),
    Inst!(
        0x10,
        SLA,
        IM::Implied,
        IM::Immediate,
        IM::Absolute,
        IM::ZeroPage
    ),
    Inst!(0x11, RB, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(0x12, WB, IM::Immediate, IM::Absolute, IM::ZeroPage),
    Inst!(0x1f, HLT, IM::Implied),
];

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
            syntax_error
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
