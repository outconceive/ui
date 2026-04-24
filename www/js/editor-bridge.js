let _editorWasmModule = null;
let _editorWasmReady = null;

function _loadEditorWasm(wasmUrl) {
    if (_editorWasmReady) return _editorWasmReady;
    _editorWasmReady = import(wasmUrl).then(function(module) {
        var binaryUrl = wasmUrl.replace('.js', '_bg.wasm');
        return module.default({ module_or_path: binaryUrl }).then(function() {
            _editorWasmModule = module;
            return module;
        });
    });
    return _editorWasmReady;
}

var FEATURE_MAP = {
    'bold':          { shortcut: 'ctrl+b',     method: 'toggle_bold',      icon: 'B',  label: 'Bold' },
    'italic':        { shortcut: 'ctrl+i',     method: 'toggle_italic',    icon: 'I',  label: 'Italic' },
    'underline':     { shortcut: 'ctrl+u',     method: 'apply_underline',  icon: 'U',  label: 'Underline' },
    'strikethrough': { shortcut: 'ctrl+shift+s', method: 'apply_strikethrough', icon: 'S', label: 'Strikethrough' },
    'code':          { shortcut: 'ctrl+`',     method: 'toggle_code',      icon: '<>', label: 'Code' },
    'heading':       { shortcut: null,         method: 'set_heading',      icon: 'H',  label: 'Heading' },
    'list':          { shortcut: null,         method: 'set_list',         icon: '•',  label: 'List' },
    'ordered-list':  { shortcut: null,         method: 'set_ordered_list', icon: '1.', label: 'Ordered List' },
    'quote':         { shortcut: null,         method: 'set_quote',        icon: '"',  label: 'Quote' },
    'code-block':    { shortcut: null,         method: 'toggle_code_block',icon: '{}', label: 'Code Block' },
    'link':          { shortcut: 'ctrl+k',     method: 'add_link',         icon: '🔗', label: 'Link' },
    'hr':            { shortcut: null,         method: 'insert_hr',        icon: '—',  label: 'Divider' },
    'divider':       { shortcut: null,         method: 'insert_hr',        icon: '—',  label: 'Divider' },
};

