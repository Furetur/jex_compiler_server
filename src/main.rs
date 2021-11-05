use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpRequest};
use random_string::generate;
use tokio::fs::File;
use tokio::process::Command;

extern crate env_logger;

const RUNTIME_DATA_FOLDER: &str = "runtime_data";
const SOURCE_CODE_DATA_FOLDER: &str = "source_code";
const COMPILED_CODE_DATA_FOLDER: &str = "compiled";

const JEX_COMPILER_FILE: &str = "JexCompiler-0.2.jar";
const JEX_VM_FILE: &str = "jex_vm-0.4";

#[post("/")]
async fn index(source_code: String) -> impl Responder {
    let request_id = random_string(6);
    let source_filepath = format!("{}/{}/{}.txt", RUNTIME_DATA_FOLDER, SOURCE_CODE_DATA_FOLDER, request_id);
    let compiled_filepath = format!("{}/{}/{}", RUNTIME_DATA_FOLDER, COMPILED_CODE_DATA_FOLDER, request_id);

    // create source file
    tokio::fs::write(source_filepath.clone(), source_code.clone()).await.expect("Failed to write file");
    // try to compile it
    let compiler_output = compile_jex_file(source_filepath.clone(), compiled_filepath.clone()).await;
    if let Err(e) = compiler_output {
        return e;
    }
    // try to run it
    let vm_output = run_jex_compiled_file(compiled_filepath.clone()).await;
    match vm_output {
        Ok(msg) => msg,
        Err(msg) => msg
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    tokio::fs::create_dir_all(format!("{}/{}", RUNTIME_DATA_FOLDER, SOURCE_CODE_DATA_FOLDER))
        .await.expect("Failed to create dir for source code");

    tokio::fs::create_dir_all(format!("{}/{}", RUNTIME_DATA_FOLDER, COMPILED_CODE_DATA_FOLDER))
        .await.expect("Failed to create dir for compiled code");

    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

async fn compile_jex_file(from: String, to: String) -> Result<(), String> {
    let compiler_path = format!("res/{}", JEX_COMPILER_FILE);

    let output = Command::new("java").arg("-jar").arg(compiler_path).arg(from).arg(to)
        .output().await.expect("Jex compiler failed to run");

    if output.status.success() {
        Ok(())
    } else {
        Err(from_utf8(&output.stderr).unwrap_or("Failed to get stderr from compiler").to_string())
    }
}

async fn run_jex_compiled_file(from: String) -> Result<String, String> {
    let vm_command = format!("./res/{}", JEX_VM_FILE);

    let output = Command::new(vm_command).arg(from)
        .output().await.expect("Failed to start jex vm");

    if output.status.success() {
        Ok(from_utf8(&output.stdout).unwrap_or("Failed to get stdout from vm").to_string())
    } else {
        Err(from_utf8(&output.stdout).unwrap_or("Failed to get stdout from vm").to_string())
    }
}

async fn create_source_file(request_id: String, source_code: String) -> String {
    let filename = format!("{}.txt", request_id);
    let filepath = format!("{}/{}/{}", RUNTIME_DATA_FOLDER, SOURCE_CODE_DATA_FOLDER, filename);
    tokio::fs::write(filepath.clone(), source_code).await.expect("Failed to write file");
    filepath
}

fn random_string(length: usize) -> String {
    let charset = "1234567890abcef";
    generate(length, charset)
}
