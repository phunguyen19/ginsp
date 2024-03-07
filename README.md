# Ginsp

Ginsp is a tool that supports git workflow to picking commits from a branch to another branch with integrated project management system.

WARNING: This tool is still in development and there many more management systems to be integrated.

## Installation

From release artifact (replace `<version>` with the latest release version)
```sh
# MacOS
curl -sSL  https://github.com/phunguyen19/ginsp/releases/download/<version>/ginsp-x86_64-apple-darwin.tar.gz | tar -xz && sudo mv ginsp /usr/local/bin/ && rm -f ginsp-x86_64-apple-darwin.tar.gz
```

From source
```sh
cargo install --path .
```

## Usage

Commands
```sh
ginsp --help

# Update local branches to update to date with remote branches
ginsp update master release-v1.223.0

# Compare two branches by diffing their commit messages
ginsp diff-message master release-v1.223.0 

# Compare two branches by diffing their commit messages
# and get the tickets status from project management system configured in ~/.ginsp/config.toml
ginsp diff-message master release-v1.223.0 -t

# Compare two branches by diffing their commit messages
# and pick the commits that contains the specified tickets (separator by comma)
ginsp diff-message master release-v1.223.0 -c TICKET-1,TICKET-2,TICKET-3
````

Project management configured sample with Jira
```toml
# ~/.ginsp/config.toml

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
