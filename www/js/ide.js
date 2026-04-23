'use strict';

var IDE = (function() {

    var icon = function(svg) { return '<svg viewBox="0 0 24 24">' + svg + '</svg>'; };

    var BUTTONS = [
        { id: 'label',    label: 'Aa',  title: 'Label',        wide: true, group: 'components' },
        { id: 'input',    label: icon('<rect x="3" y="6" width="18" height="12" rx="2"/><line x1="7" y1="12" x2="7" y2="12" stroke-linecap="round" stroke-width="3"/>'), title: 'Text Input', group: 'components' },
        { id: 'password', label: icon('<rect x="3" y="6" width="18" height="12" rx="2"/><circle cx="8" cy="12" r="1.5" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.5" fill="currentColor" stroke="none"/><circle cx="16" cy="12" r="1.5" fill="currentColor" stroke="none"/>'), title: 'Password Input', group: 'components' },
        { id: 'button',   label: icon('<rect x="3" y="7" width="18" height="10" rx="3"/><line x1="8" y1="12" x2="16" y2="12" stroke-linecap="round"/>'), title: 'Button', group: 'components' },
        { id: 'checkbox',  label: icon('<rect x="4" y="4" width="16" height="16" rx="2"/><polyline points="8 12 11 15 16 9" stroke-linecap="round" stroke-linejoin="round"/>'), title: 'Checkbox', group: 'components' },
        { id: 'select',   label: icon('<rect x="3" y="6" width="18" height="12" rx="2"/><polyline points="15 10 12 14 9 10" stroke-linecap="round" stroke-linejoin="round"/>'), title: 'Select', group: 'components' },
        { id: 'textarea', label: icon('<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8" stroke-linecap="round"/><line x1="7" y1="12" x2="17" y2="12" stroke-linecap="round"/><line x1="7" y1="16" x2="13" y2="16" stroke-linecap="round"/>'), title: 'Textarea', group: 'components' },
        { id: 'divider',  label: icon('<line x1="3" y1="12" x2="21" y2="12" stroke-width="2"/>'), title: 'Divider', group: 'components' },
        '|',
        { id: 'card',     label: icon('<rect x="3" y="3" width="18" height="18" rx="3"/><line x1="3" y1="9" x2="21" y2="9"/>'), title: 'Card Container', group: 'containers' },
        { id: 'form',     label: icon('<rect x="3" y="3" width="18" height="18" rx="2"/><line x1="7" y1="8" x2="17" y2="8" stroke-linecap="round"/><rect x="7" y="11" width="10" height="3" rx="1"/><rect x="11" y="17" width="6" height="2.5" rx="1"/>'), title: 'Form Container', group: 'containers' },
        { id: 'section',  label: icon('<rect x="3" y="3" width="18" height="18" rx="2" stroke-dasharray="4 2"/>'), title: 'Section Container', group: 'containers' },
        '|',
        { id: 'delete',   label: icon('<polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/>'), title: 'Delete Selected' },
        { id: 'move-up',  label: icon('<polyline points="18 15 12 9 6 15"/>'), title: 'Move Up' },
        { id: 'move-down', label: icon('<polyline points="6 9 12 15 18 9"/>'), title: 'Move Down' },
        '>',
        { id: 'preview',  label: icon('<path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>'), title: 'Toggle Preview Mode' },
        { id: 'source',   label: icon('<polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/>'), title: 'View Markout Source' },
    ];

    function OutconceiveIDE(container, app, patcher) {
        this.container = container;
        this.app = app;
        this.patcher = patcher;
        this.selectedLine = -1;
        this.designMode = true;
        this._actionHandlers = {};
        this.elements = {};
    }

    OutconceiveIDE.prototype.build = function() {
        var self = this;
        var root = this.container;
        root.className = 'ide-layout';
        root.innerHTML = '';

        // Toolbar
        var toolbar = document.createElement('div');
        toolbar.className = 'ide-toolbar';
        this._buildToolbar(toolbar);
        root.appendChild(toolbar);
        this.elements.toolbar = toolbar;

        // Canvas
        var canvas = document.createElement('div');
        canvas.className = 'ide-canvas design-mode';
        canvas.id = 'ide-canvas';
        root.appendChild(canvas);
        this.elements.canvas = canvas;

        // Side panel
        var panel = document.createElement('div');
        panel.className = 'ide-panel';
        panel.innerHTML =
            '<div class="ide-panel-tabs">' +
                '<button class="ide-panel-tab active" data-tab="properties">Properties</button>' +
                '<button class="ide-panel-tab" data-tab="source">Source</button>' +
            '</div>' +
            '<div class="ide-panel-content" id="ide-panel-content"></div>';
        root.appendChild(panel);
        this.elements.panel = panel;
        this.elements.panelContent = panel.querySelector('#ide-panel-content');

        // Status bar
        var statusbar = document.createElement('div');
        statusbar.className = 'ide-statusbar';
        statusbar.innerHTML =
            '<div class="ide-statusbar-left">' +
                '<span id="ide-status-mode">Design Mode</span>' +
                '<span id="ide-status-lines">0 lines</span>' +
            '</div>' +
            '<div class="ide-statusbar-right">' +
                '<span>Outconceive IDE</span>' +
            '</div>';
        root.appendChild(statusbar);
        this.elements.statusbar = statusbar;

        this._attachEvents();
        this._showProperties();
        this._updateStatus();
    };

    OutconceiveIDE.prototype._buildToolbar = function(toolbar) {
        var self = this;

        for (var i = 0; i < BUTTONS.length; i++) {
            var def = BUTTONS[i];
            if (def === '|') { toolbar.appendChild(sep()); continue; }
            if (def === '>') { toolbar.appendChild(spacer()); continue; }

            var btn = document.createElement('button');
            btn.className = 'ide-btn' + (def.wide ? ' ide-btn-wide' : '');
            btn.setAttribute('data-action', def.id);
            btn.setAttribute('data-tip', def.title);
            btn.innerHTML = def.label;
            toolbar.appendChild(btn);
        }

        function sep() { var d = document.createElement('div'); d.className = 'ide-sep'; return d; }
        function spacer() { var d = document.createElement('div'); d.className = 'ide-spacer'; return d; }
    };

    OutconceiveIDE.prototype._attachEvents = function() {
        var self = this;

        // Toolbar clicks
        this.elements.toolbar.addEventListener('mousedown', function(e) {
            if (e.target.closest('[data-action]')) e.preventDefault();
        });

        this.elements.toolbar.addEventListener('click', function(e) {
            var btn = e.target.closest('[data-action]');
            if (!btn) return;
            e.preventDefault();
            self._dispatch(btn.getAttribute('data-action'));
        });

        // Canvas clicks — select rows
        this.elements.canvas.addEventListener('click', function(e) {
            if (!self.designMode) return;
            var row = e.target.closest('[data-line]');
            if (row) {
                self._selectLine(parseInt(row.getAttribute('data-line'), 10));
            } else {
                self._selectLine(-1);
            }
        });

        // Canvas double-click — edit properties
        this.elements.canvas.addEventListener('dblclick', function(e) {
            if (!self.designMode) return;
            var row = e.target.closest('[data-line]');
            if (row) {
                self._selectLine(parseInt(row.getAttribute('data-line'), 10));
                self._showTab('properties');
            }
        });

        // Runtime events (when not in design mode)
        var eventRouter = new EventRouter(this.elements.canvas, this.app, this.patcher);
        this._eventRouter = eventRouter;

        // Panel tabs
        this.elements.panel.addEventListener('click', function(e) {
            var tab = e.target.closest('[data-tab]');
            if (!tab) return;
            self._showTab(tab.getAttribute('data-tab'));
        });
    };

    OutconceiveIDE.prototype._dispatch = function(action) {
        var insertAt = this.selectedLine >= 0
            ? this.selectedLine + 1
            : this.app.get_line_count();

        switch (action) {
            case 'label':
                this._insertWithPrompt('label', 'Label text:', 'Label', '', '');
                break;
            case 'input':
                this._insertWithPrompt('input', 'State key:', '', 'field_name', '');
                break;
            case 'password':
                this._insertWithPrompt('password', 'State key:', '', 'password', '');
                break;
            case 'button':
                this._insertWithPrompt('button', 'Action name:', 'Click Me', 'action', 'primary');
                break;
            case 'checkbox':
                this._insertWithPrompt('checkbox', 'State key:', '', 'checked', '');
                break;
            case 'select':
                this._insertWithPrompt('select', 'State key:', '', 'choice', '');
                break;
            case 'textarea':
                this._insertWithPrompt('textarea', 'State key:', '', 'content', '');
                break;
            case 'divider':
                var patches = this.app.insert_component(insertAt, 'divider', '', '', '');
                this.patcher.applyPatches(patches);
                this._afterChange();
                break;
            case 'card':
                var patches = this.app.insert_container(insertAt, 'card', 'padding:16');
                this.patcher.applyPatches(patches);
                this._afterChange();
                break;
            case 'form':
                var patches = this.app.insert_container(insertAt, 'form', '');
                this.patcher.applyPatches(patches);
                this._afterChange();
                break;
            case 'section':
                var patches = this.app.insert_container(insertAt, 'section', '');
                this.patcher.applyPatches(patches);
                this._afterChange();
                break;
            case 'delete':
                if (this.selectedLine >= 0) {
                    var patches = this.app.remove_line_at(this.selectedLine);
                    this.patcher.applyPatches(patches);
                    this.selectedLine = -1;
                    this._afterChange();
                }
                break;
            case 'move-up':
                if (this.selectedLine > 0) {
                    var patches = this.app.move_line(this.selectedLine, this.selectedLine - 1);
                    this.patcher.applyPatches(patches);
                    this.selectedLine--;
                    this._afterChange();
                }
                break;
            case 'move-down':
                if (this.selectedLine >= 0 && this.selectedLine < this.app.get_line_count() - 1) {
                    var patches = this.app.move_line(this.selectedLine, this.selectedLine + 1);
                    this.patcher.applyPatches(patches);
                    this.selectedLine++;
                    this._afterChange();
                }
                break;
            case 'preview':
                this.designMode = !this.designMode;
                this.elements.canvas.classList.toggle('design-mode', this.designMode);
                if (!this.designMode) {
                    this._eventRouter.attach();
                    this._selectLine(-1);
                }
                document.getElementById('ide-status-mode').textContent =
                    this.designMode ? 'Design Mode' : 'Preview Mode';
                break;
            case 'source':
                this._showTab('source');
                break;
        }
    };

    OutconceiveIDE.prototype._insertWithPrompt = function(compType, promptLabel, defaultLabel, defaultKey, defaultStyle) {
        var insertAt = this.selectedLine >= 0
            ? this.selectedLine + 1
            : this.app.get_line_count();

        this._showInsertModal(compType, promptLabel, defaultLabel, defaultKey, defaultStyle, insertAt);
    };

    OutconceiveIDE.prototype._showInsertModal = function(compType, promptLabel, defaultLabel, defaultKey, defaultStyle, insertAt) {
        var self = this;
        var content = this.elements.panelContent;

        var isButton = compType === 'button';
        var isLabel = compType === 'label';
        var needsKey = !isLabel;
        var needsLabel = isButton || isLabel;

        var html = '<div class="ide-props-title">Insert ' + compType + '</div>';

        if (needsLabel) {
            html += '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">Label</label>' +
                '<input class="ide-prop-input" id="prop-label" value="' + defaultLabel + '">' +
            '</div>';
        }

        if (needsKey) {
            html += '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">State Key / Action</label>' +
                '<input class="ide-prop-input" id="prop-key" value="' + defaultKey + '">' +
            '</div>';
        }

        html += '<div class="ide-prop-group">' +
            '<label class="ide-prop-label">Style</label>' +
            '<select class="ide-prop-select" id="prop-style">' +
                '<option value="">Default</option>' +
                '<option value="primary"' + (defaultStyle === 'primary' ? ' selected' : '') + '>Primary</option>' +
                '<option value="secondary">Secondary</option>' +
                '<option value="danger">Danger</option>' +
                '<option value="warning">Warning</option>' +
                '<option value="outline">Outline</option>' +
                '<option value="ghost">Ghost</option>' +
            '</select>' +
        '</div>';

        html += '<button class="ide-prop-btn" id="prop-insert">Insert</button>';

        content.innerHTML = html;
        this._showTab('properties');

        var firstInput = content.querySelector('.ide-prop-input');
        if (firstInput) firstInput.focus();

        document.getElementById('prop-insert').addEventListener('click', function() {
            var label = document.getElementById('prop-label');
            var key = document.getElementById('prop-key');
            var style = document.getElementById('prop-style');

            var patches = self.app.insert_component(
                insertAt,
                compType,
                label ? label.value : '',
                key ? key.value : '',
                style ? style.value : ''
            );
            self.patcher.applyPatches(patches);
            self.selectedLine = insertAt;
            self._afterChange();
        });
    };

    OutconceiveIDE.prototype._selectLine = function(lineIndex) {
        this.selectedLine = lineIndex;

        var rows = this.elements.canvas.querySelectorAll('[data-line]');
        for (var i = 0; i < rows.length; i++) {
            rows[i].classList.toggle('selected', parseInt(rows[i].getAttribute('data-line'), 10) === lineIndex);
        }

        this._showProperties();
    };

    OutconceiveIDE.prototype._showProperties = function() {
        var content = this.elements.panelContent;
        var activeTab = this.elements.panel.querySelector('.ide-panel-tab.active');
        if (activeTab && activeTab.getAttribute('data-tab') !== 'properties') return;

        if (this.selectedLine < 0) {
            content.innerHTML =
                '<div class="ide-props-title">Properties</div>' +
                '<div class="ide-props-empty">Select a row to edit its properties</div>';
            return;
        }

        var info = this.app.get_line_info(this.selectedLine);
        if (!info) return;

        if (info.is_container_start) {
            this._showContainerProps(info);
            return;
        }

        if (info.is_container_end) {
            content.innerHTML =
                '<div class="ide-props-title">Container End</div>' +
                '<div class="ide-props-empty">End of ' + (info.tag || 'container') + '</div>';
            return;
        }

        this._showRowProps(info);
    };

    OutconceiveIDE.prototype._showContainerProps = function(info) {
        var self = this;
        var content = this.elements.panelContent;

        content.innerHTML =
            '<div class="ide-props-title">Container: ' + (info.tag || 'div') + '</div>' +
            '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">Tag</label>' +
                '<input class="ide-prop-input" id="prop-tag" value="' + (info.tag || '') + '">' +
            '</div>' +
            '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">Config</label>' +
                '<input class="ide-prop-input" id="prop-config" value="' + (info.config || '') + '">' +
            '</div>' +
            '<button class="ide-prop-btn ide-prop-btn-danger" id="prop-delete">Delete Container</button>';

        document.getElementById('prop-delete').addEventListener('click', function() {
            var patches = self.app.remove_line_at(self.selectedLine);
            self.patcher.applyPatches(patches);
            self.selectedLine = -1;
            self._afterChange();
        });
    };

    OutconceiveIDE.prototype._showRowProps = function(info) {
        var self = this;
        var content = this.elements.panelContent;

        // Parse first span type for display
        var firstComp = '';
        for (var i = 0; i < info.components.length; i++) {
            var c = info.components[i];
            if (c !== ' ' && c !== '_') { firstComp = c; break; }
        }

        var compName = {
            'L': 'Label', 'I': 'Input', 'P': 'Password', 'B': 'Button',
            'C': 'Checkbox', 'R': 'Radio', 'S': 'Select', 'T': 'Textarea',
            'D': 'Divider', 'K': 'Link', 'G': 'Image'
        }[firstComp] || 'Row';

        var stateKey = info.state_keys.replace(/_/g, '').trim();

        content.innerHTML =
            '<div class="ide-props-title">' + compName + '</div>' +
            '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">Content</label>' +
                '<input class="ide-prop-input" id="prop-content" value="' + escHtml(info.content) + '">' +
            '</div>' +
            '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">State Key</label>' +
                '<input class="ide-prop-input" id="prop-state-key" value="' + escHtml(stateKey) + '">' +
            '</div>' +
            '<div class="ide-prop-group">' +
                '<label class="ide-prop-label">Markout</label>' +
                '<div class="ide-source" style="font-size:11px;padding:6px;background:#2d2d2d;border-radius:3px;">' +
                    escHtml(this.app.to_markout().split('\n')[this._markoutLineFor(this.selectedLine)] || '') +
                '</div>' +
            '</div>' +
            '<button class="ide-prop-btn ide-prop-btn-danger" id="prop-delete" style="margin-top:16px">Delete Row</button>';

        document.getElementById('prop-delete').addEventListener('click', function() {
            var patches = self.app.remove_line_at(self.selectedLine);
            self.patcher.applyPatches(patches);
            self.selectedLine = -1;
            self._afterChange();
        });
    };

    OutconceiveIDE.prototype._markoutLineFor = function(rawLine) {
        // Approximate: markout output line ≈ raw line (containers + content in order)
        return rawLine;
    };

    OutconceiveIDE.prototype._showTab = function(tabName) {
        var tabs = this.elements.panel.querySelectorAll('.ide-panel-tab');
        for (var i = 0; i < tabs.length; i++) {
            tabs[i].classList.toggle('active', tabs[i].getAttribute('data-tab') === tabName);
        }

        if (tabName === 'source') {
            this._showSource();
        } else {
            this._showProperties();
        }
    };

    OutconceiveIDE.prototype._showSource = function() {
        var self = this;
        var source = this.app.to_markout();

        this.elements.panelContent.innerHTML =
            '<div class="ide-props-title">Markout Source</div>' +
            '<textarea class="ide-source-editor" id="ide-source-editor" spellcheck="false"></textarea>' +
            '<button class="ide-prop-btn" id="ide-source-apply" style="margin-top:8px">Apply Changes</button>';

        var editor = document.getElementById('ide-source-editor');
        editor.value = source;

        document.getElementById('ide-source-apply').addEventListener('click', function() {
            var newSource = editor.value;
            self.app.from_markout(newSource);
            var vdom = self.app.initial_render();
            self.patcher.renderInitial(vdom);
            self.selectedLine = -1;
            self._updateStatus();
        });
    };

    OutconceiveIDE.prototype._afterChange = function() {
        this._updateStatus();
        this._selectLine(this.selectedLine);

        // Refresh source tab if visible
        var activeTab = this.elements.panel.querySelector('.ide-panel-tab.active');
        if (activeTab && activeTab.getAttribute('data-tab') === 'source') {
            this._showSource();
        }
    };

    OutconceiveIDE.prototype._updateStatus = function() {
        var el = document.getElementById('ide-status-lines');
        if (el) el.textContent = this.app.get_line_count() + ' lines';
    };

    OutconceiveIDE.prototype.on = function(action, handler) {
        this._actionHandlers[action] = handler;
        this._eventRouter.on(action, handler);
    };

    function escHtml(s) {
        return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    return OutconceiveIDE;
})();

window.OutconceiveIDE = IDE;
