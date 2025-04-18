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
    let src = source::Source::new("var a = 5 * 2 + 1;return a;");
    println!("Lexing...");
    let pre_lex = std::time::Instant::now();
    let tokens = lexer::lex_file(src)?;
    println!("Lexing took: {:.2?}", pre_lex.elapsed());

    println!("Parsing AST...");
    let pre_parse = std::time::Instant::now();
    let one_expr = ast::parse(&tokens)?;
    println!("AST parsing took: {:.2?}", pre_parse.elapsed());

    println!("Generating and compiling code...");
    let pre_comp = std::time::Instant::now();
    backend::compile("main", &one_expr)?;
    println!("Compilation took: {:.2?}", pre_comp.elapsed());
    println!("Executable compiled. Available at: ./main");

    Ok(())
}

fn main() {
    if let Err(e) = real_main() {
        println!("{e}");
    }
}
