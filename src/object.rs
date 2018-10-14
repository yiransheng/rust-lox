use std::convert::Into;
use std::fmt;
use std::ops::Deref;

#[derive(Clone)]
pub enum Obj {
    Function,
    String(Box<ObjString>),
}
impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Obj::Function => write!(f, "[function]"),
            Obj::String(ref s) => s.fmt(f),
        }
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct ObjString {
    inner: String,
}
impl fmt::Display for ObjString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        self.inner.fmt(f)?;
        write!(f, "\"")
    }
}

impl ObjString {
    pub fn new<S: Into<String>>(s: S) -> Self {
        let inner = s.into();
        ObjString { inner }
    }
}

impl Deref for ObjString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
