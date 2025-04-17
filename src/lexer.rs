use crate::{error, source};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Token {
    Number { value: u64, here: usize, len: usize },
    Plus { here: usize },
    Return { here: usize },
    Semicolon { here: usize },
}

#[derive(Debug, Error)]
pub enum NumberLexError {
    Letter {
        offset: usize,
        line: String,
        line_number: usize,
        column_number: usize,
        len: usize,
    },
}

#[derive(Debug, Error)]
pub enum LexerError {
    UnexpectedChar {
        offset: usize,
        line: String,
        line_number: usize,
        column_number: usize,
        c: char,
    },
    UnexpectedEOF {
        offset: usize,
        line: String,
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
            Some(';') => {
                tokens.push(Token::Semicolon { here: src.offset() });
                src.next();
            }
            Some(c) if c.is_alphabetic() || *c == '_' => {
                let (begin, _len, ident) = lex_ident(&mut src).unwrap();
                match ident.as_str() {
                    "return" => tokens.push(Token::Return { here: begin }),
                    _ => unimplemented!(),
                }
            }
            Some(c) => {
                return Err(LexerError::UnexpectedChar {
                    offset: src.offset(),
                    line: src.get_line(l),
                    column_number: col,
                    line_number: l,
                    c: *c,
                });
            }
            None => {
                return Err(LexerError::UnexpectedEOF {
                    offset: src.offset(),
                    line: src.get_line(l),
                    column_number: col,
                    line_number: l,
                });
            }
        }
    }
    Ok(tokens)
}

fn lex_ident(src: &mut source::Source) -> LexerResult<(usize, usize, String), ()> {
    let begin = src.offset();
    src.next();

    while src.peek().is_some_and(|c| c.is_alphanumeric() || *c == '_') {
        src.next();
    }

    Ok((
        begin,
        src.offset() - begin,
        src.as_string().get(begin..src.offset()).unwrap().to_owned(),
    ))
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
                offset: begin,
                line: src.get_line(l),
                column_number: c,
                line_number: l,
                len: src.offset() - begin,
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
                offset: _,
                line,
                line_number,
                column_number,
            } => error::display_error(f, line, (*line_number, *column_number), 1, "Unexpected EOF"),
            Self::UnexpectedChar {
                offset: _,
                line,
                line_number,
                column_number,
                c,
            } => error::display_error(
                f,
                line,
                (*line_number, *column_number),
                1,
                format!("Unexpected char: {c}").as_str(),
            ),
        }
    }
}

impl std::fmt::Display for NumberLexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Letter {
                offset: _,
                line,
                line_number,
                column_number,
                len,
            } => error::display_error(
                f,
                line,
                (*line_number, *column_number),
                *len,
                "Numbers MUST be separated from letters",
            ),
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Plus { .. } => write!(f, "+"),
            Token::Number { value, .. } => write!(f, "{value}"),
            Token::Return { .. } => write!(f, "return"),
            Token::Semicolon { .. } => write!(f, ";"),
        }
    }
}
