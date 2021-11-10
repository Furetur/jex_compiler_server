use crate::constants::{
    COMPILED_CODE_DATA_FOLDER, JEX_COMPILER_FILE, JEX_COMPILE_TIMEOUT, JEX_EXEC_TIMEOUT,
    JEX_VM_FILE, RUNTIME_DATA_FOLDER, SOURCE_CODE_DATA_FOLDER,
};
use crate::jex_commands::InternalErr::FailedToRunVm;
use crate::run_command::CommandError::{RunFailure, StdioDecodingFailed, UnsuccessfulExitCode};
use crate::run_command::{run_command, CommandError};
use crate::utils::random_string;
use std::fmt::Formatter;
use std::io::Error;
use tokio::process::Command;
use tokio::time::Duration;

pub enum RunJexError {
    InternalErr(InternalErr),
    UserCompilationError(String),
    UserExecutionError(String),
}

impl std::fmt::Display for RunJexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RunJexError::InternalErr(e) => write!(f, "INTERNAL ERROR\n{}", e),
            RunJexError::UserCompilationError(e) => write!(f, "COMPILATION ERROR\n{}", e),
            RunJexError::UserExecutionError(e) => write!(f, "RUNTIME ERROR\n{}", e),
        }
    }
}

pub enum InternalErr {
    FailedToWriteFile(Error),
    FailedToRunCompiler(String),
    FailedToRunVm(String),
}

impl std::fmt::Display for InternalErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalErr::FailedToWriteFile(e) => {
                write!(f, "Failed to create file for source code: {}", e)
            }
            InternalErr::FailedToRunCompiler(e) => write!(f, "Failed to run the compiler: {}", e),
            InternalErr::FailedToRunVm(e) => write!(f, "Failed to run the VM: {}", e),
        }
    }
}

pub async fn create_jex_folders() {
    tokio::fs::create_dir_all(format!(
        "{}/{}",
        RUNTIME_DATA_FOLDER, SOURCE_CODE_DATA_FOLDER
    ))
    .await
    .expect("Failed to create dir for source code");

    tokio::fs::create_dir_all(format!(
        "{}/{}",
        RUNTIME_DATA_FOLDER, COMPILED_CODE_DATA_FOLDER
    ))
    .await
    .expect("Failed to create dir for compiled code");
}

pub async fn run_jex(code: String) -> Result<String, RunJexError> {
    let request_id = random_string(6);
    let source_filepath = format!(
        "{}/{}/{}.txt",
        RUNTIME_DATA_FOLDER, SOURCE_CODE_DATA_FOLDER, request_id
    );
    let compiled_filepath = format!(
        "{}/{}/{}",
        RUNTIME_DATA_FOLDER, COMPILED_CODE_DATA_FOLDER, request_id
    );

    // create source file
    let file_create_result = tokio::fs::write(&source_filepath, &code).await;
    if let Err(e) = file_create_result {
        return Err(RunJexError::InternalErr(InternalErr::FailedToWriteFile(e)));
    }
    // try to compile it
    let compilation_result =
        compile_jex_file(source_filepath.clone(), compiled_filepath.clone()).await;
    if let Err(e) = compilation_result {
        return Err(match e {
            RunFailure(e) => {
                RunJexError::InternalErr(InternalErr::FailedToRunCompiler(e.to_string()))
            }
            StdioDecodingFailed(e) => RunJexError::InternalErr(InternalErr::FailedToRunCompiler(e)),
            UnsuccessfulExitCode(e) => RunJexError::UserCompilationError(e),
            CommandError::Timeout => RunJexError::InternalErr(InternalErr::FailedToRunCompiler(
                "Compiler timed out".to_string(),
            )),
        });
    }
    // try to run it
    let vm_result = run_jex_compiled_file(compiled_filepath.clone()).await;
    match vm_result {
        Ok(msg) => Ok(msg),
        Err(RunFailure(e)) => Err(RunJexError::InternalErr(FailedToRunVm(e.to_string()))),
        Err(StdioDecodingFailed(e)) => Err(RunJexError::InternalErr(FailedToRunVm(e.to_string()))),
        Err(UnsuccessfulExitCode(e)) => Err(RunJexError::UserExecutionError(e)),
        Err(CommandError::Timeout) => Err(RunJexError::UserExecutionError(format!(
            "Run time exceeded {} secs.",
            JEX_EXEC_TIMEOUT
        ))),
    }
}

async fn compile_jex_file(from: String, to: String) -> Result<String, CommandError> {
    let compiler_path = format!("./res/{}", JEX_COMPILER_FILE);

    let mut command = Command::new("java");
    command.arg("-jar").arg(compiler_path).arg(from).arg(to);
    run_command(&mut command, Duration::from_secs(JEX_COMPILE_TIMEOUT)).await
}

async fn run_jex_compiled_file(from: String) -> Result<String, CommandError> {
    let vm_command = format!("./res/{}", JEX_VM_FILE);

    let mut command = Command::new(vm_command);
    command.arg(from);
    run_command(&mut command, Duration::from_secs(JEX_EXEC_TIMEOUT)).await
}
