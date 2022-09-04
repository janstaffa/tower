use std::{fs::File, io::Write};

use tower_assembler::{get_im_name, read_file_binary, AssemblerError, INSTRUCTIONS};

use crate::{InstructionDef, CONTROL_SIGNALS};

pub fn disassembler(file_in: &str, file_out: &str) -> Result<(), AssemblerError> {
    let input = read_file_binary(file_in)?;
    let disassembled = disassemble(input)?;

    let bar_len = 150;

    let mut output = String::new();
    let mut prev: Option<InstructionDef> = None;
    for ins in &disassembled {
        if ins.steps.first().unwrap().len() == 0 {
            continue;
        }
        if matches!(prev, None) || prev.as_ref().unwrap().name != ins.name {
            // print the instruction name inside frame
            output += &format!("\n{:=^bar_len$}\n", "");
            output += &format!("={: ^len$}=\n", ins.name, len = bar_len - 2);
            output += &format!("{:=^bar_len$}\n", "");
        }

        prev = Some(ins.clone());

        output += "\n";

        // print border
        output += &format!("+{:-^len$}+\n", "", len = bar_len - 2);

        let instruction_mode = get_im_name(ins.instruction_mode).unwrap();

        output += &format!(
            "|{: <len$}|\n",
            format!("Instruction mode: {}", instruction_mode),
            len = bar_len - 2
        );
        output += &format!(
            "|{: <len$}|\n",
            format!("Flag status: {}", ins.flags),
            len = bar_len - 2
        );
        output += &format!("|{:-^len$}|\n", "MICROSTEPS", len = bar_len - 2);

        for s in &ins.steps {
            if s.len() == 0 {
                continue;
            }
            let named = s
                .iter()
                .map(|&s| {
                    let si = (s as f32).log2() as usize;
                    CONTROL_SIGNALS[si]
                })
                .collect::<Vec<&str>>();
            let steps_str = named.join(", ");

            output += &format!("|{: <len$}|\n", steps_str, len = bar_len - 2);
        }
        output += &format!("+{:-^len$}+\n", "", len = bar_len - 2);
    }

    let mut output_file = File::create(file_out).unwrap();
    output_file.write(output.as_bytes()).unwrap();
    Ok(())
}

/// Takes a vector of bytes containing the microcode and generates instruction definitions for it
fn disassemble(input_bytes: Vec<u8>) -> Result<Vec<InstructionDef>, AssemblerError> {
    if input_bytes.len() % 4 != 0 {
        return Err(AssemblerError::new(
            String::from("Invalid Tower microassembly code"),
            None,
        ));
    }

    let code_len = (input_bytes.len() / 4) as u32;

    let mut output: Vec<InstructionDef> = Vec::new();
    let mut current_instruction: Option<InstructionDef> = None;

    for addr in 0..code_len {
        let abs_byte = (addr * 4) as usize;

        // get individual components of the address
        let opcode = addr >> 9;

        let instruction_mode = (addr >> 6) & 0b111;
        let flags = (addr >> 4) & 0b11;
        let _micro_step = addr & 0b1111;

        let ins_signature = INSTRUCTIONS.get(opcode as usize);
        if let None = ins_signature {
            continue;
        }
        let ins_signature = ins_signature.unwrap();

        let mut found_csignals: Vec<u32> = Vec::new();
        let mut control_bytes = input_bytes[abs_byte..(abs_byte + 4)].to_vec();

        // convert to little endian
        control_bytes.reverse();

        // construct the control word
        let mut control_word: u32 = 0;
        for (i, cb) in control_bytes.iter().enumerate() {
            control_word |= (*cb as u32) << (i * 8);
        }

        for (i, _cs) in CONTROL_SIGNALS.iter().enumerate() {
            let cs_val = 2_u32.pow(i as u32);
            if (control_word & cs_val) != 0 {
                found_csignals.push(cs_val);
            }
        }

        if let Some(current_instruction) = current_instruction.as_mut() {
            if ins_signature.0 == current_instruction.name
                && flags == current_instruction.flags
                && instruction_mode == current_instruction.instruction_mode
            {
                current_instruction.steps.push(found_csignals);
                continue;
            } else {
                output.push(current_instruction.clone());
            }
        }

        let new_current_instruction = InstructionDef {
            name: ins_signature.0.to_string(),
            flags,
            instruction_mode,
            steps: Vec::from([found_csignals]),
        };
        current_instruction = Some(new_current_instruction);
    }
    if let Some(current_instruction) = current_instruction {
        output.push(current_instruction);
    }

    Ok(output)
}
