# open-backup 開発メモ (CLAUDE.md) / Development notes

端末のデータ(指定フォルダ)を、利用者自身の GitHub **非公開**リポジトリへ自動でバックアップする。Rust(コア・エンジン)+画面は RPoem(`open-runo-crossui`)で生成+ Android アプリ。詳細は [`README.md`](README.md)、**引っ越し・再開は [`PORTING.md`](PORTING.md)**。
*Automatically backs up chosen folders to the user's own **private** GitHub repository. Rust core/engine, UI generated with RPoem, plus an Android app. See README.md; for moving to another machine / resuming, read PORTING.md.*

`aruaru-IME` とは**別のプロジェクト**(混同しない)。作業場所は `F:\open-backup`(`F:\runo` 配下ではなく兄弟ディレクトリ)。
*This is a **different project** from aruaru-IME. Working dir: `F:\open-backup` (a sibling of `F:\runo`).*

## 方針 / Policy

- GitHub は必ず**非公開**。公開リポジトリが保存先なら、データを送らず中止する。README に日英で警告を載せる。
- アカウントは作らない(利用者が自分で作る)。トークンは、最小権限(保存先のリポジトリだけに Contents の読み書き)を、利用者が自分で作る。値を画面・ログ・コミットに出さない。
- 個人データを外へ出さない(AI 分類は端末内の `aruaru-llm` だけ、送り先は 127.0.0.1 固定)。
- `aruaru-llm` は端末で 1 つだけ(既存を探して相乗り、tenant=`open-backup`)。
- 汎用化できる部分は RPoem へ(画面の生成は RPoem の `open-runo-crossui`)。Python は使わない(Rust)。
- 画面・インストーラーの試験ルール(複数の解像度・縦横・文字サイズ、下部ボタン固定)は、`open-raid-z/CLAUDE.md` の「運用ルール」。

## HANDOFF(2026-10-14 時点) / State

- **実装済み / Done**: コア(認証・非公開リポジトリの確認/作成・保存・世代の切替・索引+AI の分類)、**2 つの保存方法**(`git` の経路 `sync.rs`、GitHub Data API だけの経路 `apisync.rs`。`backend="auto"`)、CLI(`login` `add` `remove` `status` `once` `run` `serve` `index`)、常駐エンジン `serve`(127.0.0.1:4620、画面 `/ui`、`/login` `/logout` を含む)、画面(RPoem で生成、ログイン欄・おすすめフォルダ・「はじめての方へ」の日英の案内)、Android アプリ(`android/`)。
- **確認済み / Verified**: Windows の git 経路・API 経路(実 GitHub)、画面(ブラウザ)、Android x86_64 エミュレーターで起動→ログイン→フォルダ追加→実行→GitHub に保存、Android 実機(moto g53y 5G、Android 14)にアプリを導入してエンジンが起動、Linux(WSL)のビルドとテスト、macOS は `cargo check` のみ。
- **未了 / Open**: 実機でログイン以降の通し確認(ユーザーが実機で試す)、iPhone、macOS の実行、3 OS の CI(GitHub の支払い設定)、Android の署名、`open-easy-web` による `aruaru-llm` の管理、インストーラー、GitHub 以外の保存先。詳細は PORTING.md の「未了」。
- **テスト残骸 / Test leftovers**: `aon-co-jp-backup` に非公開の `open-backup-selftest-0001〜0011`。**削除はユーザーに任せる**(Claude は削除しない)。

## 注意(環境固有) / Pitfalls

- PowerShell 5.1: BOM の無い UTF-8 を読み間違える(日本語入りの `.ps1` は BOM 付き)。`Set-Content` は TOML を壊す。`Remove-Item` は F: のパスと同じ命令にあると保護で止まる(分ける)。
- リポジトリの作成・内容の作成を短時間に連打すると、二次レート制限(403)。テストは少数ずつ。API の経路は 1 ファイルごとに約 0.7 秒あける。
- **Android**: 画面(`ui/index.html`)は Rust のエンジンに `include_str!` で埋め込まれる。画面やコアを変えたら、**必ず `android\build-android.ps1` で、エンジンから作り直す**(Gradle の `assembleDebug` だけでは、アプリの中のエンジンが古いまま)。
- **adb**: ユーザーの実機(`<実機のシリアル>`)がつながっていることがある。エミュレーターは `-s emulator-5554`。**実機への導入・更新は、ユーザーの許可を得てから**。`am start` は、設定画面がアプリのタスクの上に残っていると起動しない(`-f 0x10008000`)。
- 非 ASCII を含む `perl -e`、Bash のヒアドキュメントの `\`・`\u`・`\t` は、環境で壊れることがある。編集は Edit/Write ツールで。
- テスト用トークン(この開発機のみ): `F:\backup-token.txt`(`aon-co-jp-backup`)。実際のトークンは、OS の資格情報ストアに残さない(画面の `/login` に実トークンを入れない。環境変数 `OPEN_BACKUP_TOKEN` で渡す)。

## ユーザーの方針(記録) / User policies

- 「ダウンロードするものは利用者が選んだものだけ」「GitHub は非公開限定」「アカウント作成・最小権限トークンの作り方は利用者が自分で調べる(README に日英で最低限の手順)」「Rust+RPoem で作る」「実機への導入は許可を得てから」。

## 公開(2026-10-14) / Public release

- **リポジトリを公開**し、リリース v0.1.0(プレリリース)を出した。紹介ページ: https://easy-web.tokyo/open-backup/ 。詳細は PORTING.md の「公開・配布・VPS」。
- 公開の前に、履歴の検査と、配布物のユーザー名の検査をした(`--remap-path-prefix`)。**今後も、リリースの前に、配布物に自分のユーザー名・パス・トークンが無いか確かめる**。履歴を書き換えたのは 1 回(`tools/gen-ui/target` の除去)。
- VPS: `/root/easy-web.tokyo/open-backup`、`open-backup-web.service`(127.0.0.1:8116)、`domains.toml` の経路。変更の前後で、全ホストの状態を比べる。
