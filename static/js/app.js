import ConversionWebSocket from './websocket.js';
import FileUploader from './uploader.js';

class NarayanApp {
    constructor() {
        this.ws = new ConversionWebSocket();
        this.jobs = new Map();
        this.pollInterval = null;
        this.init();
    }

    init() {
        // Initialize WebSocket
        this.ws.connect();
        this.setupWebSocketHandlers();

        // Initialize uploader
        this.uploader = new FileUploader(
            'drop-zone',
            'file-input',
            (data) => this.handleUploadComplete(data)
        );

        // Load existing jobs from localStorage
        this.loadActiveJobs();

        // Poll for job status updates
        this.startPolling();
    }

    setupWebSocketHandlers() {
        this.ws.on('progress', (message) => {
            this.updateJobProgress(message.job_id, message.percent, message.file);
        });

        this.ws.on('complete', (message) => {
            this.markJobComplete(message.job_id);
        });

        this.ws.on('error', (message) => {
            this.markJobFailed(message.job_id, message.message);
        });
    }

    async loadActiveJobs() {
        const activeJobIds = this.ws.getActiveJobs();

        for (const jobId of activeJobIds) {
            try {
                await this.fetchAndDisplayJob(jobId);
            } catch (error) {
                console.error(`Failed to load job ${jobId}:`, error);
                // Remove invalid job from active list
                this.ws.unsubscribe(jobId);
            }
        }
    }

    async fetchAndDisplayJob(jobId) {
        const response = await fetch(`/api/jobs/${jobId}`);
        if (!response.ok) {
            throw new Error('Job not found');
        }

        const job = await response.json();
        this.displayJob(job);
        this.ws.subscribe(jobId);
    }

    handleUploadComplete(data) {
        console.log('Upload complete:', data);

        // Subscribe to job updates
        this.ws.subscribe(data.job_id);

        // Fetch and display job
        setTimeout(() => {
            this.fetchAndDisplayJob(data.job_id);
        }, 500);
    }

    displayJob(job) {
        // Remove "no jobs" message
        const noJobsMsg = document.querySelector('.no-jobs');
        if (noJobsMsg) {
            noJobsMsg.remove();
        }

        // Check if job already displayed
        if (this.jobs.has(job.id)) {
            this.updateJobDisplay(job);
            return;
        }

        // Create job card
        const template = document.getElementById('job-template');
        const jobCard = template.content.cloneNode(true);

        const cardDiv = jobCard.querySelector('.job-card');
        cardDiv.dataset.jobId = job.id;

        const title = jobCard.querySelector('.job-title');
        title.textContent = `Job ${job.id.substring(0, 8)}`;

        const status = jobCard.querySelector('.job-status');
        this.updateJobStatus(status, job.status);

        // Add to jobs list
        const jobsList = document.getElementById('jobs-list');
        jobsList.prepend(jobCard);

        // Store job reference
        this.jobs.set(job.id, job);

        // Update initial state
        this.updateJobDisplay(job);
    }

    updateJobDisplay(job) {
        const card = document.querySelector(`[data-job-id="${job.id}"]`);
        if (!card) return;

        const status = card.querySelector('.job-status');
        const progressFill = card.querySelector('.progress-fill');
        const progressText = card.querySelector('.progress-text');
        const statusText = card.querySelector('.status-text');
        const downloadBtn = card.querySelector('.btn-download');

        this.updateJobStatus(status, job.status);

        // Update progress based on status
        if (job.status.type === 'processing') {
            const current = job.status.current_file || 0;
            const total = job.status.total || job.files.length;
            const percent = total > 0 ? Math.round((current / total) * 100) : 0;

            progressFill.style.width = `${percent}%`;
            progressText.textContent = `${percent}%`;
            statusText.textContent = `Processing file ${current + 1} of ${total}...`;
        } else if (job.status.type === 'complete') {
            progressFill.style.width = '100%';
            progressText.textContent = '100%';
            statusText.textContent = `Completed in ${job.status.duration || 0}s`;
            downloadBtn.style.display = 'block';

            // Setup download handler
            downloadBtn.onclick = () => this.downloadBatch(job.id);
        } else if (job.status.type === 'failed') {
            statusText.textContent = `Failed: ${job.status.error || 'Unknown error'}`;
        }

        // Update job in map
        this.jobs.set(job.id, job);
    }

    updateJobStatus(statusEl, status) {
        statusEl.className = 'job-status ' + status.type;
        statusEl.textContent = status.type.charAt(0).toUpperCase() + status.type.slice(1);
    }

    updateJobProgress(jobId, percent, file) {
        const card = document.querySelector(`[data-job-id="${jobId}"]`);
        if (!card) return;

        const progressFill = card.querySelector('.progress-fill');
        const progressText = card.querySelector('.progress-text');
        const statusText = card.querySelector('.status-text');

        progressFill.style.width = `${percent}%`;
        progressText.textContent = `${Math.round(percent)}%`;
        statusText.textContent = `Converting ${file}...`;
    }

    markJobComplete(jobId) {
        console.log('Job complete:', jobId);
        this.fetchAndDisplayJob(jobId);
        this.ws.unsubscribe(jobId);
    }

    markJobFailed(jobId, error) {
        const card = document.querySelector(`[data-job-id="${jobId}"]`);
        if (!card) return;

        const status = card.querySelector('.job-status');
        const statusText = card.querySelector('.status-text');

        status.className = 'job-status failed';
        status.textContent = 'Failed';
        statusText.textContent = `Error: ${error}`;

        this.ws.unsubscribe(jobId);
    }

    downloadBatch(jobId) {
        window.location.href = `/api/download/batch/${jobId}`;
    }

    startPolling() {
        // Poll every 5 seconds for job updates
        this.pollInterval = setInterval(async () => {
            for (const [jobId, job] of this.jobs) {
                if (job.status.type === 'processing' || job.status.type === 'pending') {
                    try {
                        await this.fetchAndDisplayJob(jobId);
                    } catch (error) {
                        console.error(`Failed to poll job ${jobId}:`, error);
                    }
                }
            }
        }, 5000);
    }

    stopPolling() {
        if (this.pollInterval) {
            clearInterval(this.pollInterval);
            this.pollInterval = null;
        }
    }
}

// Initialize app when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.app = new NarayanApp();
    });
} else {
    window.app = new NarayanApp();
}
