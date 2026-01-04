class FileUploader {
    constructor(dropZoneId, fileInputId, onUploadComplete) {
        this.dropZone = document.getElementById(dropZoneId);
        this.fileInput = document.getElementById(fileInputId);
        this.browseBtn = document.getElementById('browse-btn');
        this.onUploadComplete = onUploadComplete;
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
            });
        });

        ['dragleave', 'drop'].forEach(eventName => {
            this.dropZone.addEventListener(eventName, () => {
                this.dropZone.classList.remove('highlight');
            });
        });

        // Handle drop
        this.dropZone.addEventListener('drop', (e) => {
            const files = e.dataTransfer.files;
            this.handleFiles(files);
        });

        // Handle click to browse
        this.browseBtn.addEventListener('click', (e) => {
            e.preventDefault();
            this.fileInput.click();
        });

        this.dropZone.addEventListener('click', () => {
            this.fileInput.click();
        });

        // Handle file input change
        this.fileInput.addEventListener('change', (e) => {
            this.handleFiles(e.target.files);
        });
    }

    preventDefaults(e) {
        e.preventDefault();
        e.stopPropagation();
    }

    handleFiles(files) {
        if (files.length === 0) return;

        // Filter FLAC files
        const flacFiles = Array.from(files).filter(file =>
            file.name.toLowerCase().endsWith('.flac')
        );

        if (flacFiles.length === 0) {
            this.showError('Please select FLAC files only');
            return;
        }

        if (flacFiles.length !== files.length) {
            this.showWarning(`Filtered out ${files.length - flacFiles.length} non-FLAC files`);
        }

        this.uploadFiles(flacFiles);
    }

    async uploadFiles(files) {
        console.log('Uploading files:', files.length);

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
            console.log('Upload response:', data);

            // Clear file input
            this.fileInput.value = '';

            // Call completion callback
            if (this.onUploadComplete) {
                this.onUploadComplete(data);
            }

            this.showSuccess(`Uploaded ${data.file_count} files`);

        } catch (error) {
            console.error('Upload error:', error);
            this.showError(`Upload failed: ${error.message}`);
        }
    }

    showUploading(count) {
        // You could add a loading indicator here
        console.log(`Uploading ${count} files...`);
    }

    showSuccess(message) {
        console.log('Success:', message);
        // Could show a toast notification here
    }

    showError(message) {
        console.error('Error:', message);
        alert(message);
    }

    showWarning(message) {
        console.warn('Warning:', message);
    }
}

export default FileUploader;
