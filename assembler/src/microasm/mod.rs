pub mod asm;
pub mod disasm;

// ==============================================
// =             SHARED DEFINITIONS             =
// ==============================================

pub const COMMENT_IDENT: char = ';';

pub const CONTROL_SIGNALS: &[&'static str] = &[
    "IEND",
    "HLT",
    "PCI",
    "PCO",
    "PCJ",
    "SPI",
    "SPO",
    "SPOA",
    "AI",
    "BI",
    "BO",
    "HI",
    "HO",
    "LI",
    "LO",
    "HLO",
    "HLI",
    "ARHI",
    "ARHO",
    "ARLI",
    "ARLO",
    "ARHLO",
    "ALUO",
    "OPADD",
    "OPSUB",
    "OPNOT",
    "OPNAND",
    "OPSR",
    "INCE",
    "DEC",
    "INCI",
    "INCO",
    "FI",
    "FO",
    "MI",
    "MO",
    "INI",
    "_RAMSTART",
    "_SPSTART",
];

// constants
pub const OPCODE_BIT_SIZE: u32 = 5;
pub const STEP_COUNTER_BIT_SIZE: u32 = 4;
pub const INSTRUCTION_MODE_BIT_SIZE: u32 = 3;
pub const FLAGS_BIT_SIZE: u32 = 3;

pub const INSTRUCTION_MODE_COUNT: usize = 2_usize.pow(INSTRUCTION_MODE_BIT_SIZE);
pub const FLAG_COMBINATIONS: usize = 2_usize.pow(FLAGS_BIT_SIZE);
pub const TOTAL_DEF_COMBINATIONS: usize = INSTRUCTION_MODE_COUNT * FLAG_COMBINATIONS;

pub const MAX_MICRO_STEP_COUNT: usize = 16;

pub const FLAGS: [&'static str; FLAGS_BIT_SIZE as usize] = ["CARRY", "ZERO", "INCARRY"];

pub const CONTROL_BYTES: usize = 5;

// the values are exponents
pub type MicroStep = Vec<u64>;

#[derive(Debug, Clone)]
pub struct ConditionalStep {
    pub step: MicroStep,
    pub conditions: Vec<Conditional>,
}
pub struct MacroDef {
    name: String,
    steps: Vec<ConditionalStep>,
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
