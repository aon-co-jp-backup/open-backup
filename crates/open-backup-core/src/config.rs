use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// バックアップ対象フォルダ
    #[serde(default)]
    pub folders: Vec<PathBuf>,
    /// 保存先リポジトリ名(既定: open-backup-<端末名>)。利用者のアカウント配下に必ず非公開で作る。
    pub repo: Option<String>,
    /// 定期実行の間隔(秒)
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
    /// これを超えるファイルは対象外(GitHub は 100MB 超を拒否する)
    #[serde(default = "default_max_mb")]
    pub max_file_mb: u64,
    /// 除外するディレクトリ名/ファイル名
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    /// GitHub OAuth App の client_id(デバイス認可でログインする場合)
    pub client_id: Option<String>,
}

fn default_interval() -> u64 {
    600
}
fn default_max_mb() -> u64 {
    95
}
fn default_exclude() -> Vec<String> {
    [".git", "node_modules", "target", "Thumbs.db", "desktop.ini", ".DS_Store"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            folders: vec![],
            repo: None,
            interval_secs: default_interval(),
            max_file_mb: default_max_mb(),
            exclude: default_exclude(),
            client_id: None,
        }
    }
}

pub fn config_dir() -> Result<PathBuf> {
    Ok(dirs::config_dir().context("設定ディレクトリが不明")?.join("open-backup"))
}
pub fn data_dir() -> Result<PathBuf> {
    Ok(dirs::data_local_dir().context("データディレクトリが不明")?.join("open-backup"))
}
pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

impl Config {
    pub fn load() -> Result<Self> {
        let p = config_path()?;
        if !p.exists() {
            return Ok(Self::default());
        }
        Ok(toml::from_str(&std::fs::read_to_string(&p)?)?)
    }
    pub fn save(&self) -> Result<()> {
        let p = config_path()?;
        std::fs::create_dir_all(p.parent().unwrap())?;
        std::fs::write(p, toml::to_string_pretty(self)?)?;
        Ok(())
    }
    pub fn repo_name(&self) -> String {
        self.repo.clone().unwrap_or_else(|| {
            let host = std::env::var("COMPUTERNAME")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_else(|_| "device".into());
            let safe: String = host
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
                .collect();
            format!("open-backup-{}", safe.to_lowercase())
        })
    }
}
