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

## 使い方 / Usage

```bash
open-backup login --token     # 個人用トークン(repoスコープ)を標準入力から。または client_id を設定してデバイス認可
open-backup add <フォルダ>     # バックアップ対象を追加
open-backup once              # 1回だけ実行
open-backup run               # 常駐(変更検知 + 定期実行)
```

- 保存先: ログインしたアカウントの `open-backup-<端末名>`(非公開)。
- アカウントの新規作成は GitHub 側で利用者自身が行います(本ツールは作成しません)。
- トークンはOSの資格情報ストアに保存し、gitのコマンドラインには出しません。
- 100MB超のファイルはGitHubの制限により対象外(既定95MB超をスキップ)。
- 元で削除したファイルはバックアップ側からも消えますが、gitの履歴には残ります。

## ライセンス / License

MIT OR Apache-2.0
