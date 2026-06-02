# Contributing to WinSoftVol

Issues and pull requests are welcome.

## Reporting bugs

Open an issue with:
- Windows version
- USB audio device model
- Steps to reproduce
- What you expected vs what happened

## Pull requests

1. Fork the repo and create a branch from `main`.
2. Keep changes focused — one fix or feature per PR.
3. Test on a real Windows machine with a USB audio device if possible.
4. Update `README.md` if you add or change user-facing behaviour.

## Building

Requires Rust stable.

**macOS (cross-compile to Windows x64):**

```sh
make setup   # first time only
make
```

**Windows (native):**

```sh
cargo build --release
```

## Code style

- Run `cargo fmt` before committing.
- Run `cargo clippy` and fix warnings before submitting.
- No `unsafe` without a comment explaining why it is sound.

## License

By contributing you agree that your changes will be licensed under the [MIT License](LICENSE).
