'use strict';

function EventRouter(container, app, patcher) {
    this.container = container;
    this.app = app;
    this.patcher = patcher;
    this._actionHandlers = {};
}

EventRouter.prototype.attach = function() {
    var self = this;

    this.container.addEventListener('input', function(e) {
        var target = e.target;
        var bind = target.getAttribute('data-bind');
        if (!bind) return;

        var value = target.value;
        if (target.type === 'checkbox') {
            var patches = self.app.toggle_state(bind);
            self.patcher.applyPatches(patches);
        } else {
            var patches = self.app.update_state(bind, value);
            self.patcher.applyPatches(patches);
        }
    });

    this.container.addEventListener('click', function(e) {
        // Handle remove:list:index actions
        var logicEl = e.target.closest('[data-logic]');
        if (logicEl) {
            var logic = logicEl.getAttribute('data-logic');
            var removeMatch = logic.match(/^remove:([^:]+):(\d+)$/);
            if (removeMatch) {
                e.preventDefault();
                var patches = self.app.remove_list_item(removeMatch[1], parseInt(removeMatch[2], 10));
                self.patcher.applyPatches(patches);
                return;
            }
        }

        // Handle data-fetch buttons
        var fetchEl = e.target.closest('[data-fetch]');
        if (fetchEl) {
            e.preventDefault();
            var url = fetchEl.getAttribute('data-fetch');
            var bind = fetchEl.getAttribute('data-bind') || fetchEl.getAttribute('data-action') || 'data';
            if (self._fetchHandler) {
                self._fetchHandler(bind, url);
            }
            return;
        }

        // Handle data-action buttons
        var target = e.target.closest('[data-action]');
        if (!target) return;

        e.preventDefault();
        var action = target.getAttribute('data-action');
        if (action && self._actionHandlers[action]) {
            self._actionHandlers[action](self.app);
        }
    });
};

EventRouter.prototype.on = function(actionName, handler) {
    this._actionHandlers[actionName] = handler;
};

window.EventRouter = EventRouter;
