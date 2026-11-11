//! Where each CLI keeps the sign-in it already made, and whether it is there.
//! Only the presence of credentials is read; no value is ever returned.

use std::path::PathBuf;

fn home_dir() -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() => home,
        _ => String::new(),
    }
}

pub(crate) fn docker_config_path() -> String {
    let home = home_dir();
    if home.is_empty() {
        return String::new();
    }
    PathBuf::from(home)
        .join(".docker")
        .join("config.json")
        .display()
        .to_string()
}

pub(crate) fn terraform_credentials_path() -> String {
    let home = home_dir();
    if home.is_empty() {
        return String::new();
    }
    PathBuf::from(home)
        .join(".terraform.d")
        .join("credentials.tfrc.json")
        .display()
        .to_string()
}

/// Reports whether the Docker config lists any registry auth entry. It inspects
/// keys only and never returns credential values.
pub(crate) fn docker_has_registry_auth(config_path: &str) -> bool {
    if config_path.is_empty() {
        return false;
    }
    let Ok(raw) = std::fs::read(config_path) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&raw) else {
        return false;
    };
    parsed
        .get("auths")
        .and_then(serde_json::Value::as_object)
        .map(|auths| !auths.is_empty())
        .unwrap_or(false)
}

/// Reports whether the Terraform CLI has a credentials file with content.
pub(crate) fn terraform_credentials_present(credentials_path: &str) -> bool {
    if credentials_path.is_empty() {
        return false;
    }
    match std::fs::metadata(credentials_path) {
        Ok(info) => !info.is_dir() && info.len() > 0,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docker_has_registry_auth_reads_keys_only() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("config.json");
        let path = path.display().to_string();

        assert!(!docker_has_registry_auth(&path), "missing config is false");

        std::fs::write(&path, r#"{"auths":{}}"#).expect("write config");
        assert!(!docker_has_registry_auth(&path), "empty auths is false");

        std::fs::write(
            &path,
            r#"{"auths":{"https://index.docker.io/v1/":{"auth":"redacted"}}}"#,
        )
        .expect("write config");
        assert!(
            docker_has_registry_auth(&path),
            "non-empty auths keys are true"
        );
    }

    #[test]
    fn terraform_credentials_present_reads_the_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("credentials.tfrc.json");
        let path = path.display().to_string();

        assert!(
            !terraform_credentials_present(&path),
            "missing credentials is false"
        );

        std::fs::write(
            &path,
            r#"{"credentials":{"app.terraform.io":{"token":"redacted"}}}"#,
        )
        .expect("write credentials");
        assert!(
            terraform_credentials_present(&path),
            "a credentials file is true"
        );
    }
}
