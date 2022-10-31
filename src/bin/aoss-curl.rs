use std::str::{from_utf8, FromStr};

use anyhow::Result;
use aws_mfa::auth::authenticate;
use clap::Parser;
use hyper::body::to_bytes;
use hyper::{Method, Request};

use aoss_curl::client::request;

#[derive(Parser, Default)]
#[clap(about = "Request to Amazon OpenSearch Service with SigV4 🔏")]
pub struct Args {
    /// Name of the AWS region
    #[clap(short, long, default_value = "eu-west-1")]
    pub region: String,

    /// Name of the AWS profile
    #[clap(short, long, default_value = "default")]
    pub profile: String,

    /// Suffix of the original AWS profile
    #[clap(short, long, default_value = "noauth")]
    pub suffix: String,

    /// MFA code
    #[clap(short, long)]
    pub code: String,

    /// Session duration in seconds
    #[clap(short, long, default_value_t = 3600)]
    pub duration: i32,

    /// Home directory containing the AWS hidden folder
    #[clap(env)]
    pub home: String,

    /// Method of the HTTP request
    #[clap(short, long, default_value = "GET", value_parser = parse_method)]
    pub method: Method,

    /// URI of the HTTP request
    #[clap(short, long)]
    pub uri: String,

    /// Body of the HTTP request
    #[clap(short, long, default_value = "")]
    pub body: String,
}

fn parse_method(arg: &str) -> Result<Method> {
    Ok(Method::from_str(arg)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let credentials = authenticate(
        &args.profile,
        &args.suffix,
        &args.region,
        &args.code,
        args.duration,
        &args.home,
    )
    .await?;

    let builder = Request::builder()
        .uri(&args.uri)
        .method(&args.method)
        .header("Content-Type", "application/json");
    let mut response = request(&args.region, &credentials, builder, &args.body).await?;
    println!("{}", response.status());

    let body = to_bytes(response.body_mut()).await?;
    println!("{}", from_utf8(&body)?);

    Ok(())
}
