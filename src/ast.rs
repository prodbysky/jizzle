use crate::lexer::Token;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Number {
        value: u64,
        here: usize,
        len: usize,
    },
    Variable {
        name: String,
        here: usize,
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
    DefineVar { name: String, value: Expression },
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
        let (rest, stmt) = parse_statement(tokens)?;
        stmts.push(stmt);
        tokens = rest;
    }
    Ok(stmts)
}

fn parse_statement(tokens: &[Token]) -> Result<(&[Token], Statement), ASTError> {
    let (rest, stmt) = match tokens.split_first() {
        Some((Token::Return { .. }, rest)) => {
            let (rest, expr) = parse_expr(rest)?;
            (rest, Statement::Return(expr))
        }
        Some((Token::Var { here }, rest)) => {
            let (rest, name) = match rest.split_first() {
                Some((Token::Ident { value, .. }, rest)) => (rest, value),
                Some((t, ..)) => {
                    return Err(ASTError::UnexpectedToken {
                        got: t.clone(),
                        expected: Token::Ident {
                            value: "any".to_string(),
                            here: *here,
                        },
                    });
                }
                None => return Err(ASTError::UnexpectedEOF),
            };
            let rest = match rest.split_first() {
                Some((Token::Equal { .. }, rest)) => rest,
                Some((t, ..)) => {
                    return Err(ASTError::UnexpectedToken {
                        got: t.clone(),
                        expected: Token::Semicolon { here: *here },
                    });
                }
                None => return Err(ASTError::UnexpectedEOF),
            };
            let (rest, expr) = parse_expr(rest)?;
            (
                rest,
                Statement::DefineVar {
                    name: name.to_string(),
                    value: expr,
                },
            )
        }
        None => {
            return Err(ASTError::UnexpectedEOF);
        }
        _ => unimplemented!(),
    };
    match rest.split_first() {
        Some((Token::Semicolon { .. }, rest)) => return Ok((rest, stmt)),
        Some((t, ..)) => {
            return Err(ASTError::UnexpectedToken {
                got: t.clone(),
                expected: Token::Semicolon { here: 0 },
            });
        }
        None => {
            return Err(ASTError::UnexpectedEOF);
        }
    }
}

fn parse_expr(mut tokens: &[Token]) -> Result<(&[Token], Expression), ASTError> {
    let (ts, mut left) = parse_mult(tokens)?;
    tokens = ts;

    loop {
        let op = match tokens.first() {
            Some(Token::Plus { here }) => Token::Plus { here: *here },
            Some(Token::Minus { here }) => Token::Minus { here: *here },
            _ => return Ok((tokens, left)),
        };
        tokens = &tokens[1..];

        let (ts, right) = parse_mult(tokens)?;
        tokens = ts;
        left = Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

fn parse_mult(mut tokens: &[Token]) -> Result<(&[Token], Expression), ASTError> {
    let (ts, mut left) = parse_primary(tokens)?;
    tokens = ts;

    loop {
        let op = match tokens.first() {
            Some(Token::Star { here }) => Token::Star { here: *here },
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
        Some(Token::Ident { value, here }) => Ok((
            &tokens[1..],
            Expression::Variable {
                here: *here,
                name: value.to_string(),
            },
        )),
        Some(Token::OpenParen { .. }) => {
            let tokens = &tokens[1..];
            let (ts, expr) = parse_expr(tokens)?;
            let tokens = ts;
            match tokens.first() {
                Some(Token::CloseParen { .. }) => Ok((&tokens[1..], expr)),
                None => Err(ASTError::UnexpectedEOF)?,
                Some(t) => Err(ASTError::UnexpectedToken {
                    got: t.clone(),
                    expected: Token::CloseParen { here: 0 },
                })?,
            }
        }
        Some(t) => Err(ASTError::UnexpectedToken {
            got: t.clone(),
            expected: Token::OpenParen { here: 0 },
        })?,
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

    #[test]
    fn statement_var() {
        let tokens = vec![
            Token::Var { here: 0 },
            Token::Ident {
                value: String::from("hello"),
                here: 1,
            },
            Token::Equal { here: 2 },
            Token::Number {
                value: 0,
                len: 1,
                here: 3,
            },
            Token::Semicolon { here: 4 },
        ];

        assert_eq!(
            parse(&tokens),
            Ok(vec![Statement::DefineVar {
                name: String::from("hello"),
                value: Expression::Number {
                    value: 0,
                    len: 1,
                    here: 3
                }
            }])
        );
    }
}
