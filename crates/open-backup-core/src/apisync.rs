//! GitHub の Git Data API だけで保存する(git のコマンドも libgit2 も使わない、Rust だけの経路)。
//! `git` が無い端末(Android など)で使う。デスクトップでは、`git` があれば従来どおり `git` を使う(`sync::run_once`)。
//!
//! 流れ: 対象フォルダを走査 → 各ファイルの git の blob ハッシュ(SHA-1)を求める(サイズと更新時刻が同じなら前回の値を使う)
//! → リポジトリの現在の木と比べて、違うファイルだけ blob として送る → 木・コミットを作って `main` を進める。
//! 元で消えたファイルは、木から外す(履歴には残る)。送る前に木を比べるので、中断しても、別の端末から続けても、重複して送らない。

use crate::config::{data_dir, Config};
use crate::sync::{load_state, save_state, Report};
use crate::{auth, github, index};
use anyhow::{bail, Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

const API: &str = "https://api.github.com";
/// API で送れる1ファイルの上限(MB)。blob は 100MB までだが、base64 の JSON になるので余裕を見る。
const MAX_FILE_MB: u64 = 40;

#[derive(Default, Serialize, Deserialize)]
struct LocalState {
    /// 最後に自分が `main` に載せたコミット
    head: Option<String>,
    /// その木の、パス → blob の SHA(送る前の比較に使う)
    remote: BTreeMap<String, String>,
    /// ローカルファイルの「サイズ・更新時刻 → git の blob SHA」(毎回ファイル全体を読まないための控え)
    sha_cache: HashMap<String, (u64, u64, String)>,
    /// このリポジトリへ送った量(バイト)。上限の判定に使う
    bytes: u64,
}

fn state_path(repo: &str) -> Result<PathBuf> {
    Ok(data_dir()?.join("api-state").join(format!("{repo}.json")))
}

fn load(repo: &str) -> LocalState {
    state_path(repo).ok().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save(repo: &str, s: &LocalState) -> Result<()> {
    let p = state_path(repo)?;
    std::fs::create_dir_all(p.parent().unwrap())?;
    std::fs::write(p, serde_json::to_string(s)?)?;
    Ok(())
}

/// git の blob の SHA-1(`blob <長さ>\0<内容>`)。
pub fn blob_sha(content: &[u8]) -> String {
    let mut h = Sha1::new();
    h.update(format!("blob {}\0", content.len()).as_bytes());
    h.update(content);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

struct Gh<'a> {
    token: &'a str,
    owner: &'a str,
    repo: &'a str,
}

impl Gh<'_> {
    fn url(&self, p: &str) -> String {
        format!("{API}/repos/{}/{}/{p}", self.owner, self.repo)
    }

    /// 呼び出し。二次レート制限(403/429)・一時的なエラー(5xx)は、待って再試行する。
    fn call(&self, method: &str, path: &str, body: Option<&serde_json::Value>) -> Result<Option<serde_json::Value>> {
        let url = self.url(path);
        for attempt in 0..6 {
            let r = ureq::request(method, &url)
                .set("Authorization", &format!("Bearer {}", self.token))
                .set("Accept", "application/vnd.github+json")
                .set("User-Agent", "open-backup")
                .timeout(Duration::from_secs(300));
            let res = match body {
                Some(b) => r.send_json(b),
                None => r.call(),
            };
            match res {
                Ok(resp) => {
                    return Ok(Some(resp.into_json().unwrap_or(serde_json::Value::Null)));
                }
                Err(ureq::Error::Status(code, resp)) => {
                    let retry_after: u64 = resp.header("retry-after").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let text = resp.into_string().unwrap_or_default();
                    let limited = code == 429 || (code == 403 && (text.contains("rate limit") || text.contains("secondary")));
                    if (limited || code >= 500) && attempt < 5 {
                        let wait = if retry_after > 0 { retry_after } else { 20 * (attempt as u64 + 1) };
                        eprintln!("GitHub が待つように求めています({code})。{wait}秒待って再試行します");
                        std::thread::sleep(Duration::from_secs(wait.min(300)));
                        continue;
                    }
                    if code == 404 || code == 409 {
                        return Ok(None); // 呼び出し側で意味を判断する(空のリポジトリ・参照が無い)
                    }
                    bail!("GitHub API {method} {path}: {code} {}", text.chars().take(200).collect::<String>());
                }
                Err(e) => {
                    if attempt < 5 {
                        std::thread::sleep(Duration::from_secs(5 * (attempt as u64 + 1)));
                        continue;
                    }
                    bail!("GitHub API {method} {path}: {e}");
                }
            }
        }
        bail!("GitHub API {method} {path}: 再試行の上限")
    }

    fn json(&self, method: &str, path: &str, body: Option<&serde_json::Value>) -> Result<serde_json::Value> {
        self.call(method, path, body)?.with_context(|| format!("{method} {path}: 見つからない/空"))
    }

    /// `main` の先頭コミット(空のリポジトリ・ブランチ無しなら None)。
    fn head(&self) -> Result<Option<String>> {
        Ok(self.call("GET", "git/ref/heads/main", None)?.and_then(|v| v["object"]["sha"].as_str().map(String::from)))
    }

    /// 空のリポジトリは Git Data API が使えないので、Contents API で最初のコミットを作る。
    fn init_empty(&self) -> Result<()> {
        let body = serde_json::json!({
            "message": "open-backup: init",
            "content": base64::engine::general_purpose::STANDARD.encode("created by open-backup\n"),
            "branch": "main",
        });
        self.json("PUT", ".open-backup/created", Some(&body))?;
        Ok(())
    }

    fn commit_tree(&self, sha: &str) -> Result<String> {
        let v = self.json("GET", &format!("git/commits/{sha}"), None)?;
        Ok(v["tree"]["sha"].as_str().context("tree なし")?.to_string())
    }

    /// 木(再帰)から、パス → blob SHA の表を作る。
    fn tree_map(&self, tree_sha: &str) -> Result<BTreeMap<String, String>> {
        let v = self.json("GET", &format!("git/trees/{tree_sha}?recursive=1"), None)?;
        if v["truncated"].as_bool() == Some(true) {
            bail!("木が大きすぎて一覧を取り切れません(リポジトリを分けてください)");
        }
        let mut m = BTreeMap::new();
        for e in v["tree"].as_array().into_iter().flatten() {
            if e["type"] == "blob" {
                if let (Some(p), Some(s)) = (e["path"].as_str(), e["sha"].as_str()) {
                    m.insert(p.to_string(), s.to_string());
                }
            }
        }
        Ok(m)
    }

    fn blob(&self, content: &[u8]) -> Result<String> {
        let body = serde_json::json!({ "content": base64::engine::general_purpose::STANDARD.encode(content), "encoding": "base64" });
        let v = self.json("POST", "git/blobs", Some(&body))?;
        Ok(v["sha"].as_str().context("blob の sha なし")?.to_string())
    }
}

struct Local {
    rel: String,
    abs: PathBuf,
    size: u64,
    mtime: u64,
    sha: String,
}

fn label(p: &Path, used: &mut HashSet<String>) -> String {
    let base = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "root".into());
    let mut l = base.clone();
    let mut n = 2;
    while !used.insert(l.clone()) {
        l = format!("{base}-{n}");
        n += 1;
    }
    l
}