var SANITIZE_PATTERNS = {
    'bold':          [/\*\*(.+?)\*\*/g, '$1', /__(.+?)__/g, '$1'],
    'italic':        [/(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)/g, '$1'],
    'strikethrough': [/~~(.+?)~~/g, '$1'],
    'code':          [/`([^`]+)`/g, '$1'],
    'heading':       [/^#{1,6}\s+/gm, ''],
    'list':          [/^[-*+]\s+/gm, ''],
    'ordered-list':  [/^\d+\.\s+/gm, ''],
    'quote':         [/^>\s*/gm, ''],
    'code-block':    [/```[\s\S]*?```/g, function(m) { return m.replace(/```\w*\n?/, '').replace(/\n?```$/, ''); }],
    'link':          [/\[([^\]]+)\]\([^)]+\)/g, '$1'],
    'hr':            [/^---+$/gm, ''],
    'divider':       [/^---+$/gm, ''],
};

function EditorBridge(containerEl, outconceiveApp, options) {
    this.container = containerEl;
    this.outconceiveApp = outconceiveApp;
    this.features = new Set((containerEl.getAttribute('data-features') || '').split(',').filter(Boolean));
    this.bindKey = containerEl.getAttribute('data-bind') || null;
    this.editor = null;
    this.patcher = null;
    this.cursorEl = null;
    this.editorEl = null;
    this.toolbarEl = null;
    this._changeTimer = null;
    this._lastExported = '';
    this._wasmUrl = (options && options.wasmUrl) || '/wasm/editor/rust_markdown_editor.js';
    this._init();
}

EditorBridge.prototype._init = function() {
    this._buildDOM();
    var self = this;
    _loadEditorWasm(this._wasmUrl).then(function(module) {
        self.editor = new module.Editor();
        self.patcher = new EditorDomPatcher(self.editorEl);

        var initial = self.bindKey && self.outconceiveApp
            ? self.outconceiveApp.get_state(self.bindKey) : '';
        if (initial) {
            var vdom = self.editor.import_markdown(self._sanitize(initial));
            self.patcher.renderInitial(vdom);
        } else {
            var vdom = self.editor.get_initial_vdom();
            self.patcher.renderInitial(vdom);
        }

        self._attachEvents();
    });
};

EditorBridge.prototype._buildDOM = function() {
    this.container.innerHTML = '';
    this.container.classList.add('mc-editor-active');

    if (this.features.size > 0) {
        this.toolbarEl = document.createElement('div');
        this.toolbarEl.className = 'mc-editor-toolbar';
        this._buildToolbar();
        this.container.appendChild(this.toolbarEl);
    }

    this.editorEl = document.createElement('div');
    this.editorEl.className = 'mc-editor-content';
    this.editorEl.setAttribute('contenteditable', 'true');
    this.editorEl.setAttribute('spellcheck', 'false');
    this.container.appendChild(this.editorEl);
};

EditorBridge.prototype._buildToolbar = function() {
    var self = this;
    var features = Array.from(this.features);

    for (var i = 0; i < features.length; i++) {
        var feat = features[i];
        var info = FEATURE_MAP[feat];
        if (!info) continue;

        var btn = document.createElement('button');
        btn.className = 'mc-editor-btn';
        btn.setAttribute('data-feature', feat);
        btn.textContent = info.icon;
        btn.title = info.label;
        btn.type = 'button';

        (function(feature, featureInfo) {
            btn.addEventListener('mousedown', function(e) {
                e.preventDefault();
                self._execFeature(feature, featureInfo);
            });
        })(feat, info);

        this.toolbarEl.appendChild(btn);
    }
};

EditorBridge.prototype._execFeature = function(feature, info) {
    if (!this.editor) return;
    var result;

    if (feature === 'heading') {
        result = this.editor.set_heading(2);
    } else if (feature === 'link') {
        var url = prompt('URL:');
        if (!url) return;
        result = this.editor.add_link(url);
    } else {
        result = this.editor[info.method]();
    }

    if (result && result.patches) {
        this.patcher.applyPatches(result.patches);
        this._setCursor(result.cursor);
    }
    this._scheduleSync();
    this._updateToolbar();
};

EditorBridge.prototype._updateToolbar = function() {
    if (!this.editor || !this.toolbarEl) return;
    var fmt = this.editor.get_current_format();
    if (!fmt) return;

    var inline = fmt.inline || {};
    var block = fmt.block || {};

    var btns = this.toolbarEl.querySelectorAll('.mc-editor-btn');
    for (var i = 0; i < btns.length; i++) {
        var feat = btns[i].getAttribute('data-feature');
        var active = false;

        switch (feat) {
            case 'bold':          active = !!inline.bold; break;
            case 'italic':        active = !!inline.italic; break;
            case 'underline':     active = !!inline.underline; break;
            case 'strikethrough': active = !!inline.strikethrough; break;
            case 'code':          active = !!inline.code; break;
            case 'link':          active = !!inline.link; break;
            case 'heading':       active = block.format === '#'; break;
            case 'list':          active = block.format === '-'; break;
            case 'ordered-list':  active = block.format === '1'; break;
            case 'quote':         active = block.format === '>'; break;
            case 'code-block':    active = block.format === '`'; break;
        }

        if (active) {
            btns[i].classList.add('mc-editor-btn-active');
        } else {
            btns[i].classList.remove('mc-editor-btn-active');
        }
    }
};

