extern crate arraydeque;

use std::io;
use std::io::Write;

mod chunk;
mod common;
mod value;
mod vm;

use chunk::Chunk;
use common::*;
use vm::VM;

fn main() {
    let mut stdout = io::stdout();
    let mut chunk = Chunk::new();

    chunk.write_constant(1.2, 123);
    chunk.write(OP_NEGATE, 123);
    chunk.write_constant(1.8, 124);
    chunk.write(OP_SUBTRACT, 124);
    chunk.write(OP_RETURN, 125);

    let mut vm = VM::new(chunk, stdout);
    vm.disassemble();
    vm.interpret();

    // chunk.disassemble(&mut stdout);
}
