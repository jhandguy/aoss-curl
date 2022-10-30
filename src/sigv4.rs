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

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use aws_mfa::auth::Credentials;
    use hyper::{Body, Request};

    use crate::sigv4::get_signed_headers;

    #[tokio::test]
    async fn test_get_signed_headers() -> Result<()> {
        let credentials = Credentials::new("access_key_id", "secret_access_key", "session_token");
        let body = "";
        let request = Request::builder()
            .uri("https://opensearch-domain.eu-west-1.es.amazonaws.com/_cat/indices")
            .method("GET")
            .body(Body::from(String::from(body)))?;
        let headers = get_signed_headers("eu-west-1", "es", &credentials, &request, body).await?;

        assert_eq!(headers.len(), 3);
        assert!(headers.get("x-amz-date").is_some());
        assert!(headers.get("authorization").is_some());
        assert_eq!(
            headers
                .get("x-amz-security-token")
                .ok_or_else(|| anyhow!("x-amz-security-token header missing"))?
                .clone()
                .to_str()?,
            credentials.session_token()
        );

        Ok(())
    }
}
