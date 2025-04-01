use std::env;
use std::process;

#[derive(Debug, PartialEq, Copy, Clone)]
enum TokenKind {
    Invalid(u8),
    Num(i64),
    Add, // '+'
    Sub, // '-'
    Mul, // '*'
    Div, // '-'
    LPar,  // '('
    RPar,  // ')'
    End
}

#[derive(Debug, PartialEq)]
struct Token {
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
            _ => unreachable!()
        }
    }

    fn precedence(&self) -> u8 {
        return match &self.kind() {
            TokenKind::Num(_) => 0,
            TokenKind::Sub => 1,
            TokenKind::Add => 1,
            TokenKind::Div => 3,
            TokenKind::Mul => 3,
            _ => unreachable!()
        }
    }

    fn kind(&self) -> TokenKind {
        self.kind
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

struct Tokeniser {
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

    fn tokenise(&mut self) -> Token {
        loop {
            let ch = self.buf.next();
            if ch.is_none() { 
                break;
            }

            let ret = match ch.unwrap() {
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
                        (x as char), t.line(), t.col());
                    return Err(());
                },

                TokenKind::End => break,
                _ => exprs.push(t)
            }
        }

        Ok(exprs)
    }
}



fn add_operator(opstack: &mut Vec<Token>, output: &mut Vec<Token>, op: Token) {
    while opstack.len() > 0 {
        let opstack_top = opstack.pop().unwrap();
        // Check for '(', if we have reached one, this is a new scope
        // so popping is no longer allowed
        if opstack_top.kind() == TokenKind::LPar {
            opstack.push(opstack_top);
            break;
        }

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

fn to_rpn(expr: Vec<Token>) -> Vec<Token> {
    let mut ret: Vec<Token> = Vec::new();
    let mut opstack: Vec<Token> = Vec::new();

    // https://en.wikipedia.org/wiki/Shunting_yard_algorithm
    'outer: for e in expr {
        match e.kind() {
            TokenKind::Num(_) => ret.push(e),
            TokenKind::LPar => opstack.push(e),
            TokenKind::RPar => {
                while opstack.len() > 0 {
                    let op = opstack.pop().unwrap();
                    if op.kind() == TokenKind::LPar {
                        continue 'outer;
                    }

                    ret.push(op);
                }

                panic!("Mismatched parenthesis");
            }
            _ => add_operator(&mut opstack, &mut ret, e)
        };
    }

    while opstack.len() > 0 {
        ret.push(opstack.pop().unwrap());
    }

    ret
}


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
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        println!("Usage: {} [EXPR]", args.nth(0).unwrap());
        process::exit(1);
    }

    let expr = args.nth(1).unwrap();
    let mut tokeniser = Tokeniser::new(expr);
    let parsed = tokeniser.collect();

    let rpn = to_rpn(parsed.unwrap());
    //println!("RPN Expression = {:?}", rpn);
    code_dump(rpn).unwrap();
}
