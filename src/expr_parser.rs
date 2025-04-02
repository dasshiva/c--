use crate::tokeniser::Token;
use crate::tokeniser::TokenKind;

// https://en.wikipedia.org/wiki/Shunting_yard_algorithm
// The below two functions implement the shunting yard algorithm
// as mentioned in the above link
pub fn add_operator(opstack: &mut Vec<Token>, output: &mut Vec<Token>, op: Token) {
    while opstack.len() > 0 {
        let opstack_top = opstack.pop().unwrap();
        // Check for '(', if we have reached one, this is a new scope
        // so popping is no longer allowed
        if *opstack_top.kind() == TokenKind::LPar {
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

pub fn to_rpn(expr: Vec<Token>) -> Vec<Token> {
    let mut ret: Vec<Token> = Vec::new();
    let mut opstack: Vec<Token> = Vec::new();

    'outer: for e in expr {
        match e.kind() {
            TokenKind::Num(_) | TokenKind::Ident(_) => ret.push(e),
            TokenKind::LPar => opstack.push(e),
            TokenKind::RPar => {
                while opstack.len() > 0 {
                    let op = opstack.pop().unwrap();
                    if *op.kind() == TokenKind::LPar {
                        continue 'outer;
                    }

                    ret.push(op);
                }

                panic!("Mismatched parenthesis at line {} column {}", 
                        e.line(), e.col());
            }
            _ => add_operator(&mut opstack, &mut ret, e)
        };
    }

    while opstack.len() > 0 {
        ret.push(opstack.pop().unwrap());
    }

    ret
}

