'use strict';

function Outconceive(wasmApp) {
    this.app = wasmApp;
    this.patcher = null;
    this.router = null;
    this.container = null;
    this._actionHandlers = {};
    this._computed = [];
    this._effects = [];
    this._memos = {};
    this._computing = false;
    this._bus = null;
}

Outconceive.bus = function() {
    return new OutconceiveBus();
};

// === Global Theming ===

Outconceive.theme = function(name) {
    if (typeof name === 'string') {
        document.documentElement.setAttribute('data-theme', name);
    } else if (typeof name === 'object') {
        var root = document.documentElement;
        root.setAttribute('data-theme', 'custom');
        for (var key in name) {
            if (name.hasOwnProperty(key)) {
                root.style.setProperty('--mc-' + key, name[key]);
            }
        }
    }
};

Outconceive.themes = {
    light: 'light',
    dark: 'dark',
    nord: 'nord',
};

Outconceive.prototype.from_markout = function(source) {
    this.app.from_markout(source);
    if (this.patcher) this._render();
    return this;
};

Outconceive.prototype.mount = function(target) {
    if (typeof target === 'string') {
        this.container = document.getElementById(target);
        if (!this.container) {
            this.container = document.querySelector(target);
        }
    } else {
        this.container = target;
    }

    if (!this.container) {
        throw new Error('Outconceive: mount target not found: ' + target);
    }

    this.patcher = new DomPatcher(this.container);
    this._render();

    this.router = new EventRouter(this.container, this.app, this.patcher);
    for (var action in this._actionHandlers) {
        this.router.on(action, this._actionHandlers[action]);
    }
    this.router.attach();

    // Wire fetch handler
    var self = this;
    this.router._fetchHandler = function(stateKey, url) {
        self.fetch(stateKey, url);
    };

    // Wrap event router to run computed after input events
    this._wrapEventRouter();

    // Initialize editor containers
    this._initEditors();

    // Initial computed pass
    this._runComputed();

    return this;
};

Outconceive.prototype.hydrate = function(target) {
    if (typeof target === 'string') {
        this.container = document.getElementById(target);
        if (!this.container) {
            this.container = document.querySelector(target);
        }
    } else {
        this.container = target;
    }

    if (!this.container) {
        throw new Error('Outconceive: hydrate target not found: ' + target);
    }

    // Don't render — DOM already exists from SSR
    // Just set up the patcher (for future updates) and event router
    this.patcher = new DomPatcher(this.container);

    // Cache the initial VDOM so diffs work from here
    this.app.initial_render();

    this.router = new EventRouter(this.container, this.app, this.patcher);
    for (var action in this._actionHandlers) {
        this.router.on(action, this._actionHandlers[action]);
    }
    this.router.attach();

    var self = this;
    this.router._fetchHandler = function(stateKey, url) {
        self.fetch(stateKey, url);
    };

    this._wrapEventRouter();
    this._runComputed();

    return this;
};

Outconceive.prototype.unmount = function() {
    this._destroyEditors();
    if (this.container) {
        this.container.innerHTML = '';
    }
    this.patcher = null;
    this.router = null;
    this.container = null;
    return this;
};

Outconceive.prototype.on = function(action, handler) {
    var self = this;
    var wrapped = function(wasmApp) {
        handler(wasmApp);
        self._runComputed();
    };
    this._actionHandlers[action] = wrapped;
    if (this.router) {
        this.router.on(action, wrapped);
    }
    return this;
};

// === Computed State ===

Outconceive.prototype.computed = function(key, deps, fn) {
    this._computed.push({ key: key, deps: deps, fn: fn });
    // Run immediately to set initial value
    this._runSingleComputed({ key: key, deps: deps, fn: fn });
    return this;
};

Outconceive.prototype._runComputed = function() {
    if (this._computing) return;
    this._computing = true;

    var changed = true;
    var maxPasses = 10;
    var pass = 0;

    while (changed && pass < maxPasses) {
        changed = false;
        pass++;
        for (var i = 0; i < this._computed.length; i++) {
            if (this._runSingleComputed(this._computed[i])) {
                changed = true;
            }
        }
    }

    this._computing = false;

    // Run effects after all computed values settle
    this._runEffects();
};

Outconceive.prototype._runSingleComputed = function(def) {
    var self = this;
    var getter = function(k) { return self.app.get_state(k); };
    getter.bool = function(k) { return self.app.get_state_bool(k); };
    getter.count = function(k) { return self.app.get_list_count(k); };

    var newValue = def.fn(getter);
    var oldValue = this.app.get_state(def.key);

    if (String(newValue) !== String(oldValue)) {
        var patches = this.app.update_state(def.key, String(newValue));
        if (this.patcher) this.patcher.applyPatches(patches);
        return true;
    }
    return false;
};

