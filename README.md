# Simple Pterodactyl Backup -> Google Drive Uploader

Currently undocumeented script for my personal use :)

example output:

```log
admin@admin:/GDrive$ ./bin
2025-12-07T15:46:16.759264Z  INFO bin: Loading configuration and credentials...
2025-12-07T15:46:16.759494Z  INFO bin: Loading credentials...
2025-12-07T15:46:16.779319Z  INFO bin: Creating archive...
2025-12-07T15:46:31.633636Z  INFO ptero_gdrive::backup: Created archive: archive-2025-12-07T164616+0100.tar.gz
2025-12-07T15:46:31.633664Z  INFO bin: Uploading archive to Google Drive...
2025-12-07T15:46:32.129614Z  INFO ptero_gdrive::backup: Started uploading backup_2025-12-07_15-46-32.tar.gz to Google Drive
2025-12-07T15:47:33.489574Z  INFO ptero_gdrive::backup: Successfully uploaded backup_2025-12-07_15-46-32.tar.gz to Google Drive
2025-12-07T15:47:33.489611Z  INFO bin: Deleting archive...
2025-12-07T15:47:33.489623Z  INFO ptero_gdrive::backup: Deleting archive: archive-2025-12-07T164616+0100.tar.gz
2025-12-07T15:47:34.033319Z  INFO ptero_gdrive::backup: Successfully deleted archive: archive-2025-12-07T164616+0100.tar.gz
2025-12-07T15:47:34.033344Z  INFO bin: Checking quotas...
2025-12-07T15:47:34.033347Z  INFO ptero_gdrive::qotas: Getting all files in Google Drive backup folder
2025-12-07T15:47:34.353585Z  INFO ptero_gdrive::qotas: Current backup count (2) is within the limit (50). No files to delete.
2025-12-07T15:47:34.353611Z  INFO bin: Backup process completed successfully.
```