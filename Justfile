dev:
    cargo run -- serve mcp

build:
    cargo build

release:
    cargo build --release

check:
    cargo check

lint:
    cargo clippy -- -D warnings

fmt:
    cargo fmt

fmt-toml:
    taplo format

check-toml:
    taplo check

test:
    cargo nextest run

test-ci:
    cargo nextest run --profile ci

# ── xtask delegation ──────────────────────────────────────────────────────────

dist:
    cargo xtask dist

ci:
    cargo xtask ci

symlink-docs:
    cargo xtask symlink-docs

check-env:
    cargo xtask check-env

# ── Docker ────────────────────────────────────────────────────────────────────

docker-build:
    docker build -f config/Dockerfile -t unifi-rmcp .

docker-up:
    docker compose up -d

docker-down:
    docker compose down

restart:
    docker compose restart

logs:
    docker compose logs -f

health:
    curl -sf http://localhost:40030/health | jq .

# Recreate the container from the current image (useful after config/env changes)
repair:
    #!/usr/bin/env bash
    set -euo pipefail
    docker compose down || true
    docker compose up -d
    echo "unifi-rmcp: restarted"

# ── Install / setup ───────────────────────────────────────────────────────────

install:
    #!/usr/bin/env bash
    set -euo pipefail
    bash install.sh

setup:
    cp -n .env.example .env || true
    echo "Edit .env with your UNIFI_URL, UNIFI_API_KEY, and UNIFI_MCP_TOKEN"

gen-token:
    openssl rand -hex 32

# ── Testing ───────────────────────────────────────────────────────────────────

test-mcporter:
    bash tests/mcporter/test-tools.sh

# ── Plugin / skills validation ───────────────────────────────────────────────

validate-skills:
    bash scripts/validate-plugin-layout.sh

validate-plugin: validate-skills

runtime-current:
    bash scripts/check-runtime-current.sh --unit unifi-rmcp.service --service unifi-rmcp --expected-binary target/release/runifi

# ── Release / publish ─────────────────────────────────────────────────────────

# Generate a standalone CLI for this server (requires running server; HTTP-only transport)
generate-cli:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Server must be running on port 40030 (run 'just dev' first)"
    echo "Generated CLI embeds your token — do not commit or share"
    mkdir -p dist dist/.cache
    current_hash=$(timeout 10 curl -sf \
      -H "Authorization: Bearer ${UNIFI_MCP_TOKEN:-}" \
      -H "Accept: application/json, text/event-stream" \
      http://localhost:40030/mcp/tools/list 2>/dev/null | sha256sum | cut -d' ' -f1 || echo "nohash")
    cache_file="dist/.cache/unifi-rmcp-cli.schema_hash"
    if [[ -f "$cache_file" ]] && [[ "$(cat "$cache_file")" == "$current_hash" ]] && [[ -f "dist/unifi-rmcp-cli" ]]; then
      echo "SKIP: tool schema unchanged — use existing dist/unifi-rmcp-cli"
      exit 0
    fi
    timeout 30 mcporter generate-cli \
      --command http://localhost:40030/mcp \
      --header "Authorization: Bearer ${UNIFI_MCP_TOKEN:-}" \
      --name unifi-rmcp-cli \
      --output dist/unifi-rmcp-cli
    printf '%s' "$current_hash" > "$cache_file"
    echo "Generated dist/unifi-rmcp-cli"

clean:
    cargo clean
    rm -rf .cache/ dist/

# Install the release binary into the plugin bin/ directory (Linux only; requires git lfs)
build-plugin: release
    #!/bin/sh
    set -eu
    target_dir="${CARGO_TARGET_DIR:-target}"
    if [ ! -x "$target_dir/release/runifi" ] && [ -x ".cache/cargo/release/runifi" ]; then
      target_dir=".cache/cargo"
    fi
    mkdir -p bin plugins/unifi/bin
    install -m 755 "$target_dir/release/runifi" bin/runifi
    install -m 755 "$target_dir/release/runifi" plugins/unifi/bin/runifi

# Explicit binary artifact sync. This replaces hidden Cargo rustc-wrapper side effects.
sync-bin: build-plugin

# Publish: bump version, tag, push (triggers crates.io + Docker publish)
publish bump="patch":
    #!/usr/bin/env bash
    set -euo pipefail
    [ "$(git branch --show-current)" = "main" ] || { echo "Switch to main first"; exit 1; }
    [ -z "$(git status --porcelain)" ] || { echo "Commit or stash changes first"; exit 1; }
    git pull origin main
    CURRENT=$(grep -m1 "^version" Cargo.toml | sed "s/.*\"\(.*\)\".*/\1/")
    IFS="." read -r major minor patch <<< "$CURRENT"
    case "{{bump}}" in
      major) major=$((major+1)); minor=0; patch=0 ;;
      minor) minor=$((minor+1)); patch=0 ;;
      patch) patch=$((patch+1)) ;;
      *) echo "Usage: just publish [major|minor|patch]"; exit 1 ;;
    esac
    NEW="${major}.${minor}.${patch}"
    echo "Version: ${CURRENT} → ${NEW}"
    sed -i "s/^version = \"${CURRENT}\"/version = \"${NEW}\"/" Cargo.toml
    cargo check 2>/dev/null || true
    git add -A && git commit -m "release: v${NEW}" && git tag "v${NEW}" && git push origin main --tags
    echo "Tagged v${NEW} — publish workflow will run automatically"

# Refresh local reference documentation (crawls + repomix)
refresh-docs:
    bash scripts/refresh-docs.sh

# Refresh docs — repomix packs only (no crawl)
refresh-docs-repomix:
    bash scripts/refresh-docs.sh --skip-crawl

# Refresh docs — crawl only (no repomix)
refresh-docs-crawl:
    bash scripts/refresh-docs.sh --skip-repomix

# Dry-run: print what would be refreshed
refresh-docs-dry:
    bash scripts/refresh-docs.sh --dry-run
