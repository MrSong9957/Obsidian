#!/bin/bash
# Run the self-check client. Server is a separate process (artifact/shortlink);
# this wrapper only invokes the Python 3.12 client.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
PHASE="${1:-all}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$HERE/logs/${STAMP}-${PHASE}.log"
mkdir -p "$HERE/logs"
python3 "$HERE/selftest.py" --artifact "$ROOT/artifact" --phase "$PHASE" \
  --logdir "$HERE/logs/${STAMP}-${PHASE}" 2>&1 | tee "$OUT"
rc=${PIPESTATUS[0]}
echo "SELFCHECK_EXIT=$rc LOG=$OUT"
exit "$rc"
