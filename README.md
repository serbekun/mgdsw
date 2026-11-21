# mgdsw

<div align="center">
  <img src="assets/serbekunmgdsw.png" alt="mgdsw banner" width="300">
</div>

Maxim Games Discord Server Web Site. A Rust web server for the Discord server created by Maxim.

The server is built with [axum](https://github.com/tokio-rs/axum) and [tokio](https://tokio.rs/). Static templates and examples are embedded into the binary with [rust-embed](https://github.com/pyrossh/rust-embed) and unpacked into `data/` on first start, so a single executable is enough to run the site.

## What problems does this server solve?

**Task one: create a hub for conveniently distributing content created by members of the Discord server.** One place containing the current version of a map or build, along with a description, game version, and installation instructions. No need to search for a month-old link or resend the file again.

**Task two: take recurring work off the server members.** If a person no longer has to explain installation via screen sharing or look for a link in the chat history, the site is working.

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

The binary is written to `target/release/mgdsw` (or `target/debug/mgdsw` for a debug build). Start it with:

```bash
./target/release/mgdsw
```

On startup the server listens on `http://127.0.0.1:8080`. The listen address is currently configured in `src/main.rs`.

## Versioning

The version is read from the `VERSION` file at build time by `build.rs`. To override it without editing the file, set `MGDSW_VERSION`:

```bash
MGDSW_VERSION=dev cargo build --release
```

The resulting version is exposed by the health endpoint and is also used to decide whether packaged resources need to be unpacked again.

## Runtime data

All runtime files live in `data/`:

| Directory | Purpose |
| --- | --- |
| `data/static/` | Unpacked templates and static assets served by the server |
| `data/content/` | Operator-owned content, including Minecraft map metadata |
| `data/shared/` | Operator-owned downloadable archives (for example, map zips) |
| `data/examples/` | Unpacked examples to copy from |

`data/static/` and `data/examples/` are populated from the resources embedded in the binary on first start and refreshed when the build version changes. `data/content/` and `data/shared/` are never generated automatically: they belong to the operator.

To remove the runtime directory:

```bash
./cleanup.sh
```

The server recreates `data/` on the next start and unpacks the packaged templates again. Use `--keep-content` to preserve operator-owned files:

```bash
./cleanup.sh --keep-content   # keep data/content/ and data/shared/
./cleanup.sh --force          # no confirmation prompt
```