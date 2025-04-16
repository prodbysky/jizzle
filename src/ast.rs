use crate::lexer::Token;
use thiserror::Error;
#[derive(Debug)]
pub enum Node {
    Number {
        value: u64,
        here: usize,
        len: usize,
    },
    Binary {
        left: Box<Node>,
        op: Token,
        right: Box<Node>,
    },
}

#[derive(Debug, Error)]
pub enum ASTError {
    #[error("Unexpected EOF found")]
    UnexpectedEOF,
    #[error("Unexpected token. Got: {got}, expected: {expected}")]
    UnexpectedToken { got: Token, expected: Token },
}

pub fn parse(tokens: &[Token]) -> Result<Node, ASTError> {
    Ok(parse_expr(tokens)?.1)
}

fn parse_expr(mut tokens: &[Token]) -> Result<(&[Token], Node), ASTError> {
    let (ts, mut left) = parse_primary(tokens)?;
    tokens = ts;

    loop {
        let op = match tokens.first() {
            Some(Token::Plus { here }) => Token::Plus { here: *here },
            _ => return Ok((tokens, left)),
        };
        tokens = &tokens[1..];

        let (ts, right) = parse_primary(tokens)?;
        tokens = ts;
        left = Node::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}
fn parse_primary(tokens: &[Token]) -> Result<(&[Token], Node), ASTError> {
    match tokens.first() {
        Some(Token::Number { value, here, len }) => Ok((
            &tokens[1..],
            Node::Number {
                value: *value,
                here: *here,
                len: *len,
            },
        )),
        Some(c) => Err(ASTError::UnexpectedToken {
            got: c.clone(),
            expected: Token::Number {
                value: 0,
                len: 0,
                here: 0,
            },
        }),
        None => Err(ASTError::UnexpectedEOF),
    }
}
