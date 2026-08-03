#!/usr/bin/env bash
set -euo pipefail

max_input_bytes=65536
event_json="$(head -c "${max_input_bytes}")"
trap 'event_json=' EXIT

# Parse only required fields with a trusted JSON parser.
# Pass validated process arguments separately and never use eval.
# Filter secrets from every output.

if [[ -z "${event_json}" ]]; then
  printf '%s\n' '{"error":"invalid event"}' >&2
  exit 2
fi

printf '%s\n' '{"status":"ok"}'
