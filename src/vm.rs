use std::io::Write;
use std::result;

use arraydeque::ArrayDeque;

use chunk::Chunk;
use common::*;
use value::Value;

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub type Result<T> = result::Result<T, InterpretError>;

pub struct VM<W> {
    chunk: Chunk,
    ip: usize,
    stack: ArrayDeque<[Value; 256]>,
    output: W,
}
impl<W> VM<W> {
    pub fn new(chunk: Chunk, output: W) -> Self {
        VM {
            chunk,
            output,
            ip: 0,
            stack: ArrayDeque::new(),
        }
    }
}

impl<W: Write> VM<W> {
    pub fn disassemble(&mut self) {
        self.chunk.disassemble(&mut self.output);
    }
    pub fn interpret(&mut self) -> Result<()> {
        self.run()
    }
    fn run(&mut self) -> Result<()> {
        loop {
            let instr: u8 = self.read_byte();
            match instr {
                OP_RETURN => {
                    self.pop_value().map(|v| self.print_value(v));
                    return Ok(());
                }
                OP_NIL => {
                    self.push_value(Value::Nil);
                }
                OP_TRUE => {
                    self.push_value(Value::from(true));
                }
                OP_FALSE => {
                    self.push_value(Value::from(false));
                }
                OP_CONSTANT => {
                    let constant = self.read_constant();
                    self.push_value(constant);
                }
                OP_NEGATE => {
                    let value = self.pop_value().ok_or(InterpretError::RuntimeError)?;
                    let neg_value = (-value).ok_or(InterpretError::RuntimeError)?;
                    self.push_value(neg_value);
                }
                OP_NOT => {
                    let value = self.pop_value().ok_or(InterpretError::RuntimeError)?;
                    let value = Value::from(value.is_falsy());
                    self.push_value(value);
                }
                OP_EQUAL => {
                    if let Some(value) = self.binary_op(|a, b| Some(Value::from(a == b))) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OP_GREATER => {
                    if let Some(value) = self.binary_op(|a, b| {
                        let a = a.into_number()?;
                        let b = b.into_number()?;

                        Some(Value::from(a > b))
                    }) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OP_LESS => {
                    if let Some(value) = self.binary_op(|a, b| {
                        let a = a.into_number()?;
                        let b = b.into_number()?;

                        Some(Value::from(a < b))
                    }) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OP_ADD => {
                    if let Some(value) = self.binary_op(|a, b| a + b) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OP_SUBTRACT => {
                    if let Some(value) = self.binary_op(|a, b| a - b) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OP_MULTIPLY => {
                    if let Some(value) = self.binary_op(|a, b| a * b) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                OP_DIVIDE => {
                    if let Some(value) = self.binary_op(|a, b| a / b) {
                        self.push_value(value);
                    } else {
                        return Err(InterpretError::RuntimeError);
                    }
                }
                _ => return Err(InterpretError::CompileError),
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.read_byte(self.ip);

        self.ip += 1;

        byte
    }
    fn read_constant(&mut self) -> Value {
        let constant = self.chunk.read_constant(self.ip);

        self.ip += 1;

        constant
    }

    fn push_value(&mut self, v: Value) {
        // panic on overflow
        self.stack.push_back(v).unwrap();
    }
    fn pop_value(&mut self) -> Option<Value> {
        self.stack.pop_back()
    }
    fn peek(&self, distance: usize) -> Option<&Value> {
        let n = self.stack.len();
        let index = n - 1 - distance;

        self.stack.get(index)
    }

    fn print_value(&mut self, v: Value) {
        write!(self.output, "{}\n", v);
    }

    fn binary_op<F: Fn(Value, Value) -> Option<Value>>(&mut self, f: F) -> Option<Value> {
        let a = self.pop_value()?;
        let b = self.pop_value()?;

        f(b, a)
    }
}
