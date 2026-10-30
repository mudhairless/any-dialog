# Contributing to any-dialog

Thanks for your interest. This is a small, zero-dependency crate, and the rules below exist to keep it that way.

## Getting set up

You need a stable Rust toolchain. The minimum supported Rust version is 1.85 (set by edition 2024 and declared as `rust-version` in `Cargo.toml`); CI tests on 1.85 as well as on stable, so code that needs a newer compiler will fail there.

```sh
git clone https://github.com/mudhairless/any-dialog
cd any-dialog
cargo test
```

To see the dialogs themselves you need a desktop session and one of the helper programs: `kdialog` or `zenity` on Linux, `osascript` on macOS (bundled with the OS), or PowerShell on Windows (also bundled).

## The gate

CI runs the following, and a change is expected to pass all of it:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features   # on Linux, macOS, and Windows
```

Changes to the PowerShell scripts under `src/backends/powershell/` are also linted with PSScriptAnalyzer at Error and Warning severity, in CI on Windows.

## How the backends work, and the rules for changing them

Each dialog kind is rendered by all four backends. The important conventions:

* **Lockstep.** A new dialog kind lands with a rendering for every backend in the same change, plus probe coverage (below). A kind that only some helpers can express does not belong in the shared API.
* **User text is data, never code.** Text, titles, and choices travel to the helpers as arguments (`kdialog`, `zenity`), as `run`-handler arguments (`osascript`), or as environment variables (PowerShell). Nothing a caller passes is ever interpolated into a script.
* **Cancel is a value.** Prompt methods return `Ok(None)` when the user cancels. Each backend maps its helper's cancel signal onto that; a backend whose helper cannot distinguish cancel from an empty answer needs a rendering that can, not a special case in the API.
* **PowerShell scripts are files.** The scripts live as real `.ps1` files in `src/backends/powershell/` and are embedded with `include_str!`. Keep them clean under PSScriptAnalyzer's default rules: full cmdlet names, no `Write-Host`, and never assign to an automatic variable.
* **Tests pin shapes, not pixels.** CI machines have no desktop session, so the unit tests assert the exact arguments and scripts each backend builds, and the classification of the results. Rendering itself is verified by hand, through the probe.

## The probe

`examples/dialog_probe.rs` is the project's verification harness:

```sh
# The full human checklist: every dialog, OK and cancel paths.
cargo run --example dialog_probe -- --interactive

# Force one backend when several helpers are installed.
cargo run --example dialog_probe -- --backend zenity --interactive

# A single dialog, for spot checks.
cargo run --example dialog_probe -- --case save
```

On macOS, `--scripted` runs unattended raw probes for the dialogs whose underlying commands support a timeout; that mode is what the manually triggered macOS probe workflow in CI uses. If you add a dialog kind, add its `--case` and interactive steps to the probe in the same change.

## Proposing changes

Open an issue first for a new dialog kind or an API change, so the lockstep question (can all four backends express it, with cancel distinguishable?) gets answered before code is written. Small, focused pull requests are easier to review than large ones. By contributing, you agree that your contributions are licensed under the MIT license, the same as the rest of the project.

## Conduct

Participation in this project is covered by the [Code of Conduct](CODE_OF_CONDUCT.md).
