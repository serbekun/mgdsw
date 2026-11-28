# mgdsc

<div align="center">
  <img src="assets/serbekunmgdsw.png" alt="mgdsc banner" width="300">
</div>

Maxim Games Discord Server Core. The Rust core server of the Maxim Games Discord Server (mgds) ecosystem: it holds all information about the community (Minecraft servers, maps, mods) and serves it through an HTTP API. It does nothing else. Everything that shows this information, such as the website or a Discord bot, is a client of this API.

The server is built with [axum](https://github.com/tokio-rs/axum) and [tokio](https://tokio.rs/). Static templates and examples are embedded into the binary with [rust-embed](https://github.com/pyrossh/rust-embed) and unpacked into `data/` on first start, so a single executable is enough to run the core.

## Role in the mgds ecosystem

| Project | Role |
| --- | --- |
| **mgdsc** (this repository) | The core: the single source of truth, available only through the API |
| **mgdsw** | The website: a client for the mgdsc API |
| **mgds_minecraft_status** (in the future) | Discord bot: shows the information from the mgdsc API in a channel |
| **mgds_role_chooser** | Standalone role selection bot: does not depend on mgdsc |

The website and the bot never keep their own copy of the data. Both read it from mgdsc, so they cannot get out of sync.

## What problems does this server solve?

**Task one: keep one source of truth.** The domain, the game version and the mods of a Minecraft server are written in one place. The website page and the Discord message take them from the same answer of the API, and nobody has to update them twice.

**Task two: create a hub for conveniently distributing content created by members of the Discord server.** One place containing the current version of a map or build, along with a description, game version, and installation instructions. No need to search for a month-old link or resend the file again.

**Task three: take recurring work off the server members.** If a person no longer has to explain installation via screen sharing or look for a link in the chat history, the system is working.

## Requirements

- Rust with Cargo. The project uses the 2024 edition, so Rust **1.85 or newer** is required.
- Bash, only if you want to use `cleanup.sh`.

## Build and run

Run the server in development mode:

```bash
cargo run
```

Build an optimized binary:

```bash
cargo build --release
```

The binary is written to `target/release/mgdsc` (or `target/debug/mgdsc` for a debug build). Start it with:

```bash
./target/release/mgdsc
```

On startup the server listens on `http://127.0.0.1:8080`. The listen address is currently configured in `src/main.rs`. Because it listens only on the loopback interface, the server is meant to run behind a reverse proxy such as nginx.

The proxy is also required for static files and map archives. The core itself serves only `/api/v0/*`: templates and assets unpacked into `data/static/` are meant to be served by the proxy, and the map ZIP endpoint answers with an `X-Accel-Redirect` header that nginx turns into the file response without copying the archive through Rust.

## API

All data is served by the API, which is versioned: the current version is `v0`, under `/api/v0`.

| Endpoint | Purpose |
| --- | --- |
| `GET /api/v0/health` | Liveness check, returns `{"health":"ok"}` |
| `GET /api/v0/version` | Reports the running version of the server |
| `GET /api/v0/shared/minecraft/maps/list` | Slugs of all published Minecraft maps |
| `GET /api/v0/shared/minecraft/maps/{slug}/yaml` | Public metadata of a published map |
| `GET /api/v0/shared/minecraft/maps/{slug}/zip` | `X-Accel-Redirect` response for a published map archive |

Clients (the website, bots) should take the base address of the API from their own configuration instead of hardcoding it.

## Versioning

The version is read from the `VERSION` file at build time by `build.rs`. To override it without editing the file, set `MGDSC_VERSION`:

```bash
MGDSC_VERSION=dev cargo build --release
```

The resulting version is exposed by `GET /api/v0/version` and is also used to decide whether packaged resources need to be unpacked again.

## Runtime data

All runtime files live in `data/`:

| Directory | Purpose |
| --- | --- |
| `data/static/` | Unpacked templates and static assets, meant to be served by the reverse proxy |
| `data/content/` | Operator-owned content, including Minecraft map metadata |
| `data/shared/` | Operator-owned downloadable archives (for example, map zips) |
| `data/examples/` | Unpacked examples to copy from |

`data/static/` and `data/examples/` are populated from the resources embedded in the binary on first start. When the build version changes, the unpacker runs again and adds files that are new in that version, but it never overwrites files that already exist on disk, so local edits survive an upgrade. `data/content/` and `data/shared/` are created as empty directories and never populated automatically: their contents belong to the operator.

To remove the runtime directory:

```bash
./cleanup.sh
```

The server recreates `data/` on the next start and unpacks the packaged templates and examples again. Use `--keep-content` to preserve operator-owned files:

```bash
./cleanup.sh --keep-content   # keep data/content/ and data/shared/
./cleanup.sh --force          # no confirmation prompt
```
