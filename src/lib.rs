use reqwest::header::AUTHORIZATION;
use yup_oauth2::InstalledFlowAuthenticator;

use crate::{config::Config, credentials::Credentials, errors::AppError};

pub mod backup;
pub mod config;
pub mod credentials;
pub mod errors;
pub mod qotas;

pub fn get_client(config: &Config) -> reqwest::Client {
    reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "Authorization",
                format!("Bearer {}", config.ptero_api_token)
                    .parse()
                    .unwrap(),
            );
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert(
                "Accept",
                "Application/vnd.pterodactyl.v1+json".parse().unwrap(),
            );
            headers
        })
        .build()
        .unwrap()
}

pub async fn get_drive_client(credentials: Credentials) -> Result<reqwest::Client, AppError> {
    let auth = InstalledFlowAuthenticator::builder(
        credentials.into(),
        yup_oauth2::InstalledFlowReturnMethod::Interactive,
    )
    .persist_tokens_to_disk("token_cache.json")
    .build()
    .await
    .map_err(AppError::Io)?;

    let scopes = &["https://www.googleapis.com/auth/drive"];

    let token = auth
        .token(scopes)
        .await
        .map_err(AppError::YupOauth2)?
        .token()
        .ok_or("err")
        .unwrap()
        .to_string();

    let client = reqwest::Client::builder()
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
            headers.insert(
                "X-Upload-Content-Type",
                "application/octet-stream".parse().unwrap(),
            );
            headers
        })
        .build()
        .unwrap();

    Ok(client)
}
