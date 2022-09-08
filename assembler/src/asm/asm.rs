use crate::{AssemblerError, read_file};

use super::Token;

pub fn assembler(file_in: &str, file_out: &str) -> Result<(), AssemblerError> {
    let input = read_file(file_in)?;
    // tokenize
    let tokens = match tokenize(input) {
        Ok(t) => t,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at tokenization step"),
                Some(e),
            ))
        }
    };

    // parse
    let parsed = match parse(tokens) {
        Ok(idf) => idf,
        Err(e) => {
            return Err(AssemblerError::new(
                String::from("Assembly failed at parsing step"),
                Some(e),
            ))
        }
    };

    // assemble
    let output = assemble(parsed);

    // write to output file
    let mut file = File::create(file_out).unwrap();
    file.write_all(&output).unwrap();
    Ok(())
}


fn tokenize(code: String) -> Result<Vec<Token>, AssemblerError> {
    let mut tokens: Vec<Token> = Vec::new();

    for (line_idx, line) in code.lines().enumerate() {
        let real_line = line_idx as u32 + 1;
        let line = line.trim().to_lowercase();

        // check for comments and remove if found
        let comment_idx = line.chars().position(|c| c == COMMENT_IDENT);
        let line = if let Some(idx) = comment_idx {
            line[0..idx].trim().to_string()
        } else {
            line.trim().to_string()
        };

        // skip empty lines
        if line.len() == 0 {
            continue;
        }
    }

    Ok(tokens)
}