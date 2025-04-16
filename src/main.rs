mod ast;
mod error;
mod lexer;
mod source;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    Lexer(#[from] lexer::LexerError),
    Ast(#[from] ast::ASTError),
    Backend(#[from] BackendError),
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
                writeln!(f, "Codegen failure failed")?;
                writeln!(f, "{e}")
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Failed to initialize target: {0}")]
    Target(String),
    #[error("Something during building IR failed: {0:?}")]
    IRBuild(inkwell::builder::BuilderError),
    #[error("IR verification failed: {0}")]
    IRVerification(inkwell::support::LLVMString),
    #[error("Failed to create target: {0}")]
    CompileTarget(inkwell::support::LLVMString),
    #[error("Failed to create target machine")]
    TargetMachine,
    #[error("Failed to output IR: {0}")]
    OutputIR(inkwell::support::LLVMString),
}

fn real_main() -> Result<(), CompilerError> {
    let src = source::Source::new("123+\n420\n123");
    let tokens = lexer::lex_file(src)?;

    let one_expr = ast::parse(&tokens)?;
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(BackendError::Target)?;

    let ctx = inkwell::context::Context::create();
    let module = ctx.create_module("main");
    let i64_type = ctx.i64_type();
    let main_type = i64_type.fn_type(&[], false);
    let main_func = module.add_function("main", main_type, None);

    let main_block = ctx.append_basic_block(main_func, "entry");

    let builder = ctx.create_builder();
    builder.position_at_end(main_block);
    let int = i64_type.const_int(0, false);
    builder
        .build_return(Some(&int))
        .map_err(BackendError::IRBuild)?;

    module.verify().map_err(BackendError::IRVerification)?;

    let triple = inkwell::targets::TargetMachine::get_default_triple();
    let target =
        inkwell::targets::Target::from_triple(&triple).map_err(BackendError::CompileTarget)?;

    let target_machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            inkwell::OptimizationLevel::Aggressive,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .ok_or(BackendError::TargetMachine)?;

    target_machine
        .write_to_file(
            &module,
            inkwell::targets::FileType::Object,
            "output.o".as_ref(),
        )
        .map_err(BackendError::OutputIR)?;
    std::process::Command::new("gcc")
        .arg("output.o")
        .arg("-o")
        .arg("main")
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    Ok(())
}

fn main() {
    if let Err(e) = real_main() {
        println!("{e}");
    }
}
