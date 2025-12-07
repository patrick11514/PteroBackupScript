use tokio::fs;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub files: Vec<String>,
    pub ptero_url: String,
    pub ptero_api_token: String,
    pub server_id: String,
    pub drive_folder_id: String,
    pub max_backups: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            files: vec!["world".into(), "config".into(), "server.properties".into()],
            ptero_url: "https://panel.example.com".into(),
            ptero_api_token: "ptlc_abcefghijklmnopqrstuvwxyz".into(),
            server_id: "2f615648".into(),
            drive_folder_id: "0BwwA4oUTeiV1TGRPeTVjaWRDY1E".into(),
            max_backups: 5,
        }
    }
}

pub async fn read_config() -> Result<Config, AppError> {
    let config_path = "config.json";
    match fs::read_to_string(config_path).await {
        Ok(content) => {
            let config: Config = serde_json::from_str(&content).map_err(|err| AppError::Parse {
                err,
                body: content.clone(),
            })?;
            Ok(config)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let default_config = Config::default();
            let config_json = serde_json::to_string_pretty(&default_config)
                .map_err(|err| AppError::Format(err))?;
            fs::write(config_path, config_json)
                .await
                .map_err(AppError::Io)?;
            Err(AppError::Created)
        }
        Err(err) => Err(AppError::Io(err)),
    }
}
