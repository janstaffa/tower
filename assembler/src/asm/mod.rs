use crate::{
    InstructionMode, IM_ABSOLUTE, IM_CONSTANT, IM_IMMEDIATE, IM_INDIRECT, IM_REGA, IM_REGB,
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
    /// (Instruction, real line, macro trace)
    pub instructions: Vec<(Instruction, u32, Vec<String>)>,
}

pub struct Label {
    pub name: String,
    pub address: u32,
}

#[derive(Debug, Clone)]
pub enum Argument {
    /// literal argument
    Explicit(u32),
    /// argument index
    Implicit(u32),
    /// label name
    Label(String),
}

pub struct GenericInstruction {
    pub name: String,
    pub args: Vec<Argument>,
}
// ==============================================

pub fn analyze_arg(arg: &str) -> Result<InstructionMode, String> {
    if arg.chars().count() == 0 {
        return Err(String::from("Invalid argument, argument cannot be empty."));
    }

    let ident = arg.chars().next().unwrap();
    let im = match ident {
        '#' => IM_IMMEDIATE,
        '*' => IM_ABSOLUTE,
        '@' => IM_INDIRECT,
        '%' => {
            let reg = arg.chars().nth(1).unwrap();
            match reg {
                'a' => IM_REGA,
                'b' => IM_REGB,
                _ => return Err(format!("Invalid register '{}'.", reg)),
            }
        }
        '&' => IM_CONSTANT,
        _ => 0,
    };
    Ok(im)
}

/// Takes a raw argument as &str and parses it to an enum.
pub fn parse_arg(arg: &str) -> Result<Option<Argument>, String> {
    if arg.chars().count() == 0 {
        return Err(String::from("Invalid argument, argument cannot be empty."));
    }
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
            0 => arg,
            _ => &arg[1..],
        };

        let first_char = arg.chars().next().unwrap();

        if !first_char.is_digit(10) && im == 0 {
            return Ok(Some(Argument::Label(arg.to_string())));
        }

        // hex
        let val = if str_val.starts_with("0x") {
            u32::from_str_radix(&str_val[2..], 16)
        }
        // binary
        else if str_val.starts_with("0b") {
            u32::from_str_radix(&str_val[2..], 2)
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
