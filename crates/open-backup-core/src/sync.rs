use crate::config::{data_dir, Config};
use crate::{auth, github};
use anyhow::{bail, Context, Result};
use base64::Engine;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct Report {
    pub copied: usize,
    pub removed: usize,
    pub skipped_large: Vec<PathBuf>,
    pub pushed: bool,
}

fn git(dir: &Path, token: Option<&str>, args: &[&str]) -> Result<String> {
    let mut c = Command::new("git");
    c.current_dir(dir).args(args).env("GIT_TERMINAL_PROMPT", "0");
    if let Some(t) = token {
        // トークンはコマンドラインに出さず環境変数で渡す
        let b = base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{t}"));
        c.env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "http.extraheader")
            .env("GIT_CONFIG_VALUE_0", format!("Authorization: Basic {b}"));
    }
    let o = c.output().context("git が見つかりません")?;
    if !o.status.success() {
        let mut err = String::from_utf8_lossy(&o.stderr).to_string();
        if let Some(t) = token {
            err = err.replace(t, "***");
        }
        bail!("git {} 失敗: {}", args.join(" "), err);
    }
    Ok(String::from_utf8_lossy(&o.stdout).to_string())
}

fn label(p: &Path, used: &mut HashSet<String>) -> String {
    let base = p
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".into());
    let mut l = base.clone();
    let mut n = 2;
    while !used.insert(l.clone()) {
        l = format!("{base}-{n}");
        n += 1;
    }
    l
}

fn mirror(src: &Path, dst: &Path, cfg: &Config, rep: &mut Report) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    let max = cfg.max_file_mb * 1024 * 1024;
    let excluded = |name: &str| cfg.exclude.iter().any(|e| e == name);
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for e in WalkDir::new(src)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !excluded(&e.file_name().to_string_lossy()))
    {
        let Ok(e) = e else { continue };
        if !e.file_type().is_file() {
            continue;
        }
        let rel = e.path().strip_prefix(src)?.to_path_buf();
        let Ok(md) = e.metadata() else { continue };
        if md.len() > max {
            rep.skipped_large.push(e.path().to_path_buf());
            continue;
        }
        let to = dst.join(&rel);
        let same = to
            .metadata()
            .map(|m| m.len() == md.len() && m.modified().ok() == md.modified().ok())
            .unwrap_or(false);
        if !same {
            if let Some(p) = to.parent() {
                std::fs::create_dir_all(p)?;
            }
            // 使用中などで読めないファイルは次回に回す
            if std::fs::copy(e.path(), &to).is_err() {
                continue;
            }
            rep.copied += 1;
            if let Ok(t) = md.modified() {
                let _ = std::fs::File::options()
                    .write(true)
                    .open(&to)
                    .and_then(|f| f.set_modified(t));
            }
        }
        seen.insert(rel);
    }
    // 元で削除されたものをミラー側からも消す(git の履歴には残る)
    for e in WalkDir::new(dst).contents_first(true).into_iter().filter_map(|e| e.ok()) {
        let rel = e.path().strip_prefix(dst)?;
        if e.file_type().is_file() && !seen.contains(rel) {
            std::fs::remove_file(e.path())?;
            rep.removed += 1;
        } else if e.file_type().is_dir() && e.depth() > 0 {
            let _ = std::fs::remove_dir(e.path()); // 空なら消える
        }
    }
    Ok(())
}


#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct State {
    /// 現在書き込み中のリポジトリの世代番号
    pub(crate) active: u32,
}

pub(crate) fn load_state() -> State {
    data_dir()
        .ok()
        .and_then(|d| std::fs::read_to_string(d.join("state.json")).ok())
        .and_then(|s| sonic_rs::from_str(&s).ok())
        .unwrap_or(State { active: 1 })
}

pub(crate) fn save_state(s: &State) -> Result<()> {
    let d = data_dir()?;
    std::fs::create_dir_all(&d)?;
    std::fs::write(d.join("state.json"), serde_json::to_string(s)?)?;
    Ok(())
}

fn dir_size_kb(p: &Path) -> u64 {
    WalkDir::new(p)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum::<u64>()
        / 1024
}

