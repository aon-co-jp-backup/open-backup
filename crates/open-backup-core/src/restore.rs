//! 復元: 保存した作業コピー(暗号化でも平文でも)から、好きな場所へ戻す。
//! 画面・CLI から共通で使う。既存のファイルは、`overwrite` が無い限り上書きしない。
use crate::config::{data_dir, Config};
use crate::crypt;
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    /// `<ラベル>/<パス>`
    pub rel: String,
    pub size: u64,
    pub mtime: u64,
    pub topic: Option<String>,
}

/// 暗号化されたバックアップか(`.open-backup/crypt.json` がある)
pub fn is_encrypted(work: &Path) -> bool {
    work.join(crypt::META_DIR).join(crypt::PARAMS_FILE).exists()
}

/// 復元元の作業コピーの場所。`github` なら新しく取得(clone)する。無ければ端末内の作業コピーを使う。
pub fn workdir(cfg: &Config, gen: u32, github: bool) -> Result<PathBuf> {
    let name = cfg.repo_name_n(gen);
    let local = data_dir()?.join("repos").join(&name);
    if !github && local.join(".git").exists() {
        return Ok(local);
    }
    crate::sync::fetch_repo(cfg, &name)
}

fn plain_items(work: &Path) -> Vec<Item> {
    let mut v = vec![];
    for e in WalkDir::new(work)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || (e.file_name() != ".git" && e.file_name() != crate::index::INDEX_DIR))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let Ok(rel) = e.path().strip_prefix(work) else { continue };
        let rel = rel.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
        if rel == "INDEX.md" {
            continue;
        }
        let Ok(md) = e.metadata() else { continue };
        let mt = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        v.push(Item { rel, size: md.len(), mtime: mt, topic: None });
    }
    v.sort_by(|a, b| a.rel.cmp(&b.rel));
    v
}

/// 復元できるファイルの一覧(暗号化なら `pass` が必要)。
pub fn list(work: &Path, pass: Option<&str>) -> Result<(bool, Vec<Item>)> {
    if is_encrypted(work) {
        let pass = pass.context("暗号化されたバックアップです。パスフレーズが必要です")?;
        let (_, entries) = crypt::read_manifest(work, pass)?;
        let items = entries.into_iter().map(|e| Item { rel: e.rel, size: e.size, mtime: e.mtime, topic: e.topic }).collect();
        Ok((true, items))
    } else {
        Ok((false, plain_items(work)))
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub restored: usize,
    pub skipped: usize,
    pub bytes: u64,
    pub failed: Vec<String>,
}

fn safe_rel(rel: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for c in Path::new(rel).components() {
        match c {
            Component::Normal(s) => out.push(s),
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

/// `only` は `<ラベル>/<パス>` の先頭一致(空なら全部)。
pub fn run(work: &Path, dest: &Path, pass: Option<&str>, only: Option<&str>, overwrite: bool) -> Result<Report> {
    if dest.as_os_str().is_empty() {
        bail!("復元先のフォルダを指定してください");
    }
    std::fs::create_dir_all(dest).with_context(|| format!("復元先を作れません: {}", dest.display()))?;
    if is_encrypted(work) {
        let pass = pass.context("暗号化されたバックアップです。パスフレーズが必要です")?;
        let r = crypt::restore(work, dest, pass, only, overwrite)?;
        return Ok(Report { restored: r.restored, skipped: r.skipped, bytes: r.bytes, failed: r.failed });
    }
    let mut rep = Report::default();
    for it in plain_items(work).into_iter().filter(|i| only.map(|o| i.rel.starts_with(o)).unwrap_or(true)) {
        let Some(rel) = safe_rel(&it.rel) else {
            rep.failed.push(format!("{}: 危険なパスのため復元しません", it.rel));
            continue;
        };
        let to = dest.join(&rel);
        if !overwrite && to.exists() {
            rep.skipped += 1;
            continue;
        }
        if let Some(p) = to.parent() {
            std::fs::create_dir_all(p)?;
        }
        match std::fs::copy(work.join(&rel), &to) {
            Ok(n) => {
                rep.restored += 1;
                rep.bytes += n;
                let t = std::time::UNIX_EPOCH + std::time::Duration::from_secs(it.mtime);
                let _ = std::fs::File::options().write(true).open(&to).and_then(|f| f.set_modified(t));
            }
            Err(e) => rep.failed.push(format!("{}: {e}", it.rel)),
        }
    }
    Ok(rep)
}

/// 「元の場所へ」: 目録のラベル(= 対象フォルダの名前)を、現在の設定の対象フォルダへ対応づける。
pub fn original_targets(cfg: &Config) -> Vec<(String, PathBuf)> {
    let mut used = std::collections::HashSet::new();
    cfg.folders.iter().map(|f| (crate::sync::label(f, &mut used), f.clone())).collect()
}

/// 各ラベルを、元のフォルダ(の親)へ戻す。ラベルに対応するフォルダが現在の設定に無いものは、結果に失敗として載せる。
pub fn run_original(work: &Path, cfg: &Config, pass: Option<&str>, only: Option<&str>, overwrite: bool) -> Result<Report> {
    let mut total = Report::default();
    let (_, items) = list(work, pass)?;
    let labels: std::collections::BTreeSet<String> = items.iter().filter_map(|i| i.rel.split('/').next().map(|s| s.to_string())).collect();
    let targets = original_targets(cfg);
    for l in labels {
        if let Some(o) = only {
            if !(o.starts_with(&format!("{l}/")) || o == l || l.starts_with(o)) {
                continue;
            }
        }
        let Some((_, folder)) = targets.iter().find(|(tl, _)| *tl == l) else {
            total.failed.push(format!("{l}: 現在の設定に、元のフォルダが見つかりません(別の場所を指定してください)"));
            continue;
        };
        let Some(parent) = folder.parent() else {
            total.failed.push(format!("{l}: 元のフォルダの親が分かりません"));
            continue;
        };
        let sub = match only {
            Some(o) if o.starts_with(&format!("{l}/")) => o.to_string(),
            _ => format!("{l}/"),
        };
        let r = run(work, parent, pass, Some(&sub), overwrite)?;
        total.restored += r.restored;
        total.skipped += r.skipped;
        total.bytes += r.bytes;
        total.failed.extend(r.failed);
    }
    Ok(total)
}
