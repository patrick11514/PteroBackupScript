use std::future::Future;
use std::pin::Pin;

use reqwest::header::AUTHORIZATION;
use yup_oauth2::InstalledFlowAuthenticator;

use crate::{config::Config, credentials::Credentials, errors::AppError};

struct NoInteractionDelegate;

impl yup_oauth2::authenticator_delegate::InstalledFlowDelegate for NoInteractionDelegate {
    fn present_user_url<'a>(
        &'a self,
        _url: &'a str,
        _need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(async {
            Err("Interaction required but disabled. Use --auth to allow.".to_string())
        })
    }
}

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

pub async fn get_drive_client(
    credentials: Credentials,
    allow_auth: bool,
) -> Result<reqwest::Client, AppError> {
    let mut builder = InstalledFlowAuthenticator::builder(
        credentials.into(),
        yup_oauth2::InstalledFlowReturnMethod::Interactive,
    )
    .persist_tokens_to_disk("token_cache.json");

    if !allow_auth {
        builder = builder.flow_delegate(Box::new(NoInteractionDelegate));
    }

    let auth = builder.build().await.map_err(AppError::Io)?;

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

pub async fn send_discord_webhook(config: &Config, message: &str) -> Result<(), AppError> {
    let client = reqwest::Client::new();
    let res = client
        .post(&config.webhook_url)
        .json(&serde_json::json!({ "content": message }))
        .send()
        .await
        .map_err(AppError::Request)?;

    if !res.status().is_success() {
        return Err(AppError::Response(format!(
            "Failed to send webhook: HTTP {}",
            res.status().as_u16()
        )));
    }
    Ok(())
}
