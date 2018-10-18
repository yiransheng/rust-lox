use std::borrow::Cow;
use std::convert::{From, Into};
use std::fmt;
use std::mem;
use std::ops::{Add, Div, Mul, Neg, Sub};

use object::{Obj, ObjString};

pub type ValueOwned = Value<Obj>;
pub type ValueRef<'a> = Value<Cow<'a, Obj>>;

#[derive(Clone)]
pub enum Value<O> {
    Nil,
    Number(f64),
    Bool(bool),
    Object(O),
}

impl<'a> From<&'a ValueOwned> for ValueRef<'a> {
    fn from(v: &'a ValueOwned) -> ValueRef<'a> {
        match v {
            Value::Nil => Value::Nil,
            Value::Number(ref n) => Value::Number(*n),
            Value::Bool(ref n) => Value::Bool(*n),
            Value::Object(ref o) => Value::Object(Cow::Borrowed(o)),
        }
    }
}

impl<O> PartialEq for Value<O> {
    fn eq(&self, other: &Value<O>) -> bool {
        match (self, other) {
            (&Value::Nil, &Value::Nil) => true,
            (&Value::Number(a), &Value::Number(b)) => a == b,
            (&Value::Bool(a), &Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl<O> Value<O> {
    pub fn take(&mut self) -> Value<O> {
        mem::replace(self, Value::Nil)
    }
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
            Value::Nil => true,
            Value::Bool(false) => true,
            _ => false,
        }
    }
}

impl<O: fmt::Display> fmt::Display for Value<O> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Object(ref o) => write!(f, "{}", o),
        }
    }
}

impl<O> From<f64> for Value<O> {
    fn from(n: f64) -> Value<O> {
        Value::Number(n)
    }
}

impl<O> From<bool> for Value<O> {
    fn from(b: bool) -> Value<O> {
        Value::Bool(b)
    }
}
impl<'a> From<&'a str> for ValueOwned {
    fn from(s: &'a str) -> ValueOwned {
        let obj_s = ObjString::new(s);
        Value::Object(Obj::String(Box::new(obj_s)))
    }
}

impl<O> Neg for Value<O> {
    type Output = Option<Value<O>>;

    fn neg(self) -> Self::Output {
        self.into_number().map(|d| Value::Number(-d))
    }
}
impl<O> Add for Value<O> {
    type Output = Option<Value<O>>;

    fn add(self, other: Value<O>) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a + b))
    }
}
impl<O> Sub for Value<O> {
    type Output = Option<Value<O>>;

    fn sub(self, other: Value<O>) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a - b))
    }
}
impl<O> Mul for Value<O> {
    type Output = Option<Value<O>>;

    fn mul(self, other: Value<O>) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a * b))
    }
}
impl<O> Div for Value<O> {
    type Output = Option<Value<O>>;

    fn div(self, other: Value<O>) -> Self::Output {
        let a: f64 = self.into_number()?;
        let b: f64 = other.into_number()?;

        Some(Value::Number(a / b))
    }
}
