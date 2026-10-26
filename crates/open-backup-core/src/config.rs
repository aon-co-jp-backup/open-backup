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
    /// 1リポジトリの上限(MB)。これに近づいたら次のリポジトリへ自動で切り替える
    #[serde(default = "default_max_repo_mb")]
    pub max_repo_mb: u64,
    /// 除外するディレクトリ名/ファイル名
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    /// 保存方法: "auto"(既定。`git` があれば `git`、無ければ GitHub API)、"git"、"api"(Rust だけ。Android はこれ)
    #[serde(default = "default_backend")]
    pub backend: String,
    /// GitHub OAuth App の client_id(デバイス認可でログインする場合)
    pub client_id: Option<String>,
    /// 内容による分類(同梱 aruaru-llm を端末内で起動)。既定で有効、使えなければ拡張子分類のみ
    #[serde(default = "default_true")]
    pub ai_classify: bool,
    /// aruaru-llm の実行ファイル(未指定なら open-backup と同じフォルダ)
    pub ai_llm_exe: Option<PathBuf>,
    /// 端末内(127.0.0.1)専用のポート
    #[serde(default = "default_ai_port")]
    pub ai_port: u16,
    /// 送る前に端末で暗号化する(git の保存方法のみ。`open-backup crypt-init` で初期化)。既定は無効
    #[serde(default)]
    pub encrypt: bool,
    /// 暗号化前の圧縮レベル(zstd、1〜19。0で圧縮しない)。既定 3
    #[serde(default = "default_compress")]
    pub compress_level: i32,
    /// 暗号化・圧縮に使うスレッド数(0 = CPU のコア数の半分。常駐の邪魔をしないため)
    #[serde(default)]
    pub threads: usize,
}
fn default_compress() -> i32 {
    3
}

fn default_backend() -> String {
    "auto".into()
}
fn default_true() -> bool {
    true
}
fn default_ai_port() -> u16 {
    4611
}
fn default_interval() -> u64 {
    600
}
fn default_max_mb() -> u64 {
    95
}
fn default_max_repo_mb() -> u64 {
    4000 // GitHub の推奨(数GB以内)に余裕を持たせる
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
            max_repo_mb: default_max_repo_mb(),
            exclude: default_exclude(),
            client_id: None,
            backend: default_backend(),
            ai_classify: true,
            ai_llm_exe: None,
            ai_port: default_ai_port(),
            encrypt: false,
            compress_level: default_compress(),
            threads: 0,
        }
    }
}

/// 設定・データの置き場所を1か所にまとめたい環境(Android のアプリ専用領域など)用。`OPEN_BACKUP_HOME` があれば、その下の `config` / `data` を使う。
fn home_override() -> Option<PathBuf> {
    std::env::var_os("OPEN_BACKUP_HOME").map(PathBuf::from).filter(|p| !p.as_os_str().is_empty())
}
pub fn config_dir() -> Result<PathBuf> {
    if let Some(h) = home_override() {
        return Ok(h.join("config"));
    }
    Ok(dirs::config_dir().context("設定ディレクトリが不明")?.join("open-backup"))
}
pub fn data_dir() -> Result<PathBuf> {
    if let Some(h) = home_override() {
        return Ok(h.join("data"));
    }
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
    /// 世代番号つきのリポジトリ名(例: open-backup-aon-0001)
    pub fn repo_name_n(&self, n: u32) -> String {
        format!("{}-{:04}", self.repo_name(), n)
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
