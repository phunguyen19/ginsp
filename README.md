# GINSP

This project is under development and is not ready for use.

## Install from source locally

```sh
cargo install --path .
```

## Usage

Commands
```sh
ginsp --help
````

Project project management configured sample with Jira
```toml
[project_management]
provider = "Jira"
url = "https://my-org.atlassian.net/rest/api/3/issue/:ticket_id"
credential_key = "<email>:<key>"
ticket_id_regex = '(\w+-\d+)'
```

## Contribution

### Setup

Setup Git pre commit hooks to run `cargo fmt` and `cargo clippy` before commit.

```sh
touch .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Copy this content to the `.git/hooks/pre-commit` file.

```sh
#!/bin/sh

set -eu

echo "Running cargo fmt..."
if ! cargo fmt -- --check
then
    echo "There are some code style issues."
    echo "Run cargo fmt first."
    exit 1
fi

echo "Running cargo clippy..."
if ! cargo clippy --all-targets -- -D warnings
then
    echo "There are some clippy issues."
    exit 1
fi

exit 0
```

### Test

```sh
cargo test
```

with coverage (install [tarpualin](https://github.com/xd009642/tarpaulin) first)

```sh
cargo tarpaulin
```

## License

MIT
