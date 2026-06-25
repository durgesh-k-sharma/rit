# rit

A minimal Git implementation in Rust — written from scratch, binary-compatible
with real git's object store.

## Milestones

| Name | Status | Description |
|------|--------|-------------|
| Plumbing | ✅ | Object store, SHA-1, zlib |
| Staging | ✅ | Index, `rit add` |
| Snapshot | ✅ | `rit commit`, HEAD, refs |
| Clarity | ✅ | `rit status`, `rit log`, `rit cat-file` |
| Branches | ✅ | `rit branch`, `rit checkout`, working-tree transition |
| Diff | 🔜 | `rit diff` |
| Remote | 🔜 | `rit clone`, `rit push`, `rit fetch` |

## Usage

```sh
rit init
echo "hello" > README.md
rit add README.md
rit commit -m "initial commit"
rit log
```

## Building

```sh
cargo build --release
```

## Crate Choices

| Crate | Purpose |
|-------|---------|
| `clap` 4 + derive | CLI argument parsing |
| `sha1_smol` | SHA-1 for object addressing |
| `flate2` | zlib compression (pure Rust via miniz_oxide) |
| `chrono` | Timestamps in commits |
| `hex` | SHA hex encoding |
| `thiserror` / `anyhow` | Error handling |