// === Effects ===

Outconceive.prototype.effect = function(deps, fn) {
    var self = this;
    var prev = {};
    for (var i = 0; i < deps.length; i++) {
        prev[deps[i]] = this.app.get_state(deps[i]);
    }
    this._effects.push({ deps: deps, fn: fn, prev: prev });
    return this;
};

Outconceive.prototype._runEffects = function() {
    var self = this;
    for (var i = 0; i < this._effects.length; i++) {
        var eff = this._effects[i];
        var changed = false;
        for (var j = 0; j < eff.deps.length; j++) {
            var key = eff.deps[j];
            var current = this.app.get_state(key);
            if (current !== eff.prev[key]) {
                changed = true;
                eff.prev[key] = current;
            }
        }
        if (changed) {
            eff.fn(function(k) { return self.app.get_state(k); });
        }
    }
};

// === Memo ===

Outconceive.prototype.memo = function(key, deps, fn) {
    this._memos[key] = { deps: deps, fn: fn, value: undefined, prevDeps: null };
    return this;
};

Outconceive.prototype.getMemo = function(key) {
    var m = this._memos[key];
    if (!m) return undefined;

    var self = this;
    var currentDeps = m.deps.map(function(d) { return self.app.get_state(d); });

    if (!m.prevDeps || !arraysEqual(currentDeps, m.prevDeps)) {
        m.value = m.fn(function(k) { return self.app.get_state(k); });
        m.prevDeps = currentDeps;
    }

    return m.value;
};

// === State ===

Outconceive.prototype.set = function(key, value) {
    this._lastSetKey = key;
    if (typeof value === 'boolean') {
        if (value !== this.app.get_state_bool(key)) {
            var patches = this.app.toggle_state(key);
            if (this.patcher) this.patcher.applyPatches(patches);
        }
    } else {
        var patches = this.app.update_state(key, String(value));
        if (this.patcher) this.patcher.applyPatches(patches);
    }
    this._saveState();
    this._runComputed();
    return this;
};

Outconceive.prototype.get = function(key) {
    return this.app.get_state(key);
};

Outconceive.prototype.getBool = function(key) {
    return this.app.get_state_bool(key);
};

Outconceive.prototype.source = function() {
    return this.app.to_markout();
};

// === Bus Integration ===

Outconceive.prototype.connect = function(bus) {
    this._bus = bus;
    return this;
};

Outconceive.prototype.publish = function(event, data) {
    if (this._bus) this._bus.emit(event, data);
    return this;
};

Outconceive.prototype.subscribe = function(event, handler) {
    var self = this;
    if (this._bus) {
        this._bus.on(event, function(data) {
            handler(data, self);
        });
    }
    return this;
};

Outconceive.prototype.theme = function(name) {
    if (this.container) {
        if (typeof name === 'string') {
            this.container.setAttribute('data-theme', name);
        } else if (typeof name === 'object') {
            this.container.setAttribute('data-theme', 'custom');
            for (var key in name) {
                if (name.hasOwnProperty(key)) {
                    this.container.style.setProperty('--mc-' + key, name[key]);
                }
            }
        }
    }
    return this;
};

Outconceive.prototype.syncState = function(key, bus) {
    var b = bus || this._bus;
    if (!b) return this;
    var self = this;

    // Push local changes to bus
    this.effect([key], function(get) {
        b.set(key, get(key));
    });

    // Pull bus changes to local
    b.watch(key, function(value) {
        var current = self.app.get_state(key);
        if (current !== value) {
            self._applyState(key, String(value));
            self._runComputed();
        }
    });

    return this;
};

// === Persistence ===

Outconceive.prototype.persist = function(namespace, keys) {
    this._persist = {
        namespace: 'outconceive:' + namespace,
        keys: keys || null,
    };

    // Restore saved state
    this._restoreState();

    return this;
};

Outconceive.prototype._restoreState = function() {
    if (!this._persist) return;
    var ns = this._persist.namespace;

    try {
        var saved = localStorage.getItem(ns);
        if (!saved) return;
        var data = JSON.parse(saved);
        var keys = Object.keys(data);

        for (var i = 0; i < keys.length; i++) {
            var key = keys[i];
            var value = data[key];
            if (value.type === 'bool') {
                var current = this.app.get_state_bool(key);
                if (current !== value.value) {
                    this.app.toggle_state(key);
                }
            } else {
                this.app.update_state(key, String(value.value));
            }
        }

        // Re-render after restoring
        if (this.patcher) {
            var patches = this.app.render();
            this.patcher.applyPatches(patches);
        }
    } catch (e) {
        // Ignore corrupt data
    }
};

