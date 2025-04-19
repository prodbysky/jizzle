mod ast;
mod backend;
mod error;
mod lexer;
mod source;

use clap::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    IO(#[from] std::io::Error),
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
            Self::IO(e) => {
                writeln!(f, "IO error")?;
                writeln!(f, "{e}")
            }
        }
    }
}

fn real_main() -> Result<(), CompilerError> {
    let conf = Config::parse();
    let src = source::Source::from_file(&conf.file_name)?;
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
    backend::compile(&conf.output, &one_expr)?;
    println!("Compilation took: {:.2?}", pre_comp.elapsed());
    println!("Executable compiled. Available at: ./{}", conf.output);

    Ok(())
}

fn main() {
    if let Err(e) = real_main() {
        println!("{e}");
    }
}
#[derive(Debug, Parser)]
struct Config {
    /// The program source code file name
    #[arg()]
    file_name: String,
    /// Executable output name
    #[arg(short, default_value_t = String::from("main"))]
    output: String,
}
