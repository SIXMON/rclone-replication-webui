use std::collections::HashMap;

/// Retourne la liste des noms de champs sensibles pour un type de remote donné.
///
/// Pour les types standard, la liste est explicite et alignée avec
/// `frontend/src/config/remoteFieldSchemas.ts` (champs marqués `type: 'password'`).
///
/// Pour les types avancés (clé/valeur libre), on applique une heuristique :
/// tout nom contenant des mots-clés sensibles est considéré comme un secret.
pub fn sensitive_fields(remote_type: &str) -> &'static [&'static str] {
    match remote_type {
        "s3" => &["secret_access_key"],
        "sftp" => &["pass"],
        "ftp" => &["pass"],
        "smb" => &["pass"],
        "azureblob" => &["key", "sas_url"],
        "sharepoint" => &["client_secret"],
        "local" => &[],
        // Types avancés : on filtre côté code via is_sensitive_key()
        _ => &[],
    }
}

/// Heuristique : un champ est-il sensible d'après son nom ?
/// Utilisé pour les types avancés (clé/valeur libre) où on ne peut pas avoir une liste figée.
pub fn is_sensitive_key(key: &str) -> bool {
    let key_lower = key.to_ascii_lowercase();
    const SENSITIVE_KEYWORDS: &[&str] = &[
        "pass",          // pass, password, passphrase
        "secret",        // secret, client_secret, secret_access_key
        "token",         // token, refresh_token, access_token
        "key",           // key, api_key, secret_key (mais pas "publickey"...)
        "credential",
        "sas_url",
        "auth",
    ];
    SENSITIVE_KEYWORDS.iter().any(|kw| key_lower.contains(kw))
}

/// Sépare une config en deux : les champs sensibles (à mettre dans le SecretStore)
/// et les champs publics (à garder en BDD).
pub fn split_sensitive(
    remote_type: &str,
    config: &serde_json::Map<String, serde_json::Value>,
) -> (HashMap<String, String>, serde_json::Map<String, serde_json::Value>) {
    let standard_keys = sensitive_fields(remote_type);
    let mut sensitive = HashMap::new();
    let mut public_cfg = serde_json::Map::new();

    let is_standard_type = matches!(
        remote_type,
        "s3" | "sftp" | "ftp" | "smb" | "azureblob" | "sharepoint" | "local"
    );

    for (k, v) in config {
        let is_sensitive = if is_standard_type {
            standard_keys.contains(&k.as_str())
        } else {
            is_sensitive_key(k)
        };

        if is_sensitive {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    sensitive.insert(k.clone(), s.to_string());
                }
            }
        } else {
            public_cfg.insert(k.clone(), v.clone());
        }
    }

    (sensitive, public_cfg)
}
