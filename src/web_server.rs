use actix_web::{web, App, HttpResponse, HttpServer, Result};
use actix_multipart::Multipart;
use actix_files as fs;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs as std_fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    conversion_status: Arc<Mutex<ConversionStatus>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConversionStatus {
    is_converting: bool,
    current_file: String,
    progress: f32,
    total_files: usize,
    processed_files: usize,
    log_messages: Vec<String>,
}

impl Default for ConversionStatus {
    fn default() -> Self {
        Self {
            is_converting: false,
            current_file: String::new(),
            progress: 0.0,
            total_files: 0,
            processed_files: 0,
            log_messages: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

async fn upload_files(
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let upload_dir = PathBuf::from("/tmp/flac_uploads");
    std_fs::create_dir_all(&upload_dir).unwrap();

    let mut uploaded_files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();

        if let Some(filename) = content_disposition.get_filename() {
            if !filename.ends_with(".flac") {
                continue;
            }

            let file_id = Uuid::new_v4();
            let filepath = upload_dir.join(format!("{}_{}", file_id, filename));

            let mut f = std_fs::File::create(&filepath)?;
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                f.write_all(&data)?;
            }

            uploaded_files.push(filepath.to_string_lossy().to_string());
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: format!("Uploaded {} files", uploaded_files.len()),
        data: Some(serde_json::json!({
            "files": uploaded_files,
            "count": uploaded_files.len()
        })),
    }))
}

async fn start_conversion(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mut status = state.conversion_status.lock().unwrap();

    if status.is_converting {
        return Ok(HttpResponse::Ok().json(ApiResponse {
            success: false,
            message: "Conversion already in progress".to_string(),
            data: None,
        }));
    }

    let upload_dir = PathBuf::from("/tmp/flac_uploads");
    let output_dir = PathBuf::from("/tmp/mp3_output");
    std_fs::create_dir_all(&output_dir).unwrap();

    let flac_files: Vec<PathBuf> = std_fs::read_dir(&upload_dir)
        .ok()
        .and_then(|entries| {
            Some(
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "flac"))
                    .collect()
            )
        })
        .unwrap_or_default();

    if flac_files.is_empty() {
        return Ok(HttpResponse::Ok().json(ApiResponse {
            success: false,
            message: "No FLAC files found to convert".to_string(),
            data: None,
        }));
    }

    status.is_converting = true;
    status.total_files = flac_files.len();
    status.processed_files = 0;
    status.progress = 0.0;
    status.log_messages.clear();
    status.log_messages.push("Starting conversion...".to_string());

    let state_clone = state.clone();

    tokio::spawn(async move {
        convert_files(flac_files, output_dir, state_clone).await;
    });

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Conversion started".to_string(),
        data: Some(serde_json::json!({
            "total_files": status.total_files
        })),
    }))
}

async fn convert_files(
    input_files: Vec<PathBuf>,
    output_folder: PathBuf,
    state: web::Data<AppState>,
) {
    let total_files = input_files.len() as f32;

    for (index, input_file) in input_files.iter().enumerate() {
        let file_name = input_file.file_name().unwrap().to_string_lossy();

        {
            let mut status = state.conversion_status.lock().unwrap();
            status.current_file = format!("Converting: {}", file_name);
        }

        let output_file = output_folder.join(
            input_file.file_stem().unwrap().to_string_lossy().to_string() + ".mp3"
        );

        let result = Command::new("ffmpeg")
            .arg("-i")
            .arg(input_file)
            .arg("-ab")
            .arg("320k")
            .arg("-map_metadata")
            .arg("0")
            .arg("-id3v2_version")
            .arg("3")
            .arg("-y")
            .arg(&output_file)
            .output();

        let mut status = state.conversion_status.lock().unwrap();

        match result {
            Ok(output) => {
                if output.status.success() {
                    status.log_messages.push(format!("✓ Successfully converted: {}", file_name));
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    status.log_messages.push(format!("✗ Failed to convert {}: {}", file_name, error_msg));
                }
            }
            Err(e) => {
                status.log_messages.push(format!("✗ Error converting {}: {}", file_name, e));
            }
        }

        let progress = (index + 1) as f32 / total_files;
        status.progress = progress;
        status.processed_files = index + 1;
    }

    let mut status = state.conversion_status.lock().unwrap();
    status.is_converting = false;
    status.current_file = "Conversion completed!".to_string();
    status.log_messages.push("🎉 All conversions completed!".to_string());
}

async fn get_status(state: web::Data<AppState>) -> Result<HttpResponse> {
    let status = state.conversion_status.lock().unwrap();
    Ok(HttpResponse::Ok().json(status.clone()))
}

async fn download_files() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Download endpoint".to_string(),
        data: None,
    }))
}

async fn clear_files() -> Result<HttpResponse> {
    let upload_dir = PathBuf::from("/tmp/flac_uploads");
    let output_dir = PathBuf::from("/tmp/mp3_output");

    let _ = std_fs::remove_dir_all(&upload_dir);
    let _ = std_fs::remove_dir_all(&output_dir);

    std_fs::create_dir_all(&upload_dir).ok();
    std_fs::create_dir_all(&output_dir).ok();

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "All files cleared".to_string(),
        data: None,
    }))
}

async fn health_check() -> Result<HttpResponse> {
    let ffmpeg_available = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    Ok(HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "Service is healthy".to_string(),
        data: Some(serde_json::json!({
            "ffmpeg_available": ffmpeg_available
        })),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting FLAC to MP3 Web Converter...");
    println!("Server will be available at http://0.0.0.0:8080");

    let app_state = web::Data::new(AppState {
        conversion_status: Arc::new(Mutex::new(ConversionStatus::default())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/api/upload", web::post().to(upload_files))
            .route("/api/convert", web::post().to(start_conversion))
            .route("/api/status", web::get().to(get_status))
            .route("/api/download", web::get().to(download_files))
            .route("/api/clear", web::post().to(clear_files))
            .route("/api/health", web::get().to(health_check))
            .service(fs::Files::new("/downloads", "/tmp/mp3_output").show_files_listing())
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
