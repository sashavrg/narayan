use eframe::egui;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::fs;

#[derive(Default)]
struct FlacToMp3Converter {
    input_files: Vec<PathBuf>,
    output_folder: Option<PathBuf>,
    conversion_progress: f32,
    current_file: String,
    is_converting: bool,
    conversion_receiver: Option<Receiver<ConversionMessage>>,
    conversion_sender: Option<Sender<ConversionMessage>>,
    log_messages: Vec<String>,
    total_files: usize,
    processed_files: usize,
}

#[derive(Debug)]
enum ConversionMessage {
    Progress(f32),
    CurrentFile(String),
    LogMessage(String),
    Finished,
    Error(String),
}

impl FlacToMp3Converter {
    fn new() -> Self {
        Self::default()
    }

    fn add_files(&mut self) {
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("FLAC Audio", &["flac"])
            .set_title("Select FLAC files")
            .pick_files()
        {
            for file in files {
                if !self.input_files.contains(&file) {
                    self.input_files.push(file);
                }
            }
        }
    }

    fn add_folder(&mut self) {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select folder containing FLAC files")
            .pick_folder()
        {
            if let Ok(entries) = fs::read_dir(&folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "flac") {
                        if !self.input_files.contains(&path) {
                            self.input_files.push(path);
                        }
                    }
                }
            }
        }
    }

    fn select_output_folder(&mut self) {
        if let Some(folder) = rfd::FileDialog::new()
            .set_title("Select output folder")
            .pick_folder()
        {
            self.output_folder = Some(folder);
        }
    }

    fn start_conversion(&mut self) {
        if self.input_files.is_empty() || self.output_folder.is_none() {
            return;
        }

        let (sender, receiver) = mpsc::channel();
        self.conversion_sender = Some(sender.clone());
        self.conversion_receiver = Some(receiver);
        
        let input_files = self.input_files.clone();
        let output_folder = self.output_folder.clone().unwrap();
        self.total_files = input_files.len();
        self.processed_files = 0;
        self.is_converting = true;
        self.conversion_progress = 0.0;
        self.log_messages.clear();

        thread::spawn(move || {
            Self::convert_files_thread(input_files, output_folder, sender);
        });
    }

    fn convert_files_thread(
        input_files: Vec<PathBuf>,
        output_folder: PathBuf,
        sender: Sender<ConversionMessage>,
    ) {
        let total_files = input_files.len() as f32;
        
        for (index, input_file) in input_files.iter().enumerate() {
            let file_name = input_file.file_name().unwrap().to_string_lossy();
            sender.send(ConversionMessage::CurrentFile(format!("Converting: {}", file_name))).ok();
            
            let output_file = output_folder.join(
                input_file.file_stem().unwrap().to_string_lossy().to_string() + ".mp3"
            );

            // Use ffmpeg to convert FLAC to MP3 with metadata preservation
            let result = Command::new("ffmpeg")
                .arg("-i")
                .arg(input_file)
                .arg("-ab")
                .arg("320k")
                .arg("-map_metadata")
                .arg("0")
                .arg("-id3v2_version")
                .arg("3")
                .arg("-y") // Overwrite output files
                .arg(&output_file)
                .output();

            match result {
                Ok(output) => {
                    if output.status.success() {
                        sender.send(ConversionMessage::LogMessage(
                            format!("✓ Successfully converted: {}", file_name)
                        )).ok();
                    } else {
                        let error_msg = String::from_utf8_lossy(&output.stderr);
                        sender.send(ConversionMessage::LogMessage(
                            format!("✗ Failed to convert {}: {}", file_name, error_msg)
                        )).ok();
                    }
                }
                Err(e) => {
                    sender.send(ConversionMessage::LogMessage(
                        format!("✗ Error converting {}: {}", file_name, e)
                    )).ok();
                }
            }

            let progress = (index + 1) as f32 / total_files;
            sender.send(ConversionMessage::Progress(progress)).ok();
        }

        sender.send(ConversionMessage::Finished).ok();
    }

    fn clear_files(&mut self) {
        self.input_files.clear();
    }

    fn remove_file(&mut self, index: usize) {
        if index < self.input_files.len() {
            self.input_files.remove(index);
        }
    }

    fn check_ffmpeg_availability() -> bool {
        Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

impl eframe::App for FlacToMp3Converter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process messages from conversion thread
        if let Some(receiver) = &self.conversion_receiver {
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    ConversionMessage::Progress(progress) => {
                        self.conversion_progress = progress;
                        self.processed_files = (progress * self.total_files as f32) as usize;
                    }
                    ConversionMessage::CurrentFile(file) => {
                        self.current_file = file;
                    }
                    ConversionMessage::LogMessage(msg) => {
                        self.log_messages.push(msg);
                    }
                    ConversionMessage::Finished => {
                        self.is_converting = false;
                        self.current_file = "Conversion completed!".to_string();
                        self.log_messages.push("🎉 All conversions completed!".to_string());
                    }
                    ConversionMessage::Error(err) => {
                        self.is_converting = false;
                        self.log_messages.push(format!("❌ Error: {}", err));
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("FLAC to MP3 Converter");
            ui.separator();

            // Check ffmpeg availability
            if !Self::check_ffmpeg_availability() {
                ui.colored_label(
                    egui::Color32::RED,
                    "⚠️ FFmpeg not found! Please install FFmpeg to use this application."
                );
                ui.separator();
            }

            // File selection section
            ui.horizontal(|ui| {
                if ui.button("Add FLAC Files").clicked() {
                    self.add_files();
                }
                if ui.button("Add Folder").clicked() {
                    self.add_folder();
                }
                if ui.button("Clear All").clicked() {
                    self.clear_files();
                }
            });

            // Output folder selection
            ui.horizontal(|ui| {
                ui.label("Output folder:");
                if let Some(folder) = &self.output_folder {
                    ui.label(folder.display().to_string());
                } else {
                    ui.label("No folder selected");
                }
                if ui.button("Select Output Folder").clicked() {
                    self.select_output_folder();
                }
            });

            ui.separator();

            // File list
            ui.label(format!("Selected files: {}", self.input_files.len()));
            
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    let mut to_remove = None;
                    for (index, file) in self.input_files.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.small_button("×").clicked() {
                                to_remove = Some(index);
                            }
                            ui.label(file.file_name().unwrap().to_string_lossy());
                        });
                    }
                    if let Some(index) = to_remove {
                        self.remove_file(index);
                    }
                });

            ui.separator();

            // Conversion controls
            ui.horizontal(|ui| {
                if ui.button("Start Conversion").clicked() && !self.is_converting {
                    self.start_conversion();
                }
                
                ui.label("Bitrate: 320kbps");
                ui.label("| Metadata: Preserved");
            });

            // Progress section
            if self.is_converting || self.conversion_progress > 0.0 {
                ui.separator();
                ui.label(&self.current_file);
                ui.add(egui::ProgressBar::new(self.conversion_progress).text(
                    format!("{}/{} files", self.processed_files, self.total_files)
                ));
            }

            // Log section
            if !self.log_messages.is_empty() {
                ui.separator();
                ui.label("Conversion Log:");
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for msg in &self.log_messages {
                            ui.label(msg);
                        }
                    });
            }
        });

        // Request repaint to keep UI responsive during conversion
        if self.is_converting {
            ctx.request_repaint();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("FLAC to MP3 Converter"),
        ..Default::default()
    };

    eframe::run_native(
        "FLAC to MP3 Converter",
        options,
        Box::new(|_cc| Box::new(FlacToMp3Converter::new())),
    )
}