use std::str::{from_utf8, FromStr};

use crate::AwsMfaCmd::{Env, File};
use anyhow::Result;
use aoss_curl::Client;
use async_trait::async_trait;
use aws_mfa::{Credentials, CredentialsProvider, EnvCredentialsProvider, FileCredentialsProvider};
use clap::{Args, Parser, Subcommand};
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::Method;

use crate::Cmd::{AwsMfa, NoAuth};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Request to Amazon OpenSearch Service with SigV4
    NoAuth(RequestArgs),

    /// Request to Amazon OpenSearch Service with SigV4 and aws-mfa
    AwsMfa(AwsMfaArgs),
}

#[derive(Args)]
struct AwsMfaArgs {
    #[command(subcommand)]
    cmd: AwsMfaCmd,
}

#[derive(Subcommand)]
enum AwsMfaCmd {
    /// Request to Amazon OpenSearch Service with SigV4 and aws-mfa using config and credentials files
    File(FileArgs),

    /// Request to Amazon OpenSearch Service with SigV4 and aws-mfa using environment variables
    Env(EnvArgs),
}

#[derive(Args)]
struct AuthArgs {
    /// MFA code
    #[arg(short, long)]
    code: String,

    /// MFA device identifier (defaults to AWS username)
    #[arg(short, long)]
    identifier: Option<String>,

    /// Session duration in seconds
    #[arg(short, long, default_value_t = 3600)]
    duration: i32,
}

#[derive(Args)]
struct FileArgs {
    #[command(flatten)]
    auth: AuthArgs,

    #[command(flatten)]
    request: RequestArgs,

    /// Home directory containing the AWS hidden folder
    #[arg(env = "HOME")]
    home: String,

    /// Name of the AWS profile
    #[arg(short, long, default_value = "default", env = "AWS_PROFILE")]
    profile: String,

    /// Suffix of the original AWS profile
    #[arg(short, long, default_value = "noauth")]
    suffix: String,

    /// Force authentication even though current credentials are still valid
    #[arg(short, long)]
    force: bool,
}

#[derive(Args)]
struct EnvArgs {
    #[command(flatten)]
    auth: AuthArgs,

    #[command(flatten)]
    request: RequestArgs,
}

#[derive(Args)]
struct RequestArgs {
    /// Name of the AWS region
    #[arg(short, long, env = "AWS_REGION")]
    region: Option<String>,

    /// URI of the HTTP request
    #[arg(short, long)]
    uri: String,

    /// Method of the HTTP request
    #[arg(short, long, default_value = "GET", value_parser = parse_method)]
    method: Method,

    /// Body of the HTTP request
    #[arg(short, long, default_value = "")]
    body: String,
}

fn parse_method(arg: &str) -> Result<Method> {
    Ok(Method::from_str(arg)?)
}

#[async_trait]
trait Request {
    async fn request(&self) -> Result<()>;
}

impl Cli {
    fn args(self) -> Box<dyn Request> {
        match self.cmd {
            NoAuth(args) => Box::new(args),
            AwsMfa(args) => match args.cmd {
                File(args) => Box::new(args),
                Env(args) => Box::new(args),
            },
        }
    }
}

#[async_trait]
impl Request for RequestArgs {
    async fn request(&self) -> Result<()> {
        let client = Client::new(
            &self.uri,
            &self.method,
            &self.body,
            self.region.clone(),
            None,
            None,
        );

        let response = client.request(None).await?;
        println!("{}", response.status());
        let bytes = response.collect().await?.to_bytes();
        println!("{}", from_utf8(bytes.chunk())?);

        Ok(())
    }
}

#[async_trait]
impl Request for FileArgs {
    async fn request(&self) -> Result<()> {
        let provider = FileCredentialsProvider::new(
            &self.auth.code,
            &self.home,
            self.request.region.clone(),
            &self.profile,
            &self.suffix,
            self.auth.identifier.clone(),
            self.auth.duration,
        );

        let credentials: Credentials;
        if self.force {
            credentials = provider.authenticate().await?;
        } else if let Some(c) = provider.validate().await? {
            credentials = c;
        } else {
            credentials = provider.authenticate().await?;
        }

        let client = Client::new(
            &self.request.uri,
            &self.request.method,
            &self.request.body,
            self.request.region.clone(),
            Some(self.profile.clone()),
            Some(self.home.clone()),
        );

        let response = client
            .request(Some(credentials.to_aws_credentials()))
            .await?;
        println!("{}", response.status());
        let bytes = response.collect().await?.to_bytes();
        println!("{}", from_utf8(bytes.chunk())?);

        Ok(())
    }
}

#[async_trait]
impl Request for EnvArgs {
    async fn request(&self) -> Result<()> {
        let provider = EnvCredentialsProvider::new(
            &self.auth.code,
            self.auth.identifier.clone(),
            self.auth.duration,
        );

        let credentials: Credentials;
        if let Some(c) = provider.validate().await? {
            credentials = c;
        } else {
            credentials = provider.authenticate().await?;
        }

        let client = Client::new(
            &self.request.uri,
            &self.request.method,
            &self.request.body,
            self.request.region.clone(),
            None,
            None,
        );

        let response = client
            .request(Some(credentials.to_aws_credentials()))
            .await?;
        println!("{}", response.status());
        let bytes = response.collect().await?.to_bytes();
        println!("{}", from_utf8(bytes.chunk())?);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Cli::parse().args().request().await
}
