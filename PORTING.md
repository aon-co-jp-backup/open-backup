# open-backup PORTING / 引っ越し・再開メモ (Moving & resume notes)

端末のデータを、利用者自身の GitHub **非公開**リポジトリへ自動でバックアップする。Rust のコア+常駐エンジン+画面(RPoem で生成)+ Android アプリ。
*Automatically backs up folders on a device to the user's own **private** GitHub repository. Rust core + resident engine + a UI generated with RPoem + an Android app.*

## 引っ越し手順 / Moving to another machine

このリポジトリだけで開発を続けられる(画面の生成だけ、兄弟リポジトリ `RPoem` を使う。生成物は `ui/` にコミット済みなので、通常のビルドには要らない)。
*This repository is self-sufficient. Only regenerating the UI needs the sibling `RPoem` repository; the generated output is committed in `ui/`, so normal builds do not need it.*

### 1. 必要なもの / What you need

| 用途 / For | 内容 / What |
|---|---|
| Rust | stable(`cargo`)。Windows は `%USERPROFILE%\.cargo\bin` を PATH の先頭に / put `~/.cargo/bin` first on PATH |
| 保存方法 `git` | `git` コマンド(Windows/macOS/Linux)。無ければ、自動で GitHub API の経路になる / the `git` command; without it the GitHub API path is used |
| Linux でのビルド | `gcc`・`pkg-config`・`libdbus-1-dev`(資格情報ストア `keyring` 用)。Windows からは WSL(Ubuntu)で |
| macOS | Mac または GitHub の macOS ランナー。Windows からは zig を C コンパイラにした `cargo check --target aarch64-apple-darwin` まで(リンクは Apple SDK が要り、ライセンス上、Apple 製ハードウェア向け) |
| Android | Android SDK(platform 35・build-tools)・NDK 27・JDK 17(Android Studio の `jbr` でよい)・Gradle 8.7 以上・`cargo-ndk`・`rustup target add aarch64-linux-android x86_64-linux-android` |
| 画面の再生成(任意) | 兄弟ディレクトリに `RPoem`(`../RPoem/crates/open-runo-crossui`)/ the sibling `RPoem` repo |

### 2. 取得・ビルド・テスト / Get, build, test

```
git clone https://github.com/aon-co-jp/open-backup.git && cd open-backup
cargo test                       # 単体テスト / unit tests
cargo build --release            # open-backup(CLI+常駐エンジン) / the CLI + engine
target\release\open-backup serve # 画面: http://127.0.0.1:4620/ui
```

- 画面を変えたとき: `tools\gen-ui` で `cargo run`(`ui/` を再生成)→ エンジンを作り直す(`ui/index.html` はエンジンに `include_str!` で埋め込まれる)。
- Android: `powershell -ExecutionPolicy Bypass -File android\build-android.ps1 -Gradle <gradle.bat のパス>`(エンジン arm64・x86_64 → `jniLibs` → APK)。**画面やコアを変えたら、必ずこのスクリプトで作り直す**(Gradle だけでは、アプリの中のエンジンが古いまま)。

### 3. 設定・データ・トークンの場所 / Where things live

| もの / What | 場所 / Where |
|---|---|
| 設定 | Windows `%APPDATA%\open-backup\config.toml`、macOS/Linux は `dirs` の config。`OPEN_BACKUP_HOME` があれば、その下の `config/` と `data/`(Android は `filesDir/ob`) |
| 状態(現用の世代番号・API の控え・AIの分類結果) | データ領域(`%LOCALAPPDATA%\open-backup`)の `state.json`・`api-state/`・`topics.json` |
| トークン | 環境変数 `OPEN_BACKUP_TOKEN` → OS の資格情報ストア(Windows/macOS/Linux) → データ領域の `token`(0600。Android はこれ) |
| テスト用トークン(この開発機のみ) | `F:\backup-token.txt`(`aon-co-jp-backup`。**値を画面・ログ・コミットに出さない**) |

引っ越し先でも、保存先のリポジトリ(GitHub 上)はそのまま使える。API の経路は、リポジトリの現在の木を基準に差分だけ送るので、状態ファイルが無くても重複しない(Git の経路は、作業コピーを取り直す)。
*On a new machine the destination repositories on GitHub keep working. The API path diffs against the repository's current tree, so it does not duplicate uploads even without the local state file.*

### 4. 環境固有の落とし穴 / Environment pitfalls

- Windows の PowerShell 5.1 は BOM の無い UTF-8 を読み間違える。日本語を含む `.ps1` は BOM 付き。`Set-Content` は ANSI 既定で TOML を壊す(`-Encoding utf8`、BOM 無しが要るなら .NET の書き込み)。
- PowerShell の `Remove-Item` は、F: のパスと同じ命令にあると保護で止まる(別の命令に分けるか `[IO.File]::Delete`)。
- ureq は HTTP 304 をエラーにせず成功として返す(条件付き取得をするなら注意)。
- GitHub は、リポジトリ作成・内容の作成を短時間に連打すると二次レート制限(403)で止める。API の経路は 1 ファイルごとに約 0.7 秒あけ、403/429 は待って再試行する。
- adb: **ユーザーの実機がつながっていることがある**。エミュレーターは `-s emulator-5554` を必ず付ける。実機への導入・更新は、ユーザーの許可を得てから。`am start` は、設定画面がアプリのタスクの上に残っていると起動しない(`-f 0x10008000` を付ける)。
- 非ASCII を含む `perl -e` や、Bash のヒアドキュメントの `\\` は、環境で壊れることがある。編集は Edit/Write で。

