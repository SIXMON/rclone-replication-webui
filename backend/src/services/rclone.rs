use crate::models::remote::RcloneRemote;
use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

/// Génère un fichier de config rclone temporaire à partir des remotes en BDD.
pub fn write_rclone_config(remotes: &[RcloneRemote]) -> Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("rclone-{}.conf", uuid::Uuid::new_v4()));
    let mut content = String::new();

    for remote in remotes {
        content.push_str(&format!("[{}]\n", remote.name));

        // SharePoint est un alias UI du backend rclone `onedrive` avec drive_type=documentLibrary
        // et client_credentials=true (flow app-only).
        let (effective_type, inject_sharepoint_defaults) = if remote.remote_type == "sharepoint" {
            ("onedrive", true)
        } else {
            (remote.remote_type.as_str(), false)
        };
        content.push_str(&format!("type = {}\n", effective_type));
        if inject_sharepoint_defaults {
            content.push_str("drive_type = documentLibrary\n");
            content.push_str("client_credentials = true\n");
        }

        if let Some(obj) = remote.config.as_object() {
            for (k, v) in obj {
                // `root` est géré par notre application (concaténé au chemin de la tâche),
                // ce n'est pas une clé de config rclone valide.
                if k == "root" {
                    continue;
                }
                let val = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                content.push_str(&format!("{} = {}\n", k, val));
            }
        }
        content.push('\n');
    }

    std::fs::write(&path, content).context("Failed to write rclone config")?;
    Ok(path)
}

/// Teste la connectivité d'un remote en exécutant `rclone lsd`.
pub async fn test_remote(rclone_bin: &str, remotes: &[RcloneRemote], remote_name: &str) -> Result<String> {
    let config_path = write_rclone_config(remotes)?;
    let _cleanup = ConfigCleanup(config_path.clone());

    let output = Command::new(rclone_bin)
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "lsd",
            &format!("{}:", remote_name),
            "--max-depth",
            "1",
        ])
        .output()
        .await
        .context("Failed to run rclone")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line_count = stdout.lines().count();
        Ok(format!(
            "Connected. Found {} top-level entries.",
            line_count
        ))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{}", stderr.trim())
    }
}

pub struct RcloneProcess {
    pub child: tokio::process::Child,
    pub config_path: PathBuf,
}

/// Lance `rclone sync` et retourne le processus avec son stdout/stderr fusionnés.
pub async fn spawn_sync(
    rclone_bin: &str,
    remotes: &[RcloneRemote],
    src: &str,
    dst: &str,
    extra_flags: &[String],
) -> Result<RcloneProcess> {
    let config_path = write_rclone_config(remotes)?;

    let mut args = vec![
        "--config".to_string(),
        config_path.to_str().unwrap().to_string(),
        "sync".to_string(),
        src.to_string(),
        dst.to_string(),
        "--progress".to_string(),
        "--stats-log-level".to_string(),
        "NOTICE".to_string(),
        "--use-json-log".to_string(),
    ];
    args.extend_from_slice(extra_flags);

    let child = Command::new(rclone_bin)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn rclone")?;

    Ok(RcloneProcess { child, config_path })
}

/// Lit les lignes du stdout et stderr d'un processus rclone, appelle le callback pour chaque ligne.
pub async fn stream_output<F>(mut process: RcloneProcess, mut on_line: F) -> Result<std::process::ExitStatus>
where
    F: FnMut(String) + Send,
{
    let stdout = process.child.stdout.take().expect("stdout not piped");
    let stderr = process.child.stderr.take().expect("stderr not piped");

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut stderr_lines = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(l)) => on_line(l),
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(l)) => on_line(l),
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
        }
    }

    // Drainer stderr restant
    while let Ok(Some(line)) = stderr_lines.next_line().await {
        on_line(line);
    }

    let status = process.child.wait().await.context("Failed to wait for rclone")?;

    // Nettoyer le fichier de config temporaire
    let _ = std::fs::remove_file(&process.config_path);

    Ok(status)
}

// Guard RAII pour nettoyer le fichier config en cas d'erreur précoce
struct ConfigCleanup(PathBuf);
impl Drop for ConfigCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
