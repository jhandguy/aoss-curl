# aoss-curl

[![Version](https://img.shields.io/crates/v/aoss-curl)](https://crates.io/crates/aoss-curl)
[![Downloads](https://img.shields.io/crates/d/aoss-curl)](https://crates.io/crates/aoss-curl)
[![License](https://img.shields.io/crates/l/aoss-curl)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/jhandguy/aoss-curl/ci.yaml)](https://github.com/jhandguy/aoss-curl/actions/workflows/ci.yaml)
[![Release](https://img.shields.io/github/actions/workflow/status/jhandguy/aoss-curl/cd.yaml?label=release)](https://github.com/jhandguy/aoss-curl/actions/workflows/cd.yaml)

Request to Amazon OpenSearch Service with SigV4 🔏

```shell
➜ aoss-curl
Request to Amazon OpenSearch Service with SigV4 🔏

Usage: aoss-curl <COMMAND>

Commands:
  no-auth  Request to Amazon OpenSearch Service with SigV4
  aws-mfa  Request to Amazon OpenSearch Service with SigV4 and aws-mfa
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Installation

**aoss-curl** is published on [crates.io](https://crates.io/crates/aoss-curl) and can be installed with

```shell
cargo install aoss-curl
```

or via [homebrew-tap](https://github.com/jhandguy/homebrew-tap) with

```shell
brew install jhandguy/tap/aoss-curl
```

or downloaded as binary from the [releases page](https://github.com/jhandguy/aoss-curl/releases).

## Usage

### no-auth

Run the `aoss-curl no-auth` command:
```shell
aoss-curl no-auth -u <opensearch_domain>/_cat/indices
```
```text
200 OK
green ...
```

### aws-mfa

[aws-mfa](https://github.com/jhandguy/aws-mfa) can be used for authenticating to AWS with MFA before requesting to Amazon OpenSearch Service.

#### Config and credentials files

Add default region in `~/.aws/config`:
```text
[profile <profile_name>-noauth]
region = <aws_region>

[profile <profile_name>]
region = <aws_region>
```

Add basic credentials in `~/.aws/credentials`:

```text
[<profile_name>-noauth]
aws_access_key_id = <aws_access_key_id>
aws_secret_access_key = <aws_secret_access_key>
```

> **Note**: make sure to add the `-noauth` suffix to the profile name

Run the `aoss-curl aws-mfa file` command:
```shell
aoss-curl aws-mfa file -p <profile_name> -c <mfa_code> -u <opensearch_domain>/_cat/indices
```
```text
200 OK
green ...
```

#### Environment variables

Export default region and basic credentials as environment variables:

```shell
export AWS_REGION=<aws_region>
export AWS_ACCESS_KEY_ID=<aws_access_key_id>
export AWS_SECRET_ACCESS_KEY=<aws_secret_access_key>
```

Run the `aoss-curl aws-mfa env` command:
```shell
aoss-curl aws-mfa env -c <mfa_code> -u <opensearch_domain>/_cat/indices
```
```text
200 OK
green ...
```
