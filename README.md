# GINSP

This project is under development and is not ready for use.

## Install from source locally

```sh
cargo install --path .
```

## Contribution

Setup Git pre commit hooks to run `cargo fmt`, `cargo clippy` and `cargo test` before commit.

```sh
touch .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Copy this content to the `.git/hooks/pre-commit` file.

```sh
#!/bin/sh

set -eu

if ! cargo fmt -- --check
then
    echo "There are some code style issues."
    echo "Run cargo fmt first."
    exit 1
fi

if ! cargo clippy --all-targets -- -D warnings
then
    echo "There are some clippy issues."
    exit 1
fi

if ! cargo test
then
    echo "There are some test issues."
    exit 1
fi

exit 0
```