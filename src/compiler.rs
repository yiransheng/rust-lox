use chunk::Chunk;
use common::*;
use scanner::{Scanner, Token, TokenType};
use std::mem;
use value::Value;

pub struct CompileError {
    pub line_no: u64,
    pub payload: CompileErrorPayload,
    pub message: Option<String>,
}
pub enum CompileErrorPayload {
    ScannerError,
    UnexpectedToken(TokenType),
    TooManyConstants,
}

pub type Result<T> = ::std::result::Result<T, CompileError>;

pub fn compile<'a>(source: &'a str, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new(source);

    false
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    PREC_NONE = 0,
    PREC_ASSIGNMENT = 1, // =
    PREC_OR = 2,         // or
    PREC_AND = 3,        // and
    PREC_EQUALITY = 4,   // == !=
    PREC_COMPARISON = 5, // < > <= >=
    PREC_TERM = 6,       // + -
    PREC_FACTOR = 7,     // * /
    PREC_UNARY = 8,      // ! - +
    PREC_CALL = 9,       // . () []
    PREC_PRIMARY = 10,
}

use self::Precedence::*;

struct Parser<'a, 'b> {
    previous: Token<'a>,
    current: Token<'a>,
    scanner: Scanner<'a>,
    chunk: &'b mut Chunk,
}

impl<'a, 'b: 'a> Parser<'a, 'b> {
    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(PREC_ASSIGNMENT)
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume_with_error_message(TokenType::TOKEN_RIGHT_PAREN, "expect ) after expression")
    }
    fn unary(&mut self) -> Result<()> {
        let operatorType = self.previous.ty;

        self.parse_precedence(PREC_UNARY)?;

        match operatorType {
            TokenType::TOKEN_MINUS => {
                self.emit_byte(OP_NEGATE);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        let val: Value = self
            .previous
            .raw
            .parse()
            .expect("Not a number, scanner bugged out");

        self.emit_constant(val);
        Ok(())
    }

    #[inline]
    fn consume(&mut self, ty: TokenType) -> Result<()> {
        self._consume(ty, None)
    }
    #[inline]
    fn consume_with_error_message(&mut self, ty: TokenType, message: &str) -> Result<()> {
        self._consume(ty, Some(message))
    }

    fn _consume(&mut self, ty: TokenType, message: Option<&str>) -> Result<()> {
        if self.current.ty == ty {
            self.advance()
        } else {
            Err(CompileError {
                line_no: self.current.line,
                payload: CompileErrorPayload::UnexpectedToken(self.current.ty),
                message: message.map(|s| s.to_string()),
            })
        }
    }

    fn advance(&mut self) -> Result<()> {
        mem::swap(&mut self.previous, &mut self.current);

        self.current = self.scanner.scan_token();

        if self.current.ty == TokenType::TOKEN_ERROR {
            Err(CompileError {
                line_no: self.current.line,
                payload: CompileErrorPayload::ScannerError,
                message: Some(self.current.raw.to_string()),
            })
        } else {
            Ok(())
        }
    }

    fn emit_byte(&mut self, b: u8) {
        self.chunk.write(b, self.previous.line);
    }
    fn emit_bytes(&mut self, b1: u8, b2: u8) {
        self.emit_byte(b1);
        self.emit_byte(b2);
    }
    fn emit_constant(&mut self, c: Value) {
        // Check constants array limit
        self.chunk.write_constant(c, self.previous.line);
    }
    fn emit_return(&mut self) {
        self.emit_byte(OP_RETURN);
    }
}
