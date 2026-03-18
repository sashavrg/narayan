/**
 * Animator - Centralized anime.js animation controller
 * Provides subtle, refined animations for the Narayan UI
 */

import { animate, createTimeline } from 'animejs';

export class Animator {
    constructor() {
        this.animations = new Map();
        this.isPaused = false;
        this.setupVisibilityListener();
    }

    /**
     * Setup tab visibility listener for performance optimization
     */
    setupVisibilityListener() {
        document.addEventListener('visibilitychange', () => {
            if (document.hidden) {
                this.pauseAll();
            } else {
                this.resumeAll();
            }
        });
    }

    /**
     * Page load entrance animation sequence
     * Staggered fade-in: header -> upload section -> jobs section
     */
    animatePageLoad() {
        // Check if reduced motion is preferred
        if (this.shouldReduceMotion()) {
            this.setElementsVisible();
            return;
        }

        // Animate header
        animate('.ethereal-header', {
            opacity: [0, 1],
            translateY: [-30, 0],
            duration: 800,
            ease: 'easeOutQuad'
        });

        // Animate upload section (staggered)
        animate('.upload-section', {
            opacity: [0, 1],
            translateY: [20, 0],
            duration: 800,
            delay: 200,
            ease: 'easeOutQuad'
        });

        // Animate jobs section (staggered)
        animate('.jobs-section', {
            opacity: [0, 1],
            translateY: [20, 0],
            duration: 800,
            delay: 400,
            ease: 'easeOutQuad'
        });
    }

    /**
     * Set elements visible immediately (for reduced motion)
     */
    setElementsVisible() {
        const elements = document.querySelectorAll('.ethereal-header, .upload-section, .jobs-section');
        elements.forEach(el => {
            el.style.opacity = '1';
            el.style.transform = 'none';
        });
    }

    /**
     * Animate job card entrance
     * @param {HTMLElement} cardElement - The job card element
     */
    animateJobCardEntrance(cardElement) {
        if (!cardElement || this.shouldReduceMotion()) {
            if (cardElement) {
                cardElement.style.opacity = '1';
                cardElement.style.transform = 'none';
            }
            return;
        }

        const animation = animate(cardElement, {
            opacity: [0, 1],
            translateY: [20, 0],
            scale: [0.95, 1],
            duration: 600,
            ease: 'easeOutQuad'
        });

        this.animations.set(cardElement, animation);
    }

    /**
     * Animate progress bar
     * @param {HTMLElement} progressElement - The progress fill element
     * @param {number} targetPercent - Target percentage (0-100)
     */
    animateProgress(progressElement, targetPercent) {
        if (!progressElement) return;

        // Clamp percentage between 0 and 100
        const percent = Math.max(0, Math.min(100, targetPercent));

        if (this.shouldReduceMotion()) {
            progressElement.style.width = `${percent}%`;
            return;
        }

        // Cancel existing animation if any
        const existingAnimation = this.animations.get(progressElement);
        if (existingAnimation) {
            existingAnimation.pause();
        }

        const animation = animate(progressElement, {
            width: `${percent}%`,
            duration: 800,
            ease: 'easeOutExpo'
        });

        this.animations.set(progressElement, animation);
    }

    /**
     * Animate status badge change with morph effect
     * @param {HTMLElement} statusElement - The status badge element
     * @param {Function} callback - Function to call at midpoint to change content
     */
    animateStatusChange(statusElement, callback) {
        if (!statusElement) return;

        if (this.shouldReduceMotion()) {
            if (callback) callback();
            return;
        }

        const tl = createTimeline({
            defaults: {
                ease: 'easeOutQuad'
            }
        });

        // Phase 1: Shrink and fade
        tl.add(statusElement, {
            scale: [1, 0.9],
            opacity: [1, 0.7],
            duration: 150
        }, 0);

        // Phase 2: Change content at midpoint
        tl.add(statusElement, {
            scale: [0.9, 1.05],
            opacity: [0.7, 1],
            duration: 150,
            onBegin: () => {
                if (callback) callback();
            }
        });

        // Phase 3: Settle
        tl.add(statusElement, {
            scale: [1.05, 1],
            duration: 200
        });

        this.animations.set(statusElement, tl);
    }

