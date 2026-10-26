//! 常駐エンジン: 127.0.0.1 専用の小さな HTTP サーバー。画面(/ui)を配り、状態の取得・今すぐ実行・フォルダ追加/削除を受ける。
//! フォルダ変更の監視と定期実行も同じプロセスで行う。
use anyhow::Result;
use notify::{RecursiveMode, Watcher};
use open_backup_core::{auth, config::Config, crypt, github, restore, sync};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const UI_HTML: &str = include_str!("../../../ui/index.html");

#[derive(Default)]
struct Last {
    running: bool,
    at: u64,
    ok: bool,
    msg: String,
}

type Shared = Arc<Mutex<Last>>;

/// 復元ジョブ(画面から実行。同時に1つ)
#[derive(Default, Clone, serde::Serialize)]
struct Job {
    running: bool,
    done: bool,
    ok: bool,
    msg: String,
    restored: usize,
    skipped: usize,
    bytes: u64,
    failed: Vec<String>,
    dest: String,
}
static JOB: Mutex<Option<Job>> = Mutex::new(None);

fn jval(body: &[u8]) -> serde_json::Value {
    serde_json::from_slice(body).unwrap_or(serde_json::Value::Null)
}

/// 復元元の作業コピーとパスフレーズを用意する(暗号化なら、画面から渡された値か、保存済みの値)。
fn restore_source(v: &serde_json::Value) -> Result<(PathBuf, Option<String>, Config), (String, bool)> {
    let cfg = Config::load().unwrap_or_default();
    let gen = v["gen"].as_u64().unwrap_or(1).max(1) as u32;
    let github = v["github"].as_bool().unwrap_or(false);
    let work = restore::workdir(&cfg, gen, github).map_err(|e| (format!("{e:#}"), false))?;
    let pass = if restore::is_encrypted(&work) {
        let p = v["pass"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| crypt::passphrase().ok().map(|p| p.to_string()));
        match p {
            Some(p) => Some(p),
            None => return Err(("暗号化されたバックアップです。パスフレーズを入力してください / Passphrase required".into(), true)),
        }
    } else {
        None
    };
    Ok((work, pass, cfg))
}

fn err_json(m: &str, need: bool) -> (String, String) {
    (
        if need { "401 Unauthorized" } else { "400 Bad Request" }.into(),
        serde_json::json!({"error": m, "need_pass": need}).to_string(),
    )
}

fn restore_list(v: &serde_json::Value) -> (String, String) {
    let (work, pass, _) = match restore_source(v) {
        Ok(x) => x,
        Err((m, need)) => return err_json(&m, need),
    };
    let (enc, items) = match restore::list(&work, pass.as_deref()) {
        Ok(x) => x,
        Err(e) => return err_json(&format!("{e:#}"), false),
    };
    let q = v["q"].as_str().unwrap_or("").to_lowercase();
    let limit = v["limit"].as_u64().unwrap_or(300).min(2000) as usize;
    let mut labels: std::collections::BTreeMap<String, (usize, u64)> = Default::default();
    for i in &items {
        let l = i.rel.split('/').next().unwrap_or("").to_string();
        let e = labels.entry(l).or_default();
        e.0 += 1;
        e.1 += i.size;
    }
    let shown: Vec<&restore::Item> = items
        .iter()
        .filter(|i| q.is_empty() || i.rel.to_lowercase().contains(&q) || i.topic.as_deref().unwrap_or("").to_lowercase().contains(&q))
        .take(limit)
        .collect();
    (
        "200 OK".into(),
        serde_json::json!({
            "encrypted": enc,
            "total": items.len(),
            "total_bytes": items.iter().map(|i| i.size).sum::<u64>(),
            "labels": labels.iter().map(|(k, (n, b))| serde_json::json!({"label": k, "count": n, "bytes": b})).collect::<Vec<_>>(),
            "items": shown,
        })
        .to_string(),
    )
}

