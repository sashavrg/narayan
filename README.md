# FLAC to MP3 Converter

A simple, efficient GUI application built in Rust for converting FLAC audio files to MP3 format at 320kbps while preserving metadata. Features batch processing and a clean, user-friendly interface.

## Features

- **High-Quality Conversion**: Converts FLAC to MP3 at 320kbps bitrate
- **Metadata Preservation**: Maintains all original metadata (artist, album, title, etc.)
- **Batch Processing**: Convert multiple files or entire folders at once
- **Progress Tracking**: Real-time progress updates during conversion
- **Cross-Platform**: Built for macOS (with potential for Linux/Windows)
- **Clean GUI**: Intuitive interface built with egui

## Prerequisites

### FFmpeg Installation

This application requires FFmpeg to be installed on your system.

**Fedora Linux:**
```bash
# Enable RPM Fusion repositories (if not already enabled)
sudo dnf install https://mirrors.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm
sudo dnf install https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm

# Install FFmpeg
sudo dnf install ffmpeg
```

**Other Linux distributions:**
- Ubuntu/Debian: `sudo apt install ffmpeg`
- Arch Linux: `sudo pacman -S ffmpeg`
- openSUSE: `sudo zypper install ffmpeg`

**Verify installation:**
```bash
ffmpeg -version
```

## Building from Source

### Requirements
- Rust 1.70+ (install from https://rustup.rs/)
- Development tools: `sudo dnf groupinstall "Development Tools"`
- System dependencies: `sudo dnf install pkg-config fontconfig-devel`

### Build Steps

1. **Clone the repository:**
```bash
git clone <repository-url>
cd flac-to-mp3-converter
```

2. **Build the application:**
```bash
cargo build --release
```

3. **Run the application:**
```bash
cargo run --release
```

### Creating Distribution Packages (Linux)

1. **Make the build script executable:**
```bash
chmod +x build_and_package.sh
```

2. **Run the build script:**
```bash
./build_and_package.sh
```

This will create:
- A standalone binary at `target/release/flac-to-mp3-converter`
- An AppImage directory structure at `target/release/flac-to-mp3-converter.AppDir`
- A tarball at `target/release/flac-to-mp3-converter-1.0.0-linux-[arch].tar.gz`
- RPM spec file for building RPM packages

3. **Optional: Create AppImage** (if you have appimagetool):
```bash
# Download appimagetool
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# Create AppImage
./appimagetool-x86_64.AppImage target/release/flac-to-mp3-converter.AppDir
```

4. **Optional: Build RPM package**:
```bash
# Copy source to rpmbuild directory
cp -r . rpmbuild/BUILD/flac-to-mp3-converter-1.0.0/
cd rpmbuild/BUILD/flac-to-mp3-converter-1.0.0/
tar -czf ../../SOURCES/flac-to-mp3-converter-1.0.0.tar.gz .
cd ../../..

# Build RPM
rpmbuild -ba rpmbuild/SPECS/flac-to-mp3-converter.spec
```

## Usage

1. **Launch the application**
2. **Add FLAC files** using one of these methods:
   - Click "Add FLAC Files" to select individual files
   - Click "Add Folder" to add all FLAC files from a directory
3. **Select output folder** where converted MP3s will be saved
4. **Start conversion** - the app will convert all files at 320kbps with metadata preserved
5. **Monitor progress** through the progress bar and log messages

## Technical Details

### Conversion Specifications
- **Output Format**: MP3
- **Bitrate**: 320kbps (constant bitrate)
- **Metadata**: ID3v2.3 tags preserved from source
- **Quality**: High-quality conversion using FFmpeg

### Dependencies
- `eframe`: Modern GUI framework
- `rfd`: Native file dialogs
- `egui`: Immediate mode GUI library

### Architecture
- **Frontend**: egui-based GUI with real-time updates
- **Backend**: Multi-threaded conversion using FFmpeg
- **Communication**: Channel-based messaging between GUI and conversion threads

## Code Signing (Optional)

For distribution, you may want to code sign the application:

1. **Get a Developer ID certificate** from Apple
2. **Uncomment the code signing section** in `build_and_package.sh`
3. **Update the signing identity** to match your certificate

## Troubleshooting

### Common Issues

**"FFmpeg not found" error:**
- Ensure FFmpeg is installed and available in PATH
- Test with `ffmpeg -version` in terminal

**Build errors:**
- Ensure Rust is up to date: `rustup update`
- Install development tools: `sudo dnf groupinstall "Development Tools"`
- Install system dependencies: `sudo dnf install pkg-config fontconfig-devel`

**Permission errors:**
- Make sure the build script is executable: `chmod +x build_and_package.sh`

### Performance Tips

- **Large batches**: The application processes files sequentially to avoid overwhelming the system
- **Disk space**: Ensure adequate free space (MP3s are typically 10-15% the size of FLACs)
- **Memory usage**: Minimal memory footprint due to streaming conversion

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with the excellent `egui` immediate mode GUI framework
- Uses FFmpeg for high-quality audio conversion
- Inspired by the need for a simple, efficient FLAC to MP3 converter