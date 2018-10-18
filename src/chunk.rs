extern crate arraydeque;

use std::io::Write;

use common::*;
use value::{ValueOwned, ValueRef};

#[derive(Copy, Clone, Debug)]
struct Line {
    line_no: u64,
    repeat: usize,
}
#[derive(Debug)]
struct Lines {
    lines: Vec<Line>,
}

impl Lines {
    fn new() -> Self {
        Lines { lines: Vec::new() }
    }
    fn push_line(&mut self, line_no: u64) {
        let mut line = self
            .lines
            .pop()
            .unwrap_or_else(|| Line { line_no, repeat: 0 });

        if line_no == line.line_no {
            line.repeat += 1;
            self.lines.push(line);
        } else {
            self.lines.push(line);
            self.lines.push(Line { line_no, repeat: 1 });
        }
    }
    // TODO: binary search
    fn find_line_no(&self, offset: usize) -> u64 {
        let line_acc = Line {
            line_no: 0,
            repeat: 0,
        };

        self.lines
            .iter()
            .scan(line_acc, |acc, ref line| {
                acc.line_no = line.line_no;
                acc.repeat += line.repeat;

                Some(*acc)
            }).skip_while(|line_acc| line_acc.repeat < offset + 1)
            .map(|line_acc| line_acc.line_no)
            .next()
            .unwrap()
    }
}

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<ValueOwned>,
    lines: Lines,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Lines::new(),
        }
    }
    pub fn write(&mut self, byte: u8, line: u64) {
        self.code.push(byte);
        self.lines.push_line(line);
    }
    pub fn write_constant(&mut self, constant: ValueOwned, line: u64) -> usize {
        self.write(OP_CONSTANT, line);
        let constant_offset = self.add_constant(constant);
        self.write(constant_offset as u8, line);
        constant_offset
    }
    pub fn read_byte(&self, offset: usize) -> u8 {
        self.code[offset]
    }
    pub fn read_constant(&self, offset: usize) -> ValueRef {
        let constant_offset = self.read_byte(offset);
        (&self.constants[constant_offset as usize]).into()
    }
    pub fn add_constant(&mut self, value: ValueOwned) -> usize {
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
        let prev_line_no = if offset > 0 {
            self.lines.find_line_no(offset - 1)
        } else {
            0
        };
        let line_no = self.lines.find_line_no(offset);
        if offset > 0 && line_no == prev_line_no {
            write!(write_to, "   | ");
        } else {
            write!(write_to, "{:04} ", line_no);
        }

        let instr = self.code[offset];

        match instr {
            OP_RETURN => Self::disassemble_simple_instruction("OP_RETURN", offset, write_to),
            OP_PRINT => Self::disassemble_simple_instruction("OP_PRINT", offset, write_to),
            OP_POP => Self::disassemble_simple_instruction("OP_POP", offset, write_to),
            OP_DEFINE_GLOBAL => {
                self.disassemble_constant_instruct("OP_DEFINE_GLOBAL", offset, write_to)
            }
            OP_SET_GLOBAL => self.disassemble_constant_instruct("OP_SET_GLOBAL", offset, write_to),
            OP_GET_GLOBAL => self.disassemble_constant_instruct("OP_GET_GLOBAL", offset, write_to),
            OP_NIL => Self::disassemble_simple_instruction("OP_NIL", offset, write_to),
            OP_TRUE => Self::disassemble_simple_instruction("OP_TRUE", offset, write_to),
            OP_FALSE => Self::disassemble_simple_instruction("OP_FALSE", offset, write_to),
            OP_CONSTANT => self.disassemble_constant_instruct("OP_CONSTANT", offset, write_to),
            OP_NEGATE => Self::disassemble_simple_instruction("OP_NEGATE", offset, write_to),
            OP_ADD => Self::disassemble_simple_instruction("OP_ADD", offset, write_to),
            OP_SUBTRACT => Self::disassemble_simple_instruction("OP_SUBTRACT", offset, write_to),
            OP_MULTIPLY => Self::disassemble_simple_instruction("OP_MULTIPLY", offset, write_to),
            OP_DIVIDE => Self::disassemble_simple_instruction("OP_DIVIDE", offset, write_to),
            OP_NOT => Self::disassemble_simple_instruction("OP_NOT", offset, write_to),
            OP_EQUAL => Self::disassemble_simple_instruction("OP_EQUAL", offset, write_to),
            OP_GREATER => Self::disassemble_simple_instruction("OP_GREATER", offset, write_to),
            OP_LESS => Self::disassemble_simple_instruction("OP_LESS", offset, write_to),
            _ => {
                writeln!(write_to, "Unknown OptCode {}", instr);
                offset + 1
            }
        }
    }

    fn disassemble_simple_instruction<W: Write>(
        name: &str,
        offset: usize,
        write_to: &mut W,
    ) -> usize {
        writeln!(write_to, "{}", name);
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
        writeln!(write_to, "{}", self.constants[constant as usize]);

        offset + 2
    }
}
