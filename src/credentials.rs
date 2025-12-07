use tokio::fs;
use yup_oauth2::ApplicationSecret;

use crate::errors::AppError;

#[derive(Debug, serde::Deserialize)]
pub struct Credentials {
    pub installed: InstalledCredentials,
}

#[derive(Debug, serde::Deserialize)]
pub struct InstalledCredentials {
    pub client_id: String,
    pub project_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
}

impl From<Credentials> for ApplicationSecret {
    fn from(creds: Credentials) -> Self {
        ApplicationSecret {
            client_id: creds.installed.client_id,
            client_secret: creds.installed.client_secret,
            token_uri: creds.installed.token_uri,
            auth_uri: creds.installed.auth_uri,
            redirect_uris: creds.installed.redirect_uris,
            ..Default::default()
        }
    }
}

pub async fn load_credentials() -> Result<Credentials, AppError> {
    let creds_path = "credentials.json";

    let contents = fs::read_to_string(creds_path)
        .await
        .map_err(|err| AppError::Io(err))?;

    let creds: Credentials = serde_json::from_str(&contents).map_err(|err| AppError::Parse {
        err,
        body: contents.clone(),
    })?;

    Ok(creds)
}
