"""Temporary isolated launch probe; remove after cross-platform root causes are proven."""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time

GATE = Path(__file__).resolve().parents[2] / 'src-tauri/src/services/extensions/installer_process_gate.cjs'
NODE = shutil.which('node')


def instrument(source):
    replacements = {
        "const { Worker }": "console.error('gate:boot'); process.on('uncaughtException',()=>{console.error('gate:uncaught');process.exit(1)}); const { Worker }",
        "const launchTimeout = setTimeout(abortGroup, 5000);": "const launchTimeout = setTimeout(()=>{console.error('gate:timeout');abortGroup()},5000);",
        "process.stdin.pause();": "console.error('gate:ack'); process.stdin.pause();",
        "process.stdin.unref();": "process.stdin.unref(); console.error('gate:unref');",
        "const watcher = new Worker": "console.error('gate:worker-create'); const watcher = new Worker",
        "const owner = new Socket": "console.error('gate:worker-boot'); const owner = new Socket",
        "parentPort.postMessage('ready');": "console.error('gate:worker-ready'); parentPort.postMessage('ready');",
        "watcher.on('error', abortGroup);": "watcher.on('error',()=>{console.error('gate:worker-error');abortGroup()});",
        "watcher.on('exit', abortGroup);": "watcher.on('exit',()=>{console.error('gate:worker-exit');abortGroup()});",
        "clearTimeout(launchTimeout);": "console.error('gate:main-ready'); clearTimeout(launchTimeout);",
    }
    for before, after in replacements.items():
        if source.count(before) != 1:
            raise RuntimeError('probe no longer matches gate')
        source = source.replace(before, after)
    return source


def launch_probe(root):
    script = root / 'producer.mjs'
    script.write_text("process.stdout.write('producer-loaded');")
    for label, args in [('direct', [str(script)]), ('gated', ['--eval', instrument(GATE.read_text()), '--', str(script)])]:
        child = subprocess.Popen([NODE, *args], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, cwd=root, start_new_session=os.name != 'nt')
        try:
            if label == 'gated':
                child.stdin.write(b'\x01')
                child.stdin.flush()
            code = child.wait(timeout=8)
        except subprocess.TimeoutExpired:
            child.kill()
            code = child.wait(timeout=2)
        finally:
            child.stdin.close()
        output = child.stdout.read(4096)
        markers = [line for line in child.stderr.read(4096).decode(errors='replace').splitlines()
                   if line in {'gate:boot', 'gate:uncaught', 'gate:timeout', 'gate:ack', 'gate:unref',
                               'gate:worker-create', 'gate:worker-boot', 'gate:worker-ready',
                               'gate:worker-error', 'gate:worker-exit', 'gate:main-ready'}]
        print(json.dumps({'case': label, 'exit': code, 'loaded': output == b'producer-loaded', 'stages': markers}), flush=True)


def parent(root):
    child = subprocess.Popen([NODE, '--eval', GATE.read_text(), '--', str(root / 'blocked.mjs')],
                             stdin=subprocess.PIPE, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                             cwd=root, start_new_session=True)
    (root / 'producer.pid').write_text(str(child.pid))
    child.stdin.write(b'\x01')
    child.stdin.flush()
    child.wait(timeout=15)


def linux_death_probe(root):
    (root / 'blocked.mjs').write_text("import fs from 'node:fs';import{spawn}from'node:child_process';const c=spawn(process.execPath,['-e',\"require('fs').writeFileSync('writer.pid',String(process.pid));setInterval(()=>require('fs').appendFileSync('writes','x'),5)\"],{stdio:'ignore'});c.unref();fs.writeFileSync('blocked','1');for(;;){};")
    owner = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--parent', str(root)])
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        if (root / 'writes').exists() and (root / 'writes').stat().st_size > 0:
            break
        time.sleep(.02)
    owner.kill()
    owner.wait(timeout=2)
    pids = [int((root / name).read_text()) for name in ['producer.pid', 'writer.pid']]
    time.sleep(3)
    states = []
    for pid in pids:
        try:
            stat = Path(f'/proc/{pid}/stat').read_text()[:4096]
            states.append(stat.rsplit(')', 1)[1].split()[0])
        except FileNotFoundError:
            states.append('absent')
    before = (root / 'writes').stat().st_size
    time.sleep(.15)
    print(json.dumps({'case': 'parent-death', 'states': states,
                      'writes_stable': before == (root / 'writes').stat().st_size}), flush=True)
    if any(state not in ['Z', 'X', 'absent'] for state in states):
        os.killpg(pids[0], signal.SIGKILL)


if __name__ == '__main__':
    if len(sys.argv) == 3 and sys.argv[1] == '--parent':
        parent(Path(sys.argv[2]))
    else:
        with tempfile.TemporaryDirectory(prefix='beaver-launch-probe-') as temporary:
            root = Path(temporary)
            launch_probe(root)
            if sys.platform == 'linux':
                linux_death_probe(root)
