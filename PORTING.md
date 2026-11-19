# open-backup PORTING / 引っ越し・再開メモ (Moving and resume notes)

**日本語**: スマホやパソコンのフォルダを、利用者自身の GitHub **非公開**リポジトリへ自動でバックアップします。Rust のコア+常駐エンジン+ RPoem で作る画面+ Android アプリ。
**English**: It automatically backs up folders on a device to the user's own **private** GitHub repository. A Rust core and resident engine, a screen made with RPoem, and an Android app.

## いまの状態(2026-11-19) / Where things stand now

**日本語**: v0.2.0 を公開しました(暗号化・復元・Windows インストーラー・Linux・Android APK)。実機(A401OP / Android 15)にも入れて、起動・画面・復元カード・版表記 0.2.0 を確かめました。**ログインより先は、トークンの入力が要るので、利用者が実機で試します。**
**English**: v0.2.0 is published (encryption, restore, Windows installer, Linux, Android APK). It was also installed on a real device (A401OP / Android 15), where start-up, the screen, the restore card and the version label 0.2.0 were checked. **Login and later steps need a token, so the owner tries them on the real device.**

## 引っ越しの手順 / Moving to another computer

**日本語**: このリポジトリだけで開発を続けられます。画面を作り直すときだけ、となりのフォルダの `RPoem` が要ります(作った結果は `ui/` に入っているので、ふつうのビルドには要りません)。
**English**: This repository is enough to keep developing. Only regenerating the screen needs the sibling `RPoem` folder (the result is already in `ui/`, so a normal build does not need it).

### 1. 必要なもの / What you need

