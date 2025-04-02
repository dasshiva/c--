use std::env;
use std::process;

mod expr_parser;

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
    End
}

#[derive(Debug, PartialEq)]
pub struct Token {
    kind:   TokenKind, 
    line:   u32,
    column: u32,
}

impl Token {
    fn new(kind: TokenKind, col: u32, ln: u32) -> Self {
        Self {
            kind,
            line: ln,
            column: col
        }
    }

    fn col(&self) -> u32 {
        self.column
    }

    fn line(&self) -> u32 {
        self.line
    }


    fn to_string<'a>(&self) -> &'a str {
        match &self.kind() {
            TokenKind::Add => "add",
            TokenKind::Sub => "sub",
            TokenKind::Mul => "mul",
            TokenKind::Div => "div",
            TokenKind::Mod => "rem",
            TokenKind::And => "and",
            TokenKind::Xor => "xor",
            TokenKind::Or  => "or",
            _ => unreachable!()
        }
    }

    fn precedence(&self) -> u8 {
        return match &self.kind() {
            TokenKind::Num(_) => 0,
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

    fn kind(&self) -> &TokenKind {
        &self.kind
    }
}

#[derive(Debug)]
// Read-only buffer
struct ROBuffer {
    contents: Vec<u8>,
    offset: usize
}

impl ROBuffer {
    fn new(ty: String) -> Result<Self, ()> {
        let mut bytes: Vec<u8> = Vec::new();
        let chars: Vec<char> = ty.chars().collect();
        for ch in chars {
            let b = u32::from(ch);
            if b > 0x7F {
                return Err(())
            }

            bytes.push(b as u8)
        }

        Ok(Self {
            contents: bytes,
            offset: 0
        })
    }

    fn next(&mut self) -> Option<u8> {
        if self.contents.len() <= self.offset {
            return None;
        }

        self.offset += 1;
        Some(self.contents[self.offset - 1])
    }

    fn rewind(&mut self) {
        if self.offset == 0 {
            panic!("Internal error: Attempt to rewind at offset 0");
        }

        self.offset -= 1;
    }
}


fn is_digit(c: u8) -> bool {
    (c >= b'0') && (c <= b'9')
}

fn is_alnum(c: u8) -> bool {
    is_digit(c) || ((c >= b'a') && (c <= b'z')) ||
        ((c >= b'A') && (c <= b'Z'))
}

pub struct Tokeniser {
    buf:    ROBuffer,
    line:   u32,
    column: u32
}

impl Tokeniser {
    fn new(expr: String) -> Self {
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

    fn tokenise(&mut self) -> Token {
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

    fn collect(&mut self) -> Result<Vec<Token>, ()> {
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


/*
use std::io::Write;
fn code_dump(expr: Vec<Token>) -> std::io::Result<()> {
    use std::fs::File;
    let mut file = File::create("compile.ir")?;
    let mut id = 0u32;
    let mut stack: Vec<u32> = Vec::new();
    for e in expr {
        match e.kind() {
            TokenKind::Num(x) => {
                stack.push(id);
                writeln!(&mut file, "%{} = load i64 {}", id, x).unwrap();
                id += 1;
            }

            _ => {
                let e1 = stack.pop().unwrap(); // b
                let e2 = stack.pop().unwrap(); // a
                // Operation format: [OP_NAME] a, b
                writeln!(&mut file, "%{} = {} i64 %{}, %{}", id, e.to_string(), 
                    e2, e1).unwrap();

                stack.push(id);  
                id += 1;
            }
        };
    }

    Ok(())
} */

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        println!("Usage: {} [EXPR]", args.nth(0).unwrap());
        process::exit(1);
    }

    let expr = args.nth(1).unwrap();
    let mut tokeniser = Tokeniser::new(expr);
    let parsed = tokeniser.collect();

    let rpn = expr_parser::to_rpn(parsed.unwrap());
    println!("RPN Expression = {:?}", rpn);
    //code_dump(rpn).unwrap();
}
