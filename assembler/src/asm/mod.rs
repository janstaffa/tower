use regex::Regex;

use crate::{
    InstructionMode, IM_ABSOLUTE, IM_CONSTANT, IM_IMMEDIATE, IM_IMPLIED, IM_INDIRECT, IM_REGA,
    IM_REGB,
};

pub mod asm;

// ==============================================
// =             SHARED DEFINITIONS             =
// ==============================================

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    ///        (name, arguments)
    Instruction(String, Vec<String>),
    ///    name
    Label(String),
    ///    (name, arguments)
    Marker(String, Vec<String>),
}

#[derive(Debug, PartialEq)]
pub struct TokenizedLine(u32, Token);

#[derive(Debug, Clone)]
pub struct Instruction {
    pub name: String,
    pub argument: Option<Argument>,
    pub instruction_mode: u32,
}

#[derive(Debug, Clone)]
pub struct MacroDef {
    pub name: String,
    pub args: Vec<u32>,
    pub instructions: Vec<Instruction>,
}

pub struct Label {
    pub name: String,
    pub address: u32,
}

#[derive(Debug, Clone)]
pub enum Argument {
    //		 argument
    Explicit(u32),
    //		 argument index
    Implicit(u32),
}

pub struct GenericInstruction {
    pub name: String,
    pub args: Vec<Argument>,
}
// ==============================================

pub fn analyze_arg(arg: &str) -> Result<InstructionMode, String> {
    let im = match arg.chars().next().unwrap() {
        '#' => IM_IMMEDIATE,
        '*' => IM_ABSOLUTE,
        '@' => IM_INDIRECT,
        '%' => {
            let reg = arg.chars().nth(1).unwrap();
            match reg {
                'A' => IM_REGA,
                'B' => IM_REGB,
                _ => return Err(format!("Invalid register '{}'.", reg)),
            }
        }
        _ => IM_CONSTANT,
    };
    Ok(im)
}

/// Takes a raw argument as &str and parses it to an enum.
pub fn parse_arg(arg: &str) -> Result<Option<Argument>, String> {
    let in_place_argument_idx = arg.find('$');
    let arg = if let Some(ipa_idx) = in_place_argument_idx {
        let argument_index_str = &arg[(ipa_idx + 1)..];
        if let Ok(argument_index) = argument_index_str.parse() {
            Argument::Implicit(argument_index)
        } else {
            return Err(format!("Invalid argument index '{}'.", argument_index_str));
        }
    } else {
        let im = analyze_arg(arg)?;

        let str_val = match im {
            IM_REGA | IM_REGB => return Ok(None),
            IM_CONSTANT => arg,
            _ => &arg[1..],
        };

        // hex
        let val = if str_val.starts_with("0x") {
            u32::from_str_radix(str_val, 16)
        }
        // binary
        else if str_val.starts_with("0b") {
            u32::from_str_radix(str_val, 2)
        }
        // decimal
        else {
            str_val.parse::<u32>()
        };

        if let Err(_) = val {
            return Err(format!("Failed to parse value '{}'.", str_val));
        }
        Argument::Explicit(val.unwrap())
    };

    Ok(Some(arg))
}
