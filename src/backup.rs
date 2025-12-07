use serde_json::json;

use crate::{config::Config, errors::AppError};

#[derive(Debug, serde::Deserialize)]
pub struct ArchiveResult {
    pub object: String,
    pub attributes: ArchiveAttributes,
}

#[derive(Debug, serde::Deserialize)]
pub struct ArchiveAttributes {
    pub name: String,
    pub mode: String,
    pub mode_bits: String,
    pub size: u64,
    pub is_file: bool,
    pub is_symlink: bool,
    pub mimetype: String,
    pub created_at: String,
    pub modified_at: String,
}

pub async fn make_archive(client: &reqwest::Client, config: &Config) -> Result<String, AppError> {
    let res = client
        .post(format!(
            "{}/api/client/servers/{}/files/compress",
            config.ptero_url, config.server_id
        ))
        .body(
            serde_json::json!({
                "files": config.files,
                "root": "/"
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(AppError::Request)?;

    if !res.status().is_success() {
        eprintln!("Error creating archive: HTTP {}", res.status().as_u16());
        return Err(AppError::Response(format!(
            "HTTP {}",
            res.status().as_u16()
        )));
    }

    let text = res.text().await.unwrap_or_default();

    let archive_result: ArchiveResult = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("Error parsing archive response: {}", err);
            return Err(AppError::Parse { err, body: text });
        }
    };

    tracing::info!("Created archive: {}", archive_result.attributes.name);

    Ok(archive_result.attributes.name)
}

#[derive(Debug, serde::Deserialize)]
struct DownloadUrl {
    attributes: DownloadUrlAttributes,
}

#[derive(Debug, serde::Deserialize)]
struct DownloadUrlAttributes {
    url: String,
}

async fn get_download_url(
    archive_name: &str,
    client: &reqwest::Client,
    config: &Config,
) -> Result<String, AppError> {
    let res = client
        .get(format!(
            "{}/api/client/servers/{}/files/download",
            config.ptero_url, config.server_id
        ))
        .query(&[("file", archive_name)])
        .send()
        .await
        .map_err(AppError::Request)?;

    let text = res.text().await.map_err(AppError::Request)?;
    let download_url: DownloadUrl = serde_json::from_str(&text).map_err(|e| AppError::Parse {
        err: e,
        body: text.clone(),
    })?;

    Ok(download_url.attributes.url)
}

pub async fn upload_to_gdrive(
    client: &reqwest::Client,
    gdrive_client: &reqwest::Client,
    archive_name: &str,
    config: &Config,
) -> Result<(), AppError> {
    let get_download = get_download_url(archive_name, client, config).await?;
    let url = reqwest::Url::parse(&get_download).unwrap();

    let source = client.get(url).send().await.map_err(AppError::Request)?;

    let now = chrono::Utc::now();
    let filename = format!("backup_{}.tar.gz", now.format("%Y-%m-%d_%H-%M-%S"));

    let metadata = json!({
        "name": filename,
        "parents": [config.drive_folder_id]
    });

    tracing::info!("Started uploading {} to Google Drive", filename);

    let initiate_response =gdrive_client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable&supportsAllDrives=true")
        .json(&metadata)
        .send()
        .await
        .map_err(AppError::Request)?;

    if !initiate_response.status().is_success() {
        return Err(AppError::Response(format!(
            "Failed to initiate upload: HTTP {}",
            initiate_response.status().as_u16()
        )));
    }

    let session_uri = initiate_response
        .headers()
        .get("Location")
        .ok_or_else(|| {
            AppError::Response("Missing Location header in initiate upload response".to_string())
        })?
        .to_str()
        .map_err(|e| AppError::Response(format!("Invalid Location header: {}", e)))?
        .to_string();

    let stream = source.bytes_stream();
    let body = reqwest::Body::wrap_stream(stream);

    let upload_response = gdrive_client
        .put(session_uri)
        .body(body)
        .send()
        .await
        .map_err(AppError::Request)?;

    if !upload_response.status().is_success() {
        let status = upload_response.status();
        let error_text = upload_response.text().await.unwrap_or_default();
        println!("ERROR DETAILS: Status: {}, Body: {}", status, error_text); // <--- Add this
        return Err(AppError::Response(format!(
            "Failed to upload file: HTTP {}",
            status.as_u16()
        )));
    }

    tracing::info!("Successfully uploaded {} to Google Drive", filename);

    Ok(())
}

pub async fn cleanup_archive(
    client: &reqwest::Client,
    archive_name: &str,
    config: &Config,
) -> Result<(), AppError> {
    tracing::info!("Deleting archive: {}", archive_name);

    let res = client
        .post(format!(
            "{}/api/client/servers/{}/files/delete",
            config.ptero_url, config.server_id
        ))
        .body(
            json!({
                "root": "/",
                "files": [archive_name]
            })
            .to_string(),
        )
        .send()
        .await
        .map_err(AppError::Request)?;

    if !res.status().is_success() {
        return Err(AppError::Response(format!(
            "Failed to delete archive: HTTP {}",
            res.status().as_u16()
        )));
    }

    tracing::info!("Successfully deleted archive: {}", archive_name);

    Ok(())
}
