use anyhow::{bail, Context, Result};
use std::time::Duration;

const SERVICE: &str = "open-backup";

/// トークン取得順: 環境変数 OPEN_BACKUP_TOKEN → OS の資格情報ストア
pub fn token() -> Result<String> {
    if let Ok(t) = std::env::var("OPEN_BACKUP_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    keyring::Entry::new(SERVICE, "github")?
        .get_password()
        .context("未ログインです。`open-backup login` を実行してください")
}

pub fn store_token(t: &str) -> Result<()> {
    keyring::Entry::new(SERVICE, "github")?.set_password(t)?;
    Ok(())
}

pub fn logout() -> Result<()> {
    keyring::Entry::new(SERVICE, "github")?.delete_credential()?;
    Ok(())
}

/// GitHub デバイス認可。利用者は表示された URL とコードで自分のアカウントにログインする
/// (アカウントが無ければ GitHub 側で利用者自身が作成する。本ツールはアカウントを作らない)。
pub fn device_login(client_id: &str, show: impl Fn(&str, &str)) -> Result<String> {
    let r: serde_json::Value = ureq::post("https://github.com/login/device/code")
        .set("Accept", "application/json")
        .send_form(&[("client_id", client_id), ("scope", "repo")])?
        .into_json()?;
    let device = r["device_code"].as_str().context("device_code なし")?.to_string();
    let user_code = r["user_code"].as_str().context("user_code なし")?;
    let uri = r["verification_uri"].as_str().context("verification_uri なし")?;
    let mut interval = r["interval"].as_u64().unwrap_or(5);
    let expires = r["expires_in"].as_u64().unwrap_or(900);
    show(uri, user_code);
    let mut waited = 0;
    while waited < expires {
        std::thread::sleep(Duration::from_secs(interval));
        waited += interval;
        let t: serde_json::Value = ureq::post("https://github.com/login/oauth/access_token")
            .set("Accept", "application/json")
            .send_form(&[
                ("client_id", client_id),
                ("device_code", &device),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])?
            .into_json()?;
        if let Some(tok) = t["access_token"].as_str() {
            return Ok(tok.to_string());
        }
        match t["error"].as_str() {
            Some("authorization_pending") => {}
            Some("slow_down") => interval += 5,
            Some(e) => bail!("ログイン失敗: {e}"),
            None => bail!("想定外の応答"),
        }
    }
    bail!("ログインの有効期限が切れました")
}