EditorBridge.prototype._attachEvents = function() {
    var self = this;

    this.editorEl.addEventListener('keydown', function(e) {
        if (self._handleShortcut(e)) return;

        switch (e.key) {
            case 'Enter':
                e.preventDefault();
                self._syncCursorBeforeEdit();
                self._applyResult(self.editor.on_enter());
                return;
            case 'Backspace':
                e.preventDefault();
                self._syncCursorBeforeEdit();
                self._applyResult(self.editor.on_backspace());
                return;
            case 'Delete':
                e.preventDefault();
                self._syncCursorBeforeEdit();
                self._applyResult(self.editor.on_delete());
                return;
        }

        if (e.key.indexOf('Arrow') === 0) {
            e.preventDefault();
            self._syncCursorBeforeEdit();
            self._applyResult(self.editor.on_arrow_key(e.key.replace('Arrow', '').toLowerCase()));
            return;
        }
    });

    this.editorEl.addEventListener('keypress', function(e) {
        if (e.ctrlKey || e.metaKey || e.altKey) return;
        if (e.key && e.key.length === 1) {
            e.preventDefault();
            self._syncCursorBeforeEdit();
            self._applyResult(self.editor.on_key_press(e.key));
        }
    });

    this.editorEl.addEventListener('paste', function(e) {
        e.preventDefault();
        var text = (e.clipboardData || window.clipboardData).getData('text/plain');
        if (text) {
            text = self._sanitize(text);
            self._syncCursorBeforeEdit();
            self._applyResult(self.editor.paste(text));
        }
    });

    this.editorEl.addEventListener('mouseup', function() {
        setTimeout(function() { self._syncSelectionOrCursor(); self._updateToolbar(); }, 10);
    });

    this._selChangeTimer = null;
    document.addEventListener('selectionchange', function() {
        clearTimeout(self._selChangeTimer);
        self._selChangeTimer = setTimeout(function() {
            if (!self.editorEl || !self.editorEl.contains(document.activeElement)) return;
            self._syncSelectionOrCursor();
            self._updateToolbar();
        }, 50);
    });

    // Stop events from reaching Outconceive's EventRouter
    this.container.addEventListener('input', function(e) { e.stopPropagation(); });
    this.container.addEventListener('click', function(e) { e.stopPropagation(); });
    this.container.addEventListener('change', function(e) { e.stopPropagation(); });
};

EditorBridge.prototype._handleShortcut = function(e) {
    if (!e.ctrlKey && !e.metaKey) return false;

    if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) {
            var r = this.editor.redo();
            if (r) this._applyResult(r);
        } else {
            var r = this.editor.undo();
            if (r) this._applyResult(r);
        }
        return true;
    }

    var pressed = this._normalizeKey(e);
    var features = Array.from(this.features);
    for (var i = 0; i < features.length; i++) {
        var info = FEATURE_MAP[features[i]];
        if (info && info.shortcut === pressed) {
            e.preventDefault();
            this._execFeature(features[i], info); // _execFeature already syncs selection
            return true;
        }
    }
    return false;
};

EditorBridge.prototype._normalizeKey = function(e) {
    var parts = [];
    if (e.ctrlKey || e.metaKey) parts.push('ctrl');
    if (e.shiftKey) parts.push('shift');
    if (e.altKey) parts.push('alt');
    parts.push(e.key.toLowerCase());
    return parts.join('+');
};

EditorBridge.prototype._applyResult = function(result) {
    if (!result) return;
    if (result.patches) this.patcher.applyPatches(result.patches);
    if (result.cursor) this._setCursor(result.cursor);
    this._scheduleSync();
    this._updateToolbar();
};

EditorBridge.prototype._syncCursorBeforeEdit = function() {
    if (!this.editor || !this.editorEl) return;
    var pos = this._getCursorFromDOM();
    this.editor.set_cursor_position(pos.line, pos.col);
};

EditorBridge.prototype._syncSelectionOrCursor = function() {
    if (!this.editor || !this.editorEl) return;
    var sel = window.getSelection();
    if (sel.rangeCount && !sel.isCollapsed) {
        var range = sel.getRangeAt(0);
        var start = this._posFromNode(range.startContainer, range.startOffset);
        var end = this._posFromNode(range.endContainer, range.endOffset);
        this.editor.set_selection(start.line, start.col, end.line, end.col);
    } else {
        var pos = this._getCursorFromDOM();
        this.editor.set_cursor_position(pos.line, pos.col);
        this.editor.clear_selection();
    }
};

