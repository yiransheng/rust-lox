#![allow(non_camel_case_types)]

extern crate arraydeque;
extern crate num;

use std::io;

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
    let stdout = io::stdout();
    let mut chunk = Chunk::new();

    let s = "var y = 22;
    var x = 10; 
    x = y * 3 - 2;
    print x+y;";
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
