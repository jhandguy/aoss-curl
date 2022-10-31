# aoss-curl

[![Version](https://img.shields.io/crates/v/aoss-curl)](https://crates.io/crates/aoss-curl)
[![Downloads](https://img.shields.io/crates/d/aoss-curl)](https://crates.io/crates/aoss-curl)
[![License](https://img.shields.io/crates/l/aoss-curl)](LICENSE)
[![Build](https://img.shields.io/github/workflow/status/jhandguy/aoss-curl/CI/main)](https://github.com/jhandguy/aoss-curl/actions/workflows/ci.yaml)
[![Release](https://img.shields.io/github/workflow/status/jhandguy/aoss-curl/CD?label=release)](https://github.com/jhandguy/aoss-curl/actions/workflows/cd.yaml)

Request to Amazon OpenSearch Service with SigV4 🔏

```shell
➜ aoss-curl -h
Request to Amazon OpenSearch Service with SigV4 🔏

Usage: aoss-curl [OPTIONS] --code <CODE> --uri <URI> <HOME>

Arguments:
  <HOME>  Home directory containing the AWS hidden folder [env: HOME=/Users/JohnDoe]

Options:
  -r, --region <REGION>      Name of the AWS region [default: eu-west-1]
  -p, --profile <PROFILE>    Name of the AWS profile [default: default]
  -s, --suffix <SUFFIX>      Suffix of the original AWS profile [default: noauth]
  -c, --code <CODE>          MFA code
  -d, --duration <DURATION>  Session duration in seconds [default: 3600]
  -m, --method <METHOD>      Method of the HTTP request [default: GET]
  -u, --uri <URI>            URI of the HTTP request
  -b, --body <BODY>          Body of the HTTP request [default: ]
  -h, --help                 Print help information
```

## Installation

**aoss-curl** is published on [crates.io](https://crates.io/crates/aoss-curl) and can be installed with

```shell
cargo install aoss-curl
```

or downloaded as binary from the [releases page](https://github.com/jhandguy/aoss-curl/releases).

## Usage

> **Warning**: aoss-curl requires an MFA code and uses [aws-mfa](https://github.com/jhandguy/aws-mfa) for authenticating to AWS.

Add basic credentials in `~/.aws/credentials`:

```text
[<profile_name>-noauth]
aws_access_key_id = <aws_access_key_id>
aws_secret_access_key = <aws_secret_access_key>
```

> **Note**: make sure to add the `-noauth` suffix to the profile name

Run `aoss-curl`:
```shell
aoss-curl -p <profile_name> -c <mfa_code> -u <opensearch_domain>/_cat/indices
```

Check output:
```shell
200 OK
green ...
```