EditorBridge.prototype._getCursorFromDOM = function() {
    var sel = window.getSelection();
    if (!sel.rangeCount) return { line: 0, col: 0 };
    var range = sel.getRangeAt(0);
    return this._posFromNode(range.startContainer, range.startOffset);
};

EditorBridge.prototype._posFromNode = function(node, offset) {
    if (node.nodeType !== Node.TEXT_NODE && node.childNodes && node.childNodes.length > 0) {
        if (offset < node.childNodes.length) {
            return this._posFromNode(node.childNodes[offset], 0);
        } else if (node.childNodes.length > 0) {
            var last = node.childNodes[node.childNodes.length - 1];
            if (last.nodeType === Node.TEXT_NODE) {
                return this._posFromNode(last, last.textContent.length);
            }
            return this._posFromNode(last, 0);
        }
    }

    var el = node.nodeType === Node.TEXT_NODE ? node.parentElement : node;
    while (el && !el.hasAttribute('data-line')) {
        if (el === this.editorEl) return { line: 0, col: 0 };
        el = el.parentElement;
    }
    if (!el) return { line: 0, col: 0 };

    var line = parseInt(el.getAttribute('data-line'), 10);
    var col = this._colFromNode(el, node, offset);
    return { line: line, col: col };
};

EditorBridge.prototype._colFromNode = function(lineEl, targetNode, targetOffset) {
    var walker = document.createTreeWalker(lineEl, NodeFilter.SHOW_TEXT, null, false);
    var col = 0;
    var n;
    while ((n = walker.nextNode())) {
        if (n === targetNode) return col + targetOffset;
        col += n.textContent.length;
    }
    return col;
};

EditorBridge.prototype._setCursor = function(cursor) {
    if (!cursor) return;
    var lineEl = this.editorEl.querySelector('[data-line="' + cursor.line + '"]');
    if (!lineEl) return;

    var walker = document.createTreeWalker(lineEl, NodeFilter.SHOW_TEXT, null, false);
    var currentCol = 0;
    var node;
    var target = null;
    var targetOffset = 0;

    while ((node = walker.nextNode())) {
        var len = node.textContent.length;
        if (currentCol + len >= cursor.col) {
            target = node;
            targetOffset = cursor.col - currentCol;
            break;
        }
        currentCol += len;
    }

    if (!target) {
        target = document.createTextNode('');
        var br = lineEl.querySelector('br');
        if (br) lineEl.insertBefore(target, br);
        else lineEl.appendChild(target);
        targetOffset = 0;
    }

    try {
        var range = document.createRange();
        range.setStart(target, Math.min(targetOffset, target.textContent.length));
        range.collapse(true);
        var sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(range);
    } catch (e) {}
};

EditorBridge.prototype._sanitize = function(md) {
    var result = md;
    for (var feat in SANITIZE_PATTERNS) {
        if (this.features.has(feat)) continue;
        var pats = SANITIZE_PATTERNS[feat];
        for (var i = 0; i < pats.length; i += 2) {
            result = result.replace(pats[i], pats[i + 1]);
        }
    }
    return result;
};

EditorBridge.prototype._scheduleSync = function() {
    if (!this.bindKey || !this.outconceiveApp) return;
    var self = this;
    clearTimeout(this._changeTimer);
    this._changeTimer = setTimeout(function() {
        var md = self.editor.export_markdown();
        if (md !== self._lastExported) {
            self._lastExported = md;
            self.outconceiveApp.update_state(self.bindKey, md);
        }
    }, 300);
};

