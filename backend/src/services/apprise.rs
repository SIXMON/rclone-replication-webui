use anyhow::{Context, Result};
use tokio::process::Command;

/// Envoie une notification via le CLI apprise-go.
pub async fn send_notification(
    apprise_bin: &str,
    urls: &[String],
    title: &str,
    body: &str,
) -> Result<()> {
    if urls.is_empty() {
        return Ok(());
    }

    let mut args = vec![
        "-t".to_string(),
        title.to_string(),
        "-b".to_string(),
        body.to_string(),
    ];
    args.extend(urls.iter().cloned());

    let output = Command::new(apprise_bin)
        .args(&args)
        .output()
        .await
        .context("Failed to run apprise")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Apprise failed: {}", stderr.trim());
    }

    Ok(())
}
