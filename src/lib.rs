use anyhow::{anyhow, Result};
use aws_credential_types::Credentials;
use hyper::{Body, Method, Request, Response};
use hyper_rustls::HttpsConnectorBuilder;

use crate::config::{get_default_credentials, get_default_region};
use crate::error::Error;
use crate::error::Error::{BuildRequestError, Other, SendRequestError};
use crate::sigv4::get_signed_headers;

mod config;
pub mod error;
mod sigv4;

/// Client for requesting to Amazon OpenSearch Service with SigV4
pub struct Client {
    uri: String,
    method: Method,
    body: String,
    region: Option<String>,
    profile: Option<String>,
    home: Option<String>,
}

impl Client {
    pub fn new(
        uri: &str,
        method: &Method,
        body: &str,
        region: Option<String>,
        profile: Option<String>,
        home: Option<String>,
    ) -> Self {
        Self {
            uri: String::from(uri),
            method: method.clone(),
            body: String::from(body),
            region,
            profile,
            home,
        }
    }

    /// Request to Amazon OpenSearch Service with SigV4
    pub async fn request(&self, credentials: Option<Credentials>) -> Result<Response<Body>, Error> {
        let credentials = match credentials {
            Some(r) => r,
            None => get_default_credentials().await?,
        };
        let region = match self.region.clone() {
            Some(r) => r,
            None => get_default_region(self.profile.clone(), self.home.clone()).await?,
        };

        let mut request = Request::builder()
            .uri(self.uri.clone())
            .method(self.method.clone())
            .header("Content-Type", "application/json")
            .body(Body::from(self.body.clone()))
            .map_err(BuildRequestError)?;
        let headers = get_signed_headers(&region, "es", &credentials, &request, &self.body).await?;
        for (name, value) in headers.into_iter() {
            let key = name.ok_or_else(|| Other(anyhow!("invalid signed headers")))?;
            request.headers_mut().insert(key, value);
        }

        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http1()
            .build();
        let response = hyper::Client::builder()
            .build(connector)
            .request(request)
            .await
            .map_err(SendRequestError)?;

        Ok(response)
    }
}