| 用途 / For | 内容 / What |
|---|---|
| Rust | stable(`cargo`)。Windows は `%USERPROFILE%\.cargo\bin` を PATH の先頭に。 / Stable Rust (`cargo`). On Windows put `~/.cargo/bin` first on PATH. |
| 保存の方法 `git` / the `git` way | `git` コマンド。なければ、自動で GitHub API の方法になります。 / The `git` command. Without it, the app switches to the GitHub API way. |
| Linux で作る / Building on Linux | `gcc`・`pkg-config`・`libdbus-1-dev`(資格情報ストア `keyring` 用)。Windows からは WSL(Ubuntu)で。 / `gcc`, `pkg-config`, `libdbus-1-dev` (for the credential store `keyring`). From Windows, use WSL (Ubuntu). |
| macOS | Mac か、GitHub の macOS ランナー。Windows からは、zig を C コンパイラにした `cargo check --target aarch64-apple-darwin` まで(リンクには Apple の SDK が要り、ライセンスの都合で Apple 製の機械で行います)。 / A Mac or a GitHub macOS runner. From Windows you can only go as far as `cargo check --target aarch64-apple-darwin` with zig as the C compiler (linking needs Apple's SDK, which is licensed for Apple hardware). |
| Android | Android SDK(platform 35・build-tools)・NDK 27・JDK 17(Android Studio の `jbr` でよい)・Gradle 8.7 以上・`cargo-ndk`・`rustup target add aarch64-linux-android x86_64-linux-android`。 / Android SDK (platform 35 and build-tools), NDK 27, JDK 17 (the `jbr` of Android Studio is fine), Gradle 8.7 or newer, `cargo-ndk`, and `rustup target add aarch64-linux-android x86_64-linux-android`. |
| Windows インストーラー / Windows installer | Inno Setup 6。 / Inno Setup 6. |
| 画面の作り直し(なくてもよい) / Regenerating the screen (optional) | となりに `RPoem`(`../RPoem/crates/open-runo-crossui`)。 / The sibling `RPoem` repository. |

### 2. 取得・ビルド・テスト / Get, build, test

```bash
git clone https://github.com/aon-co-jp/open-backup.git && cd open-backup
cargo test                       # 小さなテスト / unit tests
cargo build --release            # open-backup(コマンド+常駐エンジン) / the command and the engine
target/release/open-backup serve # 画面 / the screen: http://127.0.0.1:4620/ui
```

- **画面を変えたとき / When you change the screen**: `tools/gen-ui` で `cargo run`(`ui/` を作り直す)→ エンジンを作り直す(`ui/index.html` は、エンジンに `include_str!` で入っています)。 / Run `cargo run` in `tools/gen-ui` (it regenerates `ui/`), then rebuild the engine (`ui/index.html` is built into the engine with `include_str!`).
- **Android**: `powershell -ExecutionPolicy Bypass -File android\build-android.ps1 -Gradle <gradle.bat のパス / path>`(エンジン arm64 と x86_64 → `jniLibs` → APK)。**画面やコアを変えたら、必ずこの手順で作り直す**(Gradle だけでは、アプリの中のエンジンが古いままです)。版を上げるときは `android/app/build.gradle` の `versionName` と `versionCode` も。 / (builds the engines for arm64 and x86_64, copies them into `jniLibs`, then builds the APK). **After changing the screen or the core, always rebuild this way** (with Gradle alone the engine inside the app stays old). When you raise the version, also raise `versionName` and `versionCode` in `android/app/build.gradle`.
- **Windows インストーラー / Windows installer**: `powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1 -Version 0.2.0`(先に `cargo build --release`)。 / (run `cargo build --release` first).

### 3. 設定・データ・トークンの置き場所 / Where settings, data and the token live

| もの / What | 場所 / Where |
|---|---|
| 設定 / Settings | Windows は `%APPDATA%\open-backup\config.toml`。macOS と Linux は `dirs` の config。環境変数 `OPEN_BACKUP_HOME` があれば、その下の `config/` と `data/`(Android は `filesDir/ob`)。 / `%APPDATA%\open-backup\config.toml` on Windows; the `dirs` config folder on macOS and Linux. If `OPEN_BACKUP_HOME` is set, `config/` and `data/` under it (on Android, `filesDir/ob`). |
| 状態(いまの世代の番号・API の控え・AI の分類) / State (current generation, API cache, AI topics) | データの場所(`%LOCALAPPDATA%\open-backup`)の `state.json`・`api-state/`・`topics.json`。 / `state.json`, `api-state/` and `topics.json` in the data folder (`%LOCALAPPDATA%\open-backup`). |
| トークン / Token | 環境変数 `OPEN_BACKUP_TOKEN` → OS の資格情報ストア(Windows・macOS・Linux)→ データの場所の `token`(権限 0600。Android はこれ)。 / The environment variable `OPEN_BACKUP_TOKEN`, then the OS credential store (Windows, macOS, Linux), then a `token` file in the data folder (mode 0600; Android uses this). |
| テスト用トークン(この開発機だけ) / Test token (this development machine only) | `F:\backup-token.txt`(`aon-co-jp-backup`)。**値は画面・ログ・コミットに出さない。** / `F:\backup-token.txt` (`aon-co-jp-backup`). **Never print its value on screen, in logs or in commits.** |

**日本語**: 引っ越し先でも、GitHub 上の保存先はそのまま使えます。API の方法は、リポジトリの今の状態と比べて差分だけを送るので、手元の状態ファイルがなくても二重になりません(git の方法は、作業コピーを取り直します)。
**English**: On the new computer the destination repositories on GitHub keep working. The API way compares with the repository as it is now and sends only the differences, so even without the local state file nothing is uploaded twice (the git way fetches the work copy again).

### 4. 環境ごとの落とし穴 / Pitfalls of this environment

**日本語**
- Windows の PowerShell 5.1 は、BOM の無い UTF-8 を読みまちがえます(日本語入りの `.ps1` は BOM 付き)。`Set-Content` は、そのままだと ANSI で TOML をこわします(`-Encoding utf8`。BOM が要らないときは .NET で書く)。日本語の編集は Edit/Write か Python で。
- PowerShell の `Remove-Item` は、F: のパスと同じ命令にあると保護で止まります(別の命令に分けるか `[IO.File]::Delete`)。
- ureq は、HTTP 304 をエラーにせず成功として返します(条件つきの取得をするなら気をつけます)。
- GitHub は、リポジトリや内容を短い時間にたくさん作ると、二次レート制限(403)で止めます。API の方法は 1 ファイルごとに約 0.7 秒あけ、403 と 429 は待ってやり直します。
- adb: **利用者の実機がつながっていることがあります。** 実機のシリアルは、つなぐたびに違うかもしれません(`adb devices -l` で確かめる)。エミュレーターは `-s emulator-5554`。実機への導入・更新は、許可を得てから。`am start` は、設定画面がアプリの上に残っていると起動しません(`-f 0x10008000`)。
- 日本語を含む `perl -e` や、Bash のヒアドキュメントの `\` は、環境でこわれることがあります。編集は Edit/Write で。
- `scripts\build-release.ps1` の Linux 版は WSL で作ります。以前は WSL の失敗に気づかず、古い Linux 版が混ざりました。いまは失敗すると止まります。

**English**
- Windows PowerShell 5.1 misreads UTF-8 without a BOM (a `.ps1` with Japanese needs a BOM). `Set-Content` writes ANSI by default and breaks TOML (use `-Encoding utf8`; if you need no BOM, write with .NET). Edit Japanese text with Edit/Write or Python.
- PowerShell's `Remove-Item` is blocked when it is in the same command as an F: path (split the commands, or use `[IO.File]::Delete`).
- ureq returns HTTP 304 as a success, not an error (be careful if you do conditional requests).
- GitHub stops you with a secondary rate limit (403) when you create many repositories or contents quickly. The API way waits about 0.7 seconds per file and waits and retries on 403 and 429.
- adb: **the owner's real device may be connected.** Its serial may be different each time (check with `adb devices -l`). Use `-s emulator-5554` for the emulator. Ask before installing or updating on a real device. `am start` does not start the app if a settings screen is still on top (use `-f 0x10008000`).
- `perl -e` with Japanese text and `\` in Bash here-documents can break in this environment. Edit with Edit/Write.
- The Linux binary in `scripts\build-release.ps1` is built in WSL. A WSL failure used to go unnoticed and an old Linux binary was mixed in; now the script stops on failure.

### 5. まだのこと(大事な順) / Open items (most important first)

1. **実機でのログイン → フォルダを足す → 「今すぐバックアップ」の通し確認。** アプリは実機(A401OP / Android 15)に導入済みで、起動までは確認しました。トークンを入れる作業は、利用者が行います。エミュレーターでは通っています。 / **A full run on a real device: log in, add a folder, "Back up now".** The app is installed on the real device (A401OP / Android 15) and start-up is checked. The owner enters the token. It already works on the emulator.
2. iPhone 版。macOS で動かす確認。3 つの OS の CI(`.github/workflows/ci.yml` は用意済み。GitHub の支払い設定が直れば動きます)。 / An iPhone version. Running on macOS. CI for three operating systems (`.github/workflows/ci.yml` is ready; it runs once GitHub billing is fixed).
3. Android の公開用の署名と、アップデートの配り方。Android での AI 分類(`aruaru-llm` が無いので、拡張子の分類だけ)。 / Signing the Android app for release and how to deliver updates. AI sorting on Android (there is no `aruaru-llm`, so only extension sorting).
4. `open-easy-web` による `aruaru-llm` の導入と見守り。 / Installing and looking after `aruaru-llm` with `open-easy-web`.
5. ~~Windows インストーラー~~ **完了**: `installer/open-backup.iss`(Inno Setup。ユーザー単位・管理者不要・ログイン時の自動起動は選べる)。静かなインストール → `/ui` が 200 → アンインストールを、v0.1.1 と v0.2.0 で確認。 / ~~Windows installer~~ **Done**: `installer/open-backup.iss` (Inno Setup; per-user, no administrator, optional start at sign-in). Silent install, `/ui` returns 200, uninstall were checked for v0.1.1 and v0.2.0.
6. GitHub 以外の保存先。GitHub は大きなバックアップ向きではありません(1 ファイル 100MB、このアプリは 40MB まで。リポジトリは数 GB まで)。 / Destinations other than GitHub. GitHub is not meant for huge backups (100 MB per file, this app up to 40 MB; a few GB per repository).
7. 実際の職場で使うための仕事: 大きいファイルの分割・保存する期間のきまり・失敗の知らせ・たくさんの端末の一括管理・起動のあとで少し待ってから実行。 / Work for real office use: splitting big files, retention rules, failure notices, managing many devices at once, and waiting a little after start-up before running.

## しくみ / Architecture

| 場所 / Path | はたらき / Role |
|---|---|
| `crates/open-backup-core` | どの OS でも同じ中心部分。`sync.rs`(git の方法: 作業コピーにそろえる → commit → push、世代の切り替え)、`apisync.rs`(GitHub Data API だけの方法。git は要らない)、`github.rs`(非公開リポジトリの確認と作成)、`auth.rs`(トークン)、`index.rs`(拡張子の分類と、目次 `INDEX.md`・`.open-backup/index.jsonl`)、`ai.rs`(同梱の `aruaru-llm` による内容の分類)、`crypt.rs`(暗号化・圧縮)、`restore.rs`(復元)、`config.rs`。 / The core that is the same on every OS. `sync.rs` (the git way: mirror into a work copy, commit, push, switch generations), `apisync.rs` (the GitHub Data API way, no git needed), `github.rs` (check and create private repositories), `auth.rs` (token), `index.rs` (sorting by extension and the index `INDEX.md` and `.open-backup/index.jsonl`), `ai.rs` (topics from the bundled `aruaru-llm`), `crypt.rs` (encryption and compression), `restore.rs` (restore) and `config.rs`. |
| `crates/open-backup-cli` | `open-backup` コマンド(`login` `add` `remove` `status` `once` `run` `serve` `index` `crypt-init` `restore`)。`serve.rs` は `127.0.0.1` だけの HTTP(`/ui` `/status` `/run` `/add` `/remove` `/login` `/logout` `/restore/*`。Origin と Host を調べます)。 / The `open-backup` command. `serve.rs` is an HTTP server on `127.0.0.1` only (it checks Origin and Host). |
| `tools/gen-ui` | 画面を RPoem の `open-runo-crossui` で作ります(結果は `ui/`)。 / Makes the screen with RPoem's `open-runo-crossui` (the result is `ui/`). |
| `android/` | WebView と、前面サービス `EngineService`(Rust のエンジンを `lib*.so` として同梱して動かす)。 / A WebView and the foreground service `EngineService` (it ships and runs the Rust engine as `lib*.so`). |
| `installer/` と `scripts/` | Windows インストーラー(Inno Setup)と、リリース・インストーラーを作るスクリプト。 / The Windows installer (Inno Setup) and the scripts that build releases and the installer. |

### 世代の切り替え / Switching generations

**日本語**: `<リポジトリ名>-0001`(いま使う)と `-0002`(次に使う)を一緒に作ります。いま使うものが上限(`max_repo_mb`、初期値 4000)の 9 割になったら、次のものへ切り替えて全部のデータを入れ直し、さらにその次を用意します。データそのものが上限を超えているときは、切り替えずに警告します。次を用意できないとき(レート制限など)は、切り替えを見送ります。
**English**: `<repo name>-0001` (in use) and `-0002` (next) are created together. When the one in use reaches 90% of `max_repo_mb` (default 4000), it switches to the next one, saves all the data again, and prepares another spare. If the data alone is bigger than the limit, it warns instead of switching. If the next one cannot be made (for example because of a rate limit), it postpones the switch.

### 保存の方法 / Ways to save

**日本語**: `backend = "auto"`(初期値)は、`git` があれば `git`、なければ(Android はいつも)GitHub API を使います。API の方法は、各ファイルの git 流の blob SHA-1 を求め(大きさと更新時刻が同じなら控えを使う)、リポジトリの今の木と比べて、違うものだけを blob として送り、木(1000 件ずつ)→ コミット → `main` を進めます。消えたファイルは木から外します(履歴には残ります)。空のリポジトリは、Contents API で最初のコミットを作ります(Git Data API は空のリポジトリで使えないためです)。
**English**: `backend = "auto"` (the default) uses `git` if it exists, otherwise the GitHub API (Android always does). The API way computes a git-style blob SHA-1 for each file (using a saved value when size and modified time are the same), compares with the repository's current tree, uploads only the differences as blobs, then builds the tree (1000 entries at a time), commits and advances `main`. Deleted files leave the tree (history keeps them). An empty repository gets its first commit through the Contents API (the Git Data API does not work on an empty repository).

### なぜ git2 を使わないか / Why not git2

**日本語**: Android 向けの `git2` は OpenSSL が要り、`perl` と `make` の無い Windows の開発機では作れません。API の方法は ureq + rustls だけなので、Android(arm64 と x86_64)の `cargo ndk` で作れます。
**English**: `git2` for Android needs OpenSSL, which cannot be built on a Windows machine without `perl` and `make`. The API way uses only ureq and rustls, so it builds for Android (arm64 and x86_64) with `cargo ndk`.

## 試験 / Testing

| 何を / What | どうやって / How | 結果 / Result |
|---|---|---|
| 小さなテスト / Unit tests | `cargo test` | 通る(blob のハッシュが git と同じ・索引の分類・暗号化の往復)。 / Pass (blob hash matches git, index sorting, encryption round trip). |
| Windows・git の方法 / Windows, git way | 本物の GitHub(試験用の非公開リポジトリ)へ push・差分・自動切り替え。 / Push, diffs and automatic switching on real GitHub (a private test repository). | 確認済み / Checked |
| Windows・API の方法 / Windows, API way | 最初 → 変更なし(何も送らない)→ 変更と削除と追加。 / First run, then no change (nothing sent), then change, delete and add. | 確認済み(内容と履歴も GitHub 側で確認) / Checked (contents and history verified on GitHub) |
| 暗号化の通し / Encryption round trip | `tests/e2e_github.rs`(手で動かす)。暗号化 → 本物の GitHub → 取得 → 復元 → 同じ中身・非公開・平文の名前なし。 / Run by hand: encrypt, real GitHub, fetch, restore, identical content, private, no plain names. | `git` も API も確認済み(2026-10-26) / Checked for both git and API |
| 画面 / Screen | ブラウザで、フォルダ追加・「今すぐバックアップ」、スマホの縦横・タブレット・ダーク・大きい文字。 / In a browser: add a folder, "Back up now", phone portrait and landscape, tablet, dark, large text. | 確認済み(横スクロールなし・下のボタン固定) / Checked (no sideways scroll, bottom buttons fixed) |
| Windows インストーラー / Windows installer | 静かなインストール → `/ui` が 200 → アンインストール(ファイルとショートカットが消える)。 / Silent install, `/ui` returns 200, uninstall (files and shortcuts are removed). | v0.1.1 と v0.2.0 で確認済み / Checked for v0.1.1 and v0.2.0 |
| Android(x86_64 エミュレーター) / Android (x86_64 emulator) | 起動 → ログイン → フォルダを足す → 実行。v0.2.0 では、起動・画面・`/status`・復元の窓口。 / Start, log in, add a folder, run. For v0.2.0: start-up, the screen, `/status` and the restore endpoint. | 確認済み(`a.txt` と 700KB の `photo.jpg` が GitHub に入る) / Checked (`a.txt` and a 700 KB `photo.jpg` arrive on GitHub) |
| Android(実機 A401OP / Android 15) / Android (real device) | アプリの導入・エンジンの起動・画面・`/status`・版表記 0.2.0。 / Install, engine start, the screen, `/status`, version label 0.2.0. | 確認済み。**ログインより先は未確認** / Checked. **Login and later steps are not checked** |
| Linux(WSL) | release のビルドと試験。 / Release build and tests. | 確認済み / Checked |
| macOS | `cargo check`(zig cc) | コンパイルだけ / Compile only |

## 公開・配布・VPS / Release, distribution and the VPS

**日本語**
- **リポジトリは公開**(https://github.com/aon-co-jp/open-backup 。保存先の非公開リポジトリとは別の話)。公開の前に、履歴を調べ、まちがって commit されていた作成物(`tools/gen-ui/target`)と実機のシリアルを履歴から消しました(**コミットの ID が変わりました**)。トークンらしい文字列とユーザー名のパスが、履歴に残っていないことは確かめました。
- **リリース**: v0.1.0(2026-10-14)、v0.1.1(Windows インストーラー)、**v0.2.0(暗号化・復元入り。インストーラー・Android APK・Windows exe・Linux・`SHA256SUMS-0.2.0.txt`)**。作り方: `powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1 -Version 0.2.0 -Gradle <gradle.bat>`(`dist\` に出ます。`--remap-path-prefix` で、ユーザー名・cargo のパス・リポジトリのパスをバイナリに残しません)。続けて `scripts\build-installer.ps1 -Version 0.2.0`。**公開の前に、配布物に自分のユーザー名が無いか、必ず確かめます。** `gh release create` で公開し、差し替えは `gh release upload --clobber`。
- **VPS(ConoHa)**: 紹介ページ https://easy-web.tokyo/open-backup/ 。`/root/easy-web.tokyo/open-backup`(GitHub から clone。更新は `git pull --ff-only`)の `webpage/` を、`open-backup-web.service`(`python3 -m http.server 8116 --bind 127.0.0.1`)が配り、`open-web-server` の `domains.toml`(`/root/repository/open-web-server/domains.toml`)の `path_prefix = "/open-backup"` → `127.0.0.1:8116` で公開します。ダウンロードのリンクは GitHub のリリースを指します(VPS にはバイナリを置きません)。
  経路の設定を変えるときは `open-web-server` の再起動が要ります(ほかのサイトが一瞬止まります)。バックアップは `domains.toml.bak-20261014-open-backup`。変える前と後で、全ホストと主な経路の HTTP の状態が同じことを確かめます(2026-10-14 は 32 項目すべて同じでした)。`easyweb.tokyo`(ハイフン無し)はルートだけで、`/open-backup` は `easy-web.tokyo` 側です。
- **更新の手順**: ページを直す → `webpage/index.html` を commit して push → VPS で `cd /root/easy-web.tokyo/open-backup && git pull --ff-only`(サービスの再起動は要りません)。新しい版を出す → `build-release.ps1` と `build-installer.ps1` → 新しいタグで `gh release create` → `webpage/index.html` のリンク・大きさ・SHA-256 を直す。

**English**
- **The repository is public** (https://github.com/aon-co-jp/open-backup; a separate matter from the private destination repositories). Before publishing, the history was scanned, and a build output that had been committed by mistake (`tools/gen-ui/target`) and a device serial were removed from history (**commit IDs changed**). We checked that no token-like strings and no user-name paths remain in the history.
- **Releases**: v0.1.0 (2026-10-14), v0.1.1 (Windows installer) and **v0.2.0 (with encryption and restore; installer, Android APK, Windows exe, Linux and `SHA256SUMS-0.2.0.txt`)**. To build: `powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1 -Version 0.2.0 -Gradle <gradle.bat>` (output goes to `dist\`; `--remap-path-prefix` keeps the user name, the cargo path and the repository path out of the binaries), then `scripts\build-installer.ps1 -Version 0.2.0`. **Before publishing, always check that your own user name is not in the files.** Publish with `gh release create`; replace a file with `gh release upload --clobber`.
- **VPS (ConoHa)**: the web page https://easy-web.tokyo/open-backup/ . `open-backup-web.service` (`python3 -m http.server 8116 --bind 127.0.0.1`) serves `webpage/` from `/root/easy-web.tokyo/open-backup` (a clone from GitHub; update with `git pull --ff-only`), and `open-web-server` publishes it through `path_prefix = "/open-backup"` to `127.0.0.1:8116` in `domains.toml` (`/root/repository/open-web-server/domains.toml`). Download links point to GitHub releases (no binaries on the VPS).
  Changing the route needs a restart of `open-web-server` (other sites stop for a moment). The backup is `domains.toml.bak-20261014-open-backup`. Before and after a change, check that the HTTP state of all hosts and main routes is the same (on 2026-10-14 all 32 items were the same). `easyweb.tokyo` (no hyphen) serves only the root; `/open-backup` lives on `easy-web.tokyo`.
- **How to update**: fix the page, commit and push `webpage/index.html`, then run `cd /root/easy-web.tokyo/open-backup && git pull --ff-only` on the VPS (no service restart needed). To ship a new version: run `build-release.ps1` and `build-installer.ps1`, run `gh release create` with a new tag, and fix the link, size and SHA-256 in `webpage/index.html`.

## aruaru-llm の呼び出し(Connect) / Calling aruaru-llm (Connect)

**日本語**: `aruaru-llm` は REST をやめたので、このリポジトリの呼び出しも `POST /aruaru_llm.v1.Llm/<Method>`(JSON)に変えました(古い `/v1/...` は `aruaru-llm` 側で消えています)。対象は `aruaru-llm` を呼ぶところだけです。実サーバーとの往復の試験は、到達できるときだけ動きます。
**English**: `aruaru-llm` stopped using REST, so this repository's calls were changed to `POST /aruaru_llm.v1.Llm/<Method>` (JSON); the old `/v1/...` routes are gone on the `aruaru-llm` side. Only the places that call `aruaru-llm` were changed. The round-trip test against a real server runs only when the server can be reached.

## API のきまり / API rule

**日本語**: 新しい API は REST で作りません。ブラウザやコマンドから呼ぶものは Connect(`POST /<パッケージ>.<Service>/<Method>`、JSON)、サービス同士は gRPC / gRPC-web。相手が形を決めるもの(GitHub の Webhook・OAuth の戻り先・ヘルスチェック・静的ファイルなど)だけが例外です。迷ったら Connect。くわしい表と理由は、正本の [`runo`](https://github.com/aon-co-jp/runo) の `API-POLICY.md` と `CLAUDE.md` にあります。
**English**: New APIs are not built as REST. Calls from a browser or a command use Connect (`POST /<package>.<Service>/<Method>`, JSON), and services talk to each other with gRPC / gRPC-web. Only shapes that another party decides (GitHub webhooks, OAuth return addresses, health checks, static files and so on) are exceptions. When in doubt, use Connect. The full table and the reasons are in the source of truth, `API-POLICY.md` and `CLAUDE.md` in [`runo`](https://github.com/aon-co-jp/runo).
