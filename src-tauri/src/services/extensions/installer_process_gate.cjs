// No npm or UI module is loaded until durable native ownership is acknowledged.
const { Worker } = require('node:worker_threads');
const { pathToFileURL } = require('node:url');
const abortGroup = () => {
  try { process.kill(process.platform === 'win32' ? process.pid : -process.pid, 'SIGKILL'); }
  catch { process.exit(1); }
};
const launchTimeout = setTimeout(abortGroup, 5000);
process.stdin.once('end', abortGroup);
process.stdin.once('data', (bytes) => {
  if (bytes.length !== 1 || bytes[0] !== 1) return abortGroup();
  process.stdin.pause();
  process.stdin.unref();
  process.stdin.removeAllListeners('end');
  // Windows Job Objects already kill all descendants when the owner dies.
  // Reopening its stdin handle on a second event loop blocks during Socket open.
  if (process.platform === 'win32') {
    clearTimeout(launchTimeout);
    import(pathToFileURL(process.argv[1]).href).catch(abortGroup);
    return;
  }
  const watcher = new Worker(`
    const { parentPort } = require('node:worker_threads');
    const { Socket } = require('node:net');
    const stop = () => {
      try { process.kill(process.platform === 'win32' ? process.pid : -process.pid, 'SIGKILL'); }
      catch { process.kill(process.pid, 'SIGKILL'); }
    };
    // The worker owns an independent event loop; no blocking read delays normal exit.
    const owner = new Socket({ fd: 0, readable: true, writable: false });
    owner.on('end', stop);
    owner.on('error', stop);
    owner.on('data', stop);
    owner.resume();
    parentPort.postMessage('ready');
  `, { eval: true });
  watcher.on('error', abortGroup);
  watcher.on('exit', abortGroup);
  watcher.once('message', (message) => {
    if (message !== 'ready') return abortGroup();
    clearTimeout(launchTimeout);
    watcher.unref();
    import(pathToFileURL(process.argv[1]).href).catch(abortGroup);
  });
});
