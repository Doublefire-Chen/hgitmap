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
    // Initialize logger with default level if RUST_LOG not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

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
    println!("   - POST http://{}:{}/auth/register", host, port);
    println!("   - POST http://{}:{}/auth/login", host, port);
    println!("   - GET  http://{}:{}/oauth/github/authorize", host, port);
    println!("   - GET  http://{}:{}/oauth/github/callback", host, port);
    println!("   - POST http://{}:{}/platforms/connect (JWT required)", host, port);
    println!("   - GET  http://{}:{}/platforms (JWT required)", host, port);
    println!("   - GET  http://{}:{}/contributions (JWT required)", host, port);
    println!("   - GET  http://{}:{}/settings (JWT required)", host, port);
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
            // Public endpoints (no authentication required)
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
            )
            // OAuth endpoints (authorize requires JWT, callback uses state token)
            .service(
                web::scope("/oauth")
                    .route(
                        "/github/authorize",
                        web::get()
                            .to(handlers::oauth::github_authorize)
                            .wrap(crate::middleware::auth::JwtMiddleware)
                    )
                    .route("/github/callback", web::get().to(handlers::oauth::github_callback))
            )
            // Protected endpoints (JWT required)
            .service(
                web::scope("/platforms")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("/connect", web::post().to(handlers::platform_accounts::connect_platform))
                    .route("", web::get().to(handlers::platform_accounts::list_platforms))
                    .route("/{id}", web::delete().to(handlers::platform_accounts::disconnect_platform))
                    .route("/{id}/sync", web::post().to(handlers::platform_accounts::sync_platform))
            )
            .service(
                web::scope("/contributions")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("", web::get().to(handlers::contributions::get_contributions))
                    .route("/stats", web::get().to(handlers::contributions::get_stats))
            )
            .service(
                web::scope("/activities")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("", web::get().to(handlers::activities::get_activities))
                    .route("/sync", web::post().to(handlers::activities::sync_activities))
            )
            .service(
                web::scope("/settings")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("", web::get().to(handlers::settings::get_settings))
                    .route("", web::put().to(handlers::settings::update_settings))
            )
            // Admin endpoints (JWT + admin check required)
            .service(
                web::scope("/admin/oauth-apps")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("", web::get().to(handlers::oauth_apps::list_oauth_apps))
                    .route("", web::post().to(handlers::oauth_apps::create_oauth_app))
                    .route("/{id}", web::put().to(handlers::oauth_apps::update_oauth_app))
                    .route("/{id}", web::delete().to(handlers::oauth_apps::delete_oauth_app))
            )
    })
    .bind((host, port))?
    .run()
    .await
}
