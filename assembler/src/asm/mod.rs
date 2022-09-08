mod asm;


// ==============================================
// =             SHARED DEFINITIONS             =
// ==============================================

#[derive(Debug, Clone)]
pub enum Token  {
    ///        (name, arguments)
    Instruction(String, Vec<String>),
    ///    name
    Label(String),
    ///    name
    Marker(String, Vec<String>),
    ///     (name, arguments)
    MacroDef(String, Vec<String>)
}

// ==============================================