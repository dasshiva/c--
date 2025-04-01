use std::env;
use std::process;

#[derive(Debug, PartialEq)]
enum Token {
    Invalid,
    Num(i64),
    AddOp, // '+'
    SubOp, // '-'
    MulOp, // '*'
    DivOp, // '-'
    End
}

#[derive(Debug, Copy, Clone)]
enum Expr {
    Number(i64),
    Add,
    Sub,
    Mul,
    Div
}

impl Expr {
    fn to_string<'a>(&self) -> &'a str {
        match &self {
            Expr::Add => "add",
            Expr::Sub => "sub",
            Expr::Mul => "mul",
            Expr::Div => "div",
            _ => unreachable!()
        }
    }

    fn precedence(&self) -> u8 {
        return match &self {
            Expr::Number(_) => 0,
            Expr::Sub => 1,
            Expr::Add => 2,
            Expr::Div => 3,
            Expr::Mul => 4,
            //_ => unreachable!()
        }
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
             b'+'  => Token::AddOp,
             b'-'  => Token::SubOp,
             b'*'  => Token::MulOp,
             b'/'  => Token::DivOp,
             _     => Token::Invalid
        }
    }

    Token::End
}

fn collect_expr(expr: String) -> Result<Vec<Expr>, ()> {
    let mut buf = ROBuffer::new(expr)?;
    let mut exprs: Vec<Expr> = Vec::new();
    loop {
        let t = tokeniser(&mut buf);
        match t {
            Token::Num(x) => exprs.push(Expr::Number(x)),
            Token::AddOp => exprs.push(Expr::Add),
            Token::SubOp => exprs.push(Expr::Sub),
            Token::MulOp => exprs.push(Expr::Mul),
            Token::DivOp => exprs.push(Expr::Div),
            Token::Invalid => return Err(()),
            Token::End => break
        };
    }

    Ok(exprs)
}

fn add_operator(opstack: &mut Vec<Expr>, output: &mut Vec<Expr>, op: Expr) {
    while opstack.len() > 0 {
        let opstack_top = opstack.pop().unwrap();
        // If top has lower precdence than op, then add op to opstack
        // And return as there is nothing more to do
        if opstack_top.precedence() < op.precedence() {
            opstack.push(opstack_top);
            opstack.push(op);
            return;
        }
        // Else put opstack_op back into output
        else {
            output.push(opstack_top);
        }
        
    }

    // We come here if opstack is empty or exhausted so simply put op back
    // on to opstack for further evaluation
    opstack.push(op);
}

fn to_rpn(expr: Vec<Expr>) -> Vec<Expr> {
    let mut ret: Vec<Expr> = Vec::new();
    let mut opstack: Vec<Expr> = Vec::new();

    // https://en.wikipedia.org/wiki/Shunting_yard_algorithm
    for e in expr {
        match e {
            Expr::Number(_) => ret.push(e),
            _ => add_operator(&mut opstack, &mut ret, e)
        };
    }

    while opstack.len() > 0 {
        ret.push(opstack.pop().unwrap());
    }

    ret
}

use std::io::Write;

fn codegen(expr: Vec<Expr>) -> std::io::Result<()> {
    use std::fs::File;
    let mut file = File::create("compile.ir")?;
    let mut id = 0u32;
    let mut stack: Vec<u32> = Vec::new();
    for e in expr {
        match e {
            Expr::Number(x) => {
                stack.push(id);
                writeln!(&mut file, "%{} = load i64 {}", id, x).unwrap();
                id += 1;
            }

            _ => {
                let e1 = stack.pop().unwrap(); // b
                let e2 = stack.pop().unwrap(); // a
                // Operation format: [OP_NAME] a, b
                writeln!(&mut file, "%{} = {} i64 %{}, %{}", id, e.to_string(), 
                    e1, e2).unwrap();

                // '0' is a placeholder here because we do not care about
                // the actual value in Pair
                stack.push(id);  
                id += 1;
            }
        };
    }

    Ok(())
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        println!("Usage: {} [EXPR]", args.nth(0).unwrap());
        process::exit(1);
    }

    let expr = args.nth(1).unwrap();
    let parsed = collect_expr(expr).unwrap();
    let rpn = to_rpn(parsed);
    codegen(rpn).unwrap();
}
