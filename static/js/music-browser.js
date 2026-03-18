export class MusicBrowser {
    constructor(options = {}) {
        this.listEl = document.getElementById(options.listId || 'library-list');
        this.breadcrumbsEl = document.getElementById(options.breadcrumbsId || 'breadcrumbs');
        this.selectionEl = document.getElementById(options.selectionId || 'selection-list');
        this.selectionCountEl = document.getElementById(options.selectionCountId || 'selection-count');
        this.rootPathEl = document.getElementById(options.rootPathId || 'root-path');
        this.feedbackEl = document.getElementById(options.feedbackId || 'browser-feedback');
        this.filterEl = document.getElementById(options.filterId || 'browser-filter');
        this.convertBtn = document.getElementById(options.convertBtnId || 'start-convert-btn');
        this.upBtn = document.getElementById(options.upBtnId || 'up-btn');
        this.refreshBtn = document.getElementById(options.refreshBtnId || 'refresh-btn');

        this.onJobCreated = options.onJobCreated || (() => {});
        this.onError = options.onError || (() => {});
        this.onInfo = options.onInfo || (() => {});

        this.currentPath = '';
        this.parentPath = null;
        this.rootPath = '';
        this.entries = [];
        this.filteredEntries = [];
        this.selected = new Map();
    }

    async init() {
        this.bindEvents();
        await this.loadDirectory('');
    }

    bindEvents() {
        this.upBtn?.addEventListener('click', () => {
            if (this.parentPath !== null) {
                this.loadDirectory(this.parentPath);
            }
        });

        this.refreshBtn?.addEventListener('click', () => {
            this.loadDirectory(this.currentPath);
        });

        this.filterEl?.addEventListener('input', () => {
            this.applyFilter(this.filterEl.value);
        });

        this.convertBtn?.addEventListener('click', () => {
            this.startConversion();
        });
    }

    async loadDirectory(path = '') {
        this.setFeedback('Loading folder...');
        this.renderLoading();

        try {
            const query = path ? `?path=${encodeURIComponent(path)}` : '';
            const response = await fetch(`/api/library${query}`);
            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || 'Unable to load directory');
            }

            this.currentPath = data.current_path || '';
            this.parentPath = data.parent_path ?? null;
            this.rootPath = data.root_path || '';
            this.entries = data.entries || [];

            if (this.rootPathEl) {
                this.rootPathEl.textContent = this.rootPath;
            }

            this.applyFilter(this.filterEl?.value || '');
            this.renderBreadcrumbs();
            this.updateControls();

            const directoryCount = data.directory_count || 0;
            const fileCount = data.file_count || 0;
            this.setFeedback(`${directoryCount} folder${directoryCount === 1 ? '' : 's'} • ${fileCount} FLAC file${fileCount === 1 ? '' : 's'}`);
        } catch (error) {
            this.setFeedback('Failed to load folder');
            this.renderError(error.message || 'Unable to load folder');
            this.onError(error.message || 'Unable to load folder');
        }
    }

    applyFilter(term = '') {
        const normalized = term.trim().toLowerCase();
        if (!normalized) {
            this.filteredEntries = [...this.entries];
        } else {
            this.filteredEntries = this.entries.filter((entry) =>
                entry.name.toLowerCase().includes(normalized)
            );
        }

        this.renderEntries();
    }

    renderLoading() {
        if (!this.listEl) {
            return;
        }

        this.listEl.innerHTML = '<p class="browser-empty">Loading music library...</p>';
    }

    renderError(message) {
        if (!this.listEl) {
            return;
        }

        this.listEl.innerHTML = `<p class="browser-empty">${this.escapeHtml(message)}</p>`;
    }

    renderEntries() {
        if (!this.listEl) {
            return;
        }

        if (!this.filteredEntries.length) {
            this.listEl.innerHTML = '<p class="browser-empty">No matching entries in this folder.</p>';
            return;
        }

        const rows = this.filteredEntries
            .map((entry) => {
                const isDirectory = entry.kind === 'directory';
                const isSelected = this.selected.has(entry.path);
                const icon = isDirectory ? 'DIR' : 'FL';
                const sub = isDirectory ? 'Folder' : 'FLAC file';

                return `
                    <div class="library-row" data-path="${this.escapeHtml(entry.path)}" data-kind="${entry.kind}">
                        <div class="library-row-main">
                            <span class="entry-icon ${entry.kind}">${icon}</span>
                            <div class="entry-meta">
                                <p class="entry-name">${this.escapeHtml(entry.name)}</p>
                                <p class="entry-sub">${sub}</p>
                            </div>
                        </div>
                        <div class="row-actions">
                            ${isDirectory ? '<button class="btn btn-ghost" data-action="open" type="button">Open</button>' : ''}
                            <button class="btn ${isSelected ? 'btn-primary' : 'btn-ghost'}" data-action="toggle" type="button">${isSelected ? 'Selected' : 'Select'}</button>
                        </div>
                    </div>
                `;
            })
            .join('');

        this.listEl.innerHTML = rows;

        this.listEl.querySelectorAll('[data-action="open"]').forEach((button) => {
            button.addEventListener('click', (event) => {
                const row = event.currentTarget.closest('.library-row');
                if (!row) {
                    return;
                }

                const nextPath = row.dataset.path || '';
                this.loadDirectory(nextPath);
            });
        });

        this.listEl.querySelectorAll('[data-action="toggle"]').forEach((button) => {
            button.addEventListener('click', (event) => {
                const row = event.currentTarget.closest('.library-row');
                if (!row) {
                    return;
                }

                const path = row.dataset.path || '';
                const kind = row.dataset.kind || 'file';
                const name = row.querySelector('.entry-name')?.textContent || path;
                this.toggleSelection({ path, kind, name });
            });
        });
    }

    renderBreadcrumbs() {
        if (!this.breadcrumbsEl) {
            return;
        }

        const parts = this.currentPath ? this.currentPath.split('/').filter(Boolean) : [];
        const crumbs = [
            `<button class="crumb" type="button" data-index="-1">music</button>`,
        ];

        let running = '';
        parts.forEach((part, index) => {
            running = running ? `${running}/${part}` : part;
            crumbs.push('<span class="crumb-sep">/</span>');
            crumbs.push(`<button class="crumb" type="button" data-index="${index}" data-path="${this.escapeHtml(running)}">${this.escapeHtml(part)}</button>`);
        });

        this.breadcrumbsEl.innerHTML = crumbs.join('');

        this.breadcrumbsEl.querySelectorAll('.crumb').forEach((button) => {
            button.addEventListener('click', () => {
                const targetPath = button.dataset.path || '';
                this.loadDirectory(targetPath);
            });
        });
    }

    toggleSelection(entry) {
        if (!entry.path) {
            return;
        }

        if (this.selected.has(entry.path)) {
            this.selected.delete(entry.path);
        } else {
            this.selected.set(entry.path, {
                path: entry.path,
                kind: entry.kind,
                name: entry.name,
            });
        }

        this.renderEntries();
        this.renderSelection();
    }

    renderSelection() {
        if (!this.selectionEl || !this.selectionCountEl || !this.convertBtn) {
            return;
        }

        const items = Array.from(this.selected.values());
        this.selectionCountEl.textContent = `${items.length} item${items.length === 1 ? '' : 's'} selected`;
        this.convertBtn.disabled = items.length === 0;

        if (!items.length) {
            this.selectionEl.innerHTML = '<p class="empty-selection">No files or folders selected yet.</p>';
            return;
        }

        this.selectionEl.innerHTML = items
            .map((item) => {
                const label = item.kind === 'directory' ? 'Folder' : 'File';
                return `
                    <div class="selection-chip" data-selection-path="${this.escapeHtml(item.path)}">
                        <div>
                            <strong title="${this.escapeHtml(item.path)}">${this.escapeHtml(item.name)}</strong>
                            <span>${this.escapeHtml(item.path)} <span class="type">${label}</span></span>
                        </div>
                        <button type="button" data-action="remove" aria-label="Remove selected item">x</button>
                    </div>
                `;
            })
            .join('');

        this.selectionEl.querySelectorAll('[data-action="remove"]').forEach((button) => {
            button.addEventListener('click', (event) => {
                const chip = event.currentTarget.closest('.selection-chip');
                if (!chip) {
                    return;
                }

                const targetPath = chip.dataset.selectionPath;
                this.selected.delete(targetPath);
                this.renderEntries();
                this.renderSelection();
            });
        });
    }

    async startConversion() {
        if (!this.convertBtn || this.selected.size === 0) {
            return;
        }

        this.convertBtn.disabled = true;
        const originalText = this.convertBtn.textContent;
        this.convertBtn.textContent = 'Creating Job...';

        try {
            const payload = {
                paths: Array.from(this.selected.keys()),
            };

            const response = await fetch('/api/library/convert', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload),
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || 'Unable to create conversion job');
            }

            this.onInfo(`Queued ${data.file_count} file${data.file_count === 1 ? '' : 's'} for conversion.`);
            this.selected.clear();
            this.renderEntries();
            this.renderSelection();
            this.onJobCreated(data);
        } catch (error) {
            this.onError(error.message || 'Unable to create conversion job');
        } finally {
            this.convertBtn.textContent = originalText;
            this.convertBtn.disabled = this.selected.size === 0;
        }
    }

    updateControls() {
        if (this.upBtn) {
            this.upBtn.disabled = this.parentPath === null;
        }
    }

    setFeedback(message) {
        if (this.feedbackEl) {
            this.feedbackEl.textContent = message;
        }
    }

    escapeHtml(value) {
        return String(value)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }
}
