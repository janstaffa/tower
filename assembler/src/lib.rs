use std::fs::File;
use std::io::prelude::*;

pub mod asm;
pub mod microasm;

pub type InstructionMode = u32;

pub const fn im_idx_to_val(idx: u32) -> u32 {
	1 << idx
}
pub const IM_IMPLIED: InstructionMode = im_idx_to_val(0);
pub const IM_IMMEDIATE: InstructionMode = im_idx_to_val(1);
pub const IM_CONSTANT: InstructionMode = im_idx_to_val(2);
pub const IM_ABSOLUTE: InstructionMode = im_idx_to_val(3);
pub const IM_INDIRECT: InstructionMode = im_idx_to_val(4);
pub const IM_ZEROPAGE: InstructionMode = im_idx_to_val(5);
pub const IM_REGA: InstructionMode = im_idx_to_val(6);
pub const IM_REGB: InstructionMode = im_idx_to_val(7);

pub const fn get_argument_size_by_im(im: InstructionMode) -> u32 {
	match im {
		IM_ABSOLUTE | IM_CONSTANT | IM_INDIRECT => 2,
		IM_IMMEDIATE | IM_ZEROPAGE => 1,
		_ => 0,
	}
}

pub fn get_im_name(im: InstructionMode) -> Result<&'static str, ()> {
    let im_v = im_idx_to_val(im);
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
pub fn get_available_im_names(ims: u32) -> Vec<String> {
    let mut output = Vec::new();
    for i in 0..8 {
        let checking_im = im_idx_to_val(i);
        if (ims & checking_im) != 0 {
            let name = get_im_name(i).unwrap().to_string();
            output.push(name);
        }
    }
    output
}

const IM_IMM_ABS_ZP_IND: u32 = IM_IMMEDIATE | IM_ABSOLUTE | IM_ZEROPAGE | IM_INDIRECT;
pub type Instruction = (&'static str, u32);
pub static INSTRUCTIONS: &[Instruction] = &[
    ("NOP", IM_IMPLIED),
    ("LDA", IM_IMM_ABS_ZP_IND),
    ("STA", IM_CONSTANT | IM_INDIRECT),
    ("ADC", IM_IMM_ABS_ZP_IND),
    ("ADD", IM_IMM_ABS_ZP_IND),
    ("SBB", IM_IMM_ABS_ZP_IND),
    ("SUB", IM_IMM_ABS_ZP_IND),
    ("INC", IM_CONSTANT | IM_REGA | IM_REGB),
    ("DEC", IM_CONSTANT | IM_REGA | IM_REGB),
    ("CMP", IM_IMM_ABS_ZP_IND),
    ("JMP", IM_CONSTANT),
    ("JC", IM_CONSTANT),
    ("JZ", IM_CONSTANT),
    ("JNZ", IM_CONSTANT),
    ("NOTA", IM_IMPLIED),
    ("NAND", IM_IMM_ABS_ZP_IND),
    ("SRA", IM_IMPLIED),
    ("SLA", IM_IMPLIED),
    ("JSR", IM_CONSTANT),
    ("RTS", IM_IMPLIED),
    ("TBA", IM_IMPLIED),
    ("PSA", IM_IMPLIED),
    ("PSF", IM_IMPLIED),
    ("POA", IM_IMPLIED),
    ("POF", IM_IMPLIED),
    ("TBA", IM_IMPLIED),
    ("TAB", IM_IMPLIED),
    ("TFA", IM_IMPLIED),
    ("TAF", IM_IMPLIED),
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
