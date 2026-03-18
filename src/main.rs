use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use narayan_web::{
    config::Config,
    handlers::{
        browse_library, convert_library_selection, download_batch, download_file, get_job,
        handle_upload, list_jobs, websocket_handler,
    },
    models::AppState,
    services::JobManager,
    utils::check_ffmpeg_installed,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer, limit::RequestBodyLimitLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "narayan_web=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Narayan Web Converter");

    // Load configuration
    let config = Config::load();
    info!("Configuration loaded: {:?}", config);

    // Check FFmpeg installation
    if !check_ffmpeg_installed().await {
        error!("FFmpeg is not installed or not in PATH");
        return Err("FFmpeg is required but not found".into());
    }
    info!("FFmpeg is installed");

    // Create temp directory
    tokio::fs::create_dir_all(&config.temp_dir).await?;
    info!("Temp directory created: {:?}", config.temp_dir);

    // Initialize job manager
    let job_manager = Arc::new(JobManager::new(config.clone()));

    // Create app state
    let app_state = AppState::new(job_manager);

    // Build router
    let app = create_router(app_state, config.clone()).await;

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    info!("Server listening on {}", config.bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_router(state: AppState, config: Config) -> Router {
    Router::new()
        // API routes
        .route("/api/upload", post(handle_upload))
        .route("/api/library", get(browse_library))
        .route("/api/library/convert", post(convert_library_selection))
        .route("/api/jobs", get(list_jobs))
        .route("/api/jobs/:id", get(get_job))
        .route("/api/download/:id", get(download_file))
        .route("/api/download/batch/:job_id", get(download_batch))
        .route("/ws", get(websocket_handler))
        .layer(DefaultBodyLimit::max(config.max_upload_size))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        // Static file serving (must be last)
        .nest_service("/", ServeDir::new("static"))
}
