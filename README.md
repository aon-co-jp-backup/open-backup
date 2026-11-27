# open-backup

**日本語**: スマホやパソコンの大事なファイルを、あなた専用の「ネットの金庫」(GitHub の**非公開**リポジトリ)へ、自動でコピーしておくアプリです。
**English**: open-backup copies the important files on your phone or computer into your own "online vault" (a **private** GitHub repository), all by itself.

**ダウンロードと紹介ページ / Download and web page**: [最新リリース v0.2.0 / Latest release v0.2.0](https://github.com/aon-co-jp/open-backup/releases) ・ [https://easy-web.tokyo/open-backup/](https://easy-web.tokyo/open-backup/)
(Windows インストーラー・Android の APK・Linux。まだ「開発用の署名なし」のビルドです。/ Windows installer, Android APK and Linux. These are still unsigned development builds.)

---

## ひとことで言うと / In plain words

**日本語**
- **なにをするの?** 写真や書類などを、ネットの金庫に自動でコピーします。スマホをなくしても、パソコンがこわれても、金庫から取り戻せます。
- **ほかの人に見られない?** 金庫は「非公開」だけを使います。さらに**暗号化**(自分だけが持つカギでファイルをロックすること)をオンにすれば、金庫の中身も読めません。**カギ(パスフレーズ)をなくすと、だれにも元に戻せません。紙に書いて大切にしまってください。**
- **むずかしい?** いいえ。「フォルダを選ぶ」→「ログインする」→「バックアップ開始」の3つだけです。あとは裏で静かに動きます。
- **専門の方へ**: Rust 製のコアと常駐エンジン(`127.0.0.1` だけで待ち受け)、画面は RPoem で生成、Android は前面サービス+WebView。保存方法は 2 通り(`git` と、git 不要の GitHub Data API)。暗号化は XChaCha20-Poly1305 + Argon2id、圧縮は zstd。

**English**
- **What does it do?** It copies your photos, documents and other files to the online vault automatically. If you lose your phone or your computer breaks, you can get everything back.
- **Can other people see my files?** No. The vault is always **private**. You can also turn on **encryption** (locking the files with a key that only you have), so even the contents of the vault cannot be read. **If you lose the key (your passphrase), nobody can bring the files back. Please write it on paper and keep it safe.**
- **Is it hard?** No. There are only three steps: "choose a folder", "log in", "start the backup". After that it works quietly in the background.
- **For experts**: Rust core and a resident engine (listens on `127.0.0.1` only); the UI is generated with RPoem; Android uses a foreground service and a WebView. Two storage paths: `git`, and the GitHub Data API (no git needed). Encryption is XChaCha20-Poly1305 with Argon2id; compression is zstd.

---

## ⚠️ とても大事なこと / Very important

**日本語**: バックアップ先の GitHub リポジトリは、必ず**非公開(Private)**にしてください。公開にすると、あなたのデータ(個人情報やパスワードが入っているかもしれません)が世界中の人に見えてしまいます。
このアプリは非公開のリポジトリだけを自動で作り、保存先が公開になっていたら、データを送らずに止まります。でも、あとから GitHub の設定で公開に変えてしまうと、すべてが公開されます。**絶対に変えないでください。**

**English**: Always keep the GitHub repository for your backups **private**. If it becomes public, everyone in the world can see your data (it may contain personal information or passwords).
This app only creates private repositories, and it stops without sending anything if the destination is public. But if you switch it to public later in GitHub's settings, everything becomes public. **Please never do that.**

---

## はじめての方へ / If this is your first time

**日本語**
1. **GitHub のアカウント**を作ります(無料)。すでにあるなら、ログインします。アカウントの作成とメールの確認は、**あなた自身が** GitHub の画面で行います(このアプリは代わりに作れません)。
2. 保存先の**非公開リポジトリ**を作ります。名前の例: `open-backup-<端末名>-0001`。`-0002` も作っておくと、いっぱいになったとき自動で切り替わります。
3. **トークン**(アプリ専用のカギ)を作ります。画面の名前は変わることがあるので、最新の手順は検索して確かめてください。
   1. GitHub にログイン → 右上のアイコン → Settings → Developer settings → Personal access tokens → Fine-grained tokens → Generate new token。
   2. 有効期限は短めにします。
   3. Repository access は「Only select repositories」で、保存先の**1つだけ**を選びます。
   4. Repository permissions は「Contents: Read and write」だけにします(「Metadata: Read-only」は自動で付きます)。ほかは付けません。
   5. 作られたトークンは**一度しか表示されません**。コピーして、アプリの「GitHub にログイン」欄に貼ります。
4. トークンは、他の人に見せたり、公開の場所やチャットに貼ったりしないでください。

**English**
1. Make a **GitHub account** (it is free). If you already have one, log in. **You** create the account and confirm your email on GitHub's own pages (this app cannot do it for you).
2. Make a **private repository** for the backups. Example name: `open-backup-<device-name>-0001`. If you also make `-0002`, the app can switch to it by itself when the first one gets full.
3. Make a **token** (a key just for this app). GitHub's screens change from time to time, so please search for the latest steps.
   1. Log in to GitHub, click your icon at the top right, then Settings, Developer settings, Personal access tokens, Fine-grained tokens, Generate new token.
   2. Choose a short expiry date.
   3. Under Repository access, choose "Only select repositories" and pick **only the one** destination repository.
   4. Under Repository permissions, choose only "Contents: Read and write" ("Metadata: Read-only" is added automatically). Add nothing else.
   5. The token is shown **only once**. Copy it and paste it into the "Log in to GitHub" box in the app.
4. Never show the token to other people, and never paste it in a public place or in a chat.

GitHub is not made for huge backups: one file can be up to 100 MB (this app: up to 40 MB), and one repository is best kept to a few GB.
GitHub は大きなバックアップ向きではありません。1 ファイルは 100MB まで(このアプリは 40MB まで)、リポジトリ全体は数 GB までが目安です。

---

## 使い方 / How to use

**画面から / With the screen**: `open-backup serve` を動かして `http://127.0.0.1:4620/ui` を開きます(自分のパソコンの中だけで開く画面です)。Windows は、インストーラーを使うと簡単です。
**English**: Run `open-backup serve` and open `http://127.0.0.1:4620/ui` (a screen that opens only inside your own computer). On Windows, the installer is the easy way.

**コマンドから / With commands**

```bash
open-backup login --token     # トークンを標準入力から / Give the token on standard input
open-backup add <folder>      # バックアップするフォルダを足す / Add a folder to back up
open-backup once              # 1回だけ実行 / Run one time
open-backup run               # ずっと動かす(変更を見つけて、決まった間隔で) / Keep running (watches changes and repeats on a timer)
```

- **保存先 / Where it goes**: ログインしたアカウントの `open-backup-<端末名>-0001`(非公開)。最初に `-0001`(いま使う)と `-0002`(次に使う)を一緒に作ります。/ `open-backup-<device-name>-0001` (private) in your account. At the start it makes `-0001` (in use) and `-0002` (next) together.
- **いっぱいになったら / When it gets full**: 上限(初期値 4000MB)の 9 割になると、用意しておいた次のリポジトリへ自動で切り替えます。古いものは、昔の記録としてそのまま残ります。/ At 90% of the limit (default 4000 MB) it switches to the spare repository by itself. The old one stays as a record.
- **トークンの注意 / Token note**: トークンを 1 つのリポジトリだけに限ると、新しいリポジトリは自動では作れません(作る権限がないため)。自動で増やしたいなら、アカウント全体に `Administration: write` を許したトークンにするか、先に自分で作ってください。/ If the token is limited to one repository, the app cannot create new repositories by itself. To let it grow automatically, use a token with `Administration: write` for the whole account, or create the repositories yourself first.
- **保存のしかた / How files are stored**: 元のフォルダの形のまま保存します。種類(拡張子)ごとの自動の分類と目次は、各リポジトリの `INDEX.md` と `.open-backup/index.jsonl` に書きます。/ Files keep their original folder layout. An automatic sort by type (file extension) and a table of contents are written to `INDEX.md` and `.open-backup/index.jsonl`.
- 元のフォルダで消したファイルは、バックアップ側の最新の状態からも消えますが、git の履歴には残ります。/ Files you delete from the original folder disappear from the latest backup too, but stay in git's history.
- 100MB より大きいファイルは、GitHub のきまりで送れません(初期値では 95MB より大きいものを飛ばします)。/ Files over 100 MB cannot be sent because of GitHub's rules (by default, files over 95 MB are skipped).

---

## 暗号化・圧縮・復元 / Encryption, compression and restore

**日本語**
- **暗号化(好きなときだけオン)**: 送る前に、**あなたの端末の中で**暗号化します。カギはパスフレーズから作ります(Argon2id)。中身は XChaCha20-Poly1305 で守り、1MiB ごとのかたまりで、書き換え・途中切れ・入れ替えを見つけます。
  ファイル名・フォルダの形・大きさ・AI の分類は、暗号化した目録(`.open-backup/manifest.obe`)の中だけにあり、リポジトリに見える名前は意味のない文字列です。
  同じ中身なら毎回同じ暗号文になるので、変わっていないファイルは git に差分が出ません。**パスフレーズをなくすと、元に戻せません。**
- **圧縮**: 暗号化の前に zstd で小さくします(`compress_level`、初期値 3、0 で無効)。写真・動画・圧縮ファイルは、もともと小さいので自動で飛ばします。
- **並列**: ファイルごと・かたまりごとに、複数のスレッドで同時に処理します(`threads`、初期値は CPU のコア数の半分。常駐していても邪魔にならないように)。
  測定(この開発用パソコン・8 スレッド): 暗号化 約 740MB/秒、復号 約 730MB/秒。**GPU は使いません**(CPU で十分速く、遅くなる原因はディスクと回線だからです)。
- **復元**: 好きな場所へ戻せます。暗号化したものは自動でもとに戻し、すでにあるファイルは、頼まない限り上書きしません。画面の「復元」カードか、コマンドで行います。
- **使い方**: `open-backup crypt-init`(1 行目にパスフレーズ、2 行目に確認用)→ `config.toml` の `repo` を**新しい名前**にする(暗号化していない古いバックアップがあるリポジトリには書き込みません)→ いつもどおりバックアップ。
  復元: `open-backup restore --to <フォルダ> [--github] [--gen N] [--only <ラベル/パス>] [--overwrite]`。一覧だけ見るなら `--list`。
- `git` の方法でも、GitHub API の方法(Android)でも、同じ形で暗号化して送ります。本物の GitHub で「暗号化 → 送る → 取ってくる → 復元 → 同じ中身」を確かめました。

**English**
- **Encryption (optional)**: files are encrypted **inside your own device** before they are sent. The key is made from your passphrase (Argon2id). The contents are protected with XChaCha20-Poly1305 in 1 MiB pieces, so changes, cut-off files and swapped files are noticed.
  File names, folder layout, sizes and AI topics are stored only inside an encrypted catalog (`.open-backup/manifest.obe`); the names you can see in the repository are meaningless strings.
  The same input always gives the same encrypted output, so files that did not change cause no difference in git. **If you lose the passphrase, nothing can be restored.**
- **Compression**: files are made smaller with zstd before encryption (`compress_level`, default 3, 0 = off). Photos, videos and archives are skipped automatically because they are already small.
- **Parallel work**: many threads work at the same time, per file and per piece (`threads`; by default half of the CPU cores, so it never gets in your way).
  Measured (this development PC, 8 threads): about 740 MB/s to encrypt and about 730 MB/s to decrypt. **No GPU is used**: the CPU is fast enough, and the slow parts are the disk and the network.
- **Restore**: bring files back to any folder. Encrypted files are decrypted for you, and files that already exist are never overwritten unless you ask. Use the "Restore" card on the screen or a command.
- **How to use it**: run `open-backup crypt-init` (passphrase on the first line, the same again on the second), change `repo` in `config.toml` to a **new name** (it refuses to write into a repository that already holds unencrypted backups), then back up as usual.
  Restore: `open-backup restore --to <folder> [--github] [--gen N] [--only <label/path>] [--overwrite]`. Use `--list` to only look.
- Both the `git` way and the GitHub API way (Android) encrypt in the same format. We checked on real GitHub: encrypt, send, fetch, restore, and get identical contents.

---

## AI によるなかま分け / AI sorting by topic

**日本語**: 拡張子での分類に加えて、同梱の `aruaru-llm`(あなたの端末の `127.0.0.1` の中だけで動きます)が、ファイル名と最初の部分を見て「仕事・お金・写真・学習・開発・契約・設定・趣味・その他」のどれかを付けます。中身は**外には送りません**。`aruaru-llm` が無い・動かないときは、拡張子の分類だけで続けます。やめたいときは `config.toml` に `ai_classify = false`。`aruaru-llm` の導入と見守りは [open-easy-web](https://github.com/aon-co-jp/open-easy-web) と一緒にやる予定です(まだ作っていません)。
**English**: Besides sorting by file extension, the bundled `aruaru-llm` (it runs only inside your device, at `127.0.0.1`) looks at each file's name and first part and gives it one topic: work, money, photos, study, dev, contracts, system, hobby or other. Your content is **never sent outside**. If `aruaru-llm` is missing or cannot start, the app just keeps using extension-based sorting. To turn it off, put `ai_classify = false` in `config.toml`. Installing and looking after `aruaru-llm` is planned to be done together with [open-easy-web](https://github.com/aon-co-jp/open-easy-web) (not built yet).

---

## どんな端末で使える? / Which devices?

| 端末 / Device | 状態 / Status |
|---|---|
| Windows | 使えます。インストーラーあり(ユーザー単位・管理者不要)。 / Works. Installer available (per-user, no administrator needed). |
| Linux | 使えます(WSL で試験)。 / Works (tested in WSL). |
| Android | 使えます(エミュレーターで最後まで確認。実機 A401OP / Android 15 では、起動と画面まで確認。ログインより先は、まだ実機で確認していません)。 / Works (checked all the way on an emulator; on a real device, A401OP / Android 15, start-up and the screen are checked, but login and later steps are not yet checked on it). |
| macOS | まだ動かして確認していません(コンパイルまで)。 / Not run yet (only compiled). |
| iPhone | これからです。 / Not started. |

## しくみ(くわしく) / How it works (details)

| 部品 / Part | はたらき / Role |
|---|---|
| `crates/open-backup-core` | どの OS でも同じ中心部分(設定・認証・非公開リポジトリの確認と作成・保存・世代の切り替え・暗号化・復元・索引・AI 分類)。 / The core that is the same on every OS (settings, login, checking and creating private repositories, saving, switching generations, encryption, restore, index, AI topics). |
| `crates/open-backup-cli` (`open-backup`) | コマンドと、ずっと動くエンジン(フォルダの変化を見て、時間ごとに実行)。 / The command and the resident engine (watches folders and runs on a timer). |
| `tools/gen-ui` → `ui/` | 画面。RPoem の `open-runo-crossui` で、どの OS 向けも同じ元から作ります。 / The screen, made from one source for every OS with RPoem's `open-runo-crossui`. |
| `android/` | 画面(WebView)と、エンジンを動かし続ける前面サービスだけのアプリ。**git は要りません。** / An app that is only a screen (WebView) plus a foreground service that keeps the engine running. **No git needed.** |
| `installer/` | Windows のインストーラー(Inno Setup)。 / The Windows installer (Inno Setup). |

**保存の 2 つの方法 / Two ways to save**: 初期設定(`backend = "auto"`)では、`git` があれば `git`、なければ GitHub API だけ(Android はいつもこちら)を使います。どちらも結果は同じです。API の方法は、各ファイルの git 流のハッシュ(SHA-1)を求め、リポジトリの今の状態と比べて**違うものだけ**を送ります。途中で止まっても、別の端末から続けても、二重には送りません。GitHub の送りすぎ制限(二次レート制限)に合わせて、1 ファイルごとに約 0.7 秒あけます。
By default (`backend = "auto"`) it uses `git` if it exists, otherwise only the GitHub API (Android always does). The result is the same. The API way computes git-style hashes (SHA-1) for every file, compares them with the repository as it is now, and sends **only what differs**. Stopping halfway or continuing from another device never uploads anything twice. It waits about 0.7 seconds between files to respect GitHub's secondary rate limit.

**Android アプリ / The Android app**: 写真や書類を読むため、最初に「すべてのファイルへのアクセス」の許可をお願いします。画面の「おすすめフォルダ」(DCIM・Pictures・Documents・Download など)のボタンで、かんたんに足せます。作り方: `powershell -ExecutionPolicy Bypass -File android\build-android.ps1`。
To read your photos and documents, the app asks for "All files access" the first time. The "recommended folders" buttons (DCIM, Pictures, Documents, Download and more) add them easily. To build: `powershell -ExecutionPolicy Bypass -File android\build-android.ps1`.

---

## 確かめたこと / What we checked

**日本語**
- Windows: `git` の方法・GitHub API の方法とも、本物の GitHub で、最初の保存・変更だけの追加・削除・リポジトリの自動切り替えを確認。インストーラーは、静かなインストール → 画面が開く → アンインストールまで確認。
- 画面: ブラウザで、フォルダの追加と「今すぐバックアップ」、スマホの縦横・タブレット・ダークモード・大きい文字(横スクロールなし、下のボタンは固定)。
- Android: エミュレーターで「起動 → ログイン → フォルダを足す → 実行 → GitHub に入る」まで。実機(A401OP / Android 15)で、起動・画面・復元カード・版 0.2.0 の表示。
- Linux(WSL): 作る・試す、どちらも成功。 macOS: コンパイルのみ。
- **まだのこと**: 実機でのログイン以降、iPhone、macOS で動かすこと、Android の署名、GitHub 以外の保存先。

**English**
- Windows: on real GitHub, both the `git` way and the GitHub API way were checked for the first save, adding only changes, deleting, and automatic switching of repositories. The installer was checked from a silent install, to the screen opening, to uninstalling.
- Screen: in a browser, adding a folder and "Back up now", phone portrait and landscape, tablet, dark mode and large text (no sideways scrolling, bottom buttons stay in place).
- Android: on the emulator all the way from "start, log in, add a folder, run" to the files arriving on GitHub. On a real device (A401OP / Android 15): start-up, the screen, the restore card and the version 0.2.0 label.
- Linux (WSL): build and tests both succeed. macOS: compiled only.
- **Not done yet**: login and later steps on a real device, iPhone, running on macOS, Android signing, destinations other than GitHub.

---

## 引っ越し / Moving to another computer

**日本語**: このリポジトリだけで開発を続けられます。別のパソコンへ移るときは、[`PORTING.md`](PORTING.md) と [`CLAUDE.md`](CLAUDE.md) の HANDOFF を読んでください。保存先の GitHub リポジトリは、そのまま使えます。トークンはリポジトリに入っていないので、新しい場所でもう一度ログインしてください。
**English**: You can keep developing with just this repository. To move to another computer, read [`PORTING.md`](PORTING.md) and the HANDOFF in [`CLAUDE.md`](CLAUDE.md). The destination repositories on GitHub keep working. The token is not in the repository, so please log in again on the new computer.

```bash
git clone https://github.com/aon-co-jp/open-backup.git
cd open-backup
cargo test
```

## API のきまり / API rule

**日本語**: 新しい API は REST で作りません。ブラウザやコマンドから呼ぶものは Connect(`POST /<パッケージ>.<Service>/<Method>`、JSON)、サービス同士は gRPC / gRPC-web を使います。GitHub などの相手が形を決めるもの(Webhook・OAuth の戻り先・ヘルスチェックなど)だけは例外です。正本は [`runo`](https://github.com/aon-co-jp/runo) の API 方針です。このアプリが呼ぶ `aruaru-llm` は、すでに Connect 形式 `POST /aruaru_llm.v1.Llm/<Method>` を使っています。
**English**: New APIs are not built as REST. Calls from a browser or a command use Connect (`POST /<package>.<Service>/<Method>`, JSON), and services talk to each other with gRPC / gRPC-web. Only things whose shape another party decides (webhooks, OAuth return addresses, health checks and so on) are exceptions. The source of truth is the API policy in [`runo`](https://github.com/aon-co-jp/runo). The `aruaru-llm` that this app calls already uses the Connect form `POST /aruaru_llm.v1.Llm/<Method>`.

## ライセンスと作者 / License and author

MIT OR Apache-2.0

作者 / Author: 石 塚 正 浩（MASAHIRO ISHIZUKA） (https://github.com/aon-co-jp)

## Versionless API の方針 / Versionless API policy (2026-11-25)
**日本語**: このリポジトリのAPIは **Versionless**(URL・パッケージ名に `v1` 等を付けず、互換性を壊さず足すだけで進化させる)。欄は足すだけ・足す欄は省略可能・知らない欄は無視・名前変更は新名を足して旧を非推奨に(期限は既定で6か月以上、使用0を確認してから削除)・enumは未知の値を許容・安全上の緊急修正は例外。既存の `.v1` は呼び出し側を壊さないよう当面残し、版なしの別名を追加して段階的に畳む。詳細・出典は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md) 末尾。適用はこれから順に行う。
**English**: This repository's APIs are **Versionless**: no `v1` in URLs or package names; evolve by adding only, without breaking compatibility. Only add fields, new fields are optional, ignore unknown fields, rename by adding the new name and deprecating the old (default deadline 6 months or more, remove only after confirming zero use), tolerate unknown enum values, emergency security fixes are the exception. Existing `.v1` names stay for now so callers do not break; a versionless alias is added and `.v1` is retired in stages. Details and sources: the end of [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md). Rollout is step by step.

## API 方針: REST 全廃・gRPC / Connect へ
**日本語**: このリポジトリでは **REST API を新規に作らない**。ブラウザ・`curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`)、サービス間は **gRPC / gRPC-web**。GraphQL は読み取り専用。既存の REST は触るときに移し、移したら旧経路は消す。JSON は RS-JSON(`from_str_secure` / `parse_secure`)で読む。例外は相手の規格が形を決めるもの(ACME、OAuth/OIDC のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)のみ。迷ったら Connect。正本は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md)。
**English**: This repository **does not add new REST APIs**. APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`); service-to-service calls use **gRPC / gRPC-web**. GraphQL is read-only. Existing REST is migrated when touched and the old route is removed. Read JSON with RS-JSON (`from_str_secure` / `parse_secure`). Exceptions are only shapes dictated by someone else's standard (ACME, OAuth/OIDC callbacks, third-party webhooks, health checks, static files, OTLP). When in doubt, use Connect. Source of truth: the link above.

## 完成まで作り切る方針 / Finish everything policy (2026-11-27)
**日本語**: 全リポジトリで、**未確認は自動で確認し、未実施は実施し、未了は完成させる**(確認を求めず進める)。使用頻度が低いと判断される機能や、**1から作り直す大規模開発になる内容でも、やめずに最後まで開発・TEST・DEBUGする**(別の方式で済ませない、後回しにしない)。進め方: 調査(英語・日本語の検索とGitHub) → 設計 → 実装 → 実機/実データで確認 → 失敗は原因を直して再試験 → README・CLAUDE・PORTING・POLICY を日英で更新 → push。「できた」と書くのは実測した範囲だけ、確認できなかった所は「未確認」と正直に書く。例外(確認を取る): 元に戻せない削除、秘密の公開、所有者の認証・決済・実機操作が要るもの、利用者のシステム設定の変更。
**English**: In every repository, **verify what is unverified, do what is not yet done, and finish what is unfinished** automatically, without asking. Even features that look rarely used, or work that means **a large rewrite from scratch, are developed, tested and debugged to the end** (no substituting another approach, no postponing). Method: research (English and Japanese search plus GitHub) -> design -> implement -> verify on real devices or data -> fix causes and retest on failure -> update README, CLAUDE, PORTING and POLICY in Japanese and English -> push. Write "done" only for what was measured, and say "unverified" honestly for the rest. Exceptions that still need confirmation: irreversible deletions, publishing secrets, anything needing the owner's authentication, payment or real-device actions, and changes to the user's system settings.