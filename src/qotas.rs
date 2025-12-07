use crate::{config::Config, errors::AppError};

#[derive(Debug, serde::Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "createdTime")]
    pub created_time: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct DriveFileList {
    pub files: Vec<DriveFile>,
}

//scan google drive files, and if exceeded remove oldest file
pub async fn get_files(
    client: &reqwest::Client,
    config: &Config,
) -> Result<Vec<DriveFile>, AppError> {
    let query_params = [
        (
            "q",
            format!(
                "'{}' in parents and trashed = false",
                config.drive_folder_id
            ),
        ),
        ("orderBy", "createdTime asc".into()),
        ("fields", "files(id, name, createdTime)".into()),
        ("pageSize", "1000".into()),
    ];

    let res = client
        .get("https://www.googleapis.com/drive/v3/files")
        .query(&query_params)
        .send()
        .await
        .map_err(AppError::Request)?;

    if !res.status().is_success() {
        return Err(AppError::Request(res.error_for_status().unwrap_err()));
    }

    let str = res.text().await.map_err(AppError::Request)?;

    let files: DriveFileList =
        serde_json::from_str(&str).map_err(|e| AppError::Parse { err: e, body: str })?;

    Ok(files.files)
}

pub async fn delete_file(client: &reqwest::Client, file_id: &str) -> Result<(), AppError> {
    let res = client
        .delete(&format!(
            "https://www.googleapis.com/drive/v3/files/{}",
            file_id
        ))
        .send()
        .await
        .map_err(AppError::Request)?;

    if !res.status().is_success() {
        return Err(AppError::Request(res.error_for_status().unwrap_err()));
    }

    Ok(())
}

pub async fn check_quota(client: &reqwest::Client, config: &Config) -> Result<(), AppError> {
    tracing::info!("Getting all files in Google Drive backup folder");
    let files = get_files(client, config).await?;

    if files.len() < config.max_backups {
        tracing::info!(
            "Current backup count ({}) is within the limit ({}). No files to delete.",
            files.len(),
            config.max_backups
        );
        return Ok(());
    }

    let oldest = &files[0..(files.len() - config.max_backups + 1)];

    for file in oldest {
        tracing::info!("Deleting old backup file: {}", file.name);

        delete_file(client, &file.id).await?;
    }

    tracing::info!("All old backup files deleted successfully.");

    Ok(())
}
