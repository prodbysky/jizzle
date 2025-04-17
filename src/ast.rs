use crate::lexer::Token;
use thiserror::Error;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Statement {
    Return(Expression),
}

#[derive(Debug, Error, PartialEq)]
pub enum ASTError {
    #[error("Unexpected EOF found")]
    UnexpectedEOF,
    #[error("Unexpected token. Got: {got}, expected: {expected}")]
    UnexpectedToken { got: Token, expected: Token },
}

pub fn parse(mut tokens: &[Token]) -> Result<Vec<Statement>, ASTError> {
    let mut stmts = vec![];
    while !tokens.is_empty() {
        match tokens.first() {
            Some(Token::Return { here: _ }) => {
                tokens = &tokens[1..];
                let (rest, expr) = parse_expr(tokens)?;
                stmts.push(match rest.first() {
                    Some(Token::Semicolon { .. }) => Ok(Statement::Return(expr)),
                    Some(t) => Err(ASTError::UnexpectedToken {
                        got: t.clone(),
                        expected: Token::Semicolon { here: 0 },
                    }),
                    None => Err(ASTError::UnexpectedEOF),
                }?);
                tokens = &rest[1..];
            }
            None => Err(ASTError::UnexpectedEOF)?,
            Some(t) => Err(ASTError::UnexpectedToken {
                got: t.clone(),
                expected: Token::Return { here: 0 },
            })?,
        }
    }
    Ok(stmts)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let tokens = vec![];
        assert_eq!(parse(&tokens), Ok(vec![]));
    }

    #[test]
    fn number_expr() {
        let tokens = vec![Token::Number {
            value: 0,
            here: 0,
            len: 1,
        }];
        let empty: &[Token] = &[];
        let (rest, expr) = parse_expr(&tokens).unwrap();
        assert_eq!(rest, empty);
        assert_eq!(
            expr,
            Expression::Number {
                value: 0,
                here: 0,
                len: 1
            }
        );
    }

    #[test]
    fn binary_expr() {
        let tokens = vec![
            Token::Number {
                value: 1,
                here: 0,
                len: 1,
            },
            Token::Plus { here: 1 },
            Token::Number {
                value: 1,
                here: 2,
                len: 1,
            },
        ];
        let empty: &[Token] = &[];
        let (rest, expr) = parse_expr(&tokens).unwrap();
        assert_eq!(rest, empty);
        assert_eq!(
            expr,
            Expression::Binary {
                left: Box::new(Expression::Number {
                    value: 1,
                    here: 0,
                    len: 1
                }),
                op: Token::Plus { here: 1 },
                right: Box::new(Expression::Number {
                    value: 1,
                    here: 2,
                    len: 1
                }),
            }
        );
    }
    #[test]
    fn statement_return() {
        let tokens = vec![
            Token::Return { here: 0 },
            Token::Number {
                value: 0,
                here: 7,
                len: 1,
            },
            Token::Semicolon { here: 8 },
        ];

        assert_eq!(
            parse(&tokens),
            Ok(vec![Statement::Return(Expression::Number {
                value: 0,
                here: 7,
                len: 1
            })])
        );
    }
}
