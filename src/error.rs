use aws_credential_types::provider::error::CredentialsError;
use aws_sigv4::http_request::SigningError;
use aws_sigv4::signing_params::BuildError;
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

    #[error("missing field `{0}` in signature output")]
    InvalidSignature(String),

    #[error("failed to build http request")]
    BuildRequestError(#[source] http::Error),

    #[error("failed to send http request")]
    SendRequestError(#[source] hyper::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
