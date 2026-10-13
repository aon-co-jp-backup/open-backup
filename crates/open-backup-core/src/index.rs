//! 自動カテゴリ分けと自動索引。ファイルは元の構造のまま保存し、分類は索引にだけ持たせる。
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use walkdir::WalkDir;

pub const INDEX_DIR: &str = ".open-backup";

#[derive(Serialize)]
struct Entry {
    path: String,
    size: u64,
    modified: u64,
    category: &'static str,
    topic: String,
    repo: String,
}

/// 拡張子によるカテゴリ分け(内容を見る分類は将来 aruaru-llm で追加)
pub fn category(path: &Path) -> &'static str {
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "txt" | "md" | "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "csv" | "rtf" => "文書",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "heic" | "tif" | "tiff" | "svg" | "raw" => "画像",
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "wmv" | "m4v" => "動画",
        "mp3" | "wav" | "flac" | "m4a" | "ogg" | "aac" | "dsf" | "dff" | "mqa" => "音声",
        "rs" | "py" | "js" | "ts" | "go" | "java" | "kt" | "c" | "h" | "cpp" | "cs" | "rb" | "php" | "sh" | "ps1" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sql" => "コード",
        "zip" | "7z" | "rar" | "tar" | "gz" | "xz" | "iso" => "圧縮",
        _ => "その他",
    }
}

/// 索引に載せる1ファイルの情報(パスは `/` 区切りの、リポジトリの中のパス)。
pub struct FileRec {
    pub rel: String,
    pub size: u64,
    pub modified: u64,
}

/// 索引(`index.jsonl` と `INDEX.md` の中身)をメモリ上で作る。作業フォルダの無い保存方法(GitHub API)でも使う。
pub fn render(recs: &[FileRec], repo: &str, topics: &std::collections::HashMap<String, String>) -> Result<(String, String)> {
    let mut lines = String::new();
    let mut stats: BTreeMap<&'static str, (u64, u64)> = BTreeMap::new();
    for r in recs {
        let cat = category(Path::new(&r.rel));
        let s = stats.entry(cat).or_default();
        s.0 += 1;
        s.1 += r.size;
        lines.push_str(&serde_json::to_string(&Entry {
            topic: topics.get(&r.rel).cloned().unwrap_or_default(),
            path: r.rel.clone(),
            size: r.size,
            modified: r.modified,
            category: cat,
            repo: repo.into(),
        })?);
        lines.push('\n');
    }
    let mut md = format!("# バックアップ索引 / Backup index\n\nリポジトリ / repo: `{repo}`\n\n| カテゴリ / Category | 件数 / Files | 容量 / Size (MB) |\n|---|---|---|\n");
    for (c, (n, sz)) in stats {
        md.push_str(&format!("| {c} | {n} | {} |\n", sz / (1024 * 1024)));
    }
    md.push_str("\n全ファイルの一覧: `.open-backup/index.jsonl`(1行1ファイル)\n");
    Ok((lines, md))
}

/// 作業コピー全体を走査して索引(jsonl + INDEX.md)を書く。
pub fn write_index(work: &Path, repo: &str, topics: &std::collections::HashMap<String, String>) -> Result<()> {
    let mut recs = vec![];
    for e in WalkDir::new(work)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || (e.file_name() != ".git" && e.file_name() != INDEX_DIR))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let Ok(md) = e.metadata() else { continue };
        let rel = e.path().strip_prefix(work)?.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
        if rel == "INDEX.md" {
            continue;
        }
        let modified = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        recs.push(FileRec { rel, size: md.len(), modified });
    }
    let (jsonl, md) = render(&recs, repo, topics)?;
    std::fs::create_dir_all(work.join(INDEX_DIR))?;
    std::fs::write(work.join(INDEX_DIR).join("index.jsonl"), jsonl)?;
    std::fs::write(work.join("INDEX.md"), md)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorizes_and_writes_index() {
        let d = std::env::temp_dir().join(format!("ob-idx-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("docs/sub")).unwrap();
        std::fs::write(d.join("docs/a.PDF"), b"x").unwrap();
        std::fs::write(d.join("docs/sub/b.rs"), b"fn main(){}").unwrap();
        std::fs::write(d.join("noext"), b"y").unwrap();
        write_index(&d, "repo-0001", &Default::default()).unwrap();
        let j = std::fs::read_to_string(d.join(".open-backup/index.jsonl")).unwrap();
        assert!(j.contains("\"path\":\"docs/a.PDF\"") && j.contains("文書"));
        assert!(j.contains("docs/sub/b.rs") && j.contains("コード"));
        assert!(j.contains("その他"));
        assert!(std::fs::read_to_string(d.join("INDEX.md")).unwrap().contains("repo-0001"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
