//! 実際の GitHub を通した、暗号化バックアップの通し試験(手動。`#[ignore]`)。
//! 非公開のテスト用リポジトリを作り(`open-backup-e2e-<方式>-<時刻>-0001/-0002`)、暗号化して送り、GitHub から取り直して復元し、元と比べる。
//! 実行: OPEN_BACKUP_E2E_TOKEN=<テスト用トークン> OPEN_BACKUP_E2E_PASSPHRASE=<試験用パスフレーズ> \
//!       cargo test -p open-backup-core --release --test e2e_github -- --ignored --nocapture
//! 作ったテスト用リポジトリは削除しない(GitHub の画面から、利用者が削除する)。
use open_backup_core::config::Config;
use open_backup_core::crypt::Params;
use open_backup_core::{restore, sync};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut m = BTreeMap::new();
    for e in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let rel = e.path().strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
        m.insert(rel, blake3::hash(&std::fs::read(e.path()).unwrap()).as_bytes().to_vec());
    }
    m
}

fn write(p: &Path, b: &[u8]) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, b).unwrap();
}

fn gh(token: &str, path: &str) -> serde_json::Value {
    ureq::get(&format!("https://api.github.com/{path}"))
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "open-backup-e2e")
        .set("Accept", "application/vnd.github+json")
        .call()
        .unwrap_or_else(|e| panic!("GitHub GET {path}: {e}"))
        .into_json()
        .unwrap()
}

