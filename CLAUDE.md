# open-backup 開発メモ

端末のデータを利用者自身のGitHub**非公開**リポジトリへ自動バックアップする。Rust(+RPoem)。詳細は README.md。

## HANDOFF(2026-10-12 シャットダウン時点)

- 実装済み: core(認証・非公開リポジトリ作成・ミラー/commit/push・世代切替・索引+AIトピック)、CLI(login/add/remove/status/once/run/index)。
- 検証済み: Windowsで実push・増分・自動切替。Linux(WSL Ubuntu)でrelease+テスト成功。macOSは `cargo check`(zig cc)のみ。
- AI分類: 同梱 `aruaru-llm` を端末で1つ共有(127.0.0.1のみ、tenant=open-backup)。埋め込みモデル `F:\aruaru-llm\models\multilingual-e5-small` は導入済み(git管理外)。
- 画面: RPoem crossui で生成済み(`tools/gen-ui`→`ui/`)、`open-backup serve` で配信、実機のブラウザ確認済み。
- 未了: (1) Android/iPhone のネイティブシェル(WebView) (2) Android/iPhone (3) macOS実行確認 (4) open-easy-webでaruaru-llmの導入/起動管理 (5) GitHub Actions(3OS CI)は支払い設定の問題で起動不可 (6) インストーラー。
- テスト残骸: `aon-co-jp-backup` に非公開の `open-backup-selftest-0001〜0011`。ユーザーが不要時に削除(Claudeは削除しない)。
- 注意: リポジトリ作成の連打は二次レート制限(403)。テストは少数ずつ。Windowsで `Set-Content` はTOML等の文字コードを壊すので使わない。
- 再開手順: `F:\open-backup` で `cargo test`、WSLでLinuxビルド(`CARGO_TARGET_DIR=$HOME/ob-target`)。

## HANDOFF(2026-10-13 追記)

- **Android アプリ完成(エミュレーターで通し確認)**: `android/`(WebView + 前面サービス `EngineService` が `libopen_backup.so`=Rustのエンジンを起動)。git が無い端末のため、**GitHub Data API だけで保存する経路**(`crates/open-backup-core/src/apisync.rs`)を追加。`backend = "auto"`(git があれば git、Android は常に API)。
  - `git2` ではなくこの方法にした理由: Android 向けの OpenSSL が、この PC に `perl`/`make` が無くビルドできないため。API の経路は ureq+rustls だけで、Android(arm64・x86_64)の `cargo ndk` ビルドが通る。
  - 状態: `api-state/<repo>.json`(前回のコミット・リポジトリの木・ファイルのSHAの控え)。リポジトリの先頭が自分の記録と違えば、木を取り直して基準にする。空のリポジトリは Contents API で最初のコミットを作る(Git Data API は空のリポジトリで使えない)。
  - トークン: デスクトップは OS の資格情報ストア(読み戻しで確認)、無ければ/Android はデータ領域の `token` ファイル(0600)。`OPEN_BACKUP_HOME` で設定・データの置き場所を1か所にできる(Androidは `filesDir/ob`)。
  - エンジンに `/login` `/logout`、画面にログイン欄とおすすめフォルダのボタン。
- **試験**: Windows で API 経路(初回 → 変更なし → 変更+削除+追加)、Android x86_64 エミュレーター(Android 16)で通し。**未確認**: 実機・arm64 の実行、APK のリリース署名、iPhone、macOS の実行、3OS の CI(支払い設定)、AI 分類(Android には `aruaru-llm` が無いので、拡張子の分類だけ)。
- 再開: `android\build-android.ps1`(Gradle は `~\.gradle\wrapper\dists` のものを `-Gradle` で指定。JDK は Android Studio の `jbr`)。エミュレーターは `Pixel_9_Pro`(x86_64)。**ユーザーの実機が adb につながっていることがある(ZY22…)ので、`-s emulator-5554` を必ず付ける**。PowerShell の `Remove-Item` は、F: のパスと同じ命令にあると保護で止まる(絶対パスで分けるか、`[IO.File]::Delete`)。
- **Android の落とし穴(2026-10-14)**: 画面(`ui/index.html`)は、Rust のエンジンに `include_str!` で埋め込まれている。画面を変えたら、**必ず `android\build-android.ps1` でエンジン(cargo ndk)から作り直す**(Gradle の `assembleDebug` だけでは、アプリの中のエンジンが古いまま)。更新したら、`adb forward tcp:4620 tcp:4620` で `/ui` を取って、変更が入ったか確かめる(Windows のコンソールの日本語は文字化けするので、UTF-8 で読む)。
  設定画面(すべてのファイルへのアクセス)がアプリのタスクの上に残っていると、`am start` が「前面の画面に渡しただけ」になり起動しない。`am start -n tokyo.aon.openbackup/.MainActivity -f 0x10008000`(NEW_TASK|CLEAR_TASK)で起動する。
