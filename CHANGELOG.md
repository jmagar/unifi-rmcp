# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2](https://github.com/jmagar/unifi-rmcp/compare/v0.2.1...v0.2.2) (2026-07-10)


### CI

* skip mcp registry publish without secret ([2647842](https://github.com/jmagar/unifi-rmcp/commit/26478425c4594c7c4ada00a69a7f3c50eeac463f))

## [0.2.0](https://github.com/jmagar/unifi-rmcp/compare/v0.1.1...v0.2.0) (2026-07-05)


### Features

* add canonical unifi internal endpoint models ([d59a575](https://github.com/jmagar/unifi-rmcp/commit/d59a575ab5d8c9f8baf8035794e8e9c19dbdf672))
* add full upstream network mcp parity ([94c5204](https://github.com/jmagar/unifi-rmcp/commit/94c5204a1ed7ac4463ec0324ada9eef0f08c00f5))
* add official and internal unifi api actions ([3334e1a](https://github.com/jmagar/unifi-rmcp/commit/3334e1a4a9e801f0745d9e55dd0396b38de758cc))
* add safe unifi verification modes ([fa48eb8](https://github.com/jmagar/unifi-rmcp/commit/fa48eb888f62723a0b95885aa975ec61bccf1852))
* add unifi endpoint verifier ([1760e89](https://github.com/jmagar/unifi-rmcp/commit/1760e89ecad87dae25e06429b5a585ec7ed8799d))
* **cli:** add 'unifi setup install' + self-install in plugin-hook ([33ae612](https://github.com/jmagar/unifi-rmcp/commit/33ae6121da90624998651ee6fd460467b6be540f))
* **config:** load ~/.unifi-rmcp/.env at startup (dotenvy, symlink-guarded); appdata resolves to ~/.unifi-rmcp, not CLAUDE_PLUGIN_DATA ([307d866](https://github.com/jmagar/unifi-rmcp/commit/307d8669e30aa8ef3caa5b9b0e0591321a2eda7c))
* enforce unifi capability auth scopes ([44be806](https://github.com/jmagar/unifi-rmcp/commit/44be80648eaee5be740be2d6cf61a0257c47b051))
* verify official unifi api parity ([be9a361](https://github.com/jmagar/unifi-rmcp/commit/be9a36195bcfb92e0bbcd1f7f2d3eb68753729e3))


### Bug Fixes

* address lavra review findings ([d18bacf](https://github.com/jmagar/unifi-rmcp/commit/d18bacfc6b668089316c569eddf644ec8474d9ed))
* address pr toolkit review findings ([69ab391](https://github.com/jmagar/unifi-rmcp/commit/69ab391d1639ebdafc09c165ece229ea478bd0f8))
* address unifi parity review findings ([9c11f7b](https://github.com/jmagar/unifi-rmcp/commit/9c11f7b9de55a75f4e327a74e9c23d4b984cee05))
* **auth:** pass std::env::vars() to lab-auth build_from_sources ([b5bfca7](https://github.com/jmagar/unifi-rmcp/commit/b5bfca71cf404d583746faca78b8c0caf99bcaea))
* classify firewall ordering as fixture gated ([605ce6c](https://github.com/jmagar/unifi-rmcp/commit/605ce6cefa599ed33ea2ccc4995b8596cea9af11))
* **compose:** use repo-name default for Docker network ([e3e90c2](https://github.com/jmagar/unifi-rmcp/commit/e3e90c25e330cc0c5886effe71896876ec09dd69))
* correct Tier 2 CLI binary name unifi -&gt; runifi ([09ddc93](https://github.com/jmagar/unifi-rmcp/commit/09ddc931b6b82573f59a71d4338ecefed9d4ac5d))
* harden unifi endpoint verification ([201ec50](https://github.com/jmagar/unifi-rmcp/commit/201ec50f54d8c00b0c5722d1a6e55e56bb3f61ff))
* harden unifi path substitution ([b5bff28](https://github.com/jmagar/unifi-rmcp/commit/b5bff28be20a9d4b0bdbc094dd6a205f73272f42))
* keep docker build workspace intact ([03495ef](https://github.com/jmagar/unifi-rmcp/commit/03495ef8a1f3ddf1c177d99116b9936f027075de))
* keep internal reference live verified ([06ac79e](https://github.com/jmagar/unifi-rmcp/commit/06ac79e79ef8935f504c8a240eea741edf59f95c))
* **oauth:** parse UNIFI_MCP_AUTH_MODE in config load ([8cd1a1f](https://github.com/jmagar/unifi-rmcp/commit/8cd1a1f2e186f16370025d18cbfced06f69f9a2d))
* remove mutation confirmation parameter ([c204684](https://github.com/jmagar/unifi-rmcp/commit/c204684ee24c27b5ffad5bb1663fa5f3a8d89cb9))
* resolve audit blockers ([87b3a86](https://github.com/jmagar/unifi-rmcp/commit/87b3a86d7852eb5ffba635061d66eb4dfcf111d8))
* tighten unifi api action coverage ([a5e5cd4](https://github.com/jmagar/unifi-rmcp/commit/a5e5cd4fed1f04ed8654892ab05f0da6bb1597a1))


### Documentation

* align unifi skill event coverage ([5f939fa](https://github.com/jmagar/unifi-rmcp/commit/5f939fa20a10606cf6ca623427341a382f15de6b))
* capture unifi api coverage inventories ([897129f](https://github.com/jmagar/unifi-rmcp/commit/897129f24f73d65b23c5873c94745da17eae414c))
* document unifi parity verification ([6e9bd6d](https://github.com/jmagar/unifi-rmcp/commit/6e9bd6d0fb57bb575dda1de470fcc88b12541681))
* plan unifi full parity ([745ff23](https://github.com/jmagar/unifi-rmcp/commit/745ff23a9393a99786ab55e3f7521e0ceaaf1053))
* **rust:** align .cargo/config.toml and add docs/RUST.md ([58a7a6c](https://github.com/jmagar/unifi-rmcp/commit/58a7a6c6108a41a44613dd4a98fce9fc53f78caa))
* save session log ([71504d6](https://github.com/jmagar/unifi-rmcp/commit/71504d6e3db1ca5d67ec6d6dfb3d52ed1abe9cbc))


### Refactoring

* **plugin:** call runifi binary directly from hooks; port env mapping into the binary ([bc54db8](https://github.com/jmagar/unifi-rmcp/commit/bc54db85b27656823cba503495ea43a07b1c4efe))
* rename binary unifi -&gt; runifi (avoid official-name collision) ([4060707](https://github.com/jmagar/unifi-rmcp/commit/4060707d8d270a1d50b76e0b8f7c0879a70b9d4a))


### CI

* add marketplace-no-mcp auto-sync workflow [skip ci] ([5d3257e](https://github.com/jmagar/unifi-rmcp/commit/5d3257ecaad5364b007ae709d56e315dac5af88c))
* add release-please automation ([b2faf9b](https://github.com/jmagar/unifi-rmcp/commit/b2faf9b324b209a723a4ba8d6b845cdaa2ad87cc))

## [Unreleased]

## [0.1.1] — 2026-06-01

### Changed

- Plugin `SessionStart`/`ConfigChange` hooks now call `${CLAUDE_PLUGIN_ROOT}/bin/runifi setup plugin-hook` directly instead of going through the `plugin-setup.sh` shell wrapper. The env-var mapping the script performed (`CLAUDE_PLUGIN_OPTION_*` → `UNIFI_*`, plus `CLAUDE_PLUGIN_DATA` → `UNIFI_MCP_HOME`) now lives in `apply_plugin_options()` in `src/setup.rs`, applied at the top of the plugin-hook path. The script's `.env`-fallback was dropped (immaterial: the binary never persists option values to `.env` and the setup checks read live process env).

### Removed

- `plugins/unifi/hooks/plugin-setup.sh` — the wrapper was a pure env-mapping middleman now handled by the binary's `setup plugin-hook` command.

### Added

- Initial release of `rustifi` — UniFi MCP server bridging Claude to Ubiquiti network controllers
- MCP server with action-based tool dispatch (`unifi` tool, `action` parameter)
- Actions: `clients`, `devices`, `wlans`, `health`, `alarms`, `events`, `sysinfo`, `me`, `help`
- CLI thin shim with human-readable formatters and `--json` passthrough
- Bearer token + Google OAuth authentication via `lab-auth`
- Streamable HTTP transport on port 7474 + stdio transport
- Self-signed TLS support (`UNIFI_SKIP_TLS_VERIFY=true` default)
- Docker deployment with `ghcr.io/jmagar/rustifi` image
- Claude Code plugin with userConfig
- `entrypoint.sh` with permission setup and runtime validation
- Git LFS for pre-built plugin binaries in `bin/`
- nextest configuration with `ci` profile
- taplo TOML formatter configuration
- lefthook pre-commit hooks (diff check, TOML format, env guard)
- GitHub Actions: CI, Docker publish, release workflows
- xtask crate with `dist`, `ci`, `symlink-docs`, `check-env` commands
