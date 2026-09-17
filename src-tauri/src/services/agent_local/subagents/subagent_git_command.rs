use std::path::Path;

pub async fn output(repo: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    crate::services::background_command::new_tokio("git")
        .args(["-C"])
        .arg(repo)
        .args(args)
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "Git indisponible".to_string())
}

pub async fn text(repo: &Path, args: &[&str]) -> Result<String, String> {
    let result = output(repo, args).await?;
    if !result.status.success() {
        return Err("Opération Git impossible".into());
    }
    String::from_utf8(result.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|_| "Sortie Git invalide".to_string())
}

pub async fn success(repo: &Path, args: &[&str]) -> Result<bool, String> {
    Ok(output(repo, args).await?.status.success())
}

pub async fn cherry_pick(repo: &Path, commit: &str) -> Result<bool, String> {
    let result = crate::services::background_command::new_tokio("git")
        .args(["-C"])
        .arg(repo)
        .args([
            "-c",
            crate::services::brand::GIT_AUTHOR_NAME_CONFIG,
            "-c",
            crate::services::brand::GIT_AUTHOR_EMAIL_CONFIG,
            "cherry-pick",
        ])
        .arg(commit)
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|_| "Git indisponible".to_string())?;
    Ok(result.status.success())
}

pub async fn delete_branch(repo: &Path, branch: &str) -> Result<(), String> {
    let reference = format!("refs/heads/{branch}");
    if !success(repo, &["show-ref", "--verify", "--quiet", &reference]).await? {
        return Ok(());
    }
    if success(repo, &["branch", "-D", branch]).await? {
        Ok(())
    } else {
        Err("Suppression de branche impossible".into())
    }
}
