use crate::error::{AppError, Result};
use crate::models::{
    AppState, LibraryBrowseResponse, LibraryConvertRequest, LibraryEntry, LibraryEntryKind,
    UploadResponse,
};
use axum::extract::{Query, State};
use axum::response::Json;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use tracing::{debug, info};

#[derive(Debug, Deserialize)]
pub struct LibraryBrowseQuery {
    pub path: Option<String>,
}

pub async fn browse_library(
    State(state): State<AppState>,
    Query(query): Query<LibraryBrowseQuery>,
) -> Result<Json<LibraryBrowseResponse>> {
    let root = canonical_library_root(&state).await?;
    let requested_path = query.path.unwrap_or_default();
    let current = resolve_safe_path(&root, &requested_path).await?;

    if !tokio::fs::metadata(&current).await?.is_dir() {
        return Err(AppError::InvalidPath(
            "Requested path is not a directory".to_string(),
        ));
    }

    let mut entries = Vec::new();
    let mut directory_count = 0;
    let mut file_count = 0;
    let mut dir = tokio::fs::read_dir(&current).await?;

    while let Some(entry) = dir.next_entry().await? {
        let file_type = entry.file_type().await?;
        if file_type.is_symlink() {
            continue;
        }

        let canonical = match tokio::fs::canonicalize(entry.path()).await {
            Ok(path) => path,
            Err(_) => continue,
        };

        if !canonical.starts_with(&root) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        if file_type.is_dir() {
            directory_count += 1;
            entries.push(LibraryEntry {
                name,
                path: to_relative_string(&root, &canonical),
                kind: LibraryEntryKind::Directory,
            });
            continue;
        }

        if file_type.is_file() && has_flac_extension(&canonical) {
            file_count += 1;
            entries.push(LibraryEntry {
                name,
                path: to_relative_string(&root, &canonical),
                kind: LibraryEntryKind::File,
            });
        }
    }

    entries.sort_by(|left, right| {
        let left_rank = kind_rank(&left.kind);
        let right_rank = kind_rank(&right.kind);

        left_rank
            .cmp(&right_rank)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });

    let current_relative = to_relative_string(&root, &current);
    let parent_path = parent_path(&current_relative);

    Ok(Json(LibraryBrowseResponse {
        root_path: state
            .job_manager
            .music_library_root()
            .to_string_lossy()
            .to_string(),
        current_path: current_relative,
        parent_path,
        entries,
        directory_count,
        file_count,
    }))
}

pub async fn convert_library_selection(
    State(state): State<AppState>,
    Json(payload): Json<LibraryConvertRequest>,
) -> Result<Json<UploadResponse>> {
    if payload.paths.is_empty() {
        return Err(AppError::NoFilesProvided);
    }

    let root = canonical_library_root(&state).await?;
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut files: Vec<(PathBuf, String)> = Vec::new();

    for raw_path in payload.paths {
        let selected_path = resolve_safe_path(&root, &raw_path).await?;
        let metadata = tokio::fs::metadata(&selected_path)
            .await
            .map_err(|_| AppError::InvalidPath(format!("Path does not exist: {}", raw_path)))?;

        if metadata.is_file() {
            collect_file(&selected_path, &mut seen, &mut files);
            continue;
        }

        if metadata.is_dir() {
            collect_flac_files_from_directory(&root, &selected_path, &mut seen, &mut files).await?;
            continue;
        }

        return Err(AppError::InvalidPath(format!(
            "Unsupported selection type: {}",
            raw_path
        )));
    }

    files.sort_by(|left, right| left.0.cmp(&right.0));

    if files.is_empty() {
        return Err(AppError::NoFilesProvided);
    }

    let file_count = files.len();
    let job_id = state.job_manager.create_job(files).await?;

    info!(
        "Created library conversion job {} with {} files",
        job_id, file_count
    );

    Ok(Json(UploadResponse { job_id, file_count }))
}

async fn collect_flac_files_from_directory(
    root: &Path,
    start_directory: &Path,
    seen: &mut HashSet<PathBuf>,
    files: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    let mut stack = vec![start_directory.to_path_buf()];

    while let Some(directory) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&directory).await?;

        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            if file_type.is_symlink() {
                continue;
            }

            let canonical = match tokio::fs::canonicalize(entry.path()).await {
                Ok(path) => path,
                Err(_) => continue,
            };

            if !canonical.starts_with(root) {
                continue;
            }

            if file_type.is_dir() {
                stack.push(canonical);
                continue;
            }

            if file_type.is_file() {
                collect_file(&canonical, seen, files);
            }
        }
    }

    Ok(())
}

fn collect_file(path: &Path, seen: &mut HashSet<PathBuf>, files: &mut Vec<(PathBuf, String)>) {
    if !has_flac_extension(path) {
        return;
    }

    let canonical = path.to_path_buf();
    if !seen.insert(canonical.clone()) {
        return;
    }

    let filename = canonical
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.flac")
        .to_string();

    files.push((canonical, filename));
}

fn has_flac_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("flac"))
        .unwrap_or(false)
}

fn to_relative_string(root: &Path, absolute: &Path) -> String {
    absolute
        .strip_prefix(root)
        .map(normalize_relative)
        .unwrap_or_default()
}

fn normalize_relative(path: &Path) -> String {
    let value = path.to_string_lossy().to_string();
    if value == "." {
        String::new()
    } else {
        value.replace('\\', "/")
    }
}

fn parent_path(current: &str) -> Option<String> {
    if current.is_empty() {
        return None;
    }

    Path::new(current).parent().map(normalize_relative)
}

async fn canonical_library_root(state: &AppState) -> Result<PathBuf> {
    let configured = state.job_manager.music_library_root();
    let root = tokio::fs::canonicalize(configured).await.map_err(|error| {
        AppError::Internal(format!(
            "Unable to access music library root '{}': {}",
            configured.display(),
            error
        ))
    })?;

    debug!("Using library root: {}", root.display());
    Ok(root)
}

async fn resolve_safe_path(root: &Path, raw_relative_path: &str) -> Result<PathBuf> {
    let relative = validate_relative_path(raw_relative_path)?;
    let candidate = root.join(relative);
    let canonical = tokio::fs::canonicalize(&candidate)
        .await
        .map_err(|_| AppError::InvalidPath(format!("Path not found: {}", raw_relative_path)))?;

    if !canonical.starts_with(root) {
        return Err(AppError::InvalidPath(
            "Path escapes the configured music library root".to_string(),
        ));
    }

    Ok(canonical)
}

fn validate_relative_path(raw_relative_path: &str) -> Result<PathBuf> {
    let candidate = raw_relative_path.trim();
    if candidate.is_empty() {
        return Ok(PathBuf::new());
    }

    let path = Path::new(candidate);
    if path.is_absolute() {
        return Err(AppError::InvalidPath(
            "Absolute paths are not allowed".to_string(),
        ));
    }

    let mut sanitized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(part) => sanitized.push(part),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::InvalidPath(
                    "Parent traversal and root prefixes are not allowed".to_string(),
                ));
            }
        }
    }

    Ok(sanitized)
}

fn kind_rank(kind: &LibraryEntryKind) -> u8 {
    match kind {
        LibraryEntryKind::Directory => 0,
        LibraryEntryKind::File => 1,
    }
}
