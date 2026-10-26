//! クライアント側の暗号化(送る前に端末で暗号化し、鍵は利用者だけが持つ)。
//!
//! 方式: パスフレーズ → Argon2id → 32バイトの主鍵 → BLAKE3 で用途別の副鍵。
//! ファイル本体は XChaCha20-Poly1305 の STREAM(1MiB ごとのチャンク、改ざん・切り詰めを検出)。
//! ファイル名・ディレクトリ構造・サイズ・分類(AI のトピック)は、暗号化した目録(manifest)にだけ入れ、
//! リポジトリ上のファイル名は鍵つきハッシュの不透明な名前にする。
//! 同じ内容・同じパスなら毎回同じ暗号文になる(決定的 nonce)ので、変更の無いファイルは git に差分が出ない。
//! GPU(open-cuda)は使わない: ChaCha20 は CPU(SIMD)で GB/s 級に速く、実際の律速はディスクと回線。
//! 内容の分類(aruaru-llm)は、暗号化の前に端末内で行い、結果は暗号化した目録にだけ書く。
use crate::ai;
use crate::config::{config_dir, data_dir, Config};
use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::XChaCha20Poly1305;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"OBE2";
const CHUNK: usize = 1024 * 1024;
const TAG: usize = 16;
pub const META_DIR: &str = ".open-backup";
pub const PARAMS_FILE: &str = "crypt.json";
pub const MANIFEST_FILE: &str = "manifest.obe";
pub const DATA_DIR: &str = "data";
const CHECK_CONTEXT: &[u8] = b"open-backup check v1";

fn b64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// リポジトリに置く公開パラメータ(秘密ではない)。パスフレーズが合っているかの確認値も入る。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub v: u32,
    pub kdf: String,
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
    pub salt: String,
    pub check: String,
}

pub struct Keys {
    data: Zeroizing<[u8; 32]>,
    name: Zeroizing<[u8; 32]>,
    nonce: Zeroizing<[u8; 32]>,
    manifest: Zeroizing<[u8; 32]>,
}

fn kdf(pass: &str, salt: &[u8], m_kib: u32, t: u32, p: u32) -> Result<Zeroizing<[u8; 32]>> {
    let params = argon2::Params::new(m_kib, t, p, Some(32)).map_err(|e| anyhow!("Argon2 パラメータ: {e}"))?;
    let a = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut out = Zeroizing::new([0u8; 32]);
    a.hash_password_into(pass.as_bytes(), salt, &mut *out).map_err(|e| anyhow!("鍵の導出に失敗: {e}"))?;
    Ok(out)
}

fn sub(master: &[u8; 32], ctx: &str) -> Zeroizing<[u8; 32]> {
    Zeroizing::new(blake3::derive_key(ctx, master))
}

fn check_value(master: &[u8; 32]) -> String {
    let k = sub(master, "open-backup v1 check");
    b64().encode(blake3::keyed_hash(&k, CHECK_CONTEXT).as_bytes())
}

impl Params {
    /// 新しい salt で作る(Argon2id: 64MiB / 3 回 / 並列 1)。
    pub fn create(pass: &str) -> Result<Params> {
        let mut salt = [0u8; 16];
        getrandom::getrandom(&mut salt).map_err(|e| anyhow!("乱数の取得に失敗: {e}"))?;
        let (m, t, p) = (65536, 3, 1);
        let master = kdf(pass, &salt, m, t, p)?;
        Ok(Params { v: 1, kdf: "argon2id".into(), m_kib: m, t, p, salt: b64().encode(salt), check: check_value(&master) })
    }
}

impl Keys {
    /// パスフレーズから鍵を作る。合っていなければエラー。
    pub fn derive(pass: &str, par: &Params) -> Result<Keys> {
        if par.v != 1 || par.kdf != "argon2id" {
            bail!("未対応の暗号化バージョンです(v={}, kdf={})", par.v, par.kdf);
        }
        let salt = b64().decode(&par.salt).context("salt が壊れています")?;
        let master = kdf(pass, &salt, par.m_kib, par.t, par.p)?;
        let expect = check_value(&master);
        // 定数時間比較
        let ok = expect.len() == par.check.len()
            && expect.bytes().zip(par.check.bytes()).fold(0u8, |a, (x, y)| a | (x ^ y)) == 0;
        if !ok {
            bail!("パスフレーズが違います(または暗号化の設定が別のものです)");
        }
        Ok(Keys {
            data: sub(&master, "open-backup v1 data"),
            name: sub(&master, "open-backup v1 name"),
            nonce: sub(&master, "open-backup v1 nonce"),
            manifest: sub(&master, "open-backup v1 manifest"),
        })
    }

    fn blob_name(&self, rel_key: &str) -> String {
        let h = blake3::keyed_hash(&self.name, rel_key.as_bytes()).to_hex().to_string();
        format!("{DATA_DIR}/{}/{}.obe", &h[..2], &h[2..34])
    }

    fn nonce19(&self, material: &[u8]) -> [u8; 19] {
        let h = blake3::keyed_hash(&self.nonce, material);
        let mut n = [0u8; 19];
        n.copy_from_slice(&h.as_bytes()[..19]);
        n
    }
}

fn read_full<R: Read>(r: &mut R, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut n = 0;
    while n < buf.len() {
        let m = r.read(&mut buf[n..])?;
        if m == 0 {
            break;
        }
        n += m;
    }
    Ok(n)
}

/// チャンクの nonce(24バイト) = ファイルごとの nonce(19) | チャンク番号(u32 BE) | 最後のチャンクなら 1。
/// (STREAM 方式と同じ構成。チャンクごとに独立しているので並列に処理でき、切り詰めは「最後」の印で検出できる)
fn chunk_nonce(nonce19: &[u8; 19], counter: u32, last: bool) -> [u8; 24] {
    let mut n = [0u8; 24];
    n[..19].copy_from_slice(nonce19);
    n[19..23].copy_from_slice(&counter.to_be_bytes());
    n[23] = last as u8;
    n
}

/// 一度に並列で処理するチャンク数(メモリは 約 BATCH × 1MiB × 並列ファイル数)
const BATCH: usize = 8;
const MODE_RAW: u8 = 0;
const MODE_ZSTD: u8 = 1;

fn read_chunk<R: Read>(r: &mut R) -> Result<Vec<u8>> {
    let mut v = vec![0u8; CHUNK];
    let n = read_full(r, &mut v)?;
    v.truncate(n);
    Ok(v)
}

/// 平文を 1MiB ごとに(圧縮→暗号化して)書く。`level` が 0 なら圧縮しない。形式:
/// "OBE2" | nonce(19) | { 長さ(u32 LE) | 暗号文 }... 。チャンクの中身は [モード(0=そのまま/1=zstd)] + データ。
/// チャンクはスレッドで並列に処理する(rayon)。
pub fn encrypt_stream<R: Read, W: Write>(key: &[u8; 32], nonce: &[u8; 19], aad: &[u8], level: i32, mut r: R, mut w: W) -> Result<()> {
    use rayon::prelude::*;
    w.write_all(MAGIC)?;
    w.write_all(nonce)?;
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut counter: u32 = 0;
    let mut carry = read_chunk(&mut r)?;
    let mut first = true;
    loop {
        // 今回のバッチ(最大 BATCH 個)と、その次の1個(あれば最後ではない)
        let mut batch: Vec<Vec<u8>> = vec![];
        let mut ended = false;
        loop {
            let cur = std::mem::take(&mut carry);
            if cur.is_empty() && !first {
                ended = true;
                break;
            }
            first = false;
            let next = if cur.len() == CHUNK { read_chunk(&mut r)? } else { vec![] };
            let is_last = next.is_empty();
            batch.push(cur);
            carry = next;
            if is_last {
                ended = true;
                break;
            }
            if batch.len() == BATCH {
                break;
            }
        }
        let base = counter;
        let n = batch.len();
        let outs: Vec<Result<Vec<u8>>> = batch
            .par_iter()
            .enumerate()
            .map(|(i, c)| {
                let last = ended && i + 1 == n;
                let mut frame = Vec::with_capacity(c.len() + 1);
                let z = if level > 0 && !c.is_empty() { zstd::bulk::compress(c, level).ok().filter(|z| z.len() < c.len()) } else { None };
                match z {
                    Some(z) => {
                        frame.push(MODE_ZSTD);
                        frame.extend_from_slice(&z);
                    }
                    None => {
                        frame.push(MODE_RAW);
                        frame.extend_from_slice(c);
                    }
                }
                let nn = chunk_nonce(nonce, base.wrapping_add(i as u32), last);
                cipher.encrypt((&nn).into(), Payload { msg: &frame, aad }).map_err(|_| anyhow!("暗号化に失敗"))
            })
            .collect();
        for o in outs {
            let ct = o?;
            w.write_all(&(ct.len() as u32).to_le_bytes())?;
            w.write_all(&ct)?;
        }
        counter = counter.wrapping_add(n as u32);
        if ended {
            break;
        }
    }
    Ok(())
}

