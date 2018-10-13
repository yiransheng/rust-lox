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
    ParserError,
    UnexpectedToken(TokenType),
    TooManyConstants,
}

pub type Result<T> = ::std::result::Result<T, CompileError>;

pub fn compile<'a>(source: &'a str, chunk: &mut Chunk) -> Result<()> {
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner, chunk);

    parser.expression()?;
    parser.consume(TokenType::TOKEN_EOF)?;
    parser.emit_return();

    Ok(())
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
impl Precedence {
    fn higher(self) -> Self {
        match self {
            PREC_NONE => PREC_ASSIGNMENT,
            PREC_ASSIGNMENT => PREC_OR,
            PREC_OR => PREC_AND,
            PREC_AND => PREC_EQUALITY,
            PREC_EQUALITY => PREC_COMPARISON,
            PREC_COMPARISON => PREC_TERM,
            PREC_TERM => PREC_FACTOR,
            PREC_FACTOR => PREC_UNARY,
            PREC_UNARY => PREC_CALL,
            PREC_CALL => PREC_PRIMARY,
            PREC_PRIMARY => self,
        }
    }
}

use self::Precedence::*;

type ParseFn<'a, 'b> = for<'r> fn(&'r mut Parser<'a, 'b>) -> Result<()>;

struct ParseRule<'a, 'b> {
    prefix: Option<ParseFn<'a, 'b>>,
    infix: Option<ParseFn<'a, 'b>>,
    precedence: Precedence,
}

pub struct Parser<'a, 'b> {
    previous: Token<'a>,
    current: Token<'a>,
    scanner: Scanner<'a>,
    chunk: &'b mut Chunk,
}

impl<'a, 'b: 'a> Parser<'a, 'b> {
    pub fn new(mut scanner: Scanner<'a>, chunk: &'b mut Chunk) -> Self {
        let previous = Token {
            ty: TokenType::TOKEN_EOF,
            raw: "",
            line: 0,
        };
        let current = scanner.scan_token();
        Parser {
            previous,
            current,
            scanner,
            chunk,
        }
    }
    pub fn expression(&mut self) -> Result<()> {
        self.parse_precedence(PREC_ASSIGNMENT)
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume_with_error_message(TokenType::TOKEN_RIGHT_PAREN, "expect ) after expression")
    }
    fn binary(&mut self) -> Result<()> {
        let op_type = self.previous.ty;
        let rule = Self::get_rule(op_type);

        self.parse_precedence(rule.precedence.higher())?;

        match op_type {
            TokenType::TOKEN_PLUS => {
                self.emit_byte(OP_ADD);
            }
            TokenType::TOKEN_MINUS => {
                self.emit_byte(OP_SUBTRACT);
            }
            TokenType::TOKEN_STAR => {
                self.emit_byte(OP_MULTIPLY);
            }
            TokenType::TOKEN_SLASH => {
                self.emit_byte(OP_DIVIDE);
            }
            _ => unreachable!(),
        }

        Ok(())
    }
    fn unary(&mut self) -> Result<()> {
        let op_type = self.previous.ty;

        self.parse_precedence(PREC_UNARY)?;

        match op_type {
            TokenType::TOKEN_MINUS => {
                self.emit_byte(OP_NEGATE);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.advance()?;

        let ParseRule { prefix, .. } = Self::get_rule(self.previous.ty);

        let prefix: ParseFn<'a, 'b> = prefix.ok_or(self.error("Expect expression"))?;

        prefix(self)?;

        while precedence <= Self::get_rule(self.current.ty).precedence {
            self.advance()?;
            let ParseRule { infix, .. } = Self::get_rule(self.previous.ty);
            let infix = infix.unwrap();
            infix(self)?
        }

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

    #[inline]
    fn error(&mut self, message: &str) -> CompileError {
        Self::error_at(&self.previous, message)
    }
    #[inline]
    fn error_at_current(&mut self, message: &str) -> CompileError {
        Self::error_at(&self.current, message)
    }

    fn error_at(token: &Token<'a>, message: &str) -> CompileError {
        CompileError {
            line_no: token.line,
            payload: CompileErrorPayload::ParserError,
            message: Some(message.to_string()),
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

    fn get_rule(ty: TokenType) -> ParseRule<'a, 'b> {
        match ty {
            TokenType::TOKEN_LEFT_PAREN => ParseRule {
                prefix: Some(Parser::grouping),
                infix: None,
                precedence: PREC_CALL,
            },
            TokenType::TOKEN_RIGHT_PAREN => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_LEFT_BRACE => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_RIGHT_BRACE => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_COMMA => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_DOT => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_CALL,
            },
            TokenType::TOKEN_MINUS => ParseRule {
                prefix: Some(Parser::unary),
                infix: Some(Parser::binary),
                precedence: PREC_TERM,
            },
            TokenType::TOKEN_PLUS => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_TERM,
            },
            TokenType::TOKEN_SEMICOLON => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_SLASH => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_FACTOR,
            },
            TokenType::TOKEN_STAR => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_FACTOR,
            },
            TokenType::TOKEN_BANG => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_BANG_EQUAL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_EQUALITY,
            },
            TokenType::TOKEN_EQUAL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_EQUAL_EQUAL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_EQUALITY,
            },
            TokenType::TOKEN_GREATER => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_GREATER_EQUAL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_LESS => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_LESS_EQUAL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_IDENTIFIER => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_STRING => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_NUMBER => ParseRule {
                prefix: Some(Parser::number),
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_AND => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_AND,
            },
            TokenType::TOKEN_CLASS => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_ELSE => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_FALSE => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_FUN => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_FOR => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_IF => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_NIL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_OR => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_OR,
            },
            TokenType::TOKEN_PRINT => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_RETURN => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_SUPER => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_THIS => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_TRUE => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_VAR => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_WHILE => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_ERROR => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_EOF => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
        }
    }
}
