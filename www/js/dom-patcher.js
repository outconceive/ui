'use strict';

function DomPatcher(container) {
    this.container = container;
}

DomPatcher.prototype.renderInitial = function(vdom) {
    this.container.innerHTML = '';
    if (vdom.type === 'Element' && vdom.children) {
        for (var i = 0; i < vdom.children.length; i++) {
            var child = this._vnodeToDOM(vdom.children[i]);
            if (child) this.container.appendChild(child);
        }
    } else {
        var node = this._vnodeToDOM(vdom);
        if (node) this.container.appendChild(node);
    }
};

DomPatcher.prototype.applyPatches = function(patches) {
    if (!patches || !patches.length) return;
    for (var i = 0; i < patches.length; i++) {
        this._applyPatch(patches[i]);
    }
};

DomPatcher.prototype._applyPatch = function(patch) {
    switch (patch.type) {
        case 'Replace':      this._applyReplace(patch); break;
        case 'Insert':       this._applyInsert(patch); break;
        case 'Remove':       this._applyRemove(patch); break;
        case 'UpdateText':   this._applyUpdateText(patch); break;
        case 'SetAttribute': this._applySetAttribute(patch); break;
        case 'RemoveAttribute': this._applyRemoveAttribute(patch); break;
    }
};

DomPatcher.prototype._applyReplace = function(patch) {
    if (!patch.path || patch.path.length === 0) {
        this.renderInitial(patch.node);
        return;
    }
    var target = this._nodeAtPath(patch.path);
    if (!target || !target.parentNode) return;
    var newNode = this._vnodeToDOM(patch.node);
    if (newNode) target.parentNode.replaceChild(newNode, target);
};

DomPatcher.prototype._applyInsert = function(patch) {
    var parentPath = patch.path.slice(0, -1);
    var index = patch.path[patch.path.length - 1];
    var parent = this._nodeAtPath(parentPath);
    if (!parent) return;
    var newNode = this._vnodeToDOM(patch.node);
    if (!newNode) return;
    if (index >= parent.childNodes.length) {
        parent.appendChild(newNode);
    } else {
        parent.insertBefore(newNode, parent.childNodes[index]);
    }
};

DomPatcher.prototype._applyRemove = function(patch) {
    var target = this._nodeAtPath(patch.path);
    if (target && target.parentNode) target.parentNode.removeChild(target);
};

DomPatcher.prototype._applyUpdateText = function(patch) {
    var target = this._nodeAtPath(patch.path);
    if (target && target.nodeType === Node.TEXT_NODE) target.textContent = patch.text;
};

DomPatcher.prototype._applySetAttribute = function(patch) {
    var target = this._nodeAtPath(patch.path);
    if (target && target.setAttribute) target.setAttribute(patch.key, patch.value);
};

DomPatcher.prototype._applyRemoveAttribute = function(patch) {
    var target = this._nodeAtPath(patch.path);
    if (target && target.removeAttribute) target.removeAttribute(patch.key);
};

DomPatcher.prototype._nodeAtPath = function(path) {
    if (!path || path.length === 0) return this.container;
    var node = this.container;
    for (var i = 0; i < path.length; i++) {
        if (!node || !node.childNodes) return null;
        if (path[i] >= node.childNodes.length) return null;
        node = node.childNodes[path[i]];
    }
    return node;
};

var SVG_NS = 'http://www.w3.org/2000/svg';
var SVG_TAGS = { svg:1, path:1, circle:1, rect:1, line:1, polyline:1, polygon:1, ellipse:1, g:1, text:1, tspan:1, defs:1, use:1, clipPath:1, mask:1, pattern:1, linearGradient:1, radialGradient:1, stop:1 };

DomPatcher.prototype._vnodeToDOM = function(vnode, svgContext) {
    if (!vnode) return null;
    if (vnode.type === 'Text') return document.createTextNode(vnode.content || '');
    if (vnode.type === 'Element') {
        var isSvg = svgContext || SVG_TAGS[vnode.tag];
        var el = isSvg
            ? document.createElementNS(SVG_NS, vnode.tag)
            : document.createElement(vnode.tag);
        if (vnode.attrs) {
            var keys = Object.keys(vnode.attrs);
            for (var i = 0; i < keys.length; i++) {
                if (isSvg) {
                    el.setAttributeNS(null, keys[i], vnode.attrs[keys[i]]);
                } else {
                    el.setAttribute(keys[i], vnode.attrs[keys[i]]);
                }
            }
        }
        if (vnode.children) {
            for (var j = 0; j < vnode.children.length; j++) {
                var child = this._vnodeToDOM(vnode.children[j], isSvg);
                if (child) el.appendChild(child);
            }
        }
        return el;
    }
    return document.createTextNode('');
};

window.DomPatcher = DomPatcher;