fn read_frame<R: Read>(r: &mut R) -> Result<Option<Vec<u8>>> {
    let mut l = [0u8; 4];
    let n = read_full(r, &mut l)?;
    if n == 0 {
        return Ok(None);
    }
    if n < 4 {
        bail!("暗号化ファイルが途中で切れています");
    }
    let len = u32::from_le_bytes(l) as usize;
    if len < TAG + 1 || len > 2 * CHUNK + 64 {
        bail!("暗号化ファイルのチャンク長が不正です");
    }
    let mut v = vec![0u8; len];
    r.read_exact(&mut v).context("暗号化ファイルが途中で切れています")?;
    Ok(Some(v))
}

/// `encrypt_stream` の逆(並列)。改ざん・切り詰め・鍵違い・AAD違いはエラーになる。
pub fn decrypt_stream<R: Read, W: Write>(key: &[u8; 32], aad: &[u8], mut r: R, mut w: W) -> Result<()> {
    use rayon::prelude::*;
    let mut head = [0u8; 4 + 19];
    r.read_exact(&mut head).context("暗号化ファイルの先頭が読めません")?;
    if &head[..4] != MAGIC {
        bail!("暗号化ファイルではありません(形式が違います)");
    }
    let mut nonce = [0u8; 19];
    nonce.copy_from_slice(&head[4..]);
    let cipher = XChaCha20Poly1305::new(key.into());
    let mut counter: u32 = 0;
    let mut carry = read_frame(&mut r)?.context("暗号化ファイルが空です")?;
    loop {
        let mut batch: Vec<Vec<u8>> = vec![];
        let mut ended = false;
        loop {
            let cur = std::mem::take(&mut carry);
            let next = read_frame(&mut r)?;
            batch.push(cur);
            match next {
                None => {
                    ended = true;
                    break;
                }
                Some(nx) => carry = nx,
            }
            if batch.len() == BATCH {
                break;
            }
        }
        let base = counter;
        let n = batch.len();
        let outs: Vec<Result<Vec<u8>>> = batch
            .par_iter()
            .enumerate()
            .map(|(i, ct)| {
                let last = ended && i + 1 == n;
                let nn = chunk_nonce(&nonce, base.wrapping_add(i as u32), last);
                let frame = cipher.decrypt((&nn).into(), Payload { msg: ct, aad }).map_err(|_| anyhow!("復号に失敗(鍵違い・改ざん・切り詰め・入れ替え)"))?;
                match frame.first() {
                    Some(&MODE_RAW) => Ok(frame[1..].to_vec()),
                    Some(&MODE_ZSTD) => {
                        let v = zstd::bulk::decompress(&frame[1..], CHUNK).map_err(|_| anyhow!("展開に失敗"))?;
                        Ok(v)
                    }
                    _ => bail!("不明なチャンク形式"),
                }
            })
            .collect();
        for o in outs {
            let v = o?;
            w.write_all(&v)?;
        }
        counter = counter.wrapping_add(n as u32);
        if ended {
            break;
        }
    }
    Ok(())
}

// ---- パスフレーズの保管 ----

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
const SERVICE: &str = "open-backup";

fn pass_file() -> Result<PathBuf> {
    Ok(data_dir()?.join("passphrase"))
}

/// 取得順: 環境変数 OPEN_BACKUP_PASSPHRASE → OS の資格情報ストア → アプリ専用領域のファイル(自分だけが読める)
pub fn passphrase() -> Result<Zeroizing<String>> {
    if let Ok(p) = std::env::var("OPEN_BACKUP_PASSPHRASE") {
        if !p.is_empty() {
            return Ok(Zeroizing::new(p));
        }
    }
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    if let Ok(p) = keyring::Entry::new(SERVICE, "passphrase").and_then(|e| e.get_password()) {
        return Ok(Zeroizing::new(p));
    }
    let p = std::fs::read_to_string(pass_file()?).map(|s| s.trim_end_matches(['\r', '\n']).to_string()).unwrap_or_default();
    if p.is_empty() {
        bail!("暗号化のパスフレーズが未設定です。`open-backup crypt-init` を実行してください");
    }
    Ok(Zeroizing::new(p))
}

pub fn store_passphrase(p: &str) -> Result<()> {
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    if keyring::Entry::new(SERVICE, "passphrase").and_then(|e| e.set_password(p)).is_ok()
        && keyring::Entry::new(SERVICE, "passphrase").and_then(|e| e.get_password()).ok().as_deref() == Some(p)
    {
        let _ = std::fs::remove_file(pass_file()?);
        return Ok(());
    }
    let f = pass_file()?;
    std::fs::create_dir_all(f.parent().unwrap())?;
    std::fs::write(&f, p)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn local_params_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(PARAMS_FILE))
}