Outconceive.prototype._saveState = function() {
    if (!this._persist) return;
    var ns = this._persist.namespace;
    var keys = this._persist.keys;

    try {
        var saved = {};
        var existing = localStorage.getItem(ns);
        if (existing) {
            try { saved = JSON.parse(existing); } catch(e) {}
        }

        if (keys) {
            for (var i = 0; i < keys.length; i++) {
                var k = keys[i];
                saved[k] = { type: 'text', value: this.app.get_state(k) };
            }
        } else {
            // Save the key that was just changed — we track via _lastSetKey
            if (this._lastSetKey) {
                saved[this._lastSetKey] = { type: 'text', value: this.app.get_state(this._lastSetKey) };
            }
        }

        localStorage.setItem(ns, JSON.stringify(saved));
    } catch (e) {
        // Storage full or unavailable
    }
};

// === Animations ===

Outconceive.prototype.animate = function(stateKey, animationName) {
    if (!this.container) return this;
    var el = this.container.querySelector('[data-bind="' + stateKey + '"]');
    if (!el) return this;
    el.classList.remove('mc-animate-' + animationName);
    void el.offsetWidth; // force reflow
    el.classList.add('mc-animate-' + animationName);
    return this;
};

Outconceive.prototype.clearPersisted = function() {
    if (this._persist) {
        localStorage.removeItem(this._persist.namespace);
    }
    return this;
};

Outconceive.prototype._render = function() {
    var vdom = this.app.initial_render();
    this.patcher.renderInitial(vdom);
};

Outconceive.prototype._wrapEventRouter = function() {
    var self = this;
    var origAttach = this.container;

    // Listen for input events to trigger computed + persist after user typing
    this.container.addEventListener('input', function(e) {
        var bind = e.target && e.target.getAttribute && e.target.getAttribute('data-bind');
        if (bind) self._lastSetKey = bind;
        setTimeout(function() {
            self._runComputed();
            self._saveState();
        }, 0);
    });

    this.container.addEventListener('change', function(e) {
        var bind = e.target && e.target.getAttribute && e.target.getAttribute('data-bind');
        if (bind) self._lastSetKey = bind;
        setTimeout(function() {
            self._runComputed();
            self._saveState();
        }, 0);
    });
};

// === Fetch ===

Outconceive.prototype.fetch = function(stateKey, url, options) {
    var self = this;
    options = options || {};
    var method = options.method || 'GET';
    var body = options.body || null;
    var headers = options.headers || {};
    var transform = options.transform || null;
    var target = options.target || stateKey;

    // Set loading state
    this._applyState(stateKey + '._loading', 'true');
    this._applyState(stateKey + '._error', '');

    var fetchOpts = { method: method, headers: headers };
    if (body) {
        fetchOpts.body = typeof body === 'string' ? body : JSON.stringify(body);
        if (!headers['Content-Type']) {
            fetchOpts.headers['Content-Type'] = 'application/json';
        }
    }

    return window.fetch(url, fetchOpts)
        .then(function(response) {
            if (!response.ok) throw new Error('HTTP ' + response.status);
            return response.json();
        })
        .then(function(data) {
            if (transform) data = transform(data);

            if (Array.isArray(data)) {
                // Populate as list items
                // Clear existing items
                var oldCount = self.app.get_list_count(target);
                for (var i = oldCount - 1; i >= 0; i--) {
                    self.app.remove_list_item(target, i);
                }
                // Add new items
                for (var i = 0; i < data.length; i++) {
                    self.app.add_list_item(target, JSON.stringify(data[i]));
                }
            } else if (typeof data === 'object' && data !== null) {
                // Populate as flat state
                for (var key in data) {
                    if (data.hasOwnProperty(key)) {
                        self._applyState(target + '.' + key, String(data[key]));
                    }
                }
            } else {
                self._applyState(target, String(data));
            }

            self._applyState(stateKey + '._loading', '');
            self._runComputed();

            if (options.onSuccess) options.onSuccess(data);
        })
        .catch(function(err) {
            self._applyState(stateKey + '._loading', '');
            self._applyState(stateKey + '._error', err.message);
            if (options.onError) options.onError(err);
        });
};

