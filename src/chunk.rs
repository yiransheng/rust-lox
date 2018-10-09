use std::io::Write;

use common::*;
use value::Value;

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    lines: Vec<u64>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }
    pub fn write(&mut self, byte: u8, line: u64) {
        self.code.push(byte);
        self.lines.push(line);
    }
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
    pub fn disassemble<W: Write>(&self, write_to: &mut W) {
        let count = self.code.len();
        let mut i: usize = 0;
        loop {
            if i >= count {
                break;
            }

            i = self.disassemble_instruction(i, write_to);
        }
    }
    fn disassemble_instruction<W: Write>(&self, offset: usize, write_to: &mut W) -> usize {
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(write_to, "   | ");
        } else {
            write!(write_to, "{:04} ", self.lines[offset]);
        }

        let instr = self.code[offset];

        match instr {
            OP_RETURN => Self::disassemble_simple_instruction("OP_RETURN", offset, write_to),
            OP_CONSTANT => self.disassemble_constant_instruct("OP_CONSTANT", offset, write_to),
            _ => {
                write!(write_to, "Unknown OptCode {}", instr);
                offset + 1
            }
        }
    }

    fn disassemble_simple_instruction<W: Write>(
        name: &str,
        offset: usize,
        write_to: &mut W,
    ) -> usize {
        write!(write_to, "{}\n", name);
        offset + 1
    }
    fn disassemble_constant_instruct<W: Write>(
        &self,
        name: &str,
        offset: usize,
        write_to: &mut W,
    ) -> usize {
        let constant: u8 = self.code[offset + 1];

        write!(write_to, "{:<16} {:04} ", name, constant);
        write!(write_to, "{}\n", self.constants[constant as usize]);

        offset + 2
    }
}
