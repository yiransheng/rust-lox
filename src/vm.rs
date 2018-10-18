use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;
use std::ops::Deref;
use std::result;

use arraydeque::ArrayDeque;

use chunk::Chunk;
use common::*;
use object::{Obj, ObjString};
use value::{Value, ValueRef};

pub enum InterpretError {
    CompileError,
    RuntimeError,
}

pub type Result<T> = result::Result<T, InterpretError>;

pub struct VM<'a, W> {
    chunk: &'a Chunk,
    ip: usize,
    stack: ArrayDeque<[ValueRef<'a>; 256]>,
    globals: HashMap<&'a str, ValueRef<'a>>,
    output: W,
}
impl<'a, W> VM<'a, W> {
    pub fn new(chunk: &'a Chunk, output: W) -> Self {
        VM {
            chunk,
            output,
            ip: 0,
            globals: HashMap::new(),
            stack: ArrayDeque::new(),
        }
    }
}

impl<'a, W: Write> VM<'a, W> {
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
                OP_PRINT => {
                    if let Some(v) = self.pop_value() {
                        self.print_value(&v);
                    }
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
                OP_POP => {
                    let _ = self.pop_value().ok_or(InterpretError::RuntimeError)?;
                }
                OP_GET_GLOBAL => {
                    let name = self.read_string().ok_or(InterpretError::RuntimeError)?;
                    let mut value;
                    {
                        let v = self.globals.get(name).ok_or(InterpretError::RuntimeError)?;
                        value = v.clone();
                    }
                    self.push_value(value);
                }
                OP_DEFINE_GLOBAL => {
                    let name = self.read_string().ok_or(InterpretError::RuntimeError)?;
                    let value = self.peek(0).ok_or(InterpretError::RuntimeError)?;

                    self.globals.insert(name, value);
                    let _ = self.pop_value().ok_or(InterpretError::RuntimeError)?;
                }
                OP_SET_GLOBAL => {
                    let name = self.read_string().ok_or(InterpretError::RuntimeError)?;
                    let value = self.peek(0).ok_or(InterpretError::RuntimeError)?;
                    // is new key
                    if self.globals.insert(name, value).is_none() {
                        return Err(InterpretError::RuntimeError);
                    }
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
                    let b = self.peek(0).ok_or(InterpretError::RuntimeError)?;
                    let a = self.peek(1).ok_or(InterpretError::RuntimeError)?;
                    match (b, a) {
                        (Value::Number(_), Value::Number(_)) => {
                            let value = self
                                .binary_op(|a, b| a + b)
                                .ok_or(InterpretError::RuntimeError)?;
                            self.push_value(value);
                        }
                        (Value::Object(b), Value::Object(a)) => match (&*b, &*a) {
                            (Obj::String(ref b), Obj::String(ref a)) => {
                                let mut s: String = (&**a).deref().to_owned();
                                s.push_str(b);
                                let obj_s = ObjString::new(s);
                                let value = Value::Object(Cow::Owned(Obj::String(Box::new(obj_s))));

                                self.push_value(value);
                            }
                            _ => return Err(InterpretError::RuntimeError),
                        },
                        _ => return Err(InterpretError::RuntimeError),
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
    fn read_constant(&mut self) -> ValueRef<'a> {
        let constant = self.chunk.read_constant(self.ip);

        self.ip += 1;

        constant
    }
    fn read_string(&mut self) -> Option<&'a str> {
        let value: ValueRef<'a> = self.read_constant();
        match value {
            Value::Object(Cow::Borrowed(o)) => match o {
                Obj::String(o) => Some((&**o).deref()),
                _ => None,
            },
            _ => None,
        }
    }

    fn push_value(&mut self, v: ValueRef<'a>) {
        // panic on overflow
        self.stack.push_back(v).unwrap();
    }
    fn pop_value(&mut self) -> Option<ValueRef<'a>> {
        self.stack.pop_back()
    }
    fn peek(&self, distance: usize) -> Option<ValueRef<'a>> {
        let n = self.stack.len();
        let index = n - 1 - distance;

        self.stack.get(index).cloned()
    }

    fn print_value(&mut self, v: &ValueRef) {
        writeln!(self.output, "{}", v);
    }

    fn binary_op<F>(&mut self, f: F) -> Option<ValueRef<'a>>
    where
        F: for<'b> Fn(ValueRef<'b>, ValueRef<'b>) -> Option<ValueRef<'b>>,
    {
        let a = self.pop_value()?;
        let b = self.pop_value()?;

        f(b, a)
    }
}
