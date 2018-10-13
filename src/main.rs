extern crate arraydeque;
extern crate num;

use std::io;
use std::io::Write;

mod chunk;
mod common;
mod compiler;
mod scanner;
mod value;
mod vm;

use chunk::Chunk;
use common::*;
use compiler::compile;
use scanner::Scanner;
use scanner::TokenType;
use vm::VM;

fn main() {
    let mut stdout = io::stdout();
    let mut chunk = Chunk::new();

    // chunk.write_constant(1.2, 123);
    // chunk.write(OP_NEGATE, 123);
    // chunk.write_constant(1.8, 124);
    // chunk.write(OP_SUBTRACT, 124);
    // chunk.write(OP_RETURN, 125);

    // let mut vm = VM::new(chunk, stdout);
    // vm.disassemble();
    // vm.interpret();

    let s = "-12 * 33 - (4 + 6)";
    let result = compile(s, &mut chunk);

    match result {
        Ok(_) => {
            let mut vm = VM::new(chunk, stdout);
            vm.disassemble();
            vm.interpret();
        }
        Err(e) => println!("{:?}", e),
    }
}