### 5. 未了 / Open items (priority order)

1. 実機(moto g53y 5G、Android 14)でのログイン → フォルダ追加 → 「今すぐバックアップ」の通し確認(アプリは導入済み・未ログイン。エミュレーターでは通っている)。
2. iPhone 版。macOS の実行確認。3 OS の CI(`.github/workflows/ci.yml` は用意済み。GitHub アカウントの支払い設定が直れば動く)。
3. Android の公開用の署名、アップデートの配布。Android での AI 分類(`aruaru-llm` が無いので、拡張子の分類だけ)。
4. `open-easy-web` による `aruaru-llm` の導入・起動の管理。
5. インストーラー(Windows)。
6. 保存先の選択肢(GitHub 以外)。GitHub は大容量向けではない(1 ファイル 100MB、このアプリは 40MB まで。リポジトリは数 GB まで)。

## アーキテクチャ / Architecture

| 場所 / Path | 役割 / Role |
|---|---|
| `crates/open-backup-core` | OS 非依存のコア。`sync.rs`(git の経路: 作業コピーにミラー → commit → push、世代の切替)、`apisync.rs`(GitHub Data API だけの経路。git 不要)、`github.rs`(非公開リポジトリの確認・作成)、`auth.rs`(トークン)、`index.rs`(拡張子の分類と索引 `INDEX.md`・`.open-backup/index.jsonl`)、`ai.rs`(同梱 `aruaru-llm` による内容の分類)、`config.rs` |
| `crates/open-backup-cli` | `open-backup` コマンド(`login` `add` `remove` `status` `once` `run` `serve` `index`)。`serve.rs` は 127.0.0.1 専用の HTTP(`/ui` `/status` `/run` `/add` `/remove` `/login` `/logout`。Origin/Host を検査) |
| `tools/gen-ui` | 画面を RPoem の `open-runo-crossui` で生成(生成物 `ui/`) |
| `android/` | WebView + 前面サービス `EngineService`(Rust のエンジンを `lib*.so` として同梱・起動) |

### 世代の切替 / Rollover

`<リポジトリ名>-0001`(現用)と `-0002`(次用)を同時に作る。現用が上限(`max_repo_mb`、既定 4000)の 9 割に達したら、次用へ切り替えて全データを入れ直し、さらに次を作る。データ自体が上限を超えているなら、切り替えずに警告する。次を用意できないとき(レート制限など)は、切り替えを見送る。
*`-0001` (active) and `-0002` (next) are created together; at 90% of `max_repo_mb` (default 4000 MB) it switches to the next one and keeps a new spare. If the data alone exceeds the limit it warns instead of switching; if the next repo cannot be made (rate limit) it postpones.*

### 保存方法 / Backends

`backend = "auto"`(既定): `git` があれば `git`、無ければ(Android は常に)GitHub API。API の経路: 各ファイルの git の blob SHA-1 を求め(サイズ・更新時刻が同じなら控えを使う)、リポジトリの現在の木と比べて違うものだけ blob として送る → 木(1000件ずつ)→ コミット → `main` を進める。消えたファイルは木から外す(履歴には残る)。空のリポジトリは、Contents API で最初のコミットを作る(Git Data API は空のリポジトリで使えない)。
*`auto` uses `git` if present, otherwise (always on Android) the GitHub API: git blob SHA-1 per file, diff against the repository's current tree, upload only differences, then tree (1000 entries per call) → commit → advance `main`; deleted files leave the tree (history is kept); an empty repository is initialised through the Contents API.*

### なぜ git2 ではないか / Why not git2

Android 向けの `git2` は OpenSSL が要り、`perl` と `make` の無い Windows の開発機ではビルドできない。API の経路は ureq+rustls だけで、Android(arm64・x86_64)の `cargo ndk` ビルドが通る。
*`git2` on Android needs OpenSSL, which cannot be built on a Windows machine without `perl`/`make`. The API path uses only ureq + rustls and builds for Android with `cargo ndk`.*

## 試験 / Testing

| 何を / What | どう / How | 結果 / Result |
|---|---|---|
| 単体 | `cargo test` | 通る(blob のハッシュが git と一致、索引の分類) |
| Windows・git の経路 | 実 GitHub(テスト用の非公開リポジトリ)へ push・増分・自動切替 | 確認済み |
| Windows・API の経路 | 初回 → 変更なし(何も送らない)→ 変更+削除+追加 | 確認済み(内容と履歴も GitHub 側で確認) |
| 画面 | ブラウザで、フォルダ追加・「今すぐバックアップ」、スマホ縦横・タブレット・ダーク・文字拡大 | 確認済み(横スクロールなし・下部のボタン固定) |
| Android(x86_64 エミュレーター) | 起動 → ログイン → フォルダ追加 → 実行 | 確認済み(`a.txt` と 700KB の `photo.jpg` が GitHub に入る) |
| Android(実機 arm64) | アプリの導入・エンジンの起動・状態の取得 | 確認済み。**ログイン以降は未確認** |
| Linux(WSL) | release ビルド・テスト | 確認済み |
| macOS | `cargo check`(zig cc) | コンパイルのみ |
