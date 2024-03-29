use anyhow::Result;
use aws_credential_types::Credentials;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::{Method, Request, Response};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client as HttpClient;
use hyper_util::rt::TokioExecutor;
use std::time::SystemTime;

use crate::config::{get_default_credentials, get_default_region};
use crate::error::Error;
use crate::error::Error::{BuildRequestError, ReadResponseError, SendRequestError};
use crate::sigv4::sign_request;

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
    pub async fn request(
        &self,
        credentials: Option<Credentials>,
    ) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
        let credentials = match credentials {
            Some(r) => r,
            None => get_default_credentials().await?,
        };

        let region = match self.region.clone() {
            Some(r) => r,
            None => get_default_region(self.profile.clone(), self.home.clone()).await?,
        };

        let mut request = Request::builder()
            .header("Content-Type", "application/json")
            .uri(self.uri.clone())
            .method(self.method.clone())
            .body(self.body.clone())
            .map_err(BuildRequestError)?;

        sign_request(SystemTime::now(), &region, "es", credentials, &mut request).await?;

        let connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http1()
            .build();

        let response = HttpClient::builder(TokioExecutor::new())
            .build(connector)
            .request(request)
            .await
            .map_err(SendRequestError)?
            .map(|i| i.map_err(ReadResponseError).boxed());

        Ok(response)
    }
}
