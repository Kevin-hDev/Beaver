// Test-only fake npm: real process IO, deterministic locked resolution and stopped replay.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

if (process.argv.includes('--package-lock-only')) {
  const name = process.argv.at(-1);
  fs.writeFileSync('package.json', JSON.stringify({ name: 'fixture', version: '1.0.0', dependencies: { [name]: '1.0.0' } }));
  fs.writeFileSync('package-lock.json', JSON.stringify({ lockfileVersion: 3, packages: { [`node_modules/${name}`]: { version: '1.0.0' } } }));
} else {
  fs.writeFileSync(path.join(path.dirname(fileURLToPath(import.meta.url)), 'pid'), String(process.pid));
  const marker = '.npm-cache/attempt';
  if (!fs.existsSync(marker)) {
    fs.writeFileSync(marker, 'started');
    setInterval(() => fs.appendFileSync('.npm-cache/cache/payload', Buffer.alloc(256)), 10);
  } else {
    const name = Object.keys(JSON.parse(fs.readFileSync('package.json', 'utf8')).dependencies)[0];
    const root = path.join('node_modules', name);
    fs.mkdirSync(root, { recursive: true });
    fs.writeFileSync(path.join(root, 'beaver-extension.json'), JSON.stringify({
      id: name, name: 'Volume fixture', version: '1.0.0', beaverApi: '1', runtime: 'node',
      main: 'index.mjs', access: 'full', apiLevel: 'stable', essential: false,
    }));
    fs.writeFileSync(path.join(root, 'index.mjs'), 'export default {};');
    fs.writeFileSync(path.join(root, 'payload'), Buffer.alloc(4096));
  }
}
