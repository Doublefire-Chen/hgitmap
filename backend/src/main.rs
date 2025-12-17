mod models;
mod handlers;
mod middleware;
mod services;
mod utils;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};
use utils::{config::Config, db::establish_connection};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    println!("=================================================");
    println!("🚀 hgitmap Backend Server");
    println!("=================================================");

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    let host = config.host.clone();
    let port = config.port;

    println!("📝 Configuration loaded:");
    println!("   - Database: {}", config.database_url.split('@').last().unwrap_or("***"));
    println!("   - Host: {}", host);
    println!("   - Port: {}", port);
    println!("   - Registration: {}", if config.allow_registration { "ENABLED" } else { "DISABLED" });
    println!("   - Log level: {}", std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));

    // Establish database connection
    print!("🔌 Connecting to database... ");
    let db = establish_connection(&config.database_url)
        .await
        .expect("Failed to connect to database");
    println!("✅ Connected!");

    log::info!("Database connection established");

    // Start HTTP server
    println!("🌐 Starting HTTP server at http://{}:{}", host, port);
    println!("📍 Available endpoints:");
    println!("   - POST http://{}:{}/api/auth/register", host, port);
    println!("   - POST http://{}:{}/api/auth/login", host, port);
    println!("=================================================");

    log::info!("Server started at http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/login", web::post().to(handlers::auth::login))
                    )
            )
    })
    .bind((host, port))?
    .run()
    .await
}
