# Vexen Desktop

A calm, read-only window onto one checkout's Vexen memory, built with [GPUI Kit](https://gpui-kit.com) (Rust, GPUI). It shows each saved note, where it came from (file, commit, file hash, test results), the conditions it was saved under, and whether an agent that checks that file would be shown it right now.

It does not record, edit or delete anything. Saving stays with your agent and the `vexen` command, so the window can never change what an agent is told.

## Run it

```sh
cd desktop
cargo run --release -- /path/to/your/repository
```

With no path it uses the current folder. It looks upward for the `.vexen` folder, so any folder inside the repository works. Use **Refresh** after your agent saves something.

You need Rust 1.92 or later and the platform libraries GPUI needs (see the [GPUI Kit installation guide](https://gpui-kit.com/docs/installation): Visual Studio C++ workload on Windows, Xcode tools on macOS, a few system packages on Linux). The first build compiles GPUI and takes several minutes.

## What "Still applies" means

It is the same check an agent runs (`vexen check`), asked for the note's own file and conditions. A note is **Quiet** when the file changed since it was saved, the commit is no longer in the history, the note was replaced, or its conditions differ. Quiet is a normal result, not an error.

## Design

Oxocarbon-inspired near-black, one blue signal, Geist and Geist Mono (SIL Open Font License, copied from `docs/assets/fonts`). See [brand.md](../brand.md). The wording follows the brand voice: what happened, where it came from, when it may no longer apply.

## Scope

- A separate Cargo workspace: the `vexen` crate, its tests, the npm package and the release workflow are unchanged, and this app is not part of any release yet.
- Tested here: the model (filtering, a folder without Vexen) with `cargo test`, and a manual run on Windows 11. Not yet run on macOS or Linux.
