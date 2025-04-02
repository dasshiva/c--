use crate::robuffer::ROBuffer;
use crate::utils::{is_digit, is_alnum};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Invalid(u8),
    Num(i64),
    Ident(Vec<u8>),
    Add, // '+'
    Sub, // '-'
    Mul, // '*'
    Div, // '-'
    LPar,  // '('
    RPar,  // ')'
    Mod, // '%'
    And, // '&'
    Xor, // '^'
    Or, // '|'
    Assign, // '='
    End
}

#[derive(Debug, PartialEq)]
pub struct Token {
    kind:   TokenKind, 
    line:   u32,
    column: u32,
}

impl Token {
    pub fn new(kind: TokenKind, col: u32, ln: u32) -> Self {
        Self {
            kind,
            line: ln,
            column: col
        }
    }

    pub fn col(&self) -> u32 {
        self.column
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn is_operand(&self) -> bool {
        match self.kind {
            TokenKind::Num(_) | TokenKind::Ident(_) => true,
            _ => false
        }
    }

    pub fn is_paren(&self) -> bool {
        match self.kind {
            TokenKind::LPar | TokenKind::RPar => true,
            _ => false
        }
    }

    pub fn is_operator(&self) -> bool {
        match self.kind {
            TokenKind::Add | TokenKind::Sub | TokenKind::Mul |
            TokenKind::Div | TokenKind::And | TokenKind::Mod |
            TokenKind::Xor | TokenKind::Or  | TokenKind::Assign => true,
            _ => false
        }
    }

    pub fn value(&self) -> String {
        match &self.kind {
            TokenKind::Num(int) => int.to_string(),
            TokenKind::Ident(name) => String::from_utf8(name.clone()).unwrap(),
            _ => unreachable!()
        }
    }


    pub fn to_string<'a>(&self) -> &'a str {
        match &self.kind() {
            TokenKind::Add => "+",
            TokenKind::Sub => "-",
            TokenKind::Mul => "*",
            TokenKind::Div => "/",
            TokenKind::Mod => "%",
            TokenKind::And => "&",
            TokenKind::Xor => "^",
            TokenKind::Or  => "|",
            TokenKind::Assign => "=",
            _ => unreachable!()
        }
    }

    pub fn precedence(&self) -> i8 {
        return match &self.kind() {
            TokenKind::Assign => -1, // Lowest possible priority
            TokenKind::Num(_) | TokenKind::Ident(_) => 0,
            TokenKind::Or  => 1,
            TokenKind::Xor => 2,
            TokenKind::And => 3,
            TokenKind::Sub => 5,
            TokenKind::Add => 5,
            TokenKind::Div => 6,
            TokenKind::Mul => 6,
            TokenKind::Mod => 6,
            _ => unreachable!()
        }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

pub struct Tokeniser {
    buf:    ROBuffer,
    line:   u32,
    column: u32
}

impl Tokeniser {
    pub fn new(expr: String) -> Self {
        Self {
            buf: ROBuffer::new(expr).unwrap(),
            line: 1,
            column: 1
        }
    }

    fn get_num(&mut self) -> Token {
        let mut n = 0i64;
        let saved_col = self.column;
        let saved_row = self.line;
        loop {
            let ch = self.buf.next();
            // Don't rewind if you reach EOF, it will cause an infinite loop
            if ch.is_none() {
                break;
            }

            if !is_digit(ch.unwrap()) {
                self.column -= 1;
                self.buf.rewind();
                break;
            }

            let digit = ch.unwrap() - b'0';
            n *= 10;
            n += digit as i64;
            self.column += 1;
        }

        Token::new(TokenKind::Num(n), saved_col, saved_row)
    }

    fn get_ident(&mut self) -> Token {
        let mut bvec: Vec<u8> = Vec::new();
        let saved_col = self.column;
        let saved_row = self.line;
        loop {
            let ch = self.buf.next();
            if ch.is_none() {
                break;
            }

            if !is_alnum(ch.unwrap()) {
                self.column -= 1;
                self.buf.rewind();
                break;
            }

            bvec.push(ch.unwrap());
            self.column += 1;
        }

        Token::new(TokenKind::Ident(bvec), saved_col, saved_row)
    }

    pub fn tokenise(&mut self) -> Token {
        loop {
            let ch = self.buf.next();
            if ch.is_none() { 
                break;
            }

            let ret = match ch.unwrap() {
                b'a'..=b'z' | b'A'..=b'Z' => {
                    self.buf.rewind();
                    self.get_ident()
                }

                b'0'..=b'9' => {
                    self.buf.rewind(); 
                    self.get_num() 
                }

                b' ' | b'\t' | b'\r' => {
                    self.column += 1;
                    continue;
                }

                b'\n' => {
                    self.column = 1;
                    self.line += 1;
                    continue;
                }

                b'='  => Token::new(TokenKind::Assign, self.column, self.line),
                b'+'  => Token::new(TokenKind::Add, self.column, self.line),
                b'-'  => Token::new(TokenKind::Sub, self.column, self.line),
                b'*'  => Token::new(TokenKind::Mul, self.column, self.line),
                b'/'  => Token::new(TokenKind::Div, self.column, self.line),
                b'('  => Token::new(TokenKind::LPar, self.column, self.line),
                b')'  => Token::new(TokenKind::RPar, self.column, self.line),
                b'%'  => Token::new(TokenKind::Mod, self.column, self.line),
                b'|'  => Token::new(TokenKind::Or, self.column, self.line),
                b'^'  => Token::new(TokenKind::Xor, self.column, self.line),
                b'&'  => Token::new(TokenKind::And, self.column, self.line),
                _     => Token::new(TokenKind::Invalid(ch.unwrap()),
                                    self.column, self.line)
            };

            self.column += 1;

            return ret;
        }

        Token::new(TokenKind::End, self.column, self.line)
    }

    pub fn collect(&mut self) -> Result<Vec<Token>, ()> {
        let mut exprs: Vec<Token> = Vec::new();
        loop {
            let t = self.tokenise();
            match t.kind() {
                TokenKind::Invalid(x) => {
                    println!("Invalid token {} at line {} column {}",
                        (*x as char), t.line(), t.col());
                    return Err(());
                },

                TokenKind::End => break,
                _ => exprs.push(t)
            }
        }

        Ok(exprs)
    }
}