fn run_backend(backend: &str, token: &str, pass: &str, par: &Params) {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let name = format!("open-backup-e2e-{backend}-{ts}");
    let root: PathBuf = std::env::temp_dir().join(format!("ob-e2e-gh-{backend}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (home, src) = (root.join("home"), root.join("src").join("社外秘"));
    std::fs::create_dir_all(&home).unwrap();
    // 設定・状態・作業コピーは、この試験専用の場所(利用者の本来の設定・資格情報ストアには触れない)
    std::env::set_var("OPEN_BACKUP_HOME", &home);
    std::env::set_var("OPEN_BACKUP_TOKEN", token);
    std::env::set_var("OPEN_BACKUP_PASSPHRASE", pass);
    let cdir = open_backup_core::config::config_dir().unwrap();
    std::fs::create_dir_all(&cdir).unwrap();
    std::fs::write(cdir.join("crypt.json"), serde_json::to_string(par).unwrap()).unwrap();

    let mut seed = 99u64;
    let rnd: Vec<u8> = (0..2 * 1024 * 1024 + 7).map(|_| { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; seed as u8 }).collect();
    write(&src.join("契約書.txt"), "極秘の契約内容 ".repeat(40_000).as_bytes());
    write(&src.join("写真").join("a.jpg"), &rnd);
    write(&src.join("メモ").join("深い").join("b.md"), "こんにちは".as_bytes());
    write(&src.join("空.bin"), b"");

    let cfg = Config {
        folders: vec![src.clone()],
        repo: Some(name.clone()),
        encrypt: true,
        backend: backend.into(),
        ai_classify: false,
        threads: 2,
        ..Config::default()
    };
    let repo = cfg.repo_name_n(1);

    // 1回目: 暗号化して GitHub へ
    let r1 = sync::run_auto(&cfg).unwrap_or_else(|e| panic!("[{backend}] 1回目のバックアップ失敗: {e:#}"));
    assert!(r1.pushed, "[{backend}] push された");
    let owner = open_backup_core::github::login_name(token).unwrap();
    println!("[{backend}] owner={owner} repo={repo} copied={}", r1.copied);

    // GitHub 側の確認: 非公開、平文の名前が無い、目録・設定がある
    let info = gh(token, &format!("repos/{owner}/{repo}"));
    assert_eq!(info["private"], true, "[{backend}] 非公開リポジトリ");
    let t = gh(token, &format!("repos/{owner}/{repo}/git/trees/main?recursive=1"));
    let paths: Vec<String> = t["tree"].as_array().unwrap().iter().filter(|e| e["type"] == "blob").map(|e| e["path"].as_str().unwrap().to_string()).collect();
    println!("[{backend}] GitHub 上のファイル {} 件(先頭): {:?}", paths.len(), &paths[..paths.len().min(4)]);
    for p in &paths {
        for bad in ["契約", "写真", "メモ", "深い", "社外秘", "jpg", ".md", ".txt"] {
            assert!(!p.contains(bad) || p == "README-ENCRYPTED.md", "[{backend}] 平文の名前が GitHub に出ている: {p}");
        }
    }
    assert!(paths.iter().any(|p| p == ".open-backup/manifest.obe") && paths.iter().any(|p| p == ".open-backup/crypt.json"));
    assert!(paths.iter().filter(|p| p.starts_with("data/")).count() == 4, "[{backend}] 暗号化ファイルは4件");
    assert!(!paths.iter().any(|p| p == "INDEX.md" || p == ".open-backup/index.jsonl"), "[{backend}] 平文の索引を送っていない");
    // 実際の中身(manifest と1つのデータ)に平文が無い
    for p in [".open-backup/manifest.obe", paths.iter().find(|p| p.starts_with("data/")).unwrap().as_str()] {
        let b = ureq::get(&format!("https://api.github.com/repos/{owner}/{repo}/contents/{p}"))
            .set("Authorization", &format!("Bearer {token}")).set("User-Agent", "open-backup-e2e").set("Accept", "application/vnd.github.raw+json")
            .call().unwrap().into_string().unwrap_or_default();
        assert!(!b.contains("極秘の契約内容") && !b.contains("契約書"), "[{backend}] {p} に平文が含まれる");
    }

    // 2回目: 変更なし → push されない
    let r2 = sync::run_auto(&cfg).unwrap();
    assert!(!r2.pushed, "[{backend}] 変更が無ければ push しない");

    // GitHub から取り直して復元 → 元と一致
    let work = restore::workdir(&cfg, 1, true).unwrap();
    let dest = root.join("restored");
    let rr = restore::run(&work, &dest, Some(pass), None, false).unwrap();
    assert_eq!((rr.restored, rr.failed.len()), (4, 0), "[{backend}] {:?}", rr.failed);
    assert_eq!(tree(&dest.join("社外秘")), tree(&src), "[{backend}] GitHub 経由の復元が元と完全一致");
    assert!(restore::list(&work, Some("間違ったパスフレーズです")).is_err(), "[{backend}] 誤ったパスフレーズは拒否");

    // 変更・削除・追加 → 差分だけ送られ、復元にも反映
    write(&src.join("メモ").join("深い").join("b.md"), "更新しました".as_bytes());
    std::fs::remove_file(src.join("空.bin")).unwrap();
    write(&src.join("新規.txt"), b"new");
    let r3 = sync::run_auto(&cfg).unwrap();
    assert!(r3.pushed, "[{backend}] 変更を push");
    let work2 = restore::workdir(&cfg, 1, true).unwrap();
    let dest2 = root.join("restored2");
    let rr2 = restore::run(&work2, &dest2, Some(pass), None, false).unwrap();
    assert_eq!(rr2.failed.len(), 0);
    assert_eq!(tree(&dest2.join("社外秘")), tree(&src), "[{backend}] 更新後の復元も一致");
    println!("[{backend}] OK: 暗号化→GitHub→取得→復元→一致(リポジトリ {repo} は非公開のまま残します)");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
#[ignore]
fn real_github_git_and_api() {
    let token = std::env::var("OPEN_BACKUP_E2E_TOKEN").expect("OPEN_BACKUP_E2E_TOKEN");
    let pass = std::env::var("OPEN_BACKUP_E2E_PASSPHRASE").expect("OPEN_BACKUP_E2E_PASSPHRASE");
    let par = Params::create(&pass).unwrap();
    for backend in ["git", "api"] {
        run_backend(backend, &token, &pass, &par);
    }
}
