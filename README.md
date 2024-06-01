# Ginsp

Ginsp is a tool that supports git workflow to picking commits from a branch to another branch with integrated project management system.

WARNING: This tool is still in development and there many more management systems to be integrated.

## Install from release artifacts

1. Go to [release page](https://github.com/phunguyen19/ginsp/releases)
1. Copy the link of the release artifact that matches your OS and architecture.
1. Download the artifact and extract it to the directory in your PATH.

## Install from source
```sh
cargo install --path .
```

## Usage

Update local branches to update to date with remote branches
```sh
ginsp update master release-v1.223.0
```

Compare the difference of messages between two branches
```sh
ginsp diff-message master release-v1.223.0
```

Pick commits that contain messages from a branch to another branch
```sh
ginsp diff-message master release-v1.223.0 -c TICKET-1234,TICKET-1235
```

## Fetching tickets status (optional)

Only Jira is supported at the moment.

```toml
# ~/.ginsp/config.toml

[project_management]
provider = "Jira"
url = "https://my-org.atlassian.net/rest/api/3/issue/:ticket_id"
credential_key = "<email>:<key>"
ticket_id_regex = '(\w+-\d+)'
```

Then we can use `-t` option to fetch the tickets status.

```sh
ginsp diff-message master release-v1.223.0 -t
```

## License

MIT
