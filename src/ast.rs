use crate::lexer::Token;
use thiserror::Error;

#[derive(Debug)]
pub enum Expression {
    Number {
        value: u64,
        here: usize,
        len: usize,
    },
    Binary {
        left: Box<Expression>,
        op: Token,
        right: Box<Expression>,
    },
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, Error)]
pub enum ASTError {
    #[error("Unexpected EOF found")]
    UnexpectedEOF,
    #[error("Unexpected token. Got: {got}, expected: {expected}")]
    UnexpectedToken { got: Token, expected: Token },
}

pub fn parse(mut tokens: &[Token]) -> Result<Statement, ASTError> {
    match tokens.first() {
        Some(Token::Return { here: _ }) => {
            tokens = &tokens[1..];
            let (rest, expr) = parse_expr(tokens)?;
            let tk = match rest.first() {
                Some(Token::Semicolon { .. }) => Ok(Statement::Return(expr)),
                Some(t) => Err(ASTError::UnexpectedToken {
                    got: t.clone(),
                    expected: Token::Semicolon { here: 0 },
                }),
                None => Err(ASTError::UnexpectedEOF),
            }?;
            tokens = &rest[1..];
            Ok(tk)
        }
        None => Err(ASTError::UnexpectedEOF),
        Some(t) => Err(ASTError::UnexpectedToken {
            got: t.clone(),
            expected: Token::Return { here: 0 },
        }),
    }
}

fn parse_expr(mut tokens: &[Token]) -> Result<(&[Token], Expression), ASTError> {
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
        left = Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}
fn parse_primary(tokens: &[Token]) -> Result<(&[Token], Expression), ASTError> {
    match tokens.first() {
        Some(Token::Number { value, here, len }) => Ok((
            &tokens[1..],
            Expression::Number {
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
