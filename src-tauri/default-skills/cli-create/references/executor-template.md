# Runtime Executor Templates

A runtime executor lets the LLM write custom code and execute it.
The LLM has access to the full library API, not just pre-built commands.

## Architecture

```
my-skill/
├── SKILL.md                ← Essential commands + workflow
├── run.js (or run.py)      ← Universal executor
├── lib/
│   └── helpers.js          ← Reusable functions
├── references/
│   └── api-reference.md    ← Full API docs (loaded on demand)
└── package.json            ← Dependencies
```

---

## Node.js Runner Template

```javascript
#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

process.chdir(__dirname);

if (!fs.existsSync(path.join(__dirname, 'node_modules'))) {
    console.error('[setup] Installing dependencies...');
    execSync('npm install', { cwd: __dirname, stdio: 'inherit' });
}

let code;
const args = process.argv.slice(2);

if (args[0] === '-e' && args[1]) {
    code = args[1];
} else if (args[0] === '-') {
    code = fs.readFileSync('/dev/stdin', 'utf8');
} else if (args[0]) {
    code = fs.readFileSync(args[0], 'utf8');
} else {
    console.error('Usage: node run.js <file.js | -e "code" | ->');
    process.exit(1);
}

const isCompleteScript = code.includes('require(') ||
    code.includes('import ') ||
    code.includes('module.exports');

if (!isCompleteScript) {
    code = `
const { helpers } = require('./lib/helpers');
(async () => {
    try {
        ${code}
    } catch (err) {
        console.error('Execution error:', err.message);
        process.exit(1);
    }
})();
`;
}

const tempFile = path.join(__dirname, `.temp-execution-${Date.now()}.js`);
fs.writeFileSync(tempFile, code);

try {
    require(tempFile);
} finally {
    if (fs.existsSync(tempFile)) fs.unlinkSync(tempFile);
}
```

---

## Python Runner Template

```python
#!/usr/bin/env python3
"""Runtime executor — runs LLM-generated code."""
import sys, os, tempfile, importlib.util, argparse, glob

os.chdir(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'lib'))

def execute_code(code):
    fd, temp_path = tempfile.mkstemp(prefix='.temp-exec-', suffix='.py', dir='.')
    try:
        with os.fdopen(fd, 'w') as f:
            f.write(code)
        spec = importlib.util.spec_from_file_location("__main__", temp_path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    finally:
        if os.path.exists(temp_path):
            os.unlink(temp_path)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", nargs="?")
    parser.add_argument("-e", "--eval", help="Code to execute")
    args = parser.parse_args()

    if args.eval:
        code = args.eval
    elif args.source == "-":
        code = sys.stdin.read()
    elif args.source:
        with open(args.source) as f:
            code = f.read()
    else:
        print("Usage: python run.py <file.py | -e 'code' | ->", file=sys.stderr)
        sys.exit(1)

    try:
        execute_code(code)
    except Exception as e:
        print(f"Execution failed: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
```

---

## Helpers Template (lib/helpers.js)

```javascript
async function retry(fn, maxRetries = 3, delay = 1000) {
    for (let i = 0; i < maxRetries; i++) {
        try { return await fn(); }
        catch (err) {
            if (i === maxRetries - 1) throw err;
            await new Promise(r => setTimeout(r, delay * (i + 1)));
        }
    }
}

async function withTimeout(fn, ms = 30000) {
    return Promise.race([
        fn(),
        new Promise((_, reject) =>
            setTimeout(() => reject(new Error(`Timeout after ${ms}ms`)), ms))
    ]);
}

function writeResult(data, outputPath) {
    const fs = require('fs');
    const content = typeof data === 'string' ? data : JSON.stringify(data, null, 2);
    fs.writeFileSync(outputPath, content);
    console.log(`Output written to ${outputPath}`);
}

module.exports = { retry, withTimeout, writeResult };
```

---

## When to use executor vs pre-built commands

| Criteria | Pre-built commands | Runtime executor |
|----------|-------------------|------------------|
| Library API | Simple, < 20 commands | Rich, 100+ methods |
| Needs | Predictable, repetitive | Variable per use |
| Code complexity | 1-2 lines per action | Multi-line scripts |
| Security | More controlled | LLM writes arbitrary code |
| Setup | Zero (documented commands) | package.json + install |
