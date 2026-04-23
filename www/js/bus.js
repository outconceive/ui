'use strict';

function OutconceiveBus() {
    this._listeners = {};
    this._state = {};
}

OutconceiveBus.prototype.on = function(event, handler) {
    if (!this._listeners[event]) this._listeners[event] = [];
    this._listeners[event].push(handler);
    return this;
};

OutconceiveBus.prototype.off = function(event, handler) {
    if (!this._listeners[event]) return this;
    if (!handler) {
        delete this._listeners[event];
    } else {
        this._listeners[event] = this._listeners[event].filter(function(h) { return h !== handler; });
    }
    return this;
};

OutconceiveBus.prototype.emit = function(event, data) {
    var handlers = this._listeners[event];
    if (!handlers) return this;
    for (var i = 0; i < handlers.length; i++) {
        handlers[i](data);
    }
    return this;
};

OutconceiveBus.prototype.once = function(event, handler) {
    var self = this;
    var wrapper = function(data) {
        self.off(event, wrapper);
        handler(data);
    };
    return this.on(event, wrapper);
};

OutconceiveBus.prototype.set = function(key, value) {
    this._state[key] = value;
    this.emit('state:' + key, value);
    return this;
};

OutconceiveBus.prototype.get = function(key) {
    return this._state[key];
};

OutconceiveBus.prototype.watch = function(key, handler) {
    if (key in this._state) handler(this._state[key]);
    return this.on('state:' + key, handler);
};

// Global singleton
OutconceiveBus.global = new OutconceiveBus();

window.OutconceiveBus = OutconceiveBus;
