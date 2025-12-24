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
    // Load .env file FIRST before anything else
    dotenv::dotenv().ok();

    // Initialize logger with default level if RUST_LOG not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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

    // Start background job processor for heatmap generation
    log::info!("Starting heatmap generation job processor");
    services::job_processor::start_job_processor(db.clone());

    // Start sync scheduler for automatic platform data syncing
    log::info!("Starting platform sync scheduler");
    let scheduler = std::sync::Arc::new(services::sync_scheduler::SyncScheduler::new(db.clone(), config.clone()));
    let scheduler_clone = scheduler.clone();
    tokio::spawn(async move {
        scheduler_clone.start().await;
    });

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
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .wrap(cors)  // CORS must be wrapped AFTER Logger to ensure headers are added to all responses
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
                    .route("/{id}/sync-preferences", web::put().to(handlers::platform_accounts::update_sync_preferences))
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
            // Sync endpoints (JWT required)
            .service(
                web::scope("/sync")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    .route("/trigger", web::post().to(handlers::sync::trigger_sync))
                    .route("/status", web::get().to(handlers::sync::get_sync_status))
            )
            // Heatmap theme and generation endpoints
            .service(
                web::scope("/heatmap")
                    .wrap(crate::middleware::auth::JwtMiddleware)
                    // Theme management
                    .route("/themes", web::get().to(handlers::heatmap_themes::list_themes))
                    .route("/themes", web::post().to(handlers::heatmap_themes::create_theme))
                    .route("/themes/{slug}", web::get().to(handlers::heatmap_themes::get_theme))
                    .route("/themes/{slug}", web::put().to(handlers::heatmap_themes::update_theme))
                    .route("/themes/{slug}", web::delete().to(handlers::heatmap_themes::delete_theme))
                    .route("/themes/{slug}/set-default", web::post().to(handlers::heatmap_themes::set_default_theme))
                    .route("/themes/{slug}/duplicate", web::post().to(handlers::heatmap_themes::duplicate_theme))
                    // Generation settings
                    .route("/settings", web::get().to(handlers::heatmap_generation::get_generation_settings))
                    .route("/settings", web::put().to(handlers::heatmap_generation::update_generation_settings))
                    // Manual generation triggers
                    .route("/generate", web::post().to(handlers::heatmap_generation::trigger_generation))
                    .route("/generate/{slug}", web::post().to(handlers::heatmap_generation::trigger_theme_generation))
                    // View generated heatmaps and jobs
                    .route("/generated", web::get().to(handlers::heatmap_generation::list_generated_heatmaps))
                    .route("/jobs", web::get().to(handlers::heatmap_generation::list_generation_jobs))
                    // Preview theme (POST with theme parameters)
                    .route("/preview", web::post().to(handlers::heatmap_generation::preview_theme))
            )
            // Public static file endpoints (no authentication required)
            .service(
                web::scope("/static/heatmaps")
                    .route("/{user_id}/{filename}", web::get().to(handlers::static_files::serve_heatmap))
            )
            .service(
                web::scope("/embed")
                    .route("/{username}/{theme_file}", web::get().to(handlers::static_files::serve_embed))
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