/// 端末に保存した公開パラメータ。無ければ None。
pub fn load_local_params() -> Option<Params> {
    let s = std::fs::read_to_string(local_params_path().ok()?).ok()?;
    serde_json::from_str(&s).ok()
}

/// 初期化: 新しい salt を作って端末に保存し、パスフレーズを資格情報ストアへ保存する。
pub fn init(pass: &str) -> Result<Params> {
    if pass.chars().count() < 12 {
        bail!("パスフレーズは12文字以上にしてください(長い文章が安全です)");
    }
    let par = Params::create(pass)?;
    let p = local_params_path()?;
    std::fs::create_dir_all(p.parent().unwrap())?;
    std::fs::write(&p, serde_json::to_string_pretty(&par)?)?;
    store_passphrase(pass)?;
    Ok(par)
}

// ---- 目録(manifest) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// `<ラベル>/<元の相対パス>`('/' 区切り)
    pub rel: String,
    pub size: u64,
    pub mtime: u64,
    pub topic: Option<String>,
    /// リポジトリ内の暗号化ファイルの場所
    pub blob: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheEnt {
    size: u64,
    mtime: u64,
    blob: String,
}

fn cache_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("crypt-cache.json"))
}
fn load_cache() -> HashMap<String, CacheEnt> {
    cache_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
fn save_cache(c: &HashMap<String, CacheEnt>) -> Result<()> {
    let p = cache_path()?;
    std::fs::create_dir_all(p.parent().unwrap())?;
    std::fs::write(p, serde_json::to_string(c)?)?;
    Ok(())
}

fn mtime_secs(md: &std::fs::Metadata) -> u64 {
    md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0)
}

/// 圧縮しても小さくならない拡張子(写真・動画・音声・圧縮ファイルなど)
fn already_compressed(p: &Path) -> bool {
    const EXT: &[&str] = &[
        "jpg", "jpeg", "png", "gif", "webp", "heic", "avif", "mp3", "aac", "m4a", "ogg", "opus", "flac", "mp4", "mkv", "mov", "avi", "webm", "m4v",
        "zip", "7z", "rar", "gz", "bz2", "xz", "zst", "lz4", "cab", "docx", "xlsx", "pptx", "jar", "apk", "pdf",
    ];
    p.extension().and_then(|e| e.to_str()).map(|e| EXT.iter().any(|x| x.eq_ignore_ascii_case(e))).unwrap_or(false)
}

const NOTICE: &str = "# 暗号化されたバックアップ / Encrypted backup\n\n\
このリポジトリの中身は open-backup で**暗号化**されています(XChaCha20-Poly1305、鍵は Argon2id でパスフレーズから導出)。\n\
パスフレーズが無ければ、作者でも GitHub でも読めません。**パスフレーズを失うと復元できません。**\n\
復元: `open-backup restore --from <このリポジトリの作業コピー> --to <復元先>`\n\n\
The contents of this repository are **encrypted** by open-backup (XChaCha20-Poly1305; key derived from your passphrase with Argon2id).\n\
Without the passphrase nobody can read them. **If you lose the passphrase, the data cannot be recovered.**\n\
Restore: `open-backup restore --from <working copy of this repo> --to <destination>`\n";

