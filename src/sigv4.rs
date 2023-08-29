use std::time::SystemTime;

use crate::error::Error;
use crate::error::Error::{BuildParamsError, InvalidSignature, SignRequestError};
use anyhow::Result;
use aws_credential_types::Credentials;
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
) -> Result<HeaderMap, Error> {
    let now = SystemTime::now();
    let mut builder = SigningParams::builder()
        .settings(SigningSettings::default())
        .time(now)
        .region(region)
        .service_name(service)
        .access_key(credentials.access_key_id())
        .secret_key(credentials.secret_access_key());

    if let Some(session_token) = credentials.session_token() {
        builder = builder.security_token(session_token);
    }

    let params = builder.build().map_err(BuildParamsError)?;

    let headers = sign(
        SignableRequest::new(
            request.method(),
            request.uri(),
            request.headers(),
            SignableBody::Bytes(body.as_bytes()),
        ),
        &params,
    )
    .map_err(SignRequestError)?
    .output()
    .headers()
    .ok_or_else(|| InvalidSignature(String::from("headers")))?
    .clone();

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use aws_credential_types::Credentials;
    use hyper::{Body, Request};
    use std::time::SystemTime;

    use crate::sigv4::get_signed_headers;

    #[tokio::test]
    async fn test_get_signed_headers() -> Result<()> {
        let credentials = Credentials::new(
            "access_key_id",
            "secret_access_key",
            Some(String::from("session_token")),
            Some(SystemTime::now()),
            "aoss-curl",
        );
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
                .to_str()?,
            "session_token"
        );

        Ok(())
    }
}
