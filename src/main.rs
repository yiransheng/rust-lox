use std::io;

mod chunk;
mod common;
mod value;

use chunk::Chunk;
use common::*;

fn main() {
    let mut stdout = io::stdout();
    let mut chunk = Chunk::new();

    chunk.write(OP_CONSTANT, 123);
    let offset = chunk.add_constant(1.2);
    chunk.write(offset as u8, 123);
    chunk.write(OP_RETURN as u8, 123);

    chunk.disassemble(&mut stdout);
}