Outconceive.prototype._applyState = function(key, value) {
    var patches = this.app.update_state(key, value);
    if (this.patcher) this.patcher.applyPatches(patches);
};

// === Validation ===

var VALIDATORS = {
    required: function(value) {
        return value.trim() !== '' ? null : 'This field is required';
    },
    email: function(value) {
        if (!value) return null;
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value) ? null : 'Invalid email address';
    },
    number: function(value) {
        if (!value) return null;
        return !isNaN(parseFloat(value)) ? null : 'Must be a number';
    },
    url: function(value) {
        if (!value) return null;
        try { new URL(value); return null; } catch(e) { return 'Invalid URL'; }
    }
};

function parseValidateRule(rule) {
    var colonIdx = rule.indexOf(':');
    if (colonIdx === -1) return { name: rule, param: null };
    return { name: rule.substring(0, colonIdx), param: rule.substring(colonIdx + 1) };
}

function runValidation(value, rules) {
    var ruleList = rules.split(',');
    for (var i = 0; i < ruleList.length; i++) {
        var parsed = parseValidateRule(ruleList[i].trim());
        var error = null;

        if (VALIDATORS[parsed.name]) {
            error = VALIDATORS[parsed.name](value, parsed.param);
        } else if (parsed.name === 'min') {
            var min = parseFloat(parsed.param);
            if (value.length < min) error = 'Minimum ' + min + ' characters';
        } else if (parsed.name === 'max') {
            var max = parseFloat(parsed.param);
            if (value.length > max) error = 'Maximum ' + max + ' characters';
        } else if (parsed.name === 'pattern') {
            if (value && !new RegExp(parsed.param).test(value)) error = 'Invalid format';
        }

        if (error) return error;
    }
    return null;
}

Outconceive.prototype.validate = function() {
    if (!this.container) return { valid: true, errors: {} };

    var errors = {};
    var valid = true;
    var fields = this.container.querySelectorAll('[data-validate]');

    for (var i = 0; i < fields.length; i++) {
        var field = fields[i];
        var rules = field.getAttribute('data-validate');
        var bind = field.getAttribute('data-bind');
        var value = field.value || '';

        var error = runValidation(value, rules);

        // Update DOM
        var errorEl = field.parentNode.querySelector('.mc-error[data-for="' + bind + '"]');
        if (error) {
            valid = false;
            errors[bind] = error;
            field.classList.add('mc-invalid');
            field.classList.remove('mc-valid');
            if (!errorEl) {
                errorEl = document.createElement('span');
                errorEl.className = 'mc-error';
                errorEl.setAttribute('data-for', bind);
                field.parentNode.appendChild(errorEl);
            }
            errorEl.textContent = error;
        } else {
            field.classList.remove('mc-invalid');
            field.classList.add('mc-valid');
            if (errorEl) errorEl.remove();
        }
    }

    return { valid: valid, errors: errors };
};

Outconceive.prototype.clearValidation = function() {
    if (!this.container) return;
    var invalids = this.container.querySelectorAll('.mc-invalid');
    for (var i = 0; i < invalids.length; i++) {
        invalids[i].classList.remove('mc-invalid');
    }
    var errors = this.container.querySelectorAll('.mc-error');
    for (var i = 0; i < errors.length; i++) {
        errors[i].remove();
    }
};

Outconceive.prototype.addValidator = function(name, fn) {
    VALIDATORS[name] = fn;
    return this;
};

function arraysEqual(a, b) {
    if (a.length !== b.length) return false;
    for (var i = 0; i < a.length; i++) {
        if (a[i] !== b[i]) return false;
    }
    return true;
}

// === Editor Integration ===

Outconceive.prototype._initEditors = function() {
    if (!this.container || typeof EditorBridge === 'undefined') return;
    this._editors = [];
    var editorDivs = this.container.querySelectorAll('[data-editor="true"]');
    for (var i = 0; i < editorDivs.length; i++) {
        var bridge = new EditorBridge(editorDivs[i], this.app);
        this._editors.push(bridge);
    }
};

Outconceive.prototype._destroyEditors = function() {
    if (this._editors) {
        for (var i = 0; i < this._editors.length; i++) {
            this._editors[i].destroy();
        }
        this._editors = [];
    }
};

Outconceive.prototype.getEditor = function(bindKey) {
    if (!this._editors) return null;
    for (var i = 0; i < this._editors.length; i++) {
        if (this._editors[i].bindKey === bindKey) return this._editors[i];
    }
    return null;
};

window.Outconceive = Outconceive;
