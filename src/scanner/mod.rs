mod keyword;
mod token;

pub use self::token::*;

use self::keyword::match_keyword;

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u64,
}

impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Self {
        Scanner {
            source: s,
            start: 0,
            current: 0,
            line: 0,
        }
    }
    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whites();

        self.start = self.current;

        if self.is_at_end() {
            return self.mk_token(TokenType::TOKEN_EOF);
        }

        self.advance()
            .map(|c| match c {
                '(' => self.mk_token(TokenType::TOKEN_LEFT_PAREN),
                ')' => self.mk_token(TokenType::TOKEN_RIGHT_PAREN),
                '{' => self.mk_token(TokenType::TOKEN_LEFT_BRACE),
                '}' => self.mk_token(TokenType::TOKEN_RIGHT_BRACE),
                ';' => self.mk_token(TokenType::TOKEN_SEMICOLON),
                ',' => self.mk_token(TokenType::TOKEN_COMMA),
                '.' => self.mk_token(TokenType::TOKEN_DOT),
                '-' => self.mk_token(TokenType::TOKEN_MINUS),
                '+' => self.mk_token(TokenType::TOKEN_PLUS),
                '/' => self.mk_token(TokenType::TOKEN_SLASH),
                '*' => self.mk_token(TokenType::TOKEN_STAR),
                '!' => {
                    let match_equal = self.match_advance('=');
                    self.mk_token(if match_equal {
                        TokenType::TOKEN_BANG_EQUAL
                    } else {
                        TokenType::TOKEN_BANG
                    })
                }
                '=' => {
                    let match_equal = self.match_advance('=');
                    self.mk_token(if match_equal {
                        TokenType::TOKEN_EQUAL_EQUAL
                    } else {
                        TokenType::TOKEN_EQUAL
                    })
                }
                '<' => {
                    let match_equal = self.match_advance('=');
                    self.mk_token(if match_equal {
                        TokenType::TOKEN_LESS_EQUAL
                    } else {
                        TokenType::TOKEN_LESS
                    })
                }
                '>' => {
                    let match_equal = self.match_advance('=');
                    self.mk_token(if match_equal {
                        TokenType::TOKEN_GREATER_EQUAL
                    } else {
                        TokenType::TOKEN_GREATER
                    })
                }
                '"' => self
                    .scan_string()
                    .unwrap_or_else(|| self.mk_error_token("Invalid string literal")),
                c => {
                    if c.is_digit(10) {
                        self.scan_number()
                    } else if c.is_alphabetic() {
                        self.scan_identifier()
                            .unwrap_or_else(|| self.mk_error_token("Invalid identifier"))
                    } else {
                        unreachable!()
                    }
                }
            }).unwrap_or_else(|| self.mk_error_token("Unexpected EOF"))
    }
    fn skip_whites(&mut self) {
        loop {
            match self.peek() {
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                Some('/') => {
                    self.skip_line_comment();
                    return;
                }
                Some(c) => {
                    if c.is_whitespace() {
                        self.advance();
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }
    fn scan_string(&mut self) -> Option<Token<'a>> {
        loop {
            let c = self.peek()?;
            if c == '\n' {
                self.line += 1;
            }
            if c == '"' {
                // The closing "
                self.advance();
                return Some(self.mk_token(TokenType::TOKEN_STRING));
            }
            self.advance();
        }
    }
    #[inline]
    fn skip_line_comment(&mut self) {
        if self.peek_next() != Some('/') {
            return;
        }
        loop {
            if self.is_at_end() {
                return;
            }
            match self.peek() {
                Some('\n') => return,
                None => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
    fn scan_number(&mut self) -> Token<'a> {
        self.consume_while(|c| c.is_digit(10));

        let d = self.peek();
        // consume dot
        if d == Some('.') {
            self.advance();
            self.consume_while(|c| c.is_digit(10));
        }

        self.mk_token(TokenType::TOKEN_NUMBER)
    }
    fn scan_identifier(&mut self) -> Option<Token<'a>> {
        self.consume_while(|c| c.is_alphanumeric());

        let ident = self.source.get(self.start..self.current)?;
        let kw_len = match_keyword(ident.chars());

        let ty = if kw_len == Some(self.current - self.start) {
            // exact match
            match ident {
                "and" => TokenType::TOKEN_AND,
                "class" => TokenType::TOKEN_CLASS,
                "else" => TokenType::TOKEN_ELSE,
                "false" => TokenType::TOKEN_FALSE,
                "fun" => TokenType::TOKEN_FUN,
                "for" => TokenType::TOKEN_FOR,
                "if" => TokenType::TOKEN_IF,
                "nil" => TokenType::TOKEN_NIL,
                "or" => TokenType::TOKEN_OR,
                "print" => TokenType::TOKEN_PRINT,
                "return" => TokenType::TOKEN_RETURN,
                "super" => TokenType::TOKEN_SUPER,
                "this" => TokenType::TOKEN_THIS,
                "true" => TokenType::TOKEN_TRUE,
                "var" => TokenType::TOKEN_VAR,
                "while" => TokenType::TOKEN_WHILE,
                _ => unreachable!(),
            }
        } else {
            TokenType::TOKEN_IDENTIFIER
        };

        Some(self.mk_token(ty))
    }
    fn consume_while<F>(&mut self, f: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(c) = self.peek() {
            if f(c) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    fn mk_error_token(&self, msg: &'static str) -> Token<'a> {
        Token::new(TokenType::TOKEN_ERROR, msg, self.line)
    }
    fn mk_token(&self, ty: TokenType) -> Token<'a> {
        let raw = self.source.get(self.start..self.current);

        if let Some(raw) = raw {
            Token::new(ty, raw, self.line)
        } else {
            self.mk_error_token("Malformed utf8")
        }
    }

    #[inline]
    fn peek(&self) -> Option<char> {
        let remaining = self.source.get(self.current..)?;
        remaining.chars().next()
    }
    #[inline]
    fn peek_next(&self) -> Option<char> {
        let remaining = self.source.get(self.current..)?;
        let mut iter = remaining.chars();
        iter.next();
        iter.next()
    }
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;

        self.current += c.len_utf8();

        Some(c)
    }
    fn match_advance(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let matched = self.peek().map_or(false, |c| c == expected);

        if matched {
            self.advance();
        }

        matched
    }
}
