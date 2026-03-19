use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub struct FileManager;

impl FileManager {
    pub async fn create_zip(files: Vec<PathBuf>, output_path: impl AsRef<Path>) -> Result<PathBuf> {
        let output = output_path.as_ref().to_path_buf();
        info!("Creating ZIP archive: {}", output.display());

        let existing_files: Vec<PathBuf> = files
            .into_iter()
            .filter_map(|file_path| {
                if file_path.exists() {
                    Some(file_path)
                } else {
                    debug!("Skipping non-existent file: {}", file_path.display());
                    None
                }
            })
            .collect();

        if existing_files.is_empty() {
            return Err(AppError::ZipError(
                "No files available for ZIP creation".to_string(),
            ));
        }

        let output_for_task = output.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let zip_file = std::fs::File::create(&output_for_task)?;
            let mut zip_writer = ZipWriter::new(zip_file);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(0o644);

            for source_path in existing_files {
                let entry_name = source_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| {
                        AppError::ZipError(format!(
                            "Invalid output filename for path: {}",
                            source_path.display()
                        ))
                    })?;

                zip_writer
                    .start_file(entry_name, options)
                    .map_err(|e| AppError::ZipError(format!("Failed to start ZIP entry: {}", e)))?;

                let mut source_file = std::fs::File::open(&source_path)?;
                std::io::copy(&mut source_file, &mut zip_writer)?;
            }

            zip_writer
                .finish()
                .map_err(|e| AppError::ZipError(format!("Failed to finalize ZIP archive: {}", e)))?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::ZipError(format!("ZIP task failed to join: {}", e)))??;

        info!("ZIP archive created successfully: {}", output.display());
        Ok(output)
    }

    pub async fn cleanup_directory(path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            tokio::fs::remove_dir_all(path).await?;
            info!("Cleaned up directory: {}", path.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FileManager;

    #[tokio::test]
    async fn create_zip_writes_entries_without_external_binary() {
        let temp_dir = tempfile::tempdir().expect("create tempdir");
        let file_a = temp_dir.path().join("a.mp3");
        let file_b = temp_dir.path().join("b.mp3");
        let zip_path = temp_dir.path().join("batch.zip");

        std::fs::write(&file_a, b"first").expect("write first file");
        std::fs::write(&file_b, b"second").expect("write second file");

        FileManager::create_zip(vec![file_a, file_b], &zip_path)
            .await
            .expect("create zip");

        let zip_file = std::fs::File::open(&zip_path).expect("open zip");
        let mut archive = zip::ZipArchive::new(zip_file).expect("read zip");

        assert_eq!(archive.len(), 2);
        let first_name = {
            let entry = archive.by_index(0).expect("first entry");
            entry.name().to_string()
        };
        let second_name = {
            let entry = archive.by_index(1).expect("second entry");
            entry.name().to_string()
        };

        let mut names = vec![first_name, second_name];
        names.sort();
        assert_eq!(names, vec!["a.mp3", "b.mp3"]);
    }
}
