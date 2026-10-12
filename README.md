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
| 画面(RPoem crossui) | Windows/macOS/Linux/Android 向けのUI | 未着手 |
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
- 元で削除したファイルはバックアップ側からも消えますが、gitの履歴には残ります。

## ライセンス / License

MIT OR Apache-2.0
