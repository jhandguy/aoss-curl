use std::time::SystemTime;

use crate::error::Error;
use crate::error::Error::{BuildParamsError, ConvertHeaderValueError, SignRequestError};
use anyhow::Result;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{sign, SignableBody, SignableRequest, SigningSettings};
use aws_sigv4::sign::v4a::SigningParams;
use aws_smithy_runtime_api::client::identity::Identity;
use hyper::header::ToStrError;
use hyper::Request;

pub async fn sign_request(
    time: SystemTime,
    region: &str,
    service: &str,
    credentials: Credentials,
    request: &mut Request<String>,
) -> Result<(), Error> {
    let identity = Identity::from(credentials);

    let params = SigningParams::builder()
        .settings(SigningSettings::default())
        .time(time)
        .region_set(region)
        .name(service)
        .identity(&identity)
        .build()
        .map_err(BuildParamsError)?
        .into();

    let headers = request
        .headers()
        .iter()
        .map(|(n, v)| Ok((n.as_str(), v.to_str()?)))
        .collect::<Result<Vec<(&str, &str)>, ToStrError>>()
        .map_err(ConvertHeaderValueError)?
        .into_iter();

    let sign_request = SignableRequest::new(
        request.method().as_str(),
        request.uri().to_string(),
        headers,
        SignableBody::Bytes(request.body().as_bytes()),
    )
    .map_err(SignRequestError)?;

    let (instructions, _) = sign(sign_request, &params)
        .map_err(SignRequestError)?
        .into_parts();

    instructions.apply_to_request_http1x(request);

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use aws_credential_types::Credentials;
    use hyper::Request;
    use std::ops::Add;
    use std::time::SystemTime;
    use time::format_description::parse;
    use time::{Duration, OffsetDateTime};

    use crate::sigv4::sign_request;

    #[tokio::test]
    async fn test_get_signed_headers() -> Result<()> {
        let time = SystemTime::now();
        let region = "eu-west-1";
        let service = "es";
        let access_key_id = "access_key_id";
        let secret_access_key = "secret_access_key";
        let session_token = "session_token";
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            Some(String::from(session_token)),
            Some(time.add(Duration::hours(1))),
            "aoss-curl",
        );
        let mut request = Request::builder()
            .uri("https://opensearch-domain.eu-west-1.es.amazonaws.com/_cat/indices")
            .method("GET")
            .body(String::from(""))?;
        sign_request(time, region, service, credentials, &mut request).await?;

        let headers = request.headers();
        let datetime = OffsetDateTime::from(time);
        let mut format = parse("[year][month][day]T[hour][minute][second]Z")?;
        assert_eq!(headers.len(), 4);
        assert_eq!(
            headers
                .get("x-amz-date")
                .ok_or_else(|| anyhow!("x-amz-date header missing"))?
                .to_str()?,
            datetime.format(&format)?
        );
        assert_eq!(
            headers
                .get("x-amz-region-set")
                .ok_or_else(|| anyhow!("x-amz-region-set header missing"))?
                .to_str()?,
            region
        );
        format = parse("[year][month][day]")?;
        assert!(headers
            .get("authorization")
            .ok_or_else(|| anyhow!("authorization header missing"))?
            .to_str()?
            .contains(
                format!(
                    "Credential={}/{}/{}/aws4_request",
                    access_key_id,
                    datetime.format(&format)?,
                    service
                )
                .as_str()
            ));
        assert_eq!(
            headers
                .get("x-amz-security-token")
                .ok_or_else(|| anyhow!("x-amz-security-token header missing"))?
                .to_str()?,
            session_token
        );

        Ok(())
    }
}
