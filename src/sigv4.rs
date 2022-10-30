use std::time::SystemTime;

use anyhow::{anyhow, Error, Result};
use aws_mfa::auth::Credentials;
use aws_sigv4::http_request::{
    sign, SignableBody, SignableRequest, SigningParams, SigningSettings,
};
use hyper::{Body, HeaderMap, Request};

pub async fn get_signed_headers(
    region: &str,
    service: &str,
    credentials: &Credentials,
    request: &Request<Body>,
    body: &str,
) -> Result<HeaderMap> {
    let now = SystemTime::now();

    let params = SigningParams::builder()
        .settings(SigningSettings::default())
        .time(now)
        .region(region)
        .service_name(service)
        .access_key(credentials.access_key_id())
        .secret_key(credentials.secret_access_key())
        .security_token(credentials.session_token())
        .build()?;

    let headers = sign(
        SignableRequest::new(
            request.method(),
            request.uri(),
            request.headers(),
            SignableBody::Bytes(body.as_bytes()),
        ),
        &params,
    )
    .map_err(Error::msg)?
    .output()
    .headers()
    .ok_or_else(|| anyhow!("signed headers missing"))?
    .clone();

    Ok(headers)
}
