use crate::error::{AppError, Result};
use crate::utils::FFmpegCommand;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub percent: f32,
}

pub struct ConversionService;

impl ConversionService {
    pub async fn convert_flac_to_mp3(
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        progress_tx: Option<mpsc::UnboundedSender<ProgressUpdate>>,
    ) -> Result<PathBuf> {
        let input_path = input.as_ref();
        let output_path = output.as_ref();

        info!(
            "Starting conversion: {} -> {}",
            input_path.display(),
            output_path.display()
        );

        // Validate input file exists and is FLAC
        if !input_path.exists() {
            return Err(AppError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Input file not found: {}", input_path.display()),
            )));
        }

        if !input_path
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("flac"))
        {
            return Err(AppError::InvalidFileType);
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Build FFmpeg command
        let ffmpeg_cmd = FFmpegCommand::new(input_path, output_path);
        let mut child = ffmpeg_cmd
            .build()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                error!("Failed to spawn FFmpeg process: {}", e);
                AppError::FFmpegNotFound
            })?;

        // Parse progress from stdout
        if let Some(stdout) = child.stdout.take() {
            let tx = progress_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(progress) = parse_ffmpeg_progress(&line) {
                        if let Some(ref sender) = tx {
                            let _ = sender.send(ProgressUpdate { percent: progress });
                        }
                    }
                }
            });
        }

        // Wait for completion
        let status = child.wait().await.map_err(|e| {
            error!("FFmpeg process error: {}", e);
            AppError::ConversionError(format!("Process error: {}", e))
        })?;

        // Check stderr for errors
        if !status.success() {
            let stderr = if let Some(stderr) = child.stderr {
                let mut stderr_reader = BufReader::new(stderr);
                let mut stderr_output = String::new();
                let mut stderr_lines = stderr_reader.lines();
                while let Ok(Some(line)) = stderr_lines.next_line().await {
                    stderr_output.push_str(&line);
                    stderr_output.push('\n');
                }
                stderr_output
            } else {
                "Unknown error".to_string()
            };

            error!("FFmpeg conversion failed: {}", stderr);
            return Err(AppError::ConversionError(format!(
                "FFmpeg failed with exit code {:?}: {}",
                status.code(),
                stderr
            )));
        }

        info!("Conversion completed successfully: {}", output_path.display());

        // Send final progress update
        if let Some(tx) = progress_tx {
            let _ = tx.send(ProgressUpdate { percent: 100.0 });
        }

        Ok(output_path.to_path_buf())
    }

    pub fn generate_output_filename(input: impl AsRef<Path>) -> String {
        let input_path = input.as_ref();
        input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| format!("{}.mp3", s))
            .unwrap_or_else(|| "output.mp3".to_string())
    }
}

fn parse_ffmpeg_progress(line: &str) -> Option<f32> {
    // FFmpeg progress format: "progress=continue" or "progress=end"
    // We also parse "out_time_ms" to calculate percentage
    // For simplicity, we'll look for "progress=continue" and estimate based on output

    if line.contains("progress=end") {
        return Some(100.0);
    }

    // Parse time progress (this is a simplified version)
    // In production, you'd want to parse "out_time_ms" and compare with duration
    if line.starts_with("out_time_ms=") {
        // This would require knowing the total duration first
        // For now, we'll return a simple increasing progress
        debug!("FFmpeg progress line: {}", line);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_output_filename() {
        let input = PathBuf::from("/path/to/song.flac");
        let output = ConversionService::generate_output_filename(&input);
        assert_eq!(output, "song.mp3");
    }

    #[test]
    fn test_parse_ffmpeg_progress_end() {
        let result = parse_ffmpeg_progress("progress=end");
        assert_eq!(result, Some(100.0));
    }
}
