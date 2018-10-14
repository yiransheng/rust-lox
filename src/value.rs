use std::convert::{From, Into};
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Nil,
    Number(f64),
    Bool(bool),
}

impl Value {
    pub fn into_number(self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }
    pub fn into_bool(self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }
    pub fn is_falsy(&self) -> bool {
        match self {
            &Value::Nil => true,
            &Value::Bool(false) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Value {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Value {
        Value::Bool(b)
    }
}

impl Neg for Value {
    type Output = Option<Value>;

    fn neg(self) -> Self::Output {
        self.into_number().map(|d| Value::Number(-d))
    }
}
impl Add for Value {
    type Output = Option<Value>;

    fn add(self, other: Value) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a + b))
    }
}
impl Sub for Value {
    type Output = Option<Value>;

    fn sub(self, other: Value) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a - b))
    }
}
impl Mul for Value {
    type Output = Option<Value>;

    fn mul(self, other: Value) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a * b))
    }
}
impl Div for Value {
    type Output = Option<Value>;

    fn div(self, other: Value) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a / b))
    }
}
