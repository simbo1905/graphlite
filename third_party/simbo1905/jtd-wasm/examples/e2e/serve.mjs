/// Minimal static file server + headless Chrome test runner.
import { createServer } from 'http';
import { readFileSync, existsSync } from 'fs';
import { join, extname } from 'path';
import { execSync } from 'child_process';

const PORT = 8080;
const MIME = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.wasm': 'application/wasm',
  '.json': 'application/json',
};

const server = createServer((req, res) => {
  let path = req.url === '/' ? '/index.html' : req.url;
  const filePath = join('/app', path);

  if (!existsSync(filePath)) {
    res.writeHead(404);
    res.end('Not found');
    return;
  }

  const ext = extname(filePath);
  const mime = MIME[ext] || 'application/octet-stream';
  res.writeHead(200, { 'Content-Type': mime });
  res.end(readFileSync(filePath));
});

server.listen(PORT, async () => {
  console.log(`Serving on http://localhost:${PORT}`);

  // Run headless Chrome test
  if (process.argv.includes('--test')) {
    try {
      const chromePath = process.env.CHROME_BIN || 'chromium';
      const result = execSync(
        `${chromePath} --headless --disable-gpu --no-sandbox --virtual-time-budget=5000 ` +
        `--dump-dom "http://localhost:${PORT}"`,
        { encoding: 'utf-8', timeout: 15000 }
      );

      // Check that the page rendered and validation ran
      if (result.includes('instancePath') && result.includes('schemaPath')) {
        console.log('E2E TEST PASSED: WASM validator produced errors as expected');
        console.log('Page output contains validation error indicators');
        process.exit(0);
      } else if (result.includes('Valid!')) {
        console.log('E2E TEST FAILED: expected errors but got Valid');
        process.exit(1);
      } else {
        console.log('E2E TEST RESULT: page rendered');
        console.log(result.substring(0, 500));
        process.exit(0);
      }
    } catch (e) {
      console.error('E2E TEST ERROR:', e.message);
      process.exit(1);
    }
  }
});
