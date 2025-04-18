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

struct Backend<'ctx> {
    ctx: &'ctx inkwell::context::Context,
    builder: inkwell::builder::Builder<'ctx>,
    module: inkwell::module::Module<'ctx>,
    variables: std::collections::HashMap<String, inkwell::values::PointerValue<'ctx>>,
}

impl<'ctx> Backend<'ctx> {
    fn new(ctx: &'ctx inkwell::context::Context) -> Result<Self, BackendError> {
        inkwell::targets::Target::initialize_native(
            &inkwell::targets::InitializationConfig::default(),
        )
        .map_err(BackendError::Target)?;

        let builder = ctx.create_builder();
        Ok(Self {
            ctx,
            builder,
            module: ctx.create_module("main"),
            variables: std::collections::HashMap::new(),
        })
    }

    fn begin_main(&mut self) {
        let i64_type = self.ctx.i64_type();
        let main_type = i64_type.fn_type(&[], false);
        let main_func = self.module.add_function("main", main_type, None);
        let main_block = self.ctx.append_basic_block(main_func, "entry");

        self.builder.position_at_end(main_block);
    }

    fn eval_expression(
        &mut self,
        value: ast::Expression,
    ) -> Result<inkwell::values::IntValue<'ctx>, inkwell::builder::BuilderError> {
        let t = self.ctx.i64_type();
        match value {
            ast::Expression::Variable { name, .. } => {
                let ptr = self.variables.get(&name).unwrap();
                let loaded = self
                    .builder
                    .build_load(t, *ptr, &name)
                    .unwrap()
                    .into_int_value();
                Ok(loaded)
            }
            ast::Expression::Number { value, .. } => Ok(t.const_int(value, false)),
            ast::Expression::Binary { left, op, right } => {
                let left = self.eval_expression(*left)?;
                let right = self.eval_expression(*right)?;

                match op {
                    crate::lexer::Token::Plus { .. } => {
                        self.builder.build_int_add(left, right, "add")
                    }
                    crate::lexer::Token::Minus { .. } => {
                        self.builder.build_int_sub(left, right, "sub")
                    }
                    crate::lexer::Token::Star { .. } => {
                        self.builder.build_int_mul(left, right, "mul")
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn define_variable(&mut self, name: &str, value: ast::Expression) -> Result<(), BackendError> {
        let i64_type = self.ctx.i64_type();
        let ptr = self
            .builder
            .build_alloca(i64_type, name)
            .map_err(BackendError::IRBuild)?;
        let value = self.eval_expression(value).map_err(BackendError::IRBuild)?;
        self.builder
            .build_store(ptr, value)
            .map_err(BackendError::IRBuild)?;
        self.variables.insert(name.to_string(), ptr);
        Ok(())
    }
}

pub fn compile(name: &str, program: &[ast::Statement]) -> Result<(), BackendError> {
    let ctx = inkwell::context::Context::create();
    let mut backend = Backend::new(&ctx)?;
    backend.begin_main();

    for st in program {
        match st {
            ast::Statement::Return(value) => {
                let value = backend
                    .eval_expression(value.clone())
                    .map_err(BackendError::IRBuild)?;
                backend
                    .builder
                    .build_return(Some(&value))
                    .map_err(BackendError::IRBuild)?;
            }
            ast::Statement::DefineVar { name, value } => {
                backend.define_variable(name, value.clone())?;
            }
        }
    }

    backend
        .module
        .verify()
        .map_err(BackendError::IRVerification)?;

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
            &backend.module,
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
