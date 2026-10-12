//! 内容による分類(トピック)。同梱の aruaru-llm を端末内(127.0.0.1)で起動して使う。
//! 個人データを外へ出さないため、送信先は常にループバック固定。失敗時は拡張子分類だけで続行する。
use crate::config::{data_dir, Config};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub const TOPICS: &[&str] = &[
    "仕事・業務", "お金・家計・領収書", "写真・思い出", "学習・調べもの・資料",
    "開発・プログラム", "契約・公的書類", "設定・システム", "趣味・娯楽", "その他",
];

/// 起動した同梱エンジン。Drop で停止する。
pub struct Engine { child: Option<Child>, base: String }

impl Drop for Engine {
    fn drop(&mut self) { if let Some(c) = self.child.as_mut() { let _ = c.kill(); } }
}

fn exe_path(cfg: &Config) -> Option<PathBuf> {
    if let Some(p) = &cfg.ai_llm_exe { return Some(p.clone()); }
    let me = std::env::current_exe().ok()?;
    let name = if cfg!(windows) { "aruaru-llm.exe" } else { "aruaru-llm" };
    let p = me.parent()?.join(name);
    p.exists().then_some(p)
}

fn ping(base: &str) -> bool {
    ureq::post(&format!("{base}/v1/classify"))
        .timeout(Duration::from_secs(20))
        .send_json(serde_json::json!({"items":["ping"],"categories":["a","b"]}))
        .is_ok()
}

/// AI 分類が有効で、エンジンが使えるときだけ Some。
pub fn start(cfg: &Config) -> Option<Engine> {
    if !cfg.ai_classify { return None; }
    // aruaru-llm は端末に1つだけ。既に動いているもの(open-easy-web 管理・標準ポート4600・自前)を
    // 先に探して共有し、見つからない場合だけ自分用に1つ起動する。いずれも 127.0.0.1 のみ。
    for port in [cfg.ai_port, 4600, 4611] {
        let b = format!("http://127.0.0.1:{port}");
        if ping(&b) { return Some(Engine { child: None, base: b }); }
    }
    let base = format!("http://127.0.0.1:{}", cfg.ai_port);
    let exe = exe_path(cfg)?;
    let child = Command::new(exe)
        .env("ARUARU_LLM_BIND", format!("127.0.0.1:{}", cfg.ai_port))
        .stdout(Stdio::null()).stderr(Stdio::null())
        .spawn().ok()?;
    let mut eng = Engine { child: Some(child), base };
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(90) {
        if ping(&eng.base) { return Some(eng); }
        std::thread::sleep(Duration::from_secs(2));
    }
    eng.child.as_mut().map(|c| c.kill());
    eng.child = None;
    None
}

fn snippet(p: &Path) -> String {
    use std::io::Read;
    let mut buf = vec![0u8; 600];
    let n = std::fs::File::open(p).and_then(|mut f| f.read(&mut buf)).unwrap_or(0);
    match std::str::from_utf8(&buf[..n]) {
        Ok(s) if !s.contains('\0') => s.chars().take(300).collect(),
        _ => String::new(), // バイナリは名前だけで判断
    }
}

fn classify(eng: &Engine, items: &[String]) -> Result<Vec<String>> {
    let r: serde_json::Value = ureq::post(&format!("{}/v1/classify", eng.base))
        .timeout(Duration::from_secs(120))
        .send_json(serde_json::json!({"items": items, "categories": TOPICS, "tenant": "open-backup"}))?
        .into_json()?;
    let Some(a) = r["results"].as_array() else { bail!("応答が不正") };
    Ok(a.iter().map(|x| x["category"].as_str().unwrap_or("その他").to_string()).collect())
}

pub type Cache = HashMap<String, String>; // "path|size|mtime" -> topic

pub fn load_cache() -> Cache {
    data_dir().ok().and_then(|d| std::fs::read_to_string(d.join("topics.json")).ok())
        .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
pub fn save_cache(c: &Cache) {
    if let Ok(d) = data_dir() { let _ = std::fs::create_dir_all(&d); let _ = std::fs::write(d.join("topics.json"), serde_json::to_string(c).unwrap_or_default()); }
}

/// 未分類のファイルを一度に最大 max_new 件まで分類してキャッシュへ入れる。
/// 32件ずつのまとまりを複数スレッドで同時に投げる(エンジン側も複数リクエストを並行処理)。
pub fn fill(eng: &Engine, files: &[(String, PathBuf, String)], cache: &mut Cache, max_new: usize) {
    // files: (キャッシュキー, 実パス, 表示パス)
    let todo: Vec<_> = files.iter().filter(|f| !cache.contains_key(&f.0)).take(max_new).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 4);
    let chunks: Vec<_> = todo.chunks(32).collect();
    for group in chunks.chunks(threads) {
        let results: Vec<Option<Vec<String>>> = std::thread::scope(|s| {
            let hs: Vec<_> = group
                .iter()
                .map(|chunk| {
                    s.spawn(move || {
                        let items: Vec<String> =
                            chunk.iter().map(|f| format!("{}\n{}", f.2, snippet(&f.1))).collect();
                        classify(eng, &items).map_err(|e| eprintln!("AI分類を中断(拡張子分類で続行): {e}")).ok()
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().ok().flatten()).collect()
        });
        for (chunk, res) in group.iter().zip(results) {
            let Some(cats) = res else { return };
            for (f, c) in chunk.iter().zip(cats) {
                cache.insert(f.0.clone(), c);
            }
        }
    }
}
