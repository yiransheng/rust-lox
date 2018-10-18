use chunk::Chunk;
use common::*;
use object::*;
use scanner::{Scanner, Token, TokenType};
use std::mem;
use value::{Value, ValueOwned};

#[derive(Debug)]
pub struct CompileError {
    pub line_no: u64,
    pub payload: CompileErrorPayload,
    pub message: Option<String>,
}
#[derive(Debug)]
pub enum CompileErrorPayload {
    ScannerError,
    ParserError,
    UnexpectedToken(TokenType),
    TooManyConstants,
}

pub type Result<T> = ::std::result::Result<T, CompileError>;

pub fn compile<'a>(source: &'a str, chunk: &mut Chunk) -> Result<()> {
    let scanner = Scanner::new(source);
    let mut parser = Parser::new(scanner, chunk);

    if parser.match_ty(TokenType::TOKEN_EOF) {
        return Ok(());
    }

    loop {
        parser.declaration()?;
        if parser.match_ty(TokenType::TOKEN_EOF) {
            break;
        }
    }

    parser.consume(TokenType::TOKEN_EOF)?;

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

type ParseFn<'a, 'b> = for<'r> fn(&'r mut Parser<'a, 'b>, bool) -> Result<()>;

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

    fn declaration(&mut self) -> Result<()> {
        if self.match_ty(TokenType::TOKEN_VAR) {
            self.var_declaration()
        } else {
            self.statement()
        }

        // if self.panic_mode {
        //    self.syncronize()
        // }
    }

    fn statement(&mut self) -> Result<()> {
        if self.match_ty(TokenType::TOKEN_PRINT) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn var_declaration(&mut self) -> Result<()> {
        let offset = self.parse_variable("Expect global variable")?;

        if self.match_ty(TokenType::TOKEN_EQUAL) {
            self.expression()?;
        } else {
            self.emit_byte(OP_NIL);
        }

        self.consume_with_error_message(
            TokenType::TOKEN_SEMICOLON,
            "Expect ; after variable declaration",
        )?;

        self.define_variable(offset)
    }

    fn expression_statement(&mut self) -> Result<()> {
        self.expression()?;
        self.emit_byte(OP_POP);

        self.consume_with_error_message(TokenType::TOKEN_SEMICOLON, "Expect ; after expression")
    }

    fn print_statement(&mut self) -> Result<()> {
        self.expression()?;
        self.consume_with_error_message(
            TokenType::TOKEN_SEMICOLON,
            "Expect ; after print statement",
        )?;
        self.emit_byte(OP_PRINT);
        Ok(())
    }

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(PREC_ASSIGNMENT)
    }

    fn grouping(&mut self, _can_assign: bool) -> Result<()> {
        self.expression()?;
        self.consume_with_error_message(TokenType::TOKEN_RIGHT_PAREN, "expect ) after expression")
    }
    fn binary(&mut self, _can_assign: bool) -> Result<()> {
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
            TokenType::TOKEN_BANG_EQUAL => {
                self.emit_bytes(OP_EQUAL, OP_NOT);
            }
            TokenType::TOKEN_EQUAL_EQUAL => {
                self.emit_byte(OP_EQUAL);
            }
            TokenType::TOKEN_GREATER => {
                self.emit_byte(OP_GREATER);
            }
            TokenType::TOKEN_GREATER_EQUAL => {
                self.emit_bytes(OP_LESS, OP_NOT);
            }
            TokenType::TOKEN_LESS => {
                self.emit_byte(OP_LESS);
            }
            TokenType::TOKEN_LESS_EQUAL => {
                self.emit_bytes(OP_GREATER, OP_NOT);
            }
            _ => unreachable!(),
        }

        Ok(())
    }
    fn unary(&mut self, _can_assign: bool) -> Result<()> {
        let op_type = self.previous.ty;

        self.parse_precedence(PREC_UNARY)?;

        match op_type {
            TokenType::TOKEN_MINUS => {
                self.emit_byte(OP_NEGATE);
                Ok(())
            }
            TokenType::TOKEN_BANG => {
                self.emit_byte(OP_NOT);
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        self.advance()?;

        let ParseRule { prefix, .. } = Self::get_rule(self.previous.ty);

        let prefix: ParseFn<'a, 'b> = prefix.ok_or_else(|| self.error("Expect expression"))?;
        let can_assign = precedence <= PREC_ASSIGNMENT;

        prefix(self, can_assign)?;

        while precedence <= Self::get_rule(self.current.ty).precedence {
            self.advance()?;
            let ParseRule { infix, .. } = Self::get_rule(self.previous.ty);
            let infix = infix.unwrap();
            infix(self, can_assign)?
        }

        if can_assign && self.match_ty(TokenType::TOKEN_EQUAL) {
            let _ = self.expression();
            Err(self.error("Invalid assignment target"))
        } else {
            Ok(())
        }
    }
    fn literal(&mut self, _can_assign: bool) -> Result<()> {
        match self.previous.ty {
            TokenType::TOKEN_FALSE => {
                self.emit_byte(OP_FALSE);
                Ok(())
            }
            TokenType::TOKEN_TRUE => {
                self.emit_byte(OP_TRUE);
                Ok(())
            }
            TokenType::TOKEN_NIL => {
                self.emit_byte(OP_NIL);
                Ok(())
            }
            _ => Err(self.error("Unknown Literal")),
        }
    }

    fn number(&mut self, _can_assign: bool) -> Result<()> {
        let val: f64 = self
            .previous
            .raw
            .parse()
            .expect("Not a number, scanner bugged out");

        self.emit_constant(Value::from(val));
        Ok(())
    }

    fn string(&mut self, _can_assign: bool) -> Result<()> {
        let length = self.previous.raw.len();
        // remove open close quotes
        let value = Value::from(self.previous.raw.get(1..length - 1).unwrap());
        self.emit_constant(value);
        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> Result<()> {
        let previous = self.previous;
        self.named_variable(previous, can_assign)
    }

    fn named_variable(&mut self, name: Token<'a>, can_assign: bool) -> Result<()> {
        let identifier: ValueOwned = Value::from(name.raw);
        let offset = self.chunk.add_constant(identifier);

        if can_assign && self.match_ty(TokenType::TOKEN_EQUAL) {
            self.expression()?;
            self.emit_bytes(OP_SET_GLOBAL, offset as u8);
        } else {
            self.emit_bytes(OP_GET_GLOBAL, offset as u8);
        }

        Ok(())
    }

    fn define_variable(&mut self, global: usize) -> Result<()> {
        self.emit_bytes(OP_DEFINE_GLOBAL, global as u8);
        Ok(())
    }

    // result constant for identifier name offset
    fn parse_variable(&mut self, message: &str) -> Result<usize> {
        self.consume_with_error_message(TokenType::TOKEN_IDENTIFIER, message)?;
        let identifier: ValueOwned = Value::from(self.previous.raw);
        let offset = self.chunk.write_constant(identifier, self.previous.line);
        Ok(offset)
    }

    fn check_ty(&self, ty: TokenType) -> bool {
        self.current.ty == ty
    }
    fn match_ty(&mut self, ty: TokenType) -> bool {
        if !self.check_ty(ty) {
            false
        } else {
            self.advance().is_ok()
        }
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
    fn emit_constant(&mut self, c: ValueOwned) {
        // Check constants array limit
        self.chunk.write_constant(c, self.previous.line);
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
                prefix: Some(Parser::unary),
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_BANG_EQUAL => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_EQUALITY,
            },
            TokenType::TOKEN_EQUAL => ParseRule {
                prefix: None,
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_EQUAL_EQUAL => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_EQUALITY,
            },
            TokenType::TOKEN_GREATER => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_GREATER_EQUAL => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_LESS => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_LESS_EQUAL => ParseRule {
                prefix: None,
                infix: Some(Parser::binary),
                precedence: PREC_COMPARISON,
            },
            TokenType::TOKEN_IDENTIFIER => ParseRule {
                prefix: Some(Parser::variable),
                infix: None,
                precedence: PREC_NONE,
            },
            TokenType::TOKEN_STRING => ParseRule {
                prefix: Some(Parser::string),
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
                prefix: Some(Parser::literal),
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
                prefix: Some(Parser::literal),
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
                prefix: Some(Parser::literal),
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