/// 作業コピー(git の作業ディレクトリ)に、暗号化したバックアップを書く。平文は一切置かない。
pub fn sync_into(cfg: &Config, work: &Path, eng: Option<&ai::Engine>, rep: &mut crate::sync::Report) -> Result<()> {
    let par = load_local_params().context("暗号化が未初期化です。`open-backup crypt-init` を実行してください")?;
    let pass = passphrase()?;
    let keys = Keys::derive(&pass, &par)?;
    let existing_params = work.join(META_DIR).join(PARAMS_FILE);
    if existing_params.exists() {
        let s = std::fs::read_to_string(&existing_params)?;
        let theirs: Params = serde_json::from_str(&s).context("リポジトリの crypt.json が壊れています")?;
        if theirs.salt != par.salt {
            bail!("このリポジトリは別の鍵(salt)で暗号化されています。同じ端末設定を使うか、別のリポジトリ名にしてください");
        }
    } else if work.exists() {
        // 暗号化していない過去のバックアップが残っているリポジトリへは書かない(履歴に平文が残るため)
        let plain = std::fs::read_dir(work)?.filter_map(|e| e.ok()).any(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n != ".git" && n != META_DIR && n != "README-ENCRYPTED.md" && n != DATA_DIR
        });
        if plain {
            bail!("このリポジトリには暗号化されていないバックアップが既にあります(git の履歴に平文が残ります)。config.toml の repo を新しい名前にして、暗号化専用のリポジトリを作ってください");
        }
    }

    let max = cfg.max_file_mb * 1024 * 1024;
    let excluded = |name: &str| cfg.exclude.iter().any(|e| e == name);
    let mut cache = load_cache();
    let mut entries: Vec<Entry> = vec![];
    let mut srcs: Vec<(String, PathBuf, u64, u64)> = vec![];
    let mut used = HashSet::new();
    for f in &cfg.folders {
        if !f.is_dir() {
            eprintln!("スキップ(存在しない): {}", f.display());
            continue;
        }
        let l = crate::sync::label(f, &mut used);
        for e in WalkDir::new(f)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.depth() == 0 || !excluded(&e.file_name().to_string_lossy()))
        {
            let Ok(e) = e else { continue };
            if !e.file_type().is_file() {
                continue;
            }
            let Ok(md) = e.metadata() else { continue };
            if md.len() > max {
                rep.skipped_large.push(e.path().to_path_buf());
                continue;
            }
            let rel = e.path().strip_prefix(f)?.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            srcs.push((format!("{l}/{rel}"), e.path().to_path_buf(), md.len(), mtime_secs(&md)));
        }
    }

    // 内容の分類は、暗号化の前に端末内で行う(結果は暗号化した目録にだけ入る)
    let mut topics: HashMap<String, String> = HashMap::new();
    if let Some(eng) = eng {
        let mut aic = ai::load_cache();
        let files: Vec<(String, PathBuf, String)> =
            srcs.iter().map(|(k, p, len, mt)| (format!("{k}|{len}|{mt}"), p.clone(), k.clone())).collect();
        ai::fill(eng, &files, &mut aic, 500);
        ai::save_cache(&aic);
        for (key, _, rel) in files {
            if let Some(t) = aic.get(&key) {
                topics.insert(rel, t.clone());
            }
        }
    }

    // ファイル単位でも並列に処理する(各ファイルの中のチャンクも並列)。常駐の邪魔をしないよう、スレッド数は設定で絞る。
    use rayon::prelude::*;
    let threads = if cfg.threads == 0 { (std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2) / 2).max(1) } else { cfg.threads };
    let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().context("スレッドプールを作れません")?;
    let level = cfg.compress_level;
    struct Done {
        entry: Option<Entry>,
        copied: bool,
    }
    let results: Vec<Done> = pool.install(|| {
        srcs.par_iter()
            .map(|(rel_key, path, size, mtime)| {
                let blob = keys.blob_name(rel_key);
                let out = work.join(&blob);
                let topic = topics.get(rel_key).cloned();
                let mk = |blob: String| Entry { rel: rel_key.clone(), size: *size, mtime: *mtime, topic: topic.clone(), blob };
                let reuse = cache.get(rel_key).map(|c| c.size == *size && c.mtime == *mtime && c.blob == blob).unwrap_or(false) && out.exists();
                if reuse {
                    return Done { entry: Some(mk(blob)), copied: false };
                }
                // 使用中などで読めないファイルは次回に回す(既存の暗号文があれば残す)
                let Ok(mut f) = std::fs::File::open(path) else {
                    return Done { entry: out.exists().then(|| mk(blob)), copied: false };
                };
                let mut h = blake3::Hasher::new();
                let mut buf = vec![0u8; 4 * 1024 * 1024];
                loop {
                    let n = match read_full(&mut f, &mut buf) {
                        Ok(n) => n,
                        Err(_) => return Done { entry: out.exists().then(|| mk(blob)), copied: false },
                    };
                    if n == 0 {
                        break;
                    }
                    h.update_rayon(&buf[..n]);
                }
                let mut material = rel_key.as_bytes().to_vec();
                material.push(0);
                material.extend_from_slice(h.finalize().as_bytes());
                let nonce = keys.nonce19(&material);
                if let Some(p) = out.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                let tmp = out.with_extension("tmp");
                // 圧縮しても小さくならない形式(写真・動画・圧縮ファイル)は圧縮を省く
                let lv = if already_compressed(path) { 0 } else { level };
                let res = (|| -> Result<()> {
                    let src = std::fs::File::open(path)?;
                    let dst = std::fs::File::create(&tmp)?;
                    encrypt_stream(&keys.data, &nonce, rel_key.as_bytes(), lv, std::io::BufReader::new(src), std::io::BufWriter::new(dst))
                })();
                if res.is_err() {
                    let _ = std::fs::remove_file(&tmp);
                    return Done { entry: out.exists().then(|| mk(blob)), copied: false };
                }
                // 内容が同じなら暗号文も同じ(= git に差分が出ない)
                let same = std::fs::read(&out).ok().map(|a| std::fs::read(&tmp).ok().map(|b| a == b).unwrap_or(false)).unwrap_or(false);
                let copied = if same {
                    let _ = std::fs::remove_file(&tmp);
                    false
                } else {
                    std::fs::rename(&tmp, &out).is_ok()
                };
                Done { entry: Some(mk(blob)), copied }
            })
            .collect()
    });
    let mut keep: HashSet<String> = HashSet::new();
    let mut new_cache: HashMap<String, CacheEnt> = HashMap::new();
    for d in results {
        if d.copied {
            rep.copied += 1;
        }
        if let Some(e) = d.entry {
            keep.insert(e.blob.clone());
            new_cache.insert(e.rel.clone(), CacheEnt { size: e.size, mtime: e.mtime, blob: e.blob.clone() });
            entries.push(e);
        }
    }
    cache = new_cache;
    save_cache(&cache)?;

    // 元で消えたものを消す(git の履歴には残る)
    let data = work.join(DATA_DIR);
    if data.exists() {
        for e in WalkDir::new(&data).contents_first(true).into_iter().filter_map(|e| e.ok()) {
            let rel = e.path().strip_prefix(work)?.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            if e.file_type().is_file() && !keep.contains(&rel) {
                std::fs::remove_file(e.path())?;
                rep.removed += 1;
            } else if e.file_type().is_dir() && e.depth() > 0 {
                let _ = std::fs::remove_dir(e.path());
            }
        }
    }

    // 目録(暗号化)と公開パラメータ、案内文
    entries.sort_by(|a, b| a.rel.cmp(&b.rel));
    let mut jsonl = String::new();
    for e in &entries {
        jsonl.push_str(&serde_json::to_string(e)?);
        jsonl.push('\n');
    }
    let nonce = keys.nonce19(&[b"manifest\0".as_slice(), jsonl.as_bytes()].concat());
    let meta = work.join(META_DIR);
    std::fs::create_dir_all(&meta)?;
    let mpath = meta.join(MANIFEST_FILE);
    let mut buf = Vec::new();
    encrypt_stream(&keys.manifest, &nonce, b"manifest", 3, jsonl.as_bytes(), &mut buf)?;
    if std::fs::read(&mpath).ok().as_deref() != Some(&buf[..]) {
        std::fs::write(&mpath, &buf)?;
    }
    let ptext = serde_json::to_string_pretty(&par)?;
    if std::fs::read_to_string(meta.join(PARAMS_FILE)).ok().as_deref() != Some(&ptext) {
        std::fs::write(meta.join(PARAMS_FILE), ptext)?;
    }
    if std::fs::read_to_string(work.join("README-ENCRYPTED.md")).ok().as_deref() != Some(NOTICE) {
        std::fs::write(work.join("README-ENCRYPTED.md"), NOTICE)?;
    }
    Ok(())
}