fn restore_start(v: serde_json::Value) -> (String, String) {
    if JOB.lock().unwrap().as_ref().map(|j| j.running).unwrap_or(false) {
        return ("409 Conflict".into(), "{\"error\":\"復元を実行中です / A restore is already running\"}".into());
    }
    let original = v["original"].as_bool().unwrap_or(false);
    let dest = v["dest"].as_str().unwrap_or("").trim().to_string();
    if !original && dest.is_empty() {
        return ("400 Bad Request".into(), "{\"error\":\"復元先のフォルダを入力してください / Enter a destination folder\"}".into());
    }
    let (work, pass, cfg) = match restore_source(&v) {
        Ok(x) => x,
        Err((m, need)) => return err_json(&m, need),
    };
    let only = v["only"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let overwrite = v["overwrite"].as_bool().unwrap_or(false);
    *JOB.lock().unwrap() = Some(Job { running: true, dest: if original { "(元の場所 / original)".into() } else { dest.clone() }, ..Default::default() });
    std::thread::spawn(move || {
        let res = if original {
            restore::run_original(&work, &cfg, pass.as_deref(), only.as_deref(), overwrite)
        } else {
            restore::run(&work, std::path::Path::new(&dest), pass.as_deref(), only.as_deref(), overwrite)
        };
        let mut j = JOB.lock().unwrap();
        let mut job = j.take().unwrap_or_default();
        job.running = false;
        job.done = true;
        match res {
            Ok(r) => {
                job.ok = r.failed.is_empty();
                job.restored = r.restored;
                job.skipped = r.skipped;
                job.bytes = r.bytes;
                job.msg = format!("復元{}件 / スキップ{}件(既存) / 失敗{}件", r.restored, r.skipped, r.failed.len());
                job.failed = r.failed.into_iter().take(50).collect();
            }
            Err(e) => {
                job.ok = false;
                job.msg = format!("{e:#}");
            }
        }
        *j = Some(job);
    });
    ("200 OK".into(), "{\"started\":true}".into())
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 実行中でなければ1回バックアップする。同時に2つは走らない。
fn run_backup(last: &Shared) {
    {
        let mut l = last.lock().unwrap();
        if l.running {
            return;
        }
        l.running = true;
    }
    let res = Config::load().and_then(|c| sync::run_auto(&c));
    let mut l = last.lock().unwrap();
    l.running = false;
    l.at = now();
    match res {
        Ok(r) => {
            l.ok = true;
            l.msg = format!("コピー{} 削除{} push={}", r.copied, r.removed, r.pushed);
        }
        Err(e) => {
            l.ok = false;
            l.msg = format!("{e:#}");
        }
    }
}

/// 画面は数秒おきに状態を取るので、GitHub への問い合わせは60秒キャッシュする。
fn cached_login() -> Option<String> {
    static CACHE: Mutex<Option<(Instant, Option<String>, u64)>> = Mutex::new(None);
    let epoch = LOGIN_EPOCH.load(std::sync::atomic::Ordering::SeqCst);
    let mut c = CACHE.lock().unwrap();
    if let Some((t, v, e)) = c.as_ref() {
        if t.elapsed() < Duration::from_secs(60) && *e == epoch {
            return v.clone();
        }
    }
    let v = auth::token().ok().and_then(|t| github::login_name(&t).ok());
    *c = Some((Instant::now(), v.clone(), epoch));
    v
}

fn status_json(last: &Shared, port: u16) -> String {
    let cfg = Config::load().unwrap_or_default();
    let login = cached_login();
    let l = last.lock().unwrap();
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "port": port,
        "login": login,
        "repo": cfg.repo_name_n(1),
        "suggested": suggested_folders(&cfg),
        "folders": cfg.folders.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        "interval_secs": cfg.interval_secs,
        "ai": cfg.ai_classify,
        "running": l.running,
        "last": { "at": l.at, "ok": l.ok, "msg": l.msg },
    })
    .to_string()
}

fn respond(s: &mut TcpStream, code: &str, ctype: &str, body: &str) {
    let _ = write!(
        s,
        "HTTP/1.1 {code}\r\nContent-Type: {ctype}; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

fn header<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    head.lines().skip(1).find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case(name).then(|| v.trim())
    })
}

