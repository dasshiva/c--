use std::env;
use std::process;

#[derive(Debug, PartialEq)]
enum Token {
    Invalid,
    Num(i64),
    AddOp, // '+'
    SubOp,
    End
}

enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>)
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

fn get_num(buf: &mut ROBuffer) -> Token {
    let mut n = 0i64;
    loop {
        let ch = buf.next();
        if ch.is_none() {
            // Don't rewind if you reach EOF, otherwise infinite loop
            break;
        }

        if !is_digit(ch.unwrap()) {
            buf.rewind();
            break;
        }

        let digit = ch.unwrap() - b'0';
        n *= 10;
        n += digit as i64;
    }

    Token::Num(n)
    
}

fn tokeniser(buf: &mut ROBuffer) -> Token {
    loop {
        let ch = buf.next();
        if ch.is_none() { 
            break;
        }


        return match ch.unwrap() {
             b'0'..=b'9' => { buf.rewind(); get_num(buf) }
             b' ' | b'\t' | b'\r' |  b'\n' => continue, // skip all spaces
             b'+' => Token::AddOp,
             b'-' => Token::SubOp,
             _    => Token::Invalid
        }
    }

    Token::End
}

/*
 * expr = num |
 *        expr '+' expr |
 *        expr '-' expr
 */
fn parse_expr(expr: String) -> Result<Expr, ()> {
    let mut buf = ROBuffer::new(expr)?;
    loop {
        let t = tokeniser(&mut buf);
        if t != Token::End {
            print!("{:?} ", t);
        }
        else {
            break;
        }
    }
    Err(())
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        println!("Usage: {} [EXPR]", args.nth(0).unwrap());
        process::exit(1);
    }

    let expr = args.nth(1).unwrap();
    println!("Expression = {}", expr);

    parse_expr(expr).unwrap();
}
