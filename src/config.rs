use crate::error::Error;
use crate::error::Error::{Other, ProvideCredentialsError};
use anyhow::{anyhow, Result};
use aws_config::default_provider::credentials::default_provider;
use aws_config::meta::region::ProvideRegion;
use aws_config::profile::ProfileFileRegionProvider;
use aws_credential_types::provider::ProvideCredentials;
use aws_credential_types::Credentials;
use aws_runtime::env_config::file::EnvConfigFileKind;
use aws_runtime::env_config::file::EnvConfigFiles;

pub async fn get_default_region(
    profile: Option<String>,
    home: Option<String>,
) -> Result<String, Error> {
    let mut provider = ProfileFileRegionProvider::builder();

    if let Some(home) = home {
        let files = EnvConfigFiles::builder()
            .with_file(EnvConfigFileKind::Config, format!("{home}/.aws/config"))
            .with_file(
                EnvConfigFileKind::Credentials,
                format!("{home}/.aws/credentials"),
            )
            .build();
        provider = provider.profile_files(files);
    }

    if let Some(profile) = profile {
        provider = provider.profile_name(profile);
    }

    let region = provider
        .build()
        .region()
        .await
        .ok_or_else(|| Other(anyhow!("missing region in profile config file")))?
        .to_string();

    Ok(region)
}

pub async fn get_default_credentials() -> Result<Credentials, Error> {
    let credentials = default_provider()
        .await
        .provide_credentials()
        .await
        .map_err(ProvideCredentialsError)?;

    Ok(credentials)
}
