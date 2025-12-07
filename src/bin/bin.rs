use ptero_gdrive::{
    backup::{cleanup_archive, make_archive, upload_to_gdrive},
    config::read_config,
    credentials::load_credentials,
    get_client, get_drive_client,
    qotas::check_quota,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Loading configuration and credentials...");
    let config = read_config().await?;

    tracing::info!("Loading credentials...");
    let credentials = load_credentials().await?;

    let client = get_client(&config);
    let gdrive_client = get_drive_client(credentials).await?;

    tracing::info!("Creating archive...");
    let archive_name = make_archive(&client, &config).await?;

    tracing::info!("Uploading archive to Google Drive...");
    upload_to_gdrive(&client, &gdrive_client, &archive_name, &config).await?;

    tracing::info!("Deleting archive...");
    cleanup_archive(&client, &archive_name, &config).await?;

    tracing::info!("Checking quotas...");
    check_quota(&gdrive_client, &config).await?;

    tracing::info!("Backup process completed successfully.");

    Ok(())
}