/// 対象フォルダを走査して、リポジトリの中のパスごとに、git の blob SHA を求める。
fn scan(cfg: &Config, st: &mut LocalState, rep: &mut Report) -> Result<Vec<Local>> {
    let max = cfg.max_file_mb.min(MAX_FILE_MB) * 1024 * 1024;
    let excluded = |name: &str| cfg.exclude.iter().any(|e| e == name);
    let mut out = vec![];
    let mut used = HashSet::new();
    for f in &cfg.folders {
        if !f.is_dir() {
            eprintln!("スキップ(存在しない): {}", f.display());
            continue;
        }
        let l = label(f, &mut used);
        for e in WalkDir::new(f).follow_links(false).into_iter().filter_entry(|e| e.depth() == 0 || !excluded(&e.file_name().to_string_lossy())) {
            let Ok(e) = e else { continue };
            if !e.file_type().is_file() {
                continue;
            }
            let Ok(md) = e.metadata() else { continue };
            if md.len() > max {
                rep.skipped_large.push(e.path().to_path_buf());
                continue;
            }
            let rel = format!("{l}/{}", e.path().strip_prefix(f)?.to_string_lossy().replace('\\', "/"));
            let mtime = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            let sha = match st.sha_cache.get(&rel) {
                Some((s, m, sha)) if *s == md.len() && *m == mtime => sha.clone(),
                _ => {
                    // 使用中などで読めないファイルは次回に回す
                    let Ok(bytes) = std::fs::read(e.path()) else { continue };
                    let sha = blob_sha(&bytes);
                    st.sha_cache.insert(rel.clone(), (md.len(), mtime, sha.clone()));
                    sha
                }
            };
            out.push(Local { rel, abs: e.path().to_path_buf(), size: md.len(), mtime, sha });
        }
    }
    Ok(out)
}

