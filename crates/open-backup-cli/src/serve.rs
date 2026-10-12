//! 常駐エンジン: 127.0.0.1 専用の小さな HTTP サーバー。画面(/ui)を配り、状態の取得・今すぐ実行・フォルダ追加/削除を受ける。
//! フォルダ変更の監視と定期実行も同じプロセスで行う。
use anyhow::Result;
use notify::{RecursiveMode, Watcher};
use open_backup_core::{auth, config::Config, github, sync};
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
    let res = Config::load().and_then(|c| sync::run_once(&c));
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
    static CACHE: Mutex<Option<(Instant, Option<String>)>> = Mutex::new(None);
    let mut c = CACHE.lock().unwrap();
    if let Some((t, v)) = c.as_ref() {
        if t.elapsed() < Duration::from_secs(60) {
            return v.clone();
        }
    }
    let v = auth::token().ok().and_then(|t| github::login_name(&t).ok());
    *c = Some((Instant::now(), v.clone()));
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
