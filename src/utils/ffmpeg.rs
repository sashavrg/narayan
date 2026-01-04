use std::path::Path;
use tokio::process::Command;

pub struct FFmpegCommand {
    input: String,
    output: String,
}

impl FFmpegCommand {
    pub fn new(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Self {
        Self {
            input: input.as_ref().to_string_lossy().to_string(),
            output: output.as_ref().to_string_lossy().to_string(),
        }
    }

    pub fn build(&self) -> Command {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(&[
            "-i",
            &self.input,
            "-ab",
            "320k", // 320kbps bitrate
            "-map_metadata",
            "0", // Preserve metadata
            "-id3v2_version",
            "3", // ID3v2 version 3
            "-progress",
            "pipe:1", // Output progress to stdout
            "-y", // Overwrite output file
            &self.output,
        ]);
        cmd
    }
}

pub async fn check_ffmpeg_installed() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
        .is_ok()
}
