# Script Templates for CLI Skills

## Python — Analysis Script

```python
#!/usr/bin/env python3
"""Analyze [domain]. Stdlib only.
Usage: python analyze.py data.json [--format json] [--threshold N]
"""
import json, argparse, sys
from datetime import datetime

class Analyzer:
    def __init__(self, data):
        self.data = data
        self.errors = []

    def validate(self):
        if not isinstance(self.data, dict):
            self.errors.append("Input must be a JSON object")
        if self.errors:
            return False
        return True

    def analyze(self):
        return {
            "timestamp": datetime.now().isoformat(),
            "summary": {},
            "details": [],
            "recommendations": []
        }

    def format_text(self, result):
        lines = [f"# Analysis — {result['timestamp']}", ""]
        lines.append("## Summary")
        for k, v in result["summary"].items():
            lines.append(f"- {k}: {v}")
        if result["recommendations"]:
            lines.append("\n## Recommendations")
            for r in result["recommendations"]:
                lines.append(f"- {r}")
        return "\n".join(lines)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input_file")
    parser.add_argument("--format", choices=["text", "json"], default="text")
    parser.add_argument("--threshold", type=float, default=0.5)
    args = parser.parse_args()

    try:
        with open(args.input_file) as f:
            data = json.load(f)
        analyzer = Analyzer(data)
        if not analyzer.validate():
            for err in analyzer.errors:
                print(f"Validation error: {err}", file=sys.stderr)
            sys.exit(1)
        result = analyzer.analyze()
        if args.format == "json":
            print(json.dumps(result, indent=2, default=str))
        else:
            print(analyzer.format_text(result))
    except FileNotFoundError:
        print(f"Error: '{args.input_file}' not found", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: invalid JSON: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
```

---

## Bash — Command Wrapper

```bash
#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-help}"
shift 2>/dev/null || true

case "$ACTION" in
    scan)
        if [ -z "${1:-}" ]; then
            echo "Error: path required" >&2
            exit 1
        fi
        TARGET="$1"
        if [ ! -e "$TARGET" ]; then
            echo "Error: '$TARGET' does not exist" >&2
            exit 1
        fi
        echo "Scanning $TARGET..."
        ;;
    report)
        echo "Report generated."
        ;;
    status)
        echo "OK"
        ;;
    help|*)
        echo "Usage: $0 <scan|report|status> [args...]"
        exit 0
        ;;
esac
```

---

## Python — Generator Script

Produces a file output (not just text):

```python
#!/usr/bin/env python3
"""Generate [artifact]. Stdlib only.
Usage: python generate.py config.json --output result.html
"""
import json, argparse, sys

def validate_config(config):
    required = ["title", "data"]
    missing = [k for k in required if k not in config]
    if missing:
        print(f"Error: missing fields: {', '.join(missing)}", file=sys.stderr)
        sys.exit(1)

def generate(config):
    return f"<html><body><h1>{config['title']}</h1></body></html>"

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("config_file")
    parser.add_argument("--output", "-o", required=True)
    args = parser.parse_args()

    try:
        with open(args.config_file) as f:
            config = json.load(f)
        validate_config(config)
        content = generate(config)
        with open(args.output, "w") as f:
            f.write(content)
        print(json.dumps({
            "status": "success",
            "output": args.output,
            "size_bytes": len(content)
        }, indent=2))
    except FileNotFoundError:
        print(f"Error: '{args.config_file}' not found", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
```

---

## Common Patterns

### Safe JSON output
```python
json.dumps(result, indent=2, default=str, ensure_ascii=False)
```

### Read from stdin OR file
```python
if args.input_file == "-":
    data = json.load(sys.stdin)
else:
    with open(args.input_file) as f:
        data = json.load(f)
```

### Logging to stderr (not stdout)
```python
def log(msg):
    print(f"[INFO] {msg}", file=sys.stderr)
```
