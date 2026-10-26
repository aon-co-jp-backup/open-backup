//! 暗号化バックアップの通し試験(GitHub へは送らない): 端末 → 暗号化した作業コピー → 復元。
//! 環境変数を使うため、このファイルのテストは1つだけにしている。
use open_backup_core::config::Config;
use open_backup_core::crypt::{self, Params};
use open_backup_core::restore;
use open_backup_core::sync::Report;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn tree(dir: &Path) -> BTreeMap<String, (u64, Vec<u8>)> {
    let mut m = BTreeMap::new();
    for e in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let rel = e.path().strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(e.path()).unwrap();
        m.insert(rel, (bytes.len() as u64, blake3::hash(&bytes).as_bytes().to_vec()));
    }
    m
}

fn write(p: &Path, b: &[u8]) {
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, b).unwrap();
}

#[test]
fn encrypted_backup_roundtrip() {
    let root: PathBuf = std::env::temp_dir().join(format!("ob-e2e-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (home, src, work, dest) = (root.join("home"), root.join("src").join("書類"), root.join("work"), root.join("dest"));
    std::fs::create_dir_all(&home).unwrap();
    std::env::set_var("OPEN_BACKUP_HOME", &home);
    let pass = "open-backup e2e passphrase 2026";
    std::env::set_var("OPEN_BACKUP_PASSPHRASE", pass);

    // 端末の公開パラメータ(軽い Argon2 でも良いが、実際の設定で作る)
    let par = Params::create(pass).unwrap();
    let cdir = open_backup_core::config::config_dir().unwrap();
    std::fs::create_dir_all(&cdir).unwrap();
    std::fs::write(cdir.join("crypt.json"), serde_json::to_string(&par).unwrap()).unwrap();

    // 元データ: 日本語名・入れ子・空ファイル・複数チャンク・圧縮できる/できない
    let mut seed = 7u64;
    let rnd: Vec<u8> = (0..3 * 1024 * 1024 + 123).map(|_| { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; seed as u8 }).collect();
    write(&src.join("契約書.txt"), "秘密の契約内容 ".repeat(50_000).as_bytes());
    write(&src.join("写真").join("img.jpg"), &rnd);
    write(&src.join("写真").join("空.bin"), b"");
    write(&src.join("深い").join("a").join("b").join("c.dat"), &rnd[..1_000_000]);
    write(&src.join("メモ.md"), "こんにちは".as_bytes());

    let cfg = Config { folders: vec![src.clone()], encrypt: true, ai_classify: false, threads: 4, ..Config::default() };

    // 1回目のバックアップ
    let mut rep = Report::default();
    crypt::sync_into(&cfg, &work, None, &mut rep).unwrap();
    assert_eq!(rep.copied, 5, "5ファイルを暗号化");

    // 作業コピーに平文・元の名前が一切無いこと
    for e in walkdir::WalkDir::new(&work).into_iter().filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        for bad in ["契約", "写真", "img", "メモ", "深い", "書類", "jpg", "txt"] {
            assert!(!name.contains(bad), "名前に平文が出ている: {name}");
        }
        if e.file_type().is_file() && e.path().extension().map(|x| x == "obe").unwrap_or(false) {
            let b = std::fs::read(e.path()).unwrap();
            assert!(!String::from_utf8_lossy(&b).contains("秘密の契約内容"), "本文が平文で出ている");
        }
    }
    let manifest_bytes = std::fs::read(work.join(".open-backup/manifest.obe")).unwrap();
    assert!(!String::from_utf8_lossy(&manifest_bytes).contains("契約書"), "目録が平文");

    // 復元(暗号化を自動判定)
    assert!(restore::is_encrypted(&work));
    let (enc, items) = restore::list(&work, Some(pass)).unwrap();
    assert!(enc && items.len() == 5);
    assert!(restore::list(&work, Some("間違ったパスフレーズです")).is_err(), "パスフレーズ違いは拒否");
    let r = restore::run(&work, &dest, Some(pass), None, false).unwrap();
    assert_eq!((r.restored, r.failed.len()), (5, 0), "{:?}", r.failed);
    assert_eq!(tree(&dest.join("書類")), tree(&src), "復元内容が元と完全一致");
    // 更新時刻の復元
    let (a, b) = (std::fs::metadata(src.join("メモ.md")).unwrap().modified().unwrap(), std::fs::metadata(dest.join("書類/メモ.md")).unwrap().modified().unwrap());
    let d = if a > b { a.duration_since(b) } else { b.duration_since(a) }.unwrap().as_secs();
    assert!(d <= 1, "更新時刻が復元される");
    // 上書きしない(既定)
    let r2 = restore::run(&work, &dest, Some(pass), None, false).unwrap();
    assert_eq!((r2.restored, r2.skipped), (0, 5));
    // 一部だけ復元(先頭一致)
    let part = root.join("part");
    let r3 = restore::run(&work, &part, Some(pass), Some("書類/写真/"), false).unwrap();
    assert_eq!(r3.restored, 2);
    // 元の場所へ(上書きしない設定なので、既存はスキップされる)
    let r4 = restore::run_original(&work, &cfg, Some(pass), None, false).unwrap();
    assert_eq!((r4.restored, r4.skipped, r4.failed.len()), (0, 5, 0), "{:?}", r4.failed);

    // 変更が無ければ作業コピーは1バイトも変わらない(git に差分が出ない)
    let before = tree(&work);
    let mut rep2 = Report::default();
    crypt::sync_into(&cfg, &work, None, &mut rep2).unwrap();
    assert_eq!(rep2.copied, 0);
    assert_eq!(tree(&work), before, "変更無しなら暗号文も目録も同一");

    // 1つ変更・1つ削除・1つ追加 → 差分は最小
    write(&src.join("メモ.md"), "更新しました".as_bytes());
    std::fs::remove_file(src.join("写真").join("空.bin")).unwrap();
    write(&src.join("新規.txt"), b"new file");
    let mut rep3 = Report::default();
    crypt::sync_into(&cfg, &work, None, &mut rep3).unwrap();
    assert_eq!((rep3.copied, rep3.removed), (2, 1), "変更1+追加1=2、削除1");
    let after = tree(&work);
    let changed = after.iter().filter(|(k, v)| before.get(*k) != Some(*v)).count();
    assert!(changed <= 4, "変わったのは 変更1・追加1・目録・(目録の旧版) 程度: {changed}");

    // 改ざんの検出: 暗号文を1バイト書き換えると、その1件だけ失敗し、他は復元できる
    let victim = walkdir::WalkDir::new(work.join("data")).into_iter().filter_map(|e| e.ok()).find(|e| e.file_type().is_file()).unwrap();
    let mut bytes = std::fs::read(victim.path()).unwrap();
    let n = bytes.len() - 5;
    bytes[n] ^= 0x55;
    std::fs::write(victim.path(), bytes).unwrap();
    let bad = root.join("bad");
    let r5 = restore::run(&work, &bad, Some(pass), None, false).unwrap();
    assert_eq!(r5.failed.len(), 1, "改ざんした1件だけ失敗: {:?}", r5.failed);
    assert_eq!(r5.restored, 4, "残りの4件は復元できる");

    // 暗号化していない過去のバックアップが残るリポジトリには書かない
    let plain_work = root.join("plainrepo");
    write(&plain_work.join("書類").join("a.txt"), b"plain");
    let mut rep4 = Report::default();
    assert!(crypt::sync_into(&cfg, &plain_work, None, &mut rep4).is_err(), "平文リポジトリへの暗号化書き込みは拒否");

    // 平文のバックアップの復元(暗号化していない場合)
    let r6 = restore::run(&plain_work, &root.join("plain-out"), None, None, false).unwrap();
    assert_eq!(r6.restored, 1);
    assert_eq!(std::fs::read(root.join("plain-out/書類/a.txt")).unwrap(), b"plain");

    let _ = std::fs::remove_dir_all(&root);
}