/// 1回分: 現用のリポジトリへ、差分だけ送る。上限に近ければ次のリポジトリへ切り替える。
pub fn run_once(cfg: &Config) -> Result<Report> {
    if cfg.folders.is_empty() {
        bail!("対象フォルダが未設定です");
    }
    let token = auth::token()?;
    let owner = github::login_name(&token)?;
    let mut st = load_state();
    github::ensure_private_repo(&token, &owner, &cfg.repo_name_n(st.active))?;
    if let Err(e) = github::ensure_private_repo(&token, &owner, &cfg.repo_name_n(st.active + 1)) {
        eprintln!("警告: 予備リポジトリの作成に失敗(次回再試行): {e:#}");
    }
    let mut total = Report::default();
    for _ in 0..2 {
        let name = cfg.repo_name_n(st.active);
        let r = backup_into(cfg, &token, &owner, &name)?;
        total.copied += r.report.copied;
        total.removed += r.report.removed;
        total.skipped_large.extend(r.report.skipped_large);
        total.pushed |= r.report.pushed;
        if r.bytes < cfg.max_repo_mb * 1024 * 1024 * 9 / 10 {
            break;
        }
        if r.data_bytes >= cfg.max_repo_mb * 1024 * 1024 * 9 / 10 {
            eprintln!("警告: バックアップ対象のデータが1リポジトリの上限に近いため、自動切り替えをしません。max_repo_mb を上げるか対象を減らしてください");
            break;
        }
        let next = cfg.repo_name_n(st.active + 1);
        if let Err(e) = github::ensure_private_repo(&token, &owner, &next) {
            eprintln!("警告: 次のリポジトリ {next} を用意できないため切り替えを見送ります: {e:#}");
            break;
        }
        eprintln!("{name} が上限に近いため次のリポジトリへ切り替えます");
        st.active += 1;
        save_state(&st)?;
        if let Err(e) = github::ensure_private_repo(&token, &owner, &cfg.repo_name_n(st.active + 1)) {
            eprintln!("警告: 予備リポジトリの作成に失敗(次回再試行): {e:#}");
        }
    }
    save_state(&st)?;
    Ok(total)
}

struct Done {
    report: Report,
    /// このリポジトリへ送った量(履歴を含む概算)
    bytes: u64,
    /// いまの対象データの合計
    data_bytes: u64,
}