EditorBridge.prototype.setContent = function(md) {
    if (!this.editor) return;
    var sanitized = this._sanitize(md);
    var vdom = this.editor.import_markdown(sanitized);
    this.patcher.renderInitial(vdom);
    this._lastExported = sanitized;
};

EditorBridge.prototype.getContent = function() {
    if (!this.editor) return '';
    return this.editor.export_markdown();
};

EditorBridge.prototype.destroy = function() {
    clearTimeout(this._changeTimer);
    if (this.editor) {
        this.editor.free();
        this.editor = null;
    }
    this.container.innerHTML = '';
    this.container.classList.remove('mc-editor-active');
};

// Minimal DOM patcher for the editor (vanilla JS, no jQuery)
function EditorDomPatcher(container) {
    this.container = container;
}

EditorDomPatcher.prototype.renderInitial = function(vdom) {
    this.container.innerHTML = '';
    if (vdom.type === 'Element' && vdom.children) {
        for (var i = 0; i < vdom.children.length; i++) {
            var child = this._vnodeToDOM(vdom.children[i]);
            if (child) this.container.appendChild(child);
        }
    }
};

EditorDomPatcher.prototype.applyPatches = function(patches) {
    if (!patches || !patches.length) return;
    for (var i = 0; i < patches.length; i++) {
        this._applyPatch(patches[i]);
    }
};

EditorDomPatcher.prototype._applyPatch = function(patch) {
    switch (patch.type) {
        case 'Replace':
            if (!patch.path || patch.path.length === 0) {
                this.renderInitial(patch.node);
            } else {
                var target = this._nodeAtPath(patch.path);
                if (target && target.parentNode) {
                    var n = this._vnodeToDOM(patch.node);
                    if (n) target.parentNode.replaceChild(n, target);
                }
            }
            break;
        case 'Insert':
            var pp = patch.path.slice(0, -1);
            var idx = patch.path[patch.path.length - 1];
            var parent = this._nodeAtPath(pp);
            if (parent) {
                var n = this._vnodeToDOM(patch.node);
                if (n) {
                    if (idx >= parent.childNodes.length) parent.appendChild(n);
                    else parent.insertBefore(n, parent.childNodes[idx]);
                }
            }
            break;
        case 'Remove':
            var t = this._nodeAtPath(patch.path);
            if (t && t.parentNode) t.parentNode.removeChild(t);
            break;
        case 'UpdateText':
            var t = this._nodeAtPath(patch.path);
            if (t && t.nodeType === Node.TEXT_NODE) t.textContent = patch.text;
            break;
        case 'SetAttribute':
            var t = this._nodeAtPath(patch.path);
            if (t && t.setAttribute) t.setAttribute(patch.key, patch.value);
            break;
        case 'RemoveAttribute':
            var t = this._nodeAtPath(patch.path);
            if (t && t.removeAttribute) t.removeAttribute(patch.key);
            break;
    }
};

EditorDomPatcher.prototype._nodeAtPath = function(path) {
    if (!path || path.length === 0) return this.container;
    var node = this.container;
    for (var i = 0; i < path.length; i++) {
        if (!node || !node.childNodes) return null;
        if (path[i] >= node.childNodes.length) return null;
        node = node.childNodes[path[i]];
    }
    return node;
};

EditorDomPatcher.prototype._vnodeToDOM = function(vnode) {
    if (!vnode) return null;
    if (vnode.type === 'Text') return document.createTextNode(vnode.content || '');
    if (vnode.type === 'Element') {
        var el = document.createElement(vnode.tag);
        if (vnode.attrs) {
            var keys = Object.keys(vnode.attrs);
            for (var i = 0; i < keys.length; i++) {
                el.setAttribute(keys[i], vnode.attrs[keys[i]]);
            }
        }
        if (vnode.children) {
            for (var j = 0; j < vnode.children.length; j++) {
                var child = this._vnodeToDOM(vnode.children[j]);
                if (child) el.appendChild(child);
            }
        }
        return el;
    }
    return document.createTextNode('');
};

export { EditorBridge };
