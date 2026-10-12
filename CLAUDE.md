# open-backup 開発メモ

端末のデータを利用者自身のGitHub**非公開**リポジトリへ自動バックアップする。Rust(+RPoem)。詳細は README.md。

## HANDOFF(2026-10-12 シャットダウン時点)

- 実装済み: core(認証・非公開リポジトリ作成・ミラー/commit/push・世代切替・索引+AIトピック)、CLI(login/add/remove/status/once/run/index)。
- 検証済み: Windowsで実push・増分・自動切替。Linux(WSL Ubuntu)でrelease+テスト成功。macOSは `cargo check`(zig cc)のみ。
- AI分類: 同梱 `aruaru-llm` を端末で1つ共有(127.0.0.1のみ、tenant=open-backup)。埋め込みモデル `F:\aruaru-llm\models\multilingual-e5-small` は導入済み(git管理外)。
- 未了: (1) 画面(RPoem `open-runo-crossui`) (2) Android/iPhone (3) macOS実行確認 (4) open-easy-webでaruaru-llmの導入/起動管理 (5) GitHub Actions(3OS CI)は支払い設定の問題で起動不可 (6) インストーラー。
- テスト残骸: `aon-co-jp-backup` に非公開の `open-backup-selftest-0001〜0011`。ユーザーが不要時に削除(Claudeは削除しない)。
- 注意: リポジトリ作成の連打は二次レート制限(403)。テストは少数ずつ。Windowsで `Set-Content` はTOML等の文字コードを壊すので使わない。
- 再開手順: `F:\open-backup` で `cargo test`、WSLでLinuxビルド(`CARGO_TARGET_DIR=$HOME/ob-target`)。
