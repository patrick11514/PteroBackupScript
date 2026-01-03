use ptero_gdrive::{
    backup::{cleanup_archive, make_archive, upload_to_gdrive},
    config::read_config,
    credentials::load_credentials,
    errors::AppError,
    get_client, get_drive_client, send_discord_webhook,
    qotas::check_quota,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Loading configuration and credentials...");
    let config = read_config().await?;

    tracing::info!("Loading credentials...");
    let credentials = load_credentials().await?;

    let args: Vec<String> = std::env::args().collect();
    let allow_auth = args.contains(&"--auth".to_string());

    let client = get_client(&config);
    let gdrive_client = match get_drive_client(credentials, allow_auth).await {
        Ok(client) => client,
        Err(AppError::YupOauth2(e)) => {
            tracing::error!("Google Drive Auth Error: {}", e);
            if let Err(webhook_err) = send_discord_webhook(
                &config,
                "Google Drive Authentication Failed! Token might be expired.",
            )
            .await
            {
                tracing::error!("Failed to send webhook: {}", webhook_err);
            }
            return Err(AppError::YupOauth2(e).into());
        }
        Err(e) => return Err(e.into()),
    };

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
