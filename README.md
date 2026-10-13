# open-backup

端末のデータ(指定フォルダ)を、利用者自身のGitHubアカウントの**非公開リポジトリ**へ自動でバックアップします。
Automatically backs up folders you choose on your device to a **private repository** in your own GitHub account.

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
| Android アプリ | `android/`(WebView + 前面サービスで Rust のエンジンを起動。**git 不要**、GitHub API だけで保存) | 実装済み(エミュレーターで通し確認。実機は未確認) |
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

## 動作確認の記録 / Verification log (2026-10-12)

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
- 確認済み(Android 16 / x86_64 エミュレーター): 起動 → エンジン起動 → ログイン → `/storage/emulated/0/Documents` のフォルダを追加 → 「今すぐバックアップ」で、`a.txt` と 700KB の `photo.jpg` が非公開リポジトリに入る。**実機・arm64 の実行は未確認**(arm64 のエンジンのビルドは成功)。