    /**
     * Fade out element
     * @param {HTMLElement} element - Element to fade out
     * @param {Function} callback - Optional callback when complete
     */
    fadeOut(element, callback) {
        if (!element) return;

        if (this.shouldReduceMotion()) {
            element.style.opacity = '0';
            if (callback) callback();
            return;
        }

        const animation = animate(element, {
            opacity: [1, 0],
            duration: 300,
            ease: 'easeOutQuad',
            onComplete: () => {
                if (callback) callback();
            }
        });

        this.animations.set(element, animation);
    }

    /**
     * Fade in element
     * @param {HTMLElement} element - Element to fade in
     * @param {Function} callback - Optional callback when complete
     */
    fadeIn(element, callback) {
        if (!element) return;

        if (this.shouldReduceMotion()) {
            element.style.opacity = '1';
            if (callback) callback();
            return;
        }

        const animation = animate(element, {
            opacity: [0, 1],
            duration: 300,
            ease: 'easeInQuad',
            onComplete: () => {
                if (callback) callback();
            }
        });

        this.animations.set(element, animation);
    }

    /**
     * Pulse animation for drop zone icon on drag
     * @param {HTMLElement} dropZoneElement - The drop zone container
     */
    animateDropZonePulse(dropZoneElement) {
        if (!dropZoneElement || this.shouldReduceMotion()) return;

        const icon = dropZoneElement.querySelector('.upload-icon');
        if (!icon) return;

        // Cancel existing pulse if any
        const existingAnimation = this.animations.get(icon);
        if (existingAnimation) {
            existingAnimation.pause();
        }

        const animation = animate(icon, {
            scale: [1, 1.1, 1],
            duration: 600,
            ease: 'easeInOutQuad'
        });

        this.animations.set(icon, animation);
    }

    /**
     * Stop pulse animation for drop zone icon
     * @param {HTMLElement} dropZoneElement - The drop zone container
     */
    stopDropZonePulse(dropZoneElement) {
        if (!dropZoneElement) return;

        const icon = dropZoneElement.querySelector('.upload-icon');
        if (!icon) return;

        const animation = this.animations.get(icon);
        if (animation) {
            animation.pause();
            this.animations.delete(icon);
        }

        // Reset scale
        animate(icon, {
            scale: 1,
            duration: 200,
            ease: 'easeOutQuad'
        });
    }

    /**
     * Pause all active animations
     */
    pauseAll() {
        if (this.isPaused) return;

        this.animations.forEach(animation => {
            if (animation && typeof animation.pause === 'function') {
                animation.pause();
            }
        });

        this.isPaused = true;
    }

    /**
     * Resume all paused animations
     */
    resumeAll() {
        if (!this.isPaused) return;

        this.animations.forEach(animation => {
            if (animation && typeof animation.play === 'function') {
                animation.play();
            }
        });

        this.isPaused = false;
    }

    /**
     * Check if user prefers reduced motion
     * @returns {boolean}
     */
    shouldReduceMotion() {
        return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    }

    /**
     * Clean up completed animations
     */
    cleanupAnimations() {
        this.animations.forEach((animation, element) => {
            if (animation.completed) {
                this.animations.delete(element);
            }
        });
    }

    /**
     * Cancel specific animation
     * @param {HTMLElement} element - Element whose animation to cancel
     */
    cancelAnimation(element) {
        const animation = this.animations.get(element);
        if (animation) {
            animation.pause();
            this.animations.delete(element);
        }
    }

    /**
     * Cancel all animations and clear
     */
    destroy() {
        this.animations.forEach(animation => {
            if (animation && typeof animation.pause === 'function') {
                animation.pause();
            }
        });
        this.animations.clear();
    }
}
