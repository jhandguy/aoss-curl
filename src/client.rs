use anyhow::{anyhow, Result};
use aws_mfa::auth::Credentials;
use hyper::http::request::Builder;
use hyper::{Body, Client, Response};
use hyper_rustls::HttpsConnectorBuilder;

use crate::sigv4::get_signed_headers;

/// Request to Amazon OpenSearch Service with SigV4
pub async fn request(
    region: &str,
    credentials: &Credentials,
    builder: Builder,
    body: &str,
) -> Result<Response<Body>> {
    let mut request = builder.body(Body::from(String::from(body)))?;
    let headers = get_signed_headers(region, "es", credentials, &request, body).await?;
    for (name, value) in headers.into_iter() {
        let key = name.ok_or_else(|| anyhow!("header name missing"))?;
        request.headers_mut().insert(key, value);
    }

    let connector = HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();
    let response = Client::builder().build(connector).request(request).await?;

    Ok(response)
}
