use crate::ast;
use thiserror::Error;

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

pub fn compile(name: &str, program: &[ast::Statement]) -> Result<(), BackendError> {
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(BackendError::Target)?;

    let ctx = inkwell::context::Context::create();
    let module = ctx.create_module("main");
    let i64_type = ctx.i64_type();
    let main_type = i64_type.fn_type(&[], false);
    let main_func = module.add_function("main", main_type, None);

    let main_block = ctx.append_basic_block(main_func, "entry");

    let builder = ctx.create_builder();
    let mut variables = std::collections::HashMap::new();
    builder.position_at_end(main_block);

    for st in program {
        match st {
            ast::Statement::Return(value) => {
                let value = eval_expression(&builder, &ctx, value, &variables)
                    .map_err(BackendError::IRBuild)?;
                builder
                    .build_return(Some(&value))
                    .map_err(BackendError::IRBuild)?;
            }
            ast::Statement::DefineVar { name, value } => {
                let ptr = builder
                    .build_alloca(i64_type, name)
                    .map_err(BackendError::IRBuild)?;
                let value = eval_expression(&builder, &ctx, value, &variables)
                    .map_err(BackendError::IRBuild)?;
                builder
                    .build_store(ptr, value)
                    .map_err(BackendError::IRBuild)?;
                variables.insert(name.to_string(), ptr);
            }
        }
    }

    module.verify().map_err(BackendError::IRVerification)?;

    let triple = inkwell::targets::TargetMachine::get_default_triple();
    let target =
        inkwell::targets::Target::from_triple(&triple).map_err(BackendError::CompileTarget)?;

    let target_machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            inkwell::OptimizationLevel::None,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .ok_or(BackendError::TargetMachine)?;

    target_machine
        .write_to_file(
            &module,
            inkwell::targets::FileType::Object,
            format!("{name}.o").as_ref(),
        )
        .map_err(BackendError::OutputIR)?;
    std::process::Command::new("gcc")
        .arg(format!("{name}.o"))
        .arg("-o")
        .arg(name)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    std::process::Command::new("rm")
        .arg(format!("{name}.o"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    Ok(())
}

fn eval_expression<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ctx: &'ctx inkwell::context::Context,
    expr: &ast::Expression,
    variables: &std::collections::HashMap<String, inkwell::values::PointerValue<'ctx>>,
) -> Result<inkwell::values::IntValue<'ctx>, inkwell::builder::BuilderError> {
    let t = ctx.i64_type();
    match expr {
        ast::Expression::Variable { name, .. } => {
            let ptr = variables.get(name).unwrap();
            let loaded = builder.build_load(t, *ptr, name).unwrap().into_int_value();
            Ok(loaded)
        }
        ast::Expression::Number { value, .. } => Ok(t.const_int(*value, false)),
        ast::Expression::Binary { left, op, right } => {
            let left = eval_expression(builder, ctx, left, variables)?;
            let right = eval_expression(builder, ctx, right, variables)?;

            match op {
                crate::lexer::Token::Plus { .. } => builder.build_int_add(left, right, "add"),
                crate::lexer::Token::Minus { .. } => builder.build_int_sub(left, right, "sub"),
                crate::lexer::Token::Star { .. } => builder.build_int_mul(left, right, "mul"),
                _ => unreachable!(),
            }
        }
    }
}
