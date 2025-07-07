#!/bin/bash

# Cross-compile Build Script for FLAC to MP3 Converter
# This script cross-compiles from Fedora Linux to macOS and creates a DMG

set -e

APP_NAME="FLAC to MP3 Converter"
BUNDLE_ID="com.yourcompany.flac-to-mp3-converter"
VERSION="1.0.0"
BINARY_NAME="flac-to-mp3-converter"

echo "Cross-compiling FLAC to MP3 Converter for macOS from Fedora..."

# Check if macOS target is installed
if ! rustup target list --installed | grep -q "x86_64-apple-darwin"; then
    echo "Installing macOS target..."
    rustup target add x86_64-apple-darwin
fi

# Check if we also want Apple Silicon support
if ! rustup target list --installed | grep -q "aarch64-apple-darwin"; then
    echo "Installing Apple Silicon target..."
    rustup target add aarch64-apple-darwin
fi

# Clean previous builds
cargo clean

# Build for Intel Macs
echo "Building for Intel Macs (x86_64)..."
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon Macs
echo "Building for Apple Silicon (aarch64)..."
cargo build --release --target aarch64-apple-darwin

# Create universal binary using lipo (if available) or just use Intel version
echo "Creating universal binary..."
if command -v lipo &> /dev/null; then
    lipo -create \
        target/x86_64-apple-darwin/release/${BINARY_NAME} \
        target/aarch64-apple-darwin/release/${BINARY_NAME} \
        -output target/release/${BINARY_NAME}-universal
    BINARY_PATH="target/release/${BINARY_NAME}-universal"
else
    echo "Warning: lipo not available, using Intel-only binary"
    cp target/x86_64-apple-darwin/release/${BINARY_NAME} target/release/${BINARY_NAME}-universal
    BINARY_PATH="target/release/${BINARY_NAME}-universal"
fi

# Create app bundle structure
echo "Creating macOS app bundle..."
APP_BUNDLE="target/release/${APP_NAME}.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy the binary
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/${APP_NAME}"
chmod +x "$APP_BUNDLE/Contents/MacOS/${APP_NAME}"

# Create Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <false/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>flac</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>CFBundleTypeDescription</key>
            <string>FLAC Audio File</string>
        </dict>
    </array>
</dict>
</plist>
EOF

# Create a placeholder icon (replace this with actual .icns file)
cat > "$APP_BUNDLE/Contents/Resources/AppIcon.icns" << 'EOF'
# This is a placeholder - replace with actual .icns file
# You can create one from a PNG using: sips -s format icns icon.png --out AppIcon.icns
EOF

echo "App bundle created at: $APP_BUNDLE"

# Create DMG using Linux tools
echo "Creating DMG..."
DMG_NAME="FLAC-to-MP3-Converter-${VERSION}.dmg"
DMG_PATH="target/release/${DMG_NAME}"

# Remove existing DMG
rm -f "$DMG_PATH"

# Create temporary DMG directory
TEMP_DMG_DIR="target/release/dmg_temp"
rm -rf "$TEMP_DMG_DIR"
mkdir -p "$TEMP_DMG_DIR"

# Copy app bundle to temp directory
cp -R "$APP_BUNDLE" "$TEMP_DMG_DIR/"

# Create a symbolic link to Applications (this won't work on Linux but will on macOS)
# We'll create a note file instead
cat > "$TEMP_DMG_DIR/INSTALL.txt" << EOF
Installation Instructions:
1. Drag "${APP_NAME}.app" to your Applications folder
2. Install FFmpeg using Homebrew: brew install ffmpeg
3. Launch the application from Applications

Note: This application requires FFmpeg to be installed on your Mac.
EOF

# Calculate size needed for DMG
SIZE=$(du -sm "$TEMP_DMG_DIR" | cut -f1)
SIZE=$((SIZE + 10)) # Add some padding

# Create DMG using Linux tools (requires libdmg-hfsplus or similar)
if command -v dmg &> /dev/null; then
    echo "Creating DMG with dmg tool..."
    dmg "$TEMP_DMG_DIR" "$DMG_PATH"
elif command -v genisoimage &> /dev/null; then
    echo "Creating ISO image (DMG alternative)..."
    genisoimage -V "${APP_NAME}" -D -R -apple -no-pad -o "${DMG_PATH%.dmg}.iso" "$TEMP_DMG_DIR"
    echo "Created ISO: ${DMG_PATH%.dmg}.iso"
else
    echo "Creating tar.gz archive (DMG alternative)..."
    tar -czf "${DMG_PATH%.dmg}.tar.gz" -C "$TEMP_DMG_DIR" .
    echo "Created archive: ${DMG_PATH%.dmg}.tar.gz"
    echo ""
    echo "To create a proper DMG, transfer the app bundle to a Mac and run:"
    echo "hdiutil create -volname '${APP_NAME}' -srcfolder '$TEMP_DMG_DIR' -ov -format UDZO '${DMG_PATH}'"
fi

# Clean up temp directory
rm -rf "$TEMP_DMG_DIR"

echo "Cross-compilation complete!"
echo "App Bundle: $APP_BUNDLE"
echo "Intel Binary: target/x86_64-apple-darwin/release/${BINARY_NAME}"
echo "Apple Silicon Binary: target/aarch64-apple-darwin/release/${BINARY_NAME}"
if [ -f "$DMG_PATH" ]; then
    echo "DMG: $DMG_PATH"
fi

# Instructions for macOS users
echo ""
echo "IMPORTANT: This application requires FFmpeg to be installed on the target Mac."
echo "macOS users can install FFmpeg using Homebrew:"
echo "  brew install ffmpeg"
echo ""
echo "Or download from: https://ffmpeg.org/download.html"
echo ""
echo "If you need a proper DMG, transfer the .app bundle to a Mac and use:"
echo "hdiutil create -volname '${APP_NAME}' -srcfolder target/release/dmg_temp -ov -format UDZO '${DMG_PATH}'"