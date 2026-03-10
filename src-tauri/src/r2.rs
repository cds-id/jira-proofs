use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use chrono::Local;
use std::path::Path;

use crate::config::R2Config;

pub fn content_type_for_extension(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

pub fn build_object_key(filename: &str) -> String {
    let date = Local::now().format("%Y-%m-%d");
    format!("captures/{}/{}", date, filename)
}

pub fn build_public_url(public_base: &str, object_key: &str) -> String {
    format!("{}/{}", public_base.trim_end_matches('/'), object_key)
}

fn create_s3_client(config: &R2Config) -> Client {
    let credentials = Credentials::new(
        &config.access_key_id, &config.secret_access_key, None, None, "jira-proofs",
    );
    let s3_config = S3ConfigBuilder::new()
        .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", config.account_id))
        .region(Region::new("auto"))
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();
    Client::from_conf(s3_config)
}

pub async fn upload_file(config: &R2Config, file_path: &Path) -> Result<String, String> {
    let client = create_s3_client(config);
    let filename = file_path.file_name().and_then(|n| n.to_str()).ok_or("Invalid filename")?;
    let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let object_key = build_object_key(filename);
    let content_type = content_type_for_extension(extension);
    let body = ByteStream::from_path(file_path).await.map_err(|e| format!("Failed to read file: {}", e))?;
    client.put_object().bucket(&config.bucket).key(&object_key).body(body)
        .content_type(content_type).send().await
        .map_err(|e| format!("R2 upload failed: {}", e))?;
    Ok(build_public_url(&config.public_url, &object_key))
}
