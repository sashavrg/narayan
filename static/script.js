let selectedFiles = [];
let statusInterval = null;

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    checkHealth();
    setupFileInput();
});

function setupFileInput() {
    const fileInput = document.getElementById('fileInput');
    fileInput.addEventListener('change', (e) => {
        const files = Array.from(e.target.files);
        selectedFiles = [...selectedFiles, ...files];
        updateFileList();
        e.target.value = ''; // Reset input
    });
}

function updateFileList() {
    const fileList = document.getElementById('fileList');

    if (selectedFiles.length === 0) {
        fileList.innerHTML = '<p style="text-align: center; color: #999; padding: 20px;">No files selected</p>';
        return;
    }

    fileList.innerHTML = selectedFiles.map((file, index) => `
        <div class="file-item">
            <span class="file-name">${file.name}</span>
            <button class="remove-btn" onclick="removeFile(${index})">Remove</button>
        </div>
    `).join('');
}

function removeFile(index) {
    selectedFiles.splice(index, 1);
    updateFileList();
}

function clearFiles() {
    if (confirm('Are you sure you want to clear all files?')) {
        selectedFiles = [];
        updateFileList();

        // Also clear server-side files
        fetch('/api/clear', { method: 'POST' })
            .then(response => response.json())
            .then(data => {
                console.log('Server files cleared:', data);
                hideElement('downloadCard');
                hideElement('logCard');
                hideElement('progressCard');
            })
            .catch(error => console.error('Error clearing files:', error));
    }
}

async function startConversion() {
    if (selectedFiles.length === 0) {
        alert('Please select FLAC files first!');
        return;
    }

    const convertBtn = document.getElementById('convertBtn');
    convertBtn.disabled = true;
    convertBtn.textContent = 'Uploading...';

    try {
        // Upload files
        const formData = new FormData();
        selectedFiles.forEach(file => {
            formData.append('files', file);
        });

        const uploadResponse = await fetch('/api/upload', {
            method: 'POST',
            body: formData
        });

        if (!uploadResponse.ok) {
            throw new Error('Upload failed');
        }

        convertBtn.textContent = 'Converting...';

        // Start conversion
        const convertResponse = await fetch('/api/convert', {
            method: 'POST'
        });

        if (!convertResponse.ok) {
            throw new Error('Conversion start failed');
        }

        // Show progress card
        showElement('progressCard');
        showElement('logCard');
        hideElement('downloadCard');

        // Start polling for status
        startStatusPolling();

    } catch (error) {
        console.error('Error:', error);
        alert('An error occurred: ' + error.message);
        convertBtn.disabled = false;
        convertBtn.textContent = 'Start Conversion';
    }
}

function startStatusPolling() {
    if (statusInterval) {
        clearInterval(statusInterval);
    }

    statusInterval = setInterval(async () => {
        try {
            const response = await fetch('/api/status');
            const status = await response.json();

            updateProgress(status);

            if (!status.is_converting && status.progress > 0) {
                clearInterval(statusInterval);
                statusInterval = null;

                const convertBtn = document.getElementById('convertBtn');
                convertBtn.disabled = false;
                convertBtn.textContent = 'Start Conversion';

                showElement('downloadCard');
            }
        } catch (error) {
            console.error('Error fetching status:', error);
        }
    }, 500); // Poll every 500ms
}

function updateProgress(status) {
    const currentFile = document.getElementById('currentFile');
    const progressBar = document.getElementById('progressBar');
    const progressText = document.getElementById('progressText');
    const logMessages = document.getElementById('logMessages');

    currentFile.textContent = status.current_file || 'Waiting...';

    const progressPercent = Math.round(status.progress * 100);
    progressBar.style.width = progressPercent + '%';
    progressText.textContent = `${status.processed_files} / ${status.total_files} files (${progressPercent}%)`;

    // Update log messages
    if (status.log_messages && status.log_messages.length > 0) {
        logMessages.innerHTML = status.log_messages
            .map(msg => `<div class="log-message">${escapeHtml(msg)}</div>`)
            .join('');

        // Auto-scroll to bottom
        logMessages.scrollTop = logMessages.scrollHeight;
    }
}

async function checkHealth() {
    try {
        const response = await fetch('/api/health');
        const data = await response.json();

        if (!data.data.ffmpeg_available) {
            showElement('ffmpeg-warning');
        }
    } catch (error) {
        console.error('Health check failed:', error);
    }
}

function showElement(id) {
    const element = document.getElementById(id);
    if (element) {
        element.classList.remove('hidden');
    }
}

function hideElement(id) {
    const element = document.getElementById(id);
    if (element) {
        element.classList.add('hidden');
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