fn handle(mut s: TcpStream, last: Shared, port: u16) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buf = vec![0u8; 16 * 1024];
    let mut n = 0;
    // ヘッダの終わりまで読む
    while n < buf.len() {
        match s.read(&mut buf[n..]) {
            Ok(0) | Err(_) => break,
            Ok(k) => n += k,
        }
        if buf[..n].windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let text = String::from_utf8_lossy(&buf[..n]).to_string();
    let Some((head, rest)) = text.split_once("\r\n\r\n") else { return };
    let mut first = head.lines().next().unwrap_or("").split(' ');
    let (method, path) = (first.next().unwrap_or(""), first.next().unwrap_or(""));

    // 他サイトのページ(CSRF)や DNS リバインディングからの操作を拒否する
    let host_ok = header(head, "host")
        .map(|h| h == format!("127.0.0.1:{port}") || h == format!("localhost:{port}"))
        .unwrap_or(false);
    let origin_ok = header(head, "origin")
        .map(|o| o == format!("http://127.0.0.1:{port}") || o == format!("http://localhost:{port}"))
        .unwrap_or(true);
    if !host_ok || !origin_ok {
        return respond(&mut s, "403 Forbidden", "text/plain", "forbidden");
    }

    // 本文(POST)
    let clen: usize = header(head, "content-length").and_then(|v| v.parse().ok()).unwrap_or(0).min(8192);
    let mut body = rest.as_bytes().to_vec();
    while body.len() < clen {
        let mut b = [0u8; 2048];
        match s.read(&mut b) {
            Ok(0) | Err(_) => break,
            Ok(k) => body.extend_from_slice(&b[..k]),
        }
    }
    let folder = || {
        serde_json::from_slice::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v["folder"].as_str().map(PathBuf::from))
    };

    match (method, path) {
        ("GET", "/ui") | ("GET", "/") => respond(&mut s, "200 OK", "text/html", UI_HTML),
        ("GET", "/health") => respond(&mut s, "200 OK", "text/plain", "ok"),
        ("GET", "/status") => respond(&mut s, "200 OK", "application/json", &status_json(&last, port)),
        ("POST", "/login") => {
            // 画面から貼り付けたトークンを確かめて保存する(値は返さない・ログにも出さない)
            let tok = serde_json::from_slice::<serde_json::Value>(&body).ok().and_then(|v| v["token"].as_str().map(|s| s.trim().to_string())).unwrap_or_default();
            if tok.len() < 20 {
                return respond(&mut s, "400 Bad Request", "application/json", "{\"error\":\"トークンが短すぎます / token too short\"}");
            }
            match github::login_name(&tok) {
                Ok(name) if !name.is_empty() => match auth::store_token(&tok) {
                    Ok(()) => {
                        reset_login_cache();
                        respond(&mut s, "200 OK", "application/json", &status_json(&last, port));
                    }
                    Err(_) => respond(&mut s, "500 Internal Server Error", "application/json", "{\"error\":\"トークンを保存できませんでした / could not store the token\"}"),
                },
                _ => respond(&mut s, "401 Unauthorized", "application/json", "{\"error\":\"このトークンではログインできません(権限・有効期限を確認) / token rejected\"}"),
            }
        }
        ("POST", "/logout") => {
            let _ = auth::logout();
            reset_login_cache();
            respond(&mut s, "200 OK", "application/json", &status_json(&last, port));
        }
        ("POST", "/restore/list") => {
            let (code, b) = restore_list(&jval(&body));
            respond(&mut s, &code, "application/json", &b);
        }
        ("POST", "/restore/run") => {
            let (code, b) = restore_start(jval(&body));
            respond(&mut s, &code, "application/json", &b);
        }
        ("GET", "/restore/status") => {
            let j = JOB.lock().unwrap().clone().unwrap_or_default();
            respond(&mut s, "200 OK", "application/json", &serde_json::to_string(&j).unwrap_or_default());
        }
        ("POST", "/run") => {
            let l = last.clone();
            std::thread::spawn(move || run_backup(&l));
            respond(&mut s, "200 OK", "application/json", "{\"started\":true}");
        }
        ("POST", "/add") | ("POST", "/remove") => {
            let Some(f) = folder() else {
                return respond(&mut s, "400 Bad Request", "application/json", "{\"error\":\"folder が必要です\"}");
            };
            let mut cfg = Config::load().unwrap_or_default();
            let canon = std::fs::canonicalize(&f)
                .map(|c| PathBuf::from(c.to_string_lossy().trim_start_matches("\\\\?\\")))
                .ok();
            if path == "/add" {
                match canon {
                    Some(c) if c.is_dir() => {
                        if !cfg.folders.contains(&c) {
                            cfg.folders.push(c);
                            let _ = cfg.save();
                        }
                    }
                    _ => return respond(&mut s, "400 Bad Request", "application/json", "{\"error\":\"フォルダが見つかりません\"}"),
                }
            } else {
                let c = canon.unwrap_or(f);
                cfg.folders.retain(|x| *x != c);
                let _ = cfg.save();
            }
            respond(&mut s, "200 OK", "application/json", &status_json(&last, port));
        }
        _ => respond(&mut s, "404 Not Found", "text/plain", "not found"),
    }
}

