use anyhow::{bail, Result};

fn req(method: &str, url: &str, token: &str) -> ureq::Request {
    ureq::request(method, url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "open-backup")
}

pub fn login_name(token: &str) -> Result<String> {
    let v: serde_json::Value = req("GET", "https://api.github.com/user", token).call()?.into_json()?;
    Ok(v["login"].as_str().unwrap_or_default().to_string())
}

/// 保存先は必ず非公開。既存が公開リポジトリだった場合は、データを送らず中止する。
pub fn ensure_private_repo(token: &str, owner: &str, name: &str) -> Result<()> {
    match req("GET", &format!("https://api.github.com/repos/{owner}/{name}"), token).call() {
        Ok(r) => {
            let v: serde_json::Value = r.into_json()?;
            if v["private"].as_bool() != Some(true) {
                bail!("{owner}/{name} は公開リポジトリです。非公開にするか別名を指定してください(送信を中止)");
            }
            return Ok(());
        }
        Err(ureq::Error::Status(404, _)) => {}
        Err(e) => bail!("リポジトリ確認に失敗: {e}"),
    }
    req("POST", "https://api.github.com/user/repos", token).send_json(serde_json::json!({
        "name": name, "private": true, "description": "open-backup による自動バックアップ"
    }))?;
    Ok(())
}
