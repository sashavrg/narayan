/**
 * ToastManager - Glassmorphic toast notification system
 * Replaces alerts and console feedback with elegant notifications
 */

export class ToastManager {
    constructor() {
        this.container = document.getElementById('toast-container');
        this.activeToasts = new Set();
        this.maxToasts = 5;

        if (!this.container) {
            console.warn('Toast container not found');
        }
    }

    /**
     * Show a toast notification
     * @param {string} message - The message to display
     * @param {string} type - Toast type: 'connected', 'disconnected', 'error', 'warning', 'info'
     * @param {number} duration - Duration in ms (0 for persistent)
     * @returns {HTMLElement} The toast element
     */
    show(message, type = 'info', duration = 4000) {
        if (!this.container) return null;

        // Limit number of concurrent toasts
        if (this.activeToasts.size >= this.maxToasts) {
            const oldest = Array.from(this.activeToasts)[0];
            this.dismiss(oldest);
        }

        // Create toast element
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;
        toast.setAttribute('role', 'alert');
        toast.setAttribute('aria-live', 'polite');

        // Create toast content
        const content = document.createElement('div');
        content.className = 'toast-content';

        const icon = this.getIcon(type);
        const messageEl = document.createElement('span');
        messageEl.className = 'toast-message';
        messageEl.textContent = message;

        content.appendChild(icon);
        content.appendChild(messageEl);

        // Create dismiss button
        const dismissBtn = document.createElement('button');
        dismissBtn.className = 'toast-dismiss';
        dismissBtn.textContent = '×';
        dismissBtn.setAttribute('aria-label', 'Dismiss notification');
        dismissBtn.onclick = () => this.dismiss(toast);

        toast.appendChild(content);
        toast.appendChild(dismissBtn);

        // Add to container and animate in
        this.container.appendChild(toast);
        this.activeToasts.add(toast);

        // Trigger entrance animation
        requestAnimationFrame(() => {
            this.animateIn(toast);
        });

        // Auto-dismiss after duration
        if (duration > 0) {
            setTimeout(() => {
                this.dismiss(toast);
            }, duration);
        }

        return toast;
    }

    /**
     * Get icon SVG for toast type
     * @param {string} type - Toast type
     * @returns {HTMLElement}
     */
    getIcon(type) {
        const icon = document.createElement('div');
        icon.className = 'toast-icon';

        const svgNS = 'http://www.w3.org/2000/svg';
        const svg = document.createElementNS(svgNS, 'svg');
        svg.setAttribute('fill', 'none');
        svg.setAttribute('viewBox', '0 0 24 24');
        svg.setAttribute('stroke', 'currentColor');

        const path = document.createElementNS(svgNS, 'path');
        path.setAttribute('stroke-linecap', 'round');
        path.setAttribute('stroke-linejoin', 'round');
        path.setAttribute('stroke-width', '2');

        switch (type) {
            case 'connected':
                path.setAttribute('d', 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z');
                break;
            case 'disconnected':
            case 'error':
                path.setAttribute('d', 'M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z');
                break;
            case 'warning':
                path.setAttribute('d', 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z');
                break;
            default: // info
                path.setAttribute('d', 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z');
        }

        svg.appendChild(path);
        icon.appendChild(svg);
        return icon;
    }

    /**
     * Animate toast entrance
     * @param {HTMLElement} toast - Toast element
     */
    animateIn(toast) {
        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            toast.style.opacity = '1';
            toast.style.transform = 'translateX(0)';
            return;
        }

        if (typeof anime !== 'undefined') {
            anime({
                targets: toast,
                opacity: [0, 1],
                translateX: [100, 0],
                duration: 400,
                easing: 'cubicBezier(0.4, 0, 0.2, 1)'
            });
        } else {
            toast.style.opacity = '1';
            toast.style.transform = 'translateX(0)';
        }
    }

    /**
     * Dismiss a toast with animation
     * @param {HTMLElement} toast - Toast element to dismiss
     */
    dismiss(toast) {
        if (!toast || !this.activeToasts.has(toast)) return;

        const complete = () => {
            if (toast.parentNode) {
                toast.parentNode.removeChild(toast);
            }
            this.activeToasts.delete(toast);
        };

        if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
            complete();
            return;
        }

        if (typeof anime !== 'undefined') {
            anime({
                targets: toast,
                opacity: [1, 0],
                translateX: [0, 100],
                duration: 300,
                easing: 'cubicBezier(0.4, 0, 0.2, 1)',
                complete
            });
        } else {
            complete();
        }
    }

    /**
     * Show success toast
     * @param {string} message
     * @returns {HTMLElement}
     */
    success(message) {
        return this.show(message, 'connected', 4000);
    }

    /**
     * Show error toast
     * @param {string} message
     * @returns {HTMLElement}
     */
    error(message) {
        return this.show(message, 'error', 5000);
    }

    /**
     * Show warning toast
     * @param {string} message
     * @returns {HTMLElement}
     */
    warning(message) {
        return this.show(message, 'warning', 4000);
    }

    /**
     * Show info toast
     * @param {string} message
     * @returns {HTMLElement}
     */
    info(message) {
        return this.show(message, 'info', 3000);
    }

    /**
     * Setup WebSocket connection status listener
     * Replaces the old connection status element
     */
    setupConnectionListener() {
        window.addEventListener('ws-connection-status', (event) => {
            const { isConnected } = event.detail;

            if (isConnected) {
                this.show('Connected to server', 'connected', 3000);
            } else {
                this.show('Disconnected from server', 'disconnected', 0);
            }
        });
    }

    /**
     * Clear all active toasts
     */
    clearAll() {
        const toasts = Array.from(this.activeToasts);
        toasts.forEach(toast => this.dismiss(toast));
    }

    /**
     * Destroy the toast manager
     */
    destroy() {
        this.clearAll();
        this.activeToasts.clear();
    }
}
