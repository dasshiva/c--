use crate::tokeniser::{Token, TokenKind};

/* Infix expressions are validated against the following rules
 * 1) No two operands (Num or Ident) should be right beside each other 
 *  eg. a) 9 9 + b) 1 + 2 3 - c) 6 7 + 8 9 are all invalid infix expressions
 * 2) No two operators must be right beside each other
 *  eg. a) 1 + - 9 b) 2 - - 9 c) 1 * & 8 are all invalid infix expressions
 * 3) Number of left brackets == Number of right brackets
 *  eg. a) 1 + 3 (( 8 + 9) b) ((((1 + 2)) are all invalid infix expressions
 *  Note: The actual matching of left to right brackets is left to the 
 *  expression parser which can easily figure this out
 */
pub fn sema_infix(expr: &Vec<Token>) -> bool {
    // Rule 3
    let mut lpars = 0;
    let mut rpars = 0;
    let mut lastpar: &Token = &Token::new(TokenKind::End, 0, 0);
    for e in expr {
        if *e.kind() == TokenKind::LPar {
            lastpar = e;
            lpars += 1;
        }

        else if *e.kind() == TokenKind::RPar {
            lastpar = e;
            rpars += 1;
        }
    }

    if lpars != rpars {
        let ch: char;

        if lpars > rpars { 
            ch = '(';
        }
        else {
            ch = ')';
        }

        println!("Extra parenthesis {} found at line {} column {}", 
                ch, lastpar.line(), lastpar.col());
        return false;
    }

    // Rules 1 and 2
    let mut idx = 0;
    while idx < expr.len() {
        let atom = expr.get(idx).unwrap(); // This is always a valid index
        if atom.is_operand() {
            let next_atom = expr.get(idx + 1);
            if next_atom.is_none() { // last element in expression
                break;
            }

            let next_atom_uw = next_atom.unwrap();
            if next_atom_uw.is_operand() || next_atom_uw.is_paren() {
                println!("Expected operator after '{}' at line {} column {}",
                        atom.value(), atom.line(), atom.col());
                return false;
            }
        }

        else if atom.is_operator() {
            let next_atom = expr.get(idx + 1);
            if next_atom.is_none() { // last element in expression
                println!("Expected operand after '{}' at line {} column {}",
                    atom.to_string(), atom.line(), atom.col());
                return false;
            }

            let next_atom_uw = next_atom.unwrap();
            if next_atom_uw.is_operator() {
                println!("Expected operand after '{}' at line {} column {}",
                        atom.to_string(), atom.line(), atom.col());
                return false;
            }
        }

        idx += 1;
    }

    true
}

/* Runs expr in a "virtual type system" that is, emulates the running of
 * the expression by using the arity of the operators, placing dummy 
 * values on a "virtual stack" (vstack) as results of expressions in order to
 * figure out if the RPN is correct.
 * A RPN is said to be correct in context of the semantic analyser
 * when:
 * 1) After expr is executed one and only one value is left on the vstack
 * 2) vstack does not underflow at any point during execution
 */
pub fn sema_rpn(expr: &Vec<Token>) -> bool {
    let dummy = 0u32;
    let mut vstack: Vec<u32> = Vec::new();
    for e in expr {
        match e.kind() {
            TokenKind::Num(_) | TokenKind::Ident(_) => vstack.push(dummy),
            TokenKind::LPar => {
                println!("Extra '(' found at at line {} column {}",
                        e.line(), e.col());
                return false;
            }

            TokenKind::RPar => {
                println!("Extra ')' found at at line {} column {}",
                        e.line(), e.col());
                return false;
            }

            _ => {
                let op1 = vstack.pop();
                let op2 = vstack.pop();
                if op1.is_none() {
                    println!("Operator {} at line {} column {} has no operands but needs 2",
                            e.to_string(), e.line(), e.col());
                    return false;
                }

                if op2.is_none() {
                    println!("Operator {} at line {} column {} has 1 operand but needs 2",
                            e.to_string(), e.line(), e.col());
                    return false;
                }

                op1.unwrap();
                op2.unwrap();
                vstack.push(dummy);
            }
        }
    }

    if vstack.len() != 1 {
        println!("RPN generator internal error: vstack has excess elements");
        return false;
    }

    true
}
