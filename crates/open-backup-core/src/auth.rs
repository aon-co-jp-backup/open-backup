use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::time::Duration;

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
const SERVICE: &str = "open-backup";

fn token_file() -> Result<PathBuf> {
    Ok(crate::config::data_dir()?.join("token"))
}

/// OS の資格情報ストアが使えない端末(Android など)では、アプリ専用の領域のファイル(自分だけが読める)に置く。
fn read_token_file() -> Option<String> {
    let t = std::fs::read_to_string(token_file().ok()?).ok()?.trim().to_string();
    (!t.is_empty()).then_some(t)
}

fn write_token_file(t: &str) -> Result<()> {
    let p = token_file()?;
    std::fs::create_dir_all(p.parent().unwrap())?;
    std::fs::write(&p, t)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// トークン取得順: 環境変数 OPEN_BACKUP_TOKEN → OS の資格情報ストア(デスクトップ) → アプリ専用領域のファイル
pub fn token() -> Result<String> {
    if let Ok(t) = std::env::var("OPEN_BACKUP_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    if let Ok(t) = keyring::Entry::new(SERVICE, "github").and_then(|e| e.get_password()) {
        return Ok(t);
    }
    read_token_file().context("未ログインです。`open-backup login` を実行してください")
}

pub fn store_token(t: &str) -> Result<()> {
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    if keyring::Entry::new(SERVICE, "github").and_then(|e| e.set_password(t)).is_ok() {
        // 読み戻せることを確かめる(保存先が実体の無い代用品だと、次回に消えてしまう)
        if keyring::Entry::new(SERVICE, "github").and_then(|e| e.get_password()).ok().as_deref() == Some(t) {
            let _ = std::fs::remove_file(token_file()?);
            return Ok(());
        }
    }
    write_token_file(t)
}

pub fn logout() -> Result<()> {
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    {
        let _ = keyring::Entry::new(SERVICE, "github").and_then(|e| e.delete_credential());
    }
    let _ = std::fs::remove_file(token_file()?);
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
