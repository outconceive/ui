'use strict';

function OutconceiveRouter(target) {
    this.target = target;
    this.routes = {};
    this.currentRoute = null;
    this.defaultRoute = null;
    this._onHashChange = this._onHashChange.bind(this);
}

OutconceiveRouter.prototype.route = function(path, markout) {
    this.routes[path] = markout;
    return this;
};

OutconceiveRouter.prototype.default = function(path) {
    this.defaultRoute = path;
    return this;
};

OutconceiveRouter.prototype.start = function() {
    window.addEventListener('hashchange', this._onHashChange);

    // Intercept clicks on [data-route] elements
    var self = this;
    document.addEventListener('click', function(e) {
        var el = e.target.closest('[data-route]');
        if (el) {
            e.preventDefault();
            self.navigate(el.getAttribute('data-route'));
        }
    });

    // Navigate to current hash or default
    var hash = window.location.hash.slice(1) || this.defaultRoute;
    if (hash) this.navigate(hash);

    return this;
};

OutconceiveRouter.prototype.stop = function() {
    window.removeEventListener('hashchange', this._onHashChange);
};

OutconceiveRouter.prototype.navigate = function(path) {
    if (path === this.currentRoute) return;

    var markout = this.routes[path];
    if (!markout) {
        markout = this.routes[this.defaultRoute];
        path = this.defaultRoute;
    }
    if (!markout) return;

    window.location.hash = path;
    this.currentRoute = path;

    if (typeof markout === 'function') {
        markout = markout();
    }

    this.target.from_markout(markout);

    // Update [data-route] active states
    var links = document.querySelectorAll('[data-route]');
    for (var i = 0; i < links.length; i++) {
        links[i].classList.toggle('route-active', links[i].getAttribute('data-route') === path);
    }
};

OutconceiveRouter.prototype._onHashChange = function() {
    var path = window.location.hash.slice(1);
    if (path && path !== this.currentRoute) {
        this.navigate(path);
    }
};

window.OutconceiveRouter = OutconceiveRouter;
