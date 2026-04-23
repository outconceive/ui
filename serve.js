const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 9096;
const ROOT = path.join(__dirname, 'www');

const MIME = {
    '.html': 'text/html',
    '.css': 'text/css',
    '.js': 'application/javascript',
    '.wasm': 'application/wasm',
    '.json': 'application/json',
    '.png': 'image/png',
    '.svg': 'image/svg+xml',
};

http.createServer(function(req, res) {
    var url = req.url.split('?')[0];
    if (url === '/') url = '/index.html';

    var filePath = path.join(ROOT, url);
    var ext = path.extname(filePath);

    fs.readFile(filePath, function(err, data) {
        if (err) {
            res.writeHead(404);
            res.end('Not found: ' + url);
            return;
        }
        res.writeHead(200, {
            'Content-Type': MIME[ext] || 'application/octet-stream',
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp',
        });
        res.end(data);
    });
}).listen(PORT, function() {
    console.log('Outconceive dev server: http://localhost:' + PORT);
});
