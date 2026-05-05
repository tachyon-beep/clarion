#!/usr/bin/env bash
# Sprint 1 walking-skeleton end-to-end demo (WP3 Task 9 / signoffs A.4).
#
# Runs the README §3 demo script end-to-end and verifies:
#   - `clarion install` creates `.clarion/clarion.db`
#   - `clarion analyze .` spawns the Python plugin and persists at least one entity
#   - `sqlite3 .clarion/clarion.db` returns `python:function:demo.hello|function`
#
# Dependencies: cargo, Python 3.11+, sqlite3 CLI.
#
# Env overrides:
#   REPO_ROOT   — auto-detected via `git rev-parse`; override to test an external checkout.
#   VENV        — defaults to $REPO_ROOT/plugins/python/.venv; override to reuse an existing venv.
#   CARGO_BUILD — set to "0" to skip `cargo build` (assumes target/release/clarion already present).

set -euo pipefail

REPO_ROOT="${REPO_ROOT:-$(git rev-parse --show-toplevel)}"
VENV="${VENV:-$REPO_ROOT/plugins/python/.venv}"
CARGO_BUILD="${CARGO_BUILD:-1}"

log() { printf '[walking-skeleton] %s\n' "$*" >&2; }
fail() { printf '[walking-skeleton] FAIL: %s\n' "$*" >&2; exit 1; }

cd "$REPO_ROOT"

# ── 1. Build clarion binary ──────────────────────────────────────────────────
if [ "$CARGO_BUILD" = "1" ]; then
    log "building clarion (release) ..."
    cargo build --workspace --release
fi
CLARION_BIN="$REPO_ROOT/target/release/clarion"
[ -x "$CLARION_BIN" ] || fail "clarion binary missing at $CLARION_BIN"

# ── 2. Install Python plugin (editable) ──────────────────────────────────────
if [ ! -d "$VENV" ]; then
    log "creating venv at $VENV ..."
    python3 -m venv "$VENV"
fi
log "installing clarion-plugin-python (editable) ..."
"$VENV/bin/pip" install --quiet -e "$REPO_ROOT/plugins/python[dev]"
PLUGIN_BIN="$VENV/bin/clarion-plugin-python"
[ -x "$PLUGIN_BIN" ] || fail "clarion-plugin-python missing at $PLUGIN_BIN"
PLUGIN_MANIFEST="$VENV/share/clarion/plugins/python/plugin.toml"
[ -f "$PLUGIN_MANIFEST" ] || fail "plugin.toml missing at $PLUGIN_MANIFEST (WP2 L9 install-prefix fallback)"

# ── 3. Scratch project ───────────────────────────────────────────────────────
DEMO_DIR="$(mktemp -d -t clarion-demo-XXXXXX)"
trap 'rm -rf "$DEMO_DIR"' EXIT
log "scratch project: $DEMO_DIR"
cd "$DEMO_DIR"
echo 'def hello(): return "world"' > demo.py

# ── 4. PATH wiring — clarion + plugin binary ────────────────────────────────
export PATH="$REPO_ROOT/target/release:$VENV/bin:$PATH"

# ── 5. clarion install ───────────────────────────────────────────────────────
log "running: clarion install"
clarion install
[ -f "$DEMO_DIR/.clarion/clarion.db" ] || fail ".clarion/clarion.db not created by clarion install"

# ── 6. clarion analyze ───────────────────────────────────────────────────────
log "running: clarion analyze ."
clarion analyze .

# ── 7. Verify entity via sqlite3 ─────────────────────────────────────────────
log "verifying persisted entity via sqlite3 ..."
RESULT=$(sqlite3 "$DEMO_DIR/.clarion/clarion.db" "select id, kind from entities order by id;")
# B.2 (Sprint 2): every analyzed file emits a module entity in addition to
# its function/class entities. The demo file `def hello(): return "world"`
# produces exactly two rows.
EXPECTED="python:function:demo.hello|function
python:module:demo|module"

if [ "$RESULT" != "$EXPECTED" ]; then
    log "DB contents:"
    sqlite3 "$DEMO_DIR/.clarion/clarion.db" "select * from entities;" >&2 || true
    fail "expected exactly:\n$EXPECTED\ngot:\n$RESULT"
fi

log "PASS: walking skeleton persisted module + function entities"
