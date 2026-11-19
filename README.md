# open-backup

端末のデータ(指定フォルダ)を、利用者自身のGitHubアカウントの**非公開リポジトリ**へ自動でバックアップします。
Automatically backs up folders you choose on your device to a **private repository** in your own GitHub account.

**ダウンロード・紹介ページ / Download & website**: [リリース v0.2.0 / Releases](https://github.com/aon-co-jp/open-backup/releases) ・ [https://easy-web.tokyo/open-backup/](https://easy-web.tokyo/open-backup/)(Windowsインストーラー・Android の APK・Linux。開発用の署名のないビルド / Windows installer, Android APK, Linux; unsigned development builds)

## ひとことで言うと / In plain words

**日本語**
- **なにをするの?** あなたのスマホやパソコンの大事なファイル(写真・書類など)を、ネットの「あなた専用の金庫」(GitHubの非公開リポジトリ)に、自動でコピーしておきます。スマホをなくしたり、パソコンがこわれたりしても、金庫から取り戻せます。
- **だれにも見られないの?** 金庫は「非公開」だけを使います。さらに、送る前にファイルを**暗号化**(じぶんだけが読めるカギをかける)すれば、金庫の中身も読めません。**カギ(パスフレーズ)をなくすと、だれにも戻せません。** 紙に書いて、大切にしまってください。
- **むずかしい設定は?** 「フォルダを選ぶ」→「ログインする」→「バックアップ開始」だけです。あとは裏でこっそり動きます。
- **専門の方へ**: Rust製のコア+常駐エンジン(127.0.0.1のみ待受)、画面はRPoemで生成、Androidは前面サービス+WebView。保存はgit方式とGitHub Data API方式(git不要)の2通り。暗号化はXChaCha20-Poly1305+Argon2id(チャンク単位・決定的nonce・AADにパス)、圧縮はzstd、世代切替(現用+次用の2リポジトリ、上限の9割で切替)。

**English**
- **What does it do?** It automatically copies the important files on your phone or computer (photos, documents, ...) into your own private "vault" on the internet (a private GitHub repository). If you lose your phone or your computer breaks, you can get everything back from the vault.
- **Can anyone else see it?** The vault is always **private**. If you also turn on **encryption** (locking the files with a key only you know), nobody can read the contents, not even if they got into the vault. **If you lose the key (passphrase), nobody can bring the files back.** Write it on paper and keep it safe.
- **Is it hard to set up?** No: "choose a folder" -> "log in" -> "start the backup". After that it works quietly in the background.
- **For experts**: Rust core plus a resident engine (listens on 127.0.0.1 only), UI generated with RPoem, Android uses a foreground service plus a WebView. Two storage paths: git, and the GitHub Data API (no git needed). Encryption is XChaCha20-Poly1305 with Argon2id (per-chunk, deterministic nonce, path bound as AAD); compression is zstd; generation rollover (an active and a spare repository, switching at 90% of the limit).

## ⚠️ 注意 / Warning

**日本語**: バックアップ先のGitHubリポジトリは、必ず**非公開(Private)**にしてください。
公開リポジトリにすると、バックアップしたデータ(個人情報・資格情報を含む可能性があります)が世界中に公開されます。
本ツールは非公開リポジトリのみを自動作成し、既存の保存先が公開だった場合はデータを送らずに中止します。
ただし、あとから GitHub の設定画面で公開に変更すると、全データが公開されます。**絶対に変更しないでください。**

**English**: Always keep the GitHub backup repository **private**.
If it is made public, your backed-up data (which may contain personal information or credentials) is exposed to the whole world.
This tool only creates private repositories, and aborts without sending any data if an existing destination repository is public.
However, if you later switch it to public in GitHub's settings, all data becomes public. **Never do that.**

## 対応プラットフォーム / Target platforms

- Windows
- macOS
- Linux
- Android(スマートフォン・タブレット)
- 将来 / Future: iPhone(iOS)

## 構成 / Architecture

Rust のみで実装し、画面は [RPoem](https://github.com/aon-co-jp/RPoem) の `open-runo-crossui` から各OS向けに生成する方針です。

| クレート | 役割 | 状態 |
|---|---|---|
| `crates/open-backup-core` | OS非依存のコア(設定・GitHub認証・非公開リポジトリ作成・ミラー/commit/push) | 実装済み |
| `crates/open-backup-cli` (`open-backup`) | デスクトップ向けの常駐CLI(フォルダ変更の監視+定期実行) | 実装済み(Windowsでビルド確認) |
| 画面(RPoem crossui) | Windows/macOS/Linux/Android 向けのUI(`tools/gen-ui` → `ui/`、エンジンが `/ui` で配信) | 実装済み(Windows と Android エミュレーターで確認) |
| Android アプリ | `android/`(WebView + 前面サービスで Rust のエンジンを起動。**git 不要**、GitHub API だけで保存) | 実装済み(エミュレーターで通し確認。実機 A401OP / Android 15 で起動・画面を確認、ログイン以降は未確認 / on a real device: starts and shows the UI, login onward not yet verified) |
| iPhone | 将来 | 未着手 |

## 事前準備(利用者の自己責任で) / Before you start (your own responsibility)

**日本語**: GitHub のアカウントを**新規に作成**するか、**既にお持ちならログイン**してください。
次に、バックアップに必要な**最低限の権限のトークン**の作り方を、Google などで検索してご自身で調べたうえでご利用ください。
(参考: 保存先の非公開リポジトリを先に自分で作り、そのリポジトリだけに「Contents: 読み書き」を許可した Fine-grained トークンにすると、権限を最小にできます。)
トークンは他人に見せず、公開リポジトリやチャットに貼らないでください。

**English**: **Create a GitHub account**, or **log in if you already have one**.
Then look up, via Google or similar, how to create a token with the **minimum permissions** needed, and use it at your own responsibility.
(Hint: create the private destination repository yourself first, then issue a fine-grained token limited to that single repository with "Contents: read and write" only.)
Never show the token to anyone or paste it into a public repository or chat.

### トークン作成の最低限の説明 / Minimal token instructions

画面の名称は変わることがあります。最新の手順は必ずご自身で検索して確認してください。
GitHub's UI changes over time. Always confirm the current steps yourself via search.

**日本語**
1. GitHub にログイン → 右上のアイコン → Settings → Developer settings → Personal access tokens → Fine-grained tokens → Generate new token。
2. Expiration(有効期限)を短めに設定する。
3. Repository access は **Only select repositories** で、保存先の非公開リポジトリ**1つだけ**を選ぶ。
4. Repository permissions は **Contents: Read and write** のみ(Metadata: Read-only は自動で付く)。他は付けない。
5. 生成されたトークンを一度だけ表示されるうちにコピーし、`open-backup login --token` で入力する。

**English**
1. Log in to GitHub → profile icon → Settings → Developer settings → Personal access tokens → Fine-grained tokens → Generate new token.
2. Set a short Expiration.
3. Repository access: **Only select repositories**, and pick **only the one** private destination repository.
4. Repository permissions: **Contents: Read and write** only (Metadata: Read-only is added automatically). Grant nothing else.
5. Copy the token while it is shown (only once) and enter it with `open-backup login --token`.

## 使い方 / Usage

```bash
open-backup login --token     # 個人用トークン(repoスコープ)を標準入力から。または client_id を設定してデバイス認可
open-backup add <フォルダ>     # バックアップ対象を追加
open-backup once              # 1回だけ実行
open-backup run               # 常駐(変更検知 + 定期実行)
```

- 保存先: ログインしたアカウントの `open-backup-<端末名>-0001`(非公開)。初回に `-0001`(現用)と `-0002`(次用)を同時に作ります。
- 容量: 現用が上限(既定4000MB)の9割に達すると、あらかじめ作ってある次用へ切り替えて全データをそこから保存し、さらに次の新規リポジトリを自動で作ります。古いリポジトリは過去分としてそのまま残ります。
  Rollover: when the active repo reaches 90% of its limit (default 4000 MB), backups switch to the pre-created next repo and a new spare is created automatically. Old repos are kept as history.
- ※ Fine-grained トークンを1リポジトリに限定している場合、自動作成には使えません(新規作成の権限が必要)。自動で増やすなら、アカウント全体の `Administration: write` を許可したトークンにするか、手動で先に作ってください。
- アカウントの新規作成は GitHub 側で利用者自身が行います(本ツールは作成しません)。
- トークンはOSの資格情報ストアに保存し、gitのコマンドラインには出しません。
- 100MB超のファイルはGitHubの制限により対象外(既定95MB超をスキップ)。
- 保存形式は普通のフォルダ(元の構造のまま)。自動カテゴリ分け(拡張子)と自動索引は、各リポジトリの `INDEX.md` と `.open-backup/index.jsonl` に出力します。
  Stored as plain folders (original layout). Auto-categorization (by extension) and an auto-index are written to `INDEX.md` and `.open-backup/index.jsonl` in each repo.
- 元で削除したファイルはバックアップ側からも消えますが、gitの履歴には残ります。

## 暗号化・圧縮・復元 / Encryption, compression, restore (2026-10-26)

**日本語**
- **暗号化(任意)**: 送る前に**端末で**暗号化します。鍵はパスフレーズから Argon2id で作り、本体は XChaCha20-Poly1305(1MiB ごとのチャンク、改ざん・切り詰め・入れ替えを検出)。
  ファイル名・フォルダ構造・サイズ・AI の分類は、暗号化した目録(`.open-backup/manifest.obe`)にだけ入り、リポジトリ上の名前は不透明なハッシュです。
  同じ内容なら毎回同じ暗号文になるので、変更の無いファイルは git に差分が出ません。**パスフレーズを失うと復元できません。**
- **圧縮**: 暗号化の前に zstd で圧縮します(`compress_level`、既定3、0で無効)。写真・動画・圧縮ファイルは自動で省きます。
- **並列**: ファイル単位・チャンク単位でマルチスレッドに処理します(`threads`、既定は CPU のコア数の半分。常駐の邪魔をしないため)。
  実測(この開発機・8スレッド): 暗号化 約 740MB/s、復号 約 730MB/s、日本語の文章は圧縮込みで約 1.8GB/s。**GPU(open-cuda)は使いません**: ChaCha20 は CPU で十分速く、律速はディスクと回線のためです。
- **復元**: 好きな場所へ戻せます(暗号化は自動で復号。既存のファイルは、指定しない限り上書きしません)。画面の「復元」カード、または CLI。
- 使い方: `open-backup crypt-init`(標準入力の1行目にパスフレーズ。2行目に確認用) → `config.toml` の `repo` を**新しい名前**に(暗号化していない過去のバックアップが残るリポジトリには書き込みません) → 通常どおりバックアップ。
  復元: `open-backup restore --to <フォルダ> [--github] [--gen N] [--only <ラベル/パス>] [--overwrite]`。一覧だけ見るなら `--list`。
- 保存方法: **git 方式も GitHub API 方式(Android)も**、同じ形式で暗号化して送ります(実際の GitHub を通して、暗号化→取得→復元→一致を確認済み)。

**English**
- **Encryption (optional)**: encrypted **on your device** before anything is sent. Key = Argon2id(passphrase); data = XChaCha20-Poly1305 in 1 MiB chunks (tampering, truncation and swapped files are detected).
  File names, folder layout, sizes and AI topics live only in an encrypted manifest (`.open-backup/manifest.obe`); names in the repo are opaque hashes.
  Identical input gives identical ciphertext, so unchanged files produce no git diff. **If you lose the passphrase, the data cannot be recovered.**
- **Compression**: zstd before encryption (`compress_level`, default 3, 0 = off); photos, videos and archives are skipped automatically.
- **Parallel**: multi-threaded per file and per chunk (`threads`; default = half the CPU cores, so the resident process stays unobtrusive).
  Measured (this dev machine, 8 threads): ~740 MB/s encrypt, ~730 MB/s decrypt, ~1.8 GB/s on Japanese text with compression. **No GPU (open-cuda) is used**: ChaCha20 is fast enough on the CPU and the real bottleneck is disk and network.
- **Restore**: restore to any folder (encrypted backups are decrypted automatically; existing files are not overwritten unless you ask) from the "Restore" card in the UI or from the CLI.
- Usage: `open-backup crypt-init` (passphrase on the first line of stdin, confirmation on the second) → change `repo` in `config.toml` to a **new name** (it refuses to write into a repository that already holds unencrypted backups) → back up as usual.
  Restore: `open-backup restore --to <folder> [--github] [--gen N] [--only <label/path>] [--overwrite]`; `--list` only lists.
- Backends: encryption works with **both the git backend and the GitHub API backend (Android)**, using the same format (verified end to end against real GitHub: encrypt → push → fetch → restore → identical).

## ライセンス / License

MIT OR Apache-2.0

## AI による内容分類 / AI content classification

**日本語**: 拡張子による分類に加え、同梱の `aruaru-llm`(端末内 `127.0.0.1` のみで動作)がファイルの名前と冒頭の内容から
「仕事・お金・写真・学習・開発・契約・設定・趣味・その他」のトピックを付け、索引(`topic`)に記録します。
内容は外部へ送信しません(送信先は常にループバック固定)。`aruaru-llm` が無い・起動できない場合は拡張子分類だけで続行します。
`aruaru-llm` の導入・起動の管理は [open-easy-web](https://github.com/aon-co-jp/open-easy-web) と連携する方針です(未実装)。
無効にするには `config.toml` に `ai_classify = false`。

**English**: In addition to extension-based categories, the bundled `aruaru-llm` (running only on-device at `127.0.0.1`) assigns a topic
(work / money / photos / study / dev / contracts / system / hobby / other) from each file's name and first bytes, recorded as `topic` in the index.
Nothing is sent off the device (the endpoint is always loopback). If `aruaru-llm` is missing or fails to start, only extension categories are used.
Installing and supervising `aruaru-llm` is planned to be handled together with open-easy-web (not implemented yet).
Disable with `ai_classify = false` in `config.toml`.

## 動作確認の記録 / Verification log (2026-10-12〜14)

- 実機(Windows)で、テスト用の非公開リポジトリへ push・増分(変更/削除)・自動切り替えを確認。
- 確認で見つけて直した点: 上限の計算が MB の整数で切り捨てられていた / データ自体が上限を超えると毎回切り替わっていた(警告して現状維持に変更) / GitHub の二次レート制限で次のリポジトリを作れない場合は切り替えを見送る。
- 切り替えの再確認: 修正後、上限超過で1回だけ次のリポジトリへ切り替わり、全データが新しいリポジトリへ入り、予備が自動作成されることを確認。
- Linux: WSL(Ubuntu)で release ビルドとテストが成功、`status` コマンドの起動も確認。
- macOS: zig を C コンパイラにして `aarch64-apple-darwin` 向けの `cargo check` が成功(リンクと実行は未確認。SDK は Apple のライセンス上、Mac または GitHub の macOS ランナーで行う)。
- 未確認: macOS の実行・Android・iPhone、画面(RPoem)。`.github/workflows/ci.yml` で3OSのビルドを用意したが、GitHub アカウントの支払い設定の問題で Actions が起動できていない(支払い設定を直せば検証できる)。Windows からのクロスビルドは C コンパイラ(ring)が無く不可。

## 画面 / UI

`open-backup serve` で常駐エンジンを起動し、`http://127.0.0.1:4620/ui` を開く(127.0.0.1専用。他サイトからの操作はOrigin/Hostを検査して拒否)。
画面は [RPoem](https://github.com/aon-co-jp/RPoem) の `open-runo-crossui` から全OS共通で生成(`tools/gen-ui`、生成物は `ui/` にコミット済み。RPoemが無くてもCLIはビルドできる)。
Windows/macOS/Linux は `ui/<os>/launch.*`、Android は `ui/android/`(WebViewシェルが `http://127.0.0.1:4620/ui` を読み込む)。
確認済み: 画面からのフォルダ追加と「今すぐバックアップ」でGitHubへpush、スマホ縦横・タブレット・ダーク・文字拡大で横スクロールなし・下部ボタン固定。

## 保存方法(git と GitHub API) / Backends: git and the GitHub API

**日本語**: 既定(`backend = "auto"`)では、`git` コマンドがあれば `git` で、無ければ **GitHub API だけ**(Rust のみ。Android はこちら)で保存します。どちらも同じ結果(非公開リポジトリの中の、元のフォルダ構造のファイル+`INDEX.md`)になります。
API の経路は、ファイルごとに git の blob のハッシュ(SHA-1)を求め、リポジトリの現在の木と比べて**違うものだけ**を送ります(中断しても、別の端末から続けても、重複して送りません)。元で消えたファイルは木から外します(履歴には残ります)。1ファイルは 40MB まで(API の制限に余裕を持たせた値)。GitHub の二次レート制限に配慮して、1ファイルごとに約 0.7 秒あけます(最初の全量の保存は、ファイルが多いと時間がかかります)。
**English**: By default (`backend = "auto"`) it uses `git` when available, otherwise **only the GitHub API** (pure Rust; this is what Android uses). Both give the same result: your original folder layout plus `INDEX.md` in a private repository. The API path hashes every file like git does (SHA-1 blob), compares it with the repository's current tree and uploads **only what differs** (so interrupted runs and other devices never cause duplicates); deleted files are removed from the tree (history is kept). Files up to 40 MB; ~0.7 s between uploads to respect GitHub's secondary rate limit (the first full backup of many files takes a while).

## Android

```
powershell -ExecutionPolicy Bypass -File android\build-android.ps1     # エンジン(arm64・x86_64)+ APK を作る / builds the engine and the APK
```

- アプリは、画面(WebView)と、エンジンを動かし続ける前面サービスだけです。エンジンは `lib*.so` として同梱した Rust の実行ファイルで、アプリ専用の領域(`filesDir/ob`)に設定とトークンを置きます(自分だけが読める権限)。
- 端末内のフォルダ(写真・文書)を読むため、初回に「すべてのファイルへのアクセス」の許可を案内します。画面の「おすすめフォルダ」(DCIM・Pictures・Documents・Download …)のボタンで追加できます。
- トークンは、画面の「GitHub にログイン」欄に貼り付けます(作り方は上の「事前準備」)。
- *The app is only a WebView plus a foreground service that keeps the engine running. The engine is a Rust executable shipped as `lib*.so`; its config and token live in the app-private directory. On first launch it asks for "All files access" (to read photos/documents by path); quick-add buttons offer DCIM, Pictures, Documents, Download… Paste your token in the "Log in to GitHub" card.*
- 実機(moto g53y 5G、Android 14、arm64)には、アプリを導入し、エンジンが起動することまで確認(ログイン以降は、実機での確認待ち)。
- 確認済み(Android 16 / x86_64 エミュレーター): 起動 → エンジン起動 → ログイン → `/storage/emulated/0/Documents` のフォルダを追加 → 「今すぐバックアップ」で、`a.txt` と 700KB の `photo.jpg` が非公開リポジトリに入る。**実機・arm64 の実行は未確認**(arm64 のエンジンのビルドは成功)。

## はじめての方へ(画面の案内) / First-time guide (in the app)

**日本語**: 画面の「GitHub にログイン」欄に、「はじめての方へ(GitHub のアカウントが無い方も)」の案内があります(日英併記)。GitHub のアカウントが無い場合は、無料で作ります。**アカウントの作成とメールアドレスの確認は、ご自身で GitHub の画面で行います(このアプリは代わりに作れません)**。続けて、保存先の非公開リポジトリ(名前は画面の「アカウント・保存先」の表示どおり。`-0002` も作っておくと自動で切り替わります)と、そのリポジトリだけに「Contents: 読み書き」を許可した Fine-grained トークンを作り、貼り付けます。公式サイトへのリンク付き(Android では端末のブラウザで開きます)。GitHub は大容量のバックアップ向けではなく、1ファイルは 100MB(このアプリは 40MB)まで、リポジトリ全体は数GBまでが目安です。
**English**: The "Log in to GitHub" card has a bilingual "New here? (no GitHub account yet)" guide. If you have no GitHub account, create a free one — **you sign up and confirm your email yourself on GitHub; this app cannot create accounts for you**. Then create a private repository named as shown under "Account & destination" (also create the `-0002` one so it can switch automatically) and a fine-grained token limited to it with "Contents: Read and write", and paste the token. Links to the official pages are included (opened in the device's browser on Android). GitHub is not meant for huge backups: files up to 100 MB (this app: 40 MB), a few GB per repository.

---

## 引っ越し / Moving to another machine

**日本語**: このリポジトリだけで開発を続けられます。別の PC・別の作業環境へ移るときは、[`PORTING.md`](PORTING.md)(必要なツール・取得とビルドとテスト・設定とトークンの場所・環境固有の落とし穴・未了の一覧・アーキテクチャ・試験の結果)と、[`CLAUDE.md`](CLAUDE.md) の HANDOFF を読んでください。保存先の GitHub リポジトリは、引っ越し先でもそのまま使えます(API の経路は、リポジトリの現在の状態を基準に差分だけ送るので、手元の状態ファイルが無くても重複しません)。トークンは、リポジトリには含まれません(OS の資格情報ストア、またはアプリ専用領域)。引っ越し先で、もう一度ログインしてください。
**English**: This repository is self-sufficient. To continue on another machine, read [`PORTING.md`](PORTING.md) (tools, get/build/test, where settings and the token live, environment pitfalls, open items, architecture, test results) and the HANDOFF in [`CLAUDE.md`](CLAUDE.md). The destination repositories on GitHub keep working on the new machine (the API path diffs against the repository's current state, so a missing local state file causes no duplicates). The token is never part of the repository (OS credential store or the app-private directory); log in again on the new machine.

```
git clone https://github.com/aon-co-jp/open-backup.git
cd open-backup
cargo test
```

---

作者 / Author: 石 塚 正 浩（MASAHIRO ISHIZUKA） (https://github.com/aon-co-jp)

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**
