class FileUploader {
    constructor(dropZoneId, fileInputId, onUploadComplete, toastManager = null, animator = null) {
        this.dropZone = document.getElementById(dropZoneId);
        this.fileInput = document.getElementById(fileInputId);
        this.browseBtn = document.getElementById('browse-btn');
        this.onUploadComplete = onUploadComplete;
        this.toastManager = toastManager;
        this.animator = animator;
        this.init();
    }

    init() {
        // Prevent default drag behaviors
        ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
            this.dropZone.addEventListener(eventName, this.preventDefaults, false);
            document.body.addEventListener(eventName, this.preventDefaults, false);
        });

        // Highlight drop zone on drag
        ['dragenter', 'dragover'].forEach(eventName => {
            this.dropZone.addEventListener(eventName, () => {
                this.dropZone.classList.add('highlight');
                // Animate icon pulse on drag
                if (this.animator) {
                    const dropZoneGlass = this.dropZone.closest('.drop-zone-glass');
                    this.animator.animateDropZonePulse(dropZoneGlass || this.dropZone);
                }
            });
        });

        ['dragleave', 'drop'].forEach(eventName => {
            this.dropZone.addEventListener(eventName, () => {
                this.dropZone.classList.remove('highlight');
                // Stop pulse animation
                if (this.animator) {
                    const dropZoneGlass = this.dropZone.closest('.drop-zone-glass');
                    this.animator.stopDropZonePulse(dropZoneGlass || this.dropZone);
                }
            });
        });

        // Handle drop
        this.dropZone.addEventListener('drop', (e) => {
            const files = e.dataTransfer.files;
            this.handleFiles(files);
        });

        // Label handles file input trigger automatically on all platforms
        // No click handler needed for browse button (it's a label now)

        this.dropZone.addEventListener('click', (e) => {
            // Don't trigger if clicking on the browse label or file input
            if (e.target !== this.browseBtn &&
                !this.browseBtn.contains(e.target) &&
                e.target !== this.fileInput) {
                this.fileInput.click();
            }
        });

        // Handle file input change
        this.fileInput.addEventListener('change', (e) => {
            console.log('File input changed, files:', e.target.files);
            this.handleFiles(e.target.files);
        });
    }

    preventDefaults(e) {
        e.preventDefault();
        e.stopPropagation();
    }

    handleFiles(files) {
        console.log('handleFiles called with:', files);
        if (files.length === 0) {
            console.log('No files selected');
            return;
        }

        // Show selection confirmation
        if (this.toastManager) {
            this.toastManager.info(`Selected ${files.length} file${files.length > 1 ? 's' : ''}`);
        }

        // Filter FLAC files
        const flacFiles = Array.from(files).filter(file =>
            file.name.toLowerCase().endsWith('.flac')
        );

        if (flacFiles.length === 0) {
            this.showError('Please select FLAC files only (.flac extension required)');
            return;
        }

        if (flacFiles.length !== files.length) {
            this.showWarning(`Filtered out ${files.length - flacFiles.length} non-FLAC file${files.length - flacFiles.length > 1 ? 's' : ''}`);
        }

        this.uploadFiles(flacFiles);
    }

    async uploadFiles(files) {
        const formData = new FormData();
        files.forEach(file => {
            formData.append('files', file);
        });

        try {
            // Show uploading status
            this.showUploading(files.length);

            const response = await fetch('/api/upload', {
                method: 'POST',
                body: formData
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.error || 'Upload failed');
            }

            const data = await response.json();

            // Clear file input
            this.fileInput.value = '';

            // Call completion callback
            if (this.onUploadComplete) {
                this.onUploadComplete(data);
            }

            this.showSuccess(`Started conversion of ${data.file_count} file${data.file_count > 1 ? 's' : ''}`);

        } catch (error) {
            console.error('Upload error:', error);
            this.showError(`Upload failed: ${error.message}`);
        }
    }

    showUploading(count) {
        if (this.toastManager) {
            this.toastManager.info(`Uploading ${count} file${count > 1 ? 's' : ''}...`);
        }
    }

    showSuccess(message) {
        if (this.toastManager) {
            this.toastManager.success(message);
        }
    }

    showError(message) {
        if (this.toastManager) {
            this.toastManager.error(message);
        } else {
            alert(message);
        }
    }

    showWarning(message) {
        if (this.toastManager) {
            this.toastManager.warning(message);
        }
    }
}

export default FileUploader;
