mod serve;
use anyhow::Result;
use clap::{Parser, Subcommand};
use notify::{RecursiveMode, Watcher};
use open_backup_core::{auth, config::Config, github, sync};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(version, about = "指定フォルダを GitHub の非公開リポジトリへ自動バックアップ")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// GitHub にログイン(デバイス認可。--token で個人用トークンを標準入力から)
    Login {
        #[arg(long)]
        token: bool,
    },
    Logout,
    /// 対象フォルダを追加
    Add { folder: PathBuf },
    /// 対象フォルダを削除
    Remove { folder: PathBuf },
    /// 設定と状態を表示
    Status,
    /// 指定フォルダの索引だけ作る(GitHub へは送らない。動作確認用)
    Index { folder: PathBuf },
    /// 1回だけバックアップ
    Once,
    /// 常駐(変更を検知して自動バックアップ + 定期実行)
    Run,
    /// 常駐エンジン + 画面(http://127.0.0.1:4620/ui)。変更の監視と定期実行も行う
    Serve {
        #[arg(long, default_value_t = 4620)]
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = Config::load()?;
    match cli.cmd {
        Cmd::Login { token } => {
            let t = if token {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s)?;
                s.trim().to_string()
            } else {
                let id = cfg
                    .client_id
                    .clone()
                    .or_else(|| std::env::var("OPEN_BACKUP_CLIENT_ID").ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!("client_id 未設定。config.toml の client_id か環境変数 OPEN_BACKUP_CLIENT_ID、または --token を使用")
                    })?;
                auth::device_login(&id, |uri, code| {
                    println!("ブラウザで {uri} を開き、コード {code} を入力してください")
                })?
            };
            auth::store_token(&t)?;
            println!("ログイン成功: {}", github::login_name(&t)?);
        }
        Cmd::Logout => {
            auth::logout()?;
            println!("ログアウトしました");
        }
        Cmd::Add { folder } => {
            let f = std::fs::canonicalize(&folder)?;
            let f = PathBuf::from(f.to_string_lossy().trim_start_matches("\\\\?\\"));
            if !cfg.folders.contains(&f) {
                cfg.folders.push(f);
                cfg.save()?;
            }
            println!("対象: {:?}", cfg.folders);
        }
        Cmd::Remove { folder } => {
            let f = std::fs::canonicalize(&folder).unwrap_or(folder);
            cfg.folders.retain(|x| *x != f);
            cfg.save()?;
        }
        Cmd::Status => {
            println!(
                "リポジトリ名: {}\n間隔: {}秒\n対象: {:#?}",
                cfg.repo_name_n(1),
                cfg.interval_secs,
                cfg.folders
            );
            println!(
                "ログイン: {}",
                auth::token()
                    .ok()
                    .and_then(|t| github::login_name(&t).ok())
                    .unwrap_or_else(|| "未".into())
            );
        }
        Cmd::Index { folder } => sync::index_only(&cfg, &folder)?,
        Cmd::Serve { port } => serve::serve(port)?,
        Cmd::Once => report(sync::run_auto(&cfg)),
        Cmd::Run => {
            let (tx, rx) = mpsc::channel();
            let mut w = notify::recommended_watcher(move |e: notify::Result<notify::Event>| {
                if e.is_ok() {
                    let _ = tx.send(());
                }
            })?;
            for f in &cfg.folders {
                w.watch(f, RecursiveMode::Recursive)?;
            }
            report(sync::run_auto(&cfg));
            let every = Duration::from_secs(cfg.interval_secs);
            let mut last = Instant::now();
            loop {
                // 変更を検知しても、最後の変更から30秒静かになってから実行(連続保存のまとめ)
                let dirty = rx.recv_timeout(every).is_ok();
                if dirty {
                    while rx.recv_timeout(Duration::from_secs(30)).is_ok() {}
                }
                if dirty || last.elapsed() >= every {
                    report(sync::run_auto(&cfg));
                    last = Instant::now();
                }
            }
        }
    }
    Ok(())
}

fn report(r: Result<sync::Report>) {
    match r {
        Ok(r) => {
            println!("コピー{} 削除{} push={}", r.copied, r.removed, r.pushed);
            for p in r.skipped_large {
                eprintln!("大きすぎるため対象外: {}", p.display());
            }
        }
        Err(e) => eprintln!("エラー: {e:#}"),
    }
}
