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

# ── 8. Verify contains edge via sqlite3 (B.3) ────────────────────────────────
log "verifying persisted contains edge via sqlite3 ..."
EDGE_RESULT=$(sqlite3 "$DEMO_DIR/.clarion/clarion.db" \
    "select kind, from_id, to_id from edges order by from_id, to_id;")
EDGE_EXPECTED="contains|python:module:demo|python:function:demo.hello"

if [ "$EDGE_RESULT" != "$EDGE_EXPECTED" ]; then
    log "DB edge contents:"
    sqlite3 "$DEMO_DIR/.clarion/clarion.db" "select * from edges;" >&2 || true
    fail "expected edge row:\n$EDGE_EXPECTED\ngot:\n$EDGE_RESULT"
fi

# ── 9. Verify run stats include edges_inserted == 1 (B.3 §6) ─────────────────
log "verifying run stats include edges_inserted == 1 ..."
EDGES_INSERTED=$(sqlite3 "$DEMO_DIR/.clarion/clarion.db" \
    "select json_extract(stats, '\$.edges_inserted') from runs where status = 'completed';")
if [ "$EDGES_INSERTED" != "1" ]; then
    log "runs row:"
    sqlite3 "$DEMO_DIR/.clarion/clarion.db" "select id, status, stats from runs;" >&2 || true
    fail "expected runs.stats.edges_inserted == 1; got $EDGES_INSERTED"
fi

# ── 10. Verify dropped_edges_total == 0 (B.3 §6 / §9 exit criterion 6) ───────
log "verifying run stats include dropped_edges_total == 0 ..."
DROPPED=$(sqlite3 "$DEMO_DIR/.clarion/clarion.db" \
    "select json_extract(stats, '\$.dropped_edges_total') from runs where status = 'completed';")
if [ "$DROPPED" != "0" ]; then
    log "runs row:"
    sqlite3 "$DEMO_DIR/.clarion/clarion.db" "select id, status, stats from runs;" >&2 || true
    fail "expected runs.stats.dropped_edges_total == 0; got $DROPPED"
fi

log "PASS: walking skeleton persisted module + function entities + contains edge"
