use aws_credential_types::provider::error::CredentialsError;
use aws_sigv4::http_request::SigningError;
use aws_sigv4::sign::v4a::signing_params::BuildError;
use hyper::header::ToStrError;
use hyper::http;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to provide credentials")]
    ProvideCredentialsError(#[source] CredentialsError),

    #[error("failed to build signing parameters")]
    BuildParamsError(#[source] BuildError),

    #[error("failed to sign request")]
    SignRequestError(#[source] SigningError),

    #[error("failed to convert http header value to string")]
    ConvertHeaderValueError(#[source] ToStrError),

    #[error("failed to build http request")]
    BuildRequestError(#[source] http::Error),

    #[error("failed to send http request")]
    SendRequestError(#[source] hyper_util::client::legacy::Error),

    #[error("failed to read http response")]
    ReadResponseError(#[source] hyper::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