fn backup_into(cfg: &Config, token: &str, owner: &str, repo: &str) -> Result<Done> {
    let gh = Gh { token, owner, repo };
    let mut st = load(repo);
    let mut rep = Report::default();

    // リポジトリの現在の状態
    let head = match gh.head()? {
        Some(h) => h,
        None => {
            gh.init_empty()?;
            st = LocalState::default();
            gh.head()?.context("最初のコミットを作れませんでした")?
        }
    };
    let base_tree = gh.commit_tree(&head)?;
    // 自分が最後に載せたコミットと違えば(別の端末・手動の変更・状態の消失)、リポジトリの木を取り直して基準にする
    if st.head.as_deref() != Some(head.as_str()) {
        st.remote = gh.tree_map(&base_tree)?;
    }

    let locals = scan(cfg, &mut st, &mut rep)?;
    let data_bytes: u64 = locals.iter().map(|l| l.size).sum();

    // 索引(メモリ上で作って、同じように比べて送る)
    let recs: Vec<index::FileRec> = locals.iter().map(|l| index::FileRec { rel: l.rel.clone(), size: l.size, modified: l.mtime }).collect();
    let (jsonl, md) = index::render(&recs, repo, &HashMap::new())?;
    let extra: Vec<(String, Vec<u8>)> = vec![(".open-backup/index.jsonl".into(), jsonl.into_bytes()), ("INDEX.md".into(), md.into_bytes())];

    // 送る必要があるもの
    let mut wanted: BTreeMap<String, (String, Option<&Local>, Option<&[u8]>)> = BTreeMap::new();
    for l in &locals {
        wanted.insert(l.rel.clone(), (l.sha.clone(), Some(l), None));
    }
    for (p, c) in &extra {
        wanted.insert(p.clone(), (blob_sha(c), None, Some(c.as_slice())));
    }
    let labels: HashSet<&str> = locals.iter().filter_map(|l| l.rel.split('/').next()).collect();
    let managed = |p: &str| p == "INDEX.md" || p.starts_with(".open-backup/index") || p.split('/').next().is_some_and(|f| labels.contains(f));

    let mut entries: Vec<serde_json::Value> = vec![];
    let mut uploaded: HashMap<String, String> = HashMap::new(); // sha → 送った blob の sha(同じ内容の重複を送らない)
    let mut sent = 0u64;
    for (path, (sha, local, mem)) in &wanted {
        if st.remote.get(path) == Some(sha) {
            continue; // 変わっていない
        }
        if !uploaded.contains_key(sha) {
            let bytes = match (local, mem) {
                (Some(l), _) => match std::fs::read(&l.abs) {
                    Ok(b) => b,
                    Err(_) => continue, // 読めない(使用中)は次回
                },
                (None, Some(m)) => m.to_vec(),
                _ => continue,
            };
            // 読んでいる間に変わったファイルは、実際の内容の SHA で送る
            let real = blob_sha(&bytes);
            let got = gh.blob(&bytes)?;
            if got != real {
                bail!("blob の SHA が一致しません({path})");
            }
            sent += bytes.len() as u64;
            uploaded.insert(sha.clone(), got.clone());
            if &real != sha {
                uploaded.insert(real.clone(), got);
            }
            std::thread::sleep(Duration::from_millis(700)); // 二次レート制限への配慮(内容を作る呼び出しは1分に約80回まで)
        }
        let blob = uploaded.get(sha).cloned().unwrap_or_else(|| sha.clone());
        entries.push(serde_json::json!({ "path": path, "mode": "100644", "type": "blob", "sha": blob }));
        rep.copied += 1;
    }
    // 元で消えたもの(自分が管理しているパスだけ)
    for p in st.remote.keys() {
        if managed(p) && !wanted.contains_key(p) {
            entries.push(serde_json::json!({ "path": p, "mode": "100644", "type": "blob", "sha": serde_json::Value::Null }));
            rep.removed += 1;
        }
    }

    if entries.is_empty() {
        st.head = Some(head);
        save(repo, &st)?;
        return Ok(Done { report: rep, bytes: st.bytes, data_bytes });
    }

    // 木を作り(1000件ずつ、前の木を土台にして)、コミットして、main を進める
    let mut tree = base_tree;
    for chunk in entries.chunks(1000) {
        let v = gh.json("POST", "git/trees", Some(&serde_json::json!({ "base_tree": tree, "tree": chunk })))?;
        tree = v["sha"].as_str().context("tree の sha なし")?.to_string();
    }
    let msg = format!("backup: +{} -{}", rep.copied, rep.removed);
    let c = gh.json(
        "POST",
        "git/commits",
        Some(&serde_json::json!({ "message": msg, "tree": tree, "parents": [head], "author": { "name": "open-backup", "email": "open-backup@users.noreply.github.com" } })),
    )?;
    let new_head = c["sha"].as_str().context("commit の sha なし")?.to_string();
    gh.json("PATCH", "git/refs/heads/main", Some(&serde_json::json!({ "sha": new_head, "force": false })))?;

    // 状態を更新
    for e in &entries {
        let p = e["path"].as_str().unwrap_or_default().to_string();
        match e["sha"].as_str() {
            Some(s) => {
                st.remote.insert(p, s.to_string());
            }
            None => {
                st.remote.remove(&p);
            }
        }
    }
    st.head = Some(new_head);
    st.bytes += sent;
    rep.pushed = true;
    save(repo, &st)?;
    Ok(Done { report: rep, bytes: st.bytes, data_bytes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_sha_matches_git() {
        // `printf 'hello\n' | git hash-object --stdin` = ce013625030ba8dba906f756967f9e9ca394464a
        assert_eq!(blob_sha(b"hello\n"), "ce013625030ba8dba906f756967f9e9ca394464a");
        // 空のファイル
        assert_eq!(blob_sha(b""), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    }
}