/// 全対象フォルダをミラー → commit → push。
/// 初回は現用(-0001)と次用(-0002)の2つの非公開リポジトリを同時に作る。
/// 現用が上限に近づいたら次用へ切り替え、さらに次の新規リポジトリを作る。
pub fn run_once(cfg: &Config) -> Result<Report> {
    if cfg.folders.is_empty() {
        bail!("対象フォルダが未設定です。`open-backup add <フォルダ>`");
    }
    let token = auth::token()?;
    let owner = github::login_name(&token)?;
    let mut st = load_state();
    github::ensure_private_repo(&token, &owner, &cfg.repo_name_n(st.active))?;
    if let Err(e) = github::ensure_private_repo(&token, &owner, &cfg.repo_name_n(st.active + 1)) {
        eprintln!("警告: 予備リポジトリの作成に失敗(次回再試行): {e:#}");
    }

    let eng = crate::ai::start(cfg);
    if eng.is_none() && cfg.ai_classify {
        eprintln!("AI分類は利用できません(aruaru-llm 未同梱/起動失敗)。拡張子分類のみで続行します");
    }
    let mut total = Report::default();
    // 切り替え直後は新しい現用リポジトリへ全データを入れ直す(最大1回)
    for _ in 0..2 {
        let name = cfg.repo_name_n(st.active);
        let work = data_dir()?.join("repos").join(&name);
        let rep = backup_into(cfg, &token, &owner, &name, &work, eng.as_ref())?;
        total.copied += rep.copied;
        total.removed += rep.removed;
        total.skipped_large.extend(rep.skipped_large);
        total.pushed |= rep.pushed;

        if dir_size_kb(&work.join(".git")) < cfg.max_repo_mb * 1024 * 9 / 10 {
            break;
        }
        // データ自体が1リポジトリの上限を超えている場合、切り替えても解決しない
        // (実行のたびに新しいリポジトリを作り続けてしまう)ため、警告して現状維持にする。
        let data_kb = dir_size_kb(&work).saturating_sub(dir_size_kb(&work.join(".git")));
        let data_mb = data_kb / 1024;
        if data_kb >= cfg.max_repo_mb * 1024 * 9 / 10 {
            eprintln!(
                "警告: バックアップ対象のデータ({data_mb}MB)が1リポジトリの上限({}MB)に近いため、自動切り替えをしません。max_repo_mb を上げるか対象を減らしてください",
                cfg.max_repo_mb
            );
            break;
        }
        // 上限の9割に達した: 次へ切り替え。古い世代はGitHubに残し、手元の作業コピーは消す。
        // 次のリポジトリを用意できない(レート制限等)ときは切り替えず、現用に保存を続ける。
        let next = cfg.repo_name_n(st.active + 1);
        if let Err(e) = github::ensure_private_repo(&token, &owner, &next) {
            eprintln!("警告: 次のリポジトリ {next} を用意できないため切り替えを見送ります: {e:#}");
            break;
        }
        eprintln!("{name} が上限に近いため次のリポジトリへ切り替えます");
        let _ = std::fs::remove_dir_all(&work);
        st.active += 1;
        save_state(&st)?;
        // 予備(さらに次)の作成失敗は致命的ではない。次回以降に再試行する。
        if let Err(e) = github::ensure_private_repo(&token, &owner, &cfg.repo_name_n(st.active + 1)) {
            eprintln!("警告: 予備リポジトリの作成に失敗(次回再試行): {e:#}");
        }
    }
    save_state(&st)?;
    Ok(total)
}

fn backup_into(
    cfg: &Config,
    token: &str,
    owner: &str,
    name: &str,
    work: &Path,
    eng: Option<&crate::ai::Engine>,
) -> Result<Report> {
    std::fs::create_dir_all(work)?;
    if !work.join(".git").exists() {
        git(work, None, &["init", "-q", "-b", "main"])?;
        git(
            work,
            None,
            &["remote", "add", "origin", &format!("https://github.com/{owner}/{name}.git")],
        )?;
        // 既存の履歴があれば取り込む(別端末からの再接続など。空リポジトリなら失敗して無視)
        let _ = git(work, Some(token), &["pull", "-q", "--rebase", "origin", "main"]);
    }
    git(work, None, &["config", "core.longpaths", "true"])?;

    let mut rep = Report::default();
    let mut used = HashSet::new();
    for f in &cfg.folders {
        if !f.is_dir() {
            eprintln!("スキップ(存在しない): {}", f.display());
            continue;
        }
        let l = label(f, &mut used);
        mirror(f, &work.join(&l), cfg, &mut rep)?;
    }
    let topics = topics_for(work, eng);
    crate::index::write_index(work, name, &topics)?;
    git(work, None, &["add", "-A"])?;
    if git(work, None, &["status", "--porcelain"])?.trim().is_empty() {
        return Ok(rep);
    }
    let msg = format!("backup: +{} -{}", rep.copied, rep.removed);
    git(
        work,
        None,
        &[
            "-c", "user.name=open-backup",
            "-c", "user.email=open-backup@users.noreply.github.com",
            "commit", "-q", "-m", &msg,
        ],
    )?;
    git(work, Some(token), &["push", "-q", "-u", "origin", "main"])?;
    rep.pushed = true;
    Ok(rep)
}

/// 内容による分類(トピック)。エンジンが無ければ空(拡張子分類のみ)。
fn topics_for(work: &Path, eng: Option<&crate::ai::Engine>) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let Some(eng) = eng else { return out };
    let mut cache = crate::ai::load_cache();
    let mut files = vec![];
    for e in WalkDir::new(work)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || (e.file_name() != ".git" && e.file_name() != crate::index::INDEX_DIR))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let Ok(md) = e.metadata() else { continue };
        let rel = e.path().strip_prefix(work).map(|p| p.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/")).unwrap_or_default();
        if rel == "INDEX.md" { continue }
        let mt = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        files.push((format!("{rel}|{}|{mt}", md.len()), e.path().to_path_buf(), rel));
    }
    crate::ai::fill(eng, &files, &mut cache, 500);
    crate::ai::save_cache(&cache);
    for (k, _, rel) in files {
        if let Some(t) = cache.get(&k) { out.insert(rel, t.clone()); }
    }
    out
}

/// 指定フォルダの索引(拡張子+AIトピック)だけを作る。GitHub へは何も送らない(動作確認用)。
pub fn index_only(cfg: &Config, dir: &Path) -> Result<()> {
    let eng = crate::ai::start(cfg);
    if eng.is_none() {
        eprintln!("AI分類は利用できません。拡張子分類のみ");
    }
    let topics = topics_for(dir, eng.as_ref());
    crate::index::write_index(dir, "index-only", &topics)
}

/// `git` のコマンドが使えるか。
pub fn git_available() -> bool {
    Command::new("git").arg("--version").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
}

/// 保存方法を選んで1回バックアップする。`auto` は、Android では常に API、それ以外は `git` があれば `git`、無ければ API。
pub fn run_auto(cfg: &Config) -> Result<Report> {
    let use_git = match cfg.backend.as_str() {
        "git" => true,
        "api" => false,
        _ => !cfg!(target_os = "android") && git_available(),
    };
    if use_git {
        run_once(cfg)
    } else {
        crate::apisync::run_once(cfg)
    }
}
