mod ast;
mod backend;
mod error;
mod lexer;
mod source;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    Lexer(#[from] lexer::LexerError),
    Ast(#[from] ast::ASTError),
    Backend(#[from] backend::BackendError),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lexer(e) => {
                writeln!(f, "Lexer failed")?;
                writeln!(f, "{e}")
            }
            Self::Ast(e) => {
                writeln!(f, "Ast parsing failed")?;
                writeln!(f, "{e}")
            }
            Self::Backend(e) => {
                writeln!(f, "Codegen failure")?;
                writeln!(f, "{e}")
            }
        }
    }
}

fn real_main() -> Result<(), CompilerError> {
    let src = source::Source::new("return 1");
    let tokens = lexer::lex_file(src)?;

    let one_expr = vec![ast::parse(&tokens)?];

    backend::compile("main", &one_expr)?;

    Ok(())
}

fn main() {
    if let Err(e) = real_main() {
        println!("{e}");
    }
}
