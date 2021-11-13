use actix_cors::Cors;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use jex_compiler_server::jex_commands::{create_jex_folders, run_jex, RunJexError};

extern crate env_logger;

#[post("/")]
async fn index(source_code: String) -> impl Responder {
    match run_jex(source_code).await {
        Ok(output) => HttpResponse::Ok().body(output),
        Err(RunJexError::InternalErr(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(RunJexError::UserExecutionError(e)) => HttpResponse::BadRequest().body(e),
        Err(RunJexError::UserCompilationError(e)) => HttpResponse::BadRequest().body(e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "jex_compiler_server=info,actix_web=info");
    env_logger::init();

    create_jex_folders().await;

    let host = std::env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = std::env::var("PORT").expect("Could not get PORT env variable");

    println!("Listening on {}:{}", host, port);

    HttpServer::new(|| {
        let cors = Cors::permissive().allow_any_header().allow_any_origin().allow_any_method();
        App::new().wrap(cors).service(index)
    })
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
}
