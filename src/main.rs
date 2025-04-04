use std::env;
use std::process;

mod expr_parser;
mod tokeniser;
mod robuffer;
mod utils;
mod sema;

use crate::tokeniser::{Tokeniser, Token};

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
    let parsed: Result<Vec<Token>, ()> = tokeniser.collect();
    if parsed.is_err() {
        process::exit(1);
    }

    if !sema::sema_infix(parsed.as_ref().unwrap()) {
        process::exit(1);
    }

    let rpn: Vec<Token> = expr_parser::to_rpn(parsed.unwrap());
    if !sema::sema_rpn(&rpn) {
        process::exit(1);
    }

    println!("RPN Expression = {:?}", rpn);
    //code_dump(rpn).unwrap();
}
