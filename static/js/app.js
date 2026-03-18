import ConversionWebSocket from './websocket.js';
import { Animator } from './animator.js';
import { ToastManager } from './toast.js';
import { MusicBrowser } from './music-browser.js';

class NarayanApp {
    constructor() {
        this.ws = new ConversionWebSocket();
        this.jobs = new Map();
        this.pollInterval = null;
        this.animator = new Animator();
        this.toastManager = new ToastManager();
        this.musicBrowser = null;

        this.init();
    }

    async init() {
        this.animator.animatePageLoad();

        this.ws.connect();
        this.setupWebSocketHandlers();
        this.toastManager.setupConnectionListener();

        this.musicBrowser = new MusicBrowser({
            onJobCreated: (data) => this.handleConversionQueued(data),
            onError: (message) => this.toastManager.error(message),
            onInfo: (message) => this.toastManager.info(message),
        });
        await this.musicBrowser.init();

        await this.loadActiveJobs();
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
                console.error(`Failed to restore job ${jobId}:`, error);
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

    handleConversionQueued(data) {
        this.ws.subscribe(data.job_id);
        this.toastManager.success(`Job queued with ${data.file_count} file${data.file_count === 1 ? '' : 's'}.`);
        setTimeout(() => {
            this.fetchAndDisplayJob(data.job_id);
        }, 300);
    }

    displayJob(job) {
        const noJobsMsg = document.querySelector('.no-jobs');
        if (noJobsMsg) {
            this.animator.fadeOut(noJobsMsg, () => noJobsMsg.remove());
        }

        if (this.jobs.has(job.id)) {
            this.updateJobDisplay(job);
            return;
        }

        const template = document.getElementById('job-template');
        const jobCard = template.content.cloneNode(true);

        const cardDiv = jobCard.querySelector('.job-card');
        cardDiv.dataset.jobId = job.id;

        const title = jobCard.querySelector('.job-title');
        title.textContent = `Job ${job.id.substring(0, 8)}`;

        const status = jobCard.querySelector('.job-status');
        this.updateJobStatus(status, job.status);

        const jobsList = document.getElementById('jobs-list');
        jobsList.prepend(jobCard);

        const addedCard = jobsList.querySelector(`[data-job-id="${job.id}"]`);
        if (addedCard) {
            this.animator.animateJobCardEntrance(addedCard);
        }

        this.jobs.set(job.id, job);
        this.updateJobDisplay(job);
    }

    updateJobDisplay(job) {
        const card = document.querySelector(`[data-job-id="${job.id}"]`);
        if (!card) {
            return;
        }

        const status = card.querySelector('.job-status');
        const progressFill = card.querySelector('.progress-fill');
        const progressText = card.querySelector('.progress-text');
        const statusText = card.querySelector('.status-text');
        const downloadBtn = card.querySelector('.btn-download');

        this.updateJobStatus(status, job.status);

        if (job.status.type === 'processing') {
            const current = job.status.current_file || 0;
            const total = job.status.total || job.files.length;
            const percent = total > 0 ? Math.round((current / total) * 100) : 0;

            this.animator.animateProgress(progressFill, percent);
            progressText.textContent = `${percent}%`;
            statusText.textContent = `Processing file ${current + 1} of ${total}...`;
        } else if (job.status.type === 'complete') {
            this.animator.animateProgress(progressFill, 100);
            progressText.textContent = '100%';
            statusText.textContent = `Completed in ${job.status.duration || 0}s`;
            downloadBtn.style.display = 'block';
            downloadBtn.onclick = () => this.downloadBatch(job.id);
        } else if (job.status.type === 'failed') {
            statusText.textContent = `Failed: ${job.status.error || 'Unknown error'}`;
        }

        this.jobs.set(job.id, job);
    }

    updateJobStatus(statusEl, status) {
        const newClass = `job-status ${status.type}`;
        const newText = status.type.charAt(0).toUpperCase() + status.type.slice(1);

        if (statusEl.className !== newClass) {
            this.animator.animateStatusChange(statusEl, () => {
                statusEl.className = newClass;
                statusEl.textContent = newText;
            });
        }
    }

    updateJobProgress(jobId, percent, file) {
        const card = document.querySelector(`[data-job-id="${jobId}"]`);
        if (!card) {
            return;
        }

        const progressFill = card.querySelector('.progress-fill');
        const progressText = card.querySelector('.progress-text');
        const statusText = card.querySelector('.status-text');

        this.animator.animateProgress(progressFill, percent);
        progressText.textContent = `${Math.round(percent)}%`;
        statusText.textContent = `Converting ${file}...`;
    }

    markJobComplete(jobId) {
        this.toastManager.success('Conversion completed successfully.');
        this.fetchAndDisplayJob(jobId);
        this.ws.unsubscribe(jobId);
    }

    markJobFailed(jobId, error) {
        const card = document.querySelector(`[data-job-id="${jobId}"]`);
        if (!card) {
            return;
        }

        const status = card.querySelector('.job-status');
        const statusText = card.querySelector('.status-text');

        this.animator.animateStatusChange(status, () => {
            status.className = 'job-status failed';
            status.textContent = 'Failed';
        });

        statusText.textContent = `Error: ${error}`;
        this.toastManager.error(`Conversion failed: ${error}`);
        this.ws.unsubscribe(jobId);
    }

    downloadBatch(jobId) {
        window.location.href = `/api/download/batch/${jobId}`;
    }

    startPolling() {
        this.pollInterval = setInterval(async () => {
            for (const [jobId, job] of this.jobs) {
                if (job.status.type === 'processing' || job.status.type === 'pending') {
                    try {
                        await this.fetchAndDisplayJob(jobId);
                    } catch (error) {
                        console.error(`Polling failed for job ${jobId}:`, error);
                    }
                }
            }
        }, 5000);
    }
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        window.app = new NarayanApp();
    });
} else {
    window.app = new NarayanApp();
}
