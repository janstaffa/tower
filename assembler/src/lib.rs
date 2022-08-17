use std::io::{Error, ErrorKind};
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Instruction(u8, &'static str, &'static [IAT]);

#[derive(Debug)]
pub enum IAT {
    None = 0,
    Immediate = 1,
    Address = 2,
    ZeroPageAddress = 3
}

#[macro_export]
macro_rules! Inst {
    ( $op_code:literal, $name:ident, $($arg_type:expr),+ ) => {
            Instruction($op_code, stringify!($name), &[$($arg_type,)+])
    };
}


pub static INSTRUCTIONS: &[Instruction] = &[
    Inst!(0x00, NOP, IAT::None),
    Inst!(0x01, LDA, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x02, STA, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x03, TBA, IAT::None),
    Inst!(0x04, TAB, IAT::None),
    Inst!(0x05, TFA, IAT::None),
    Inst!(0x06, TAF, IAT::None),
    Inst!(0x07, JMP, IAT::Address),
    Inst!(0x08, JZ, IAT::Address),
    Inst!(0x09, JC, IAT::Address),
    Inst!(0x0a, ADD, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x0b, ADC, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x0c, SUB, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x0d, NOT, IAT::None, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x0e, NAND, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x0f, SRA, IAT::None, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x10, SLA, IAT::None, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x11, RB, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x12, WB, IAT::Immediate, IAT::Address, IAT::ZeroPageAddress),
    Inst!(0x1f, HLT, IAT::None)
];



pub fn read_file(path: &str) -> Result<String, AssemblerError> {
    let mut file = File::open(path).or(Err(AssemblerError::new(format!("File '{}' was not found", path))))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents).or(Err(AssemblerError::new("Failed to read the input file")))?;
                                          

    Ok(contents)
}

pub enum AssemblerError {
    message: String,
    syntax_errors: Vec<SyntaxError>
}

impl AssemblerError {
    pub fn new(message: u32, errors: Vec<SyntaxError>) -> Self {
        AssemblerError { message, syntax_errors }
    }
}


pub struct SyntaxError {
    line: u32,
    message: String
}

impl SyntaxError {
    pub fn new(line: u32, message: String) -> Self {
        SyntaxError { line, message }
    }
}
