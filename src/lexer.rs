use crate::{error, source};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number { value: u64, here: usize, len: usize },
    Plus { here: usize },
    Minus { here: usize },
    Star { here: usize },
    OpenParen { here: usize },
    CloseParen { here: usize },
    OpenCurly { here: usize },
    CloseCurly { here: usize },
    Return { here: usize },
    Var { here: usize },
    Semicolon { here: usize },
    Equal { here: usize },
    Ident { value: String, here: usize },
}

#[derive(Debug, Error, PartialEq)]
pub enum NumberLexError {
    Letter {
        file: Option<String>,
        line_number: usize,
        column_number: usize,
    },
}

#[derive(Debug, Error, PartialEq)]
pub enum LexerError {
    UnexpectedChar {
        file: Option<String>,
        line_number: usize,
        column_number: usize,
        c: char,
    },
    UnexpectedEOF {
        file: Option<String>,
        line_number: usize,
        column_number: usize,
    },
    Number(#[from] NumberLexError),
}

pub type LexerResult<T, E> = Result<T, E>;

pub fn lex_file(mut src: source::Source) -> LexerResult<Vec<Token>, LexerError> {
    let mut tokens = vec![];

    while !src.finished() {
        src.skip_whitespace();
        let (l, col) = src.get_position(src.offset());
        match src.peek() {
            Some(c) if c.is_ascii_digit() => {
                tokens.push(lex_number(&mut src)?);
            }
            Some('+') => {
                tokens.push(Token::Plus { here: src.offset() });
                src.next();
            }
            Some('-') => {
                tokens.push(Token::Minus { here: src.offset() });
                src.next();
            }
            Some('*') => {
                tokens.push(Token::Star { here: src.offset() });
                src.next();
            }
            Some('=') => {
                tokens.push(Token::Equal { here: src.offset() });
                src.next();
            }
            Some(';') => {
                tokens.push(Token::Semicolon { here: src.offset() });
                src.next();
            }
            Some('(') => {
                tokens.push(Token::OpenParen { here: src.offset() });
                src.next();
            }
            Some(')') => {
                tokens.push(Token::CloseParen { here: src.offset() });
                src.next();
            }
            Some('{') => {
                tokens.push(Token::OpenCurly { here: src.offset() });
                src.next();
            }
            Some('}') => {
                tokens.push(Token::CloseCurly { here: src.offset() });
                src.next();
            }
            Some(c) if c.is_alphabetic() || *c == '_' => {
                let (begin, _len, ident) = lex_ident(&mut src);
                match ident.as_str() {
                    "return" => tokens.push(Token::Return { here: begin }),
                    "var" => tokens.push(Token::Var { here: begin }),
                    _ => tokens.push(Token::Ident {
                        here: begin,
                        value: ident,
                    }),
                }
            }
            Some(c) => {
                return Err(LexerError::UnexpectedChar {
                    file: src.path().map(|s| s.to_string()),
                    column_number: col,
                    line_number: l,
                    c: *c,
                });
            }
            None => {
                return Err(LexerError::UnexpectedEOF {
                    file: src.path().map(|s| s.to_string()),
                    column_number: col,
                    line_number: l,
                });
            }
        }
    }
    Ok(tokens)
}

fn lex_ident(src: &mut source::Source) -> (usize, usize, String) {
    let begin = src.offset();
    src.next();

    while src.peek().is_some_and(|c| c.is_alphanumeric() || *c == '_') {
        src.next();
    }
    (
        begin,
        src.offset() - begin,
        src.as_string().get(begin..src.offset()).unwrap().to_owned(),
    )
}

fn lex_number(src: &mut source::Source) -> LexerResult<Token, NumberLexError> {
    let begin = src.offset();

    while src.peek().is_some_and(char::is_ascii_digit) {
        src.next();
    }

    match src.peek() {
        Some(c) if c.is_ascii_alphabetic() => {
            let (l, c) = src.get_position(begin);

            Err(NumberLexError::Letter {
                file: src.path().map(|s| s.to_string()),
                column_number: c,
                line_number: l,
            })
        }
        _ => Ok(Token::Number {
            here: begin,
            len: src.offset() - begin,
            value: src
                .src()
                .get(begin..src.offset())
                .unwrap()
                .iter()
                .collect::<String>()
                .parse()
                .unwrap(),
        }),
    }
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(e) => {
                writeln!(f, "{e}")
            }
            Self::UnexpectedEOF {
                line_number,
                column_number,
                file,
            } => error::display_error(
                f,
                file.as_deref(),
                (*line_number, *column_number),
                "Unexpected EOF",
            ),
            Self::UnexpectedChar {
                line_number,
                column_number,
                c,
                file,
            } => error::display_error(
                f,
                file.as_deref(),
                (*line_number, *column_number),
                format!("Unexpected char: {c}").as_str(),
            ),
        }
    }
}

impl std::fmt::Display for NumberLexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Letter {
                file,
                line_number,
                column_number,
            } => error::display_error(
                f,
                file.as_deref(),
                (*line_number, *column_number),
                "Numbers MUST be separated from letters",
            ),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plus { .. } => write!(f, "+"),
            Token::Minus { .. } => write!(f, "-"),
            Token::OpenParen { .. } => write!(f, "("),
            Token::CloseParen { .. } => write!(f, ")"),
            Token::OpenCurly { .. } => write!(f, "{{"),
            Token::CloseCurly { .. } => write!(f, "}}"),
            Token::Star { .. } => write!(f, "*"),
            Token::Equal { .. } => write!(f, "="),
            Token::Number { value, .. } => write!(f, "{value}"),
            Token::Return { .. } => write!(f, "return"),
            Token::Var { .. } => write!(f, "var"),
            Token::Ident { value, .. } => write!(f, "{value}"),
            Token::Semicolon { .. } => write!(f, ";"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let src = source::Source::new("");
        assert_eq!(lex_file(src), Ok(vec![]));
    }

    #[test]
    fn numbers() {
        let src = source::Source::new("69 123 0");
        assert_eq!(
            lex_file(src),
            Ok(vec![
                Token::Number {
                    value: 69,
                    here: 0,
                    len: 2
                },
                Token::Number {
                    value: 123,
                    here: 3,
                    len: 3
                },
                Token::Number {
                    value: 0,
                    here: 7,
                    len: 1
                },
            ])
        );
    }

    #[test]
    fn keywords() {
        let src = source::Source::new("return var");
        assert_eq!(
            lex_file(src),
            Ok(vec![Token::Return { here: 0 }, Token::Var { here: 7 }])
        );
    }
}