#[derive(Debug, Default)]
pub struct RestoreReport {
    pub restored: usize,
    pub skipped: usize,
    pub bytes: u64,
    pub failed: Vec<String>,
}

fn safe_rel(rel: &str) -> Option<PathBuf> {
    let p = Path::new(rel);
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(s) => out.push(s),
            _ => return None,
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

/// 目録を読む(パスフレーズが違えばエラー)。
pub fn read_manifest(work: &Path, pass: &str) -> Result<(Keys, Vec<Entry>)> {
    let ptxt = std::fs::read_to_string(work.join(META_DIR).join(PARAMS_FILE)).context("暗号化の設定(.open-backup/crypt.json)がありません。暗号化されたバックアップですか?")?;
    let par: Params = serde_json::from_str(&ptxt)?;
    let keys = Keys::derive(pass, &par)?;
    let f = std::fs::File::open(work.join(META_DIR).join(MANIFEST_FILE)).context("目録(manifest.obe)がありません")?;
    let mut pt = Vec::new();
    decrypt_stream(&keys.manifest, b"manifest", std::io::BufReader::new(f), &mut pt)?;
    let mut v = vec![];
    for line in String::from_utf8(pt)?.lines().filter(|l| !l.trim().is_empty()) {
        v.push(serde_json::from_str::<Entry>(line)?);
    }
    Ok((keys, v))
}

/// 作業コピーから、元のフォルダ構造のまま `dest` へ復元する。`only` は、`<ラベル>/<パス>` の先頭一致で絞り込む。
pub fn restore(work: &Path, dest: &Path, pass: &str, only: Option<&str>, overwrite: bool) -> Result<RestoreReport> {
    let (keys, entries) = read_manifest(work, pass)?;
    let mut rep = RestoreReport::default();
    for e in entries.iter().filter(|e| only.map(|o| e.rel.starts_with(o)).unwrap_or(true)) {
        let Some(rel) = safe_rel(&e.rel) else {
            rep.failed.push(format!("{}: 危険なパスのため復元しません", e.rel));
            continue;
        };
        // 目録の blob も検証(リポジトリ内の data/ 以外は読まない)
        if !e.blob.starts_with("data/") || e.blob.contains("..") {
            rep.failed.push(format!("{}: 不正な保存場所", e.rel));
            continue;
        }
        let to = dest.join(&rel);
        // 既にあるファイルは、指定が無い限り上書きしない(同じサイズ・同じ更新時刻なら「済み」とみなす)
        if !overwrite && to.exists() {
            rep.skipped += 1;
            continue;
        }
        if let Some(p) = to.parent() {
            std::fs::create_dir_all(p)?;
        }
        let res = (|| -> Result<u64> {
            let src = std::fs::File::open(work.join(&e.blob)).with_context(|| format!("暗号化ファイルがありません: {}", e.blob))?;
            let tmp = to.with_extension("obrestore.tmp");
            let n = {
                let mut w = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
                decrypt_stream(&keys.data, e.rel.as_bytes(), std::io::BufReader::new(src), &mut w)?;
                w.flush()?;
                std::fs::metadata(&tmp)?.len()
            };
            if n != e.size {
                let _ = std::fs::remove_file(&tmp);
                bail!("サイズが目録と違います({n} != {})", e.size);
            }
            std::fs::rename(&tmp, &to)?;
            let t = std::time::UNIX_EPOCH + std::time::Duration::from_secs(e.mtime);
            let _ = std::fs::File::options().write(true).open(&to).and_then(|f| f.set_modified(t));
            Ok(n)
        })();
        match res {
            Ok(n) => {
                rep.restored += 1;
                rep.bytes += n;
            }
            Err(err) => rep.failed.push(format!("{}: {err:#}", e.rel)),
        }
    }
    Ok(rep)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys_for(pass: &str) -> (Keys, Params) {
        // テストでは Argon2 を軽くする
        let salt = [7u8; 16];
        let master = kdf(pass, &salt, 256, 1, 1).unwrap();
        let par = Params { v: 1, kdf: "argon2id".into(), m_kib: 256, t: 1, p: 1, salt: b64().encode(salt), check: check_value(&master) };
        (Keys::derive(pass, &par).unwrap(), par)
    }

    fn roundtrip(data: &[u8]) {
        let (k, _) = keys_for("correct horse battery staple");
        let n = k.nonce19(b"x");
        let mut ct = vec![];
        encrypt_stream(&k.data, &n, b"aad", 3, data, &mut ct).unwrap();
        let mut pt = vec![];
        decrypt_stream(&k.data, b"aad", &ct[..], &mut pt).unwrap();
        assert_eq!(pt, data);
    }

    #[test]
    fn roundtrip_sizes() {
        roundtrip(b"");
        roundtrip(b"a");
        roundtrip(&vec![9u8; CHUNK - 1]);
        roundtrip(&vec![9u8; CHUNK]);
        roundtrip(&vec![9u8; CHUNK + 1]);
        let big: Vec<u8> = (0..(CHUNK * 3 + 12345)).map(|i| (i % 251) as u8).collect();
        roundtrip(&big);
    }

    #[test]
    fn roundtrip_batch_boundaries() {
        for chunks in [BATCH - 1, BATCH, BATCH + 1, BATCH * 2, BATCH * 2 + 1, BATCH * 3] {
            let d: Vec<u8> = (0..CHUNK * chunks).map(|i| (i.wrapping_mul(31) % 253) as u8).collect();
            roundtrip(&d); // ちょうどチャンクの整数倍(最後のチャンクが満杯)
            roundtrip(&d[..d.len() - 1]); // 1バイト少ない
        }
    }

    #[test]
    #[ignore] // 手動: cargo test -p open-backup-core --release bench -- --ignored --nocapture
    fn bench_throughput() {
        let (k, _) = keys_for("correct horse battery staple");
        let n = k.nonce19(b"b");
        let mut seed = 12345u64;
        let rnd: Vec<u8> = (0..256 * 1024 * 1024).map(|_| { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; (seed & 0xff) as u8 }).collect();
        let text: Vec<u8> = "open-backup のベンチマーク用の文章です。".repeat(8_000_000).into_bytes();
        for threads in [1usize, 2, 4, 8, 16] {
            let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
            for (name, data, lv) in [("乱数(圧縮なし)", &rnd, 0), ("乱数(圧縮試行)", &rnd, 3), ("文章(zstd 3)", &text, 3)] {
                let t = std::time::Instant::now();
                let mut ct = Vec::new();
                pool.install(|| encrypt_stream(&k.data, &n, b"b", lv, &data[..], &mut ct).unwrap());
                let enc = t.elapsed().as_secs_f64();
                let t = std::time::Instant::now();
                let mut pt = Vec::new();
                pool.install(|| decrypt_stream(&k.data, b"b", &ct[..], &mut pt).unwrap());
                let dec = t.elapsed().as_secs_f64();
                assert_eq!(pt.len(), data.len());
                let mb = data.len() as f64 / 1048576.0;
                println!("threads={threads:>2} {name:<14} 暗号化 {:>7.0} MB/s  復号 {:>7.0} MB/s  (出力 {:.0}%)", mb / enc, mb / dec, ct.len() as f64 * 100.0 / data.len() as f64);
            }
        }
    }

    #[test]
    fn deterministic_and_tamper_detected() {
        let (k, _) = keys_for("correct horse battery staple");
        let n = k.nonce19(b"same");
        let data = vec![5u8; CHUNK + 10];
        let (mut a, mut b) = (vec![], vec![]);
        encrypt_stream(&k.data, &n, b"p", 3, &data[..], &mut a).unwrap();
        encrypt_stream(&k.data, &n, b"p", 3, &data[..], &mut b).unwrap();
        assert_eq!(a, b, "同じ入力は同じ暗号文(git に差分が出ない)");
        // 1バイト改ざん
        let mut t = a.clone();
        let i = t.len() / 2;
        t[i] ^= 1;
        assert!(decrypt_stream(&k.data, b"p", &t[..], &mut Vec::new()).is_err());
        // 末尾チャンクの切り詰め(最初のチャンクの終わりで切る)
        let first_len = u32::from_le_bytes(a[23..27].try_into().unwrap()) as usize;
        let cut = 23 + 4 + first_len;
        assert!(cut < a.len());
        assert!(decrypt_stream(&k.data, b"p", &a[..cut], &mut Vec::new()).is_err(), "切り詰めを検出");
        // AAD(パス)が違うと復号できない = ファイルの入れ替えを検出
        assert!(decrypt_stream(&k.data, b"other", &a[..], &mut Vec::new()).is_err());
    }

    #[test]
    fn compression_shrinks_text_and_roundtrips() {
        let (k, _) = keys_for("correct horse battery staple");
        let n = k.nonce19(b"c");
        let text: Vec<u8> = "open-backup は日本語の文書をよく圧縮できます。".repeat(200_000).into_bytes();
        let (mut raw, mut z) = (vec![], vec![]);
        encrypt_stream(&k.data, &n, b"c", 0, &text[..], &mut raw).unwrap();
        encrypt_stream(&k.data, &n, b"c", 3, &text[..], &mut z).unwrap();
        assert!(z.len() * 10 < raw.len(), "圧縮で10分の1未満になる: {} vs {}", z.len(), raw.len());
        let mut back = vec![];
        decrypt_stream(&k.data, b"c", &z[..], &mut back).unwrap();
        assert_eq!(back, text);
        // 圧縮しない設定の暗号文も同じ復号関数で戻る
        let mut back2 = vec![];
        decrypt_stream(&k.data, b"c", &raw[..], &mut back2).unwrap();
        assert_eq!(back2, text);
    }

    #[test]
    fn wrong_passphrase_rejected() {
        let (_, par) = keys_for("correct horse battery staple");
        assert!(Keys::derive("wrong passphrase value", &par).is_err());
        assert!(Keys::derive("correct horse battery staple", &par).is_ok());
    }

    #[test]
    fn safe_rel_rejects_traversal() {
        assert!(safe_rel("a/b.txt").is_some());
        assert!(safe_rel("../x").is_none());
        assert!(safe_rel("/etc/passwd").is_none());
        assert!(safe_rel("C:\\x").is_none() || cfg!(not(windows)));
    }

    #[test]
    fn names_are_opaque() {
        let (k, _) = keys_for("correct horse battery staple");
        let n = k.blob_name("docs/契約書.pdf");
        assert!(n.starts_with("data/") && n.ends_with(".obe") && !n.contains("契約"));
        assert_ne!(n, k.blob_name("docs/契約書2.pdf"));
    }
}
