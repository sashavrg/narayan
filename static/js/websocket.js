class ConversionWebSocket {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 2000;
        this.messageHandlers = new Map();
        this.isConnecting = false;
    }

    connect() {
        if (this.isConnecting || (this.ws && this.ws.readyState === WebSocket.OPEN)) {
            return;
        }

        this.isConnecting = true;
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;

        console.log('Connecting to WebSocket:', wsUrl);

        try {
            this.ws = new WebSocket(wsUrl);
            this.setupEventHandlers();
        } catch (error) {
            console.error('WebSocket connection error:', error);
            this.isConnecting = false;
            this.scheduleReconnect();
        }
    }

    setupEventHandlers() {
        this.ws.onopen = () => {
            console.log('WebSocket connected');
            this.isConnecting = false;
            this.reconnectAttempts = 0;
            this.updateConnectionStatus(true);

            // Resubscribe to active jobs
            const activeJobs = this.getActiveJobs();
            activeJobs.forEach(jobId => this.subscribe(jobId));
        };

        this.ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                this.handleMessage(message);
            } catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            this.updateConnectionStatus(false);
        };

        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
            this.isConnecting = false;
            this.updateConnectionStatus(false);
            this.scheduleReconnect();
        };
    }

    scheduleReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            setTimeout(() => {
                this.reconnectAttempts++;
                console.log(`Reconnecting... (attempt ${this.reconnectAttempts})`);
                this.connect();
            }, this.reconnectDelay);
        } else {
            console.error('Max reconnection attempts reached');
        }
    }

    subscribe(jobId) {
        if (!jobId) return;

        const message = {
            type: 'subscribe',
            job_id: jobId
        };
        this.send(message);

        // Store in localStorage
        const activeJobs = this.getActiveJobs();
        if (!activeJobs.includes(jobId)) {
            activeJobs.push(jobId);
            localStorage.setItem('activeJobs', JSON.stringify(activeJobs));
        }
    }

    unsubscribe(jobId) {
        const message = {
            type: 'unsubscribe',
            job_id: jobId
        };
        this.send(message);

        // Remove from localStorage
        const activeJobs = this.getActiveJobs();
        const filtered = activeJobs.filter(id => id !== jobId);
        localStorage.setItem('activeJobs', JSON.stringify(filtered));
    }

    send(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.warn('WebSocket is not open. Message not sent:', message);
        }
    }

    handleMessage(message) {
        console.log('WebSocket message received:', message);

        const handler = this.messageHandlers.get(message.type);
        if (handler) {
            handler(message);
        }
    }

    on(messageType, handler) {
        this.messageHandlers.set(messageType, handler);
    }

    updateConnectionStatus(isConnected) {
        const statusEl = document.getElementById('connection-status');
        if (statusEl) {
            statusEl.className = 'connection-status ' + (isConnected ? 'connected' : 'disconnected');
            statusEl.textContent = isConnected ? 'Connected' : 'Disconnected';

            if (isConnected) {
                setTimeout(() => {
                    statusEl.style.display = 'none';
                }, 3000);
            }
        }
    }

    getActiveJobs() {
        try {
            return JSON.parse(localStorage.getItem('activeJobs') || '[]');
        } catch {
            return [];
        }
    }

    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
    }
}

export default ConversionWebSocket;