pub fn serve(port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let last: Shared = Arc::new(Mutex::new(Last::default()));
    println!("open-backup: http://127.0.0.1:{port}/ui");

    // 監視 + 定期実行
    let l = last.clone();
    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |e: notify::Result<notify::Event>| {
            if e.is_ok() {
                let _ = tx.send(());
            }
        })
        .ok();
        let mut watched: Vec<PathBuf> = vec![];
        let mut at = Instant::now() - Duration::from_secs(86400);
        loop {
            let cfg = Config::load().unwrap_or_default();
            if let Some(w) = watcher.as_mut() {
                let new: Vec<PathBuf> =
                    cfg.folders.iter().filter(|f| !watched.contains(f) && f.is_dir()).cloned().collect();
                for f in new {
                    if w.watch(&f, RecursiveMode::Recursive).is_ok() {
                        watched.push(f);
                    }
                }
            }
            let mut dirty = rx.recv_timeout(Duration::from_secs(5)).is_ok();
            if dirty {
                // 連続保存をまとめる(30秒静かになるまで待つ)
                while rx.recv_timeout(Duration::from_secs(30)).is_ok() {}
                dirty = true;
            }
            let due = at.elapsed() >= Duration::from_secs(cfg.interval_secs);
            if !cfg.folders.is_empty() && auth::token().is_ok() && (dirty || due) {
                run_backup(&l);
                at = Instant::now();
            }
        }
    });

    for conn in listener.incoming().flatten() {
        let l = last.clone();
        std::thread::spawn(move || handle(conn, l, port));
    }
    Ok(())
}

/// ログイン状態の控えを捨てる(ログイン・ログアウトの直後に、画面へすぐ反映するため)。
fn reset_login_cache() {
    // `cached_login` の控えは60秒。ログイン操作の直後だけ、すぐ取り直したい。
    LOGIN_EPOCH.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}
static LOGIN_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 画面の「追加」ボタン用の、よく使うフォルダ(存在して、まだ対象に入っていないもの)。
fn suggested_folders(cfg: &Config) -> Vec<String> {
    let mut v: Vec<PathBuf> = vec![];
    #[cfg(target_os = "android")]
    for n in ["DCIM", "Pictures", "Documents", "Download", "Movies", "Music"] {
        v.push(PathBuf::from("/storage/emulated/0").join(n));
    }
    #[cfg(not(target_os = "android"))]
    {
        v.extend([dirs::document_dir(), dirs::picture_dir(), dirs::desktop_dir(), dirs::video_dir(), dirs::audio_dir()].into_iter().flatten());
    }
    v.into_iter().filter(|p| p.is_dir() && !cfg.folders.contains(p)).map(|p| p.display().to_string()).collect()
}
