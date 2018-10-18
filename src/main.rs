extern crate arraydeque;
extern crate num;

use std::io;
use std::io::Write;

mod chunk;
mod common;
mod compiler;
mod object;
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

    let s = "var y=22; var x = y + 3; print x;";
    let result = compile(s, &mut chunk);

    match result {
        Ok(_) => {
            let mut vm = VM::new(&chunk, stdout);
            vm.disassemble();
            println!("\n == Program Output == \n");
            vm.interpret();
        }
        Err(e) => println!("{:?}", e),
    }
}
