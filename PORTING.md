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

> 2026-11-19: v0.2.0 を実機(A401OP / Android 15)にも導入し、起動・画面・復元カード・版表記 0.2.0 を確認。ログイン以降は利用者の作業。*Installed v0.2.0 on a real device (A401OP / Android 15): start, UI, restore card and version 0.2.0 verified; login onward is for the owner.*

> 2026-11-16: v0.2.0 公開(暗号化・復元入り。Windows インストーラー・exe・Linux・Android APK)。Windows インストーラーの無人インストール→`/ui` 200→アンインストール、Android x86_64 エミュレーターで起動・画面(復元カード)・状態を確認。**実機のログイン以降(トークン入力が要る)・iPhone・macOS 実行・Android 署名・GitHub 以外の保存先は未了**。

1. 実機(moto g53y 5G、Android 14)でのログイン → フォルダ追加 → 「今すぐバックアップ」の通し確認(アプリは導入済み・未ログイン。エミュレーターでは通っている)。
2. iPhone 版。macOS の実行確認。3 OS の CI(`.github/workflows/ci.yml` は用意済み。GitHub アカウントの支払い設定が直れば動く)。
3. Android の公開用の署名、アップデートの配布。Android での AI 分類(`aruaru-llm` が無いので、拡張子の分類だけ)。
4. `open-easy-web` による `aruaru-llm` の導入・起動の管理。
5. ~~インストーラー(Windows)~~ 完了(2026-10-16): `installer/open-backup.iss`(Inno Setup、ユーザー単位・管理者不要、ログイン時の自動起動は任意)、`scripts/build-installer.ps1`。無人でのインストール→`/ui`が200→アンインストール(ファイル・ショートカット削除)を確認済み。Done 2026-10-16: per-user Inno Setup installer; silent install → `/ui` 200 → uninstall verified.
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

## 公開・配布・VPS / Release, distribution and the VPS (2026-10-14)

- **リポジトリは公開**(https://github.com/aon-co-jp/open-backup。保存先のリポジトリが非公開であることとは別)。公開前に、履歴を検査し、誤って commit されていたビルド生成物(`tools/gen-ui/target`)と実機のシリアルを履歴から取り除いた(**コミットの ID が変わった**)。トークンの形の文字列・ユーザー名のパスは、履歴に無いことを確認済み。
  *The repository is public (separate from the private destination repositories). Before publishing the history was scanned and build artifacts (`tools/gen-ui/target`) and a device serial were removed from history (commit IDs changed); no token-like strings or user-name paths remain.*
- **リリース**: v0.1.0(プレリリース)。配布物 = Android の APK(開発用の署名)、Windows の exe、Linux のバイナリ、`SHA256SUMS.txt`。作り方: `powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1 -Version 0.1.0 -Gradle <gradle.bat>`(`dist\` に出る。`--remap-path-prefix` で、ユーザー名・cargo のパス・リポジトリのパスをバイナリに残さない。**公開の前に、必ず `noruk` のような自分の名前が配布物に無いかを確かめる**)。`gh release create v0.1.0 --prerelease ...` で公開。
  *Release v0.1.0 (pre-release): debug-signed APK, Windows exe, Linux binary, SHA256SUMS. Build with `scripts\build-release.ps1`, which remaps local paths; always verify your own user name is absent from the artifacts before publishing.*
- **VPS(ConoHa)**: 紹介ページ https://easy-web.tokyo/open-backup/ 。`/root/easy-web.tokyo/open-backup`(GitHub から clone。更新は `git pull --ff-only`)の `webpage/` を、`open-backup-web.service`(`python3 -m http.server 8116 --bind 127.0.0.1`。`make-disk` などと同じ方式)が配信し、`open-web-server` の `domains.toml`(`/root/repository/open-web-server/domains.toml`)の `path_prefix = "/open-backup"` → `127.0.0.1:8116` で公開する。ダウンロードのリンクは GitHub のリリースを指す(VPS には、バイナリを置かない)。
  設定を変えたときは `open-web-server` の再起動が要る(他のサイトが一瞬止まる)。バックアップは `domains.toml.bak-20261014-open-backup`。変更の前後で、全ホスト・主要な経路の HTTP の状態が同じことを確かめる(2026-10-14 は、32 項目すべて同じだった)。`easyweb.tokyo`(ハイフン無し)は、他のページと同じく、ルートだけで、`/open-backup` は `easy-web.tokyo` 側。
  *Landing page at https://easy-web.tokyo/open-backup/, served from `/root/easy-web.tokyo/open-backup/webpage` by `open-backup-web.service` (static, 127.0.0.1:8116) through a route in open-web-server's `domains.toml`. Downloads link to the GitHub release (no binaries on the VPS). Changing the route needs an open-web-server restart; the backup is `domains.toml.bak-20261014-open-backup`. `easyweb.tokyo` (no hyphen) serves only the root, like the other pages.*
- **更新の手順 / To update**: ページを直す → `webpage/index.html` を commit・push → VPS で `cd /root/easy-web.tokyo/open-backup && git pull --ff-only`(サービスの再起動は不要)。新しい版を出す → `build-release.ps1` → 新しいタグで `gh release create` → `webpage/index.html` のリンク・サイズ・SHA-256 を直す。

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

## aruaru-llm 呼び出しの Connect 化 (2026-11-05)

- aruaru-llm の REST 全廃に伴い、本リポジトリの呼び出し先を `POST /aruaru_llm.v1.Llm/<Method>`(JSON本文)へ変更した(旧 `/v1/...` は aruaru-llm 側で削除済み)。
- 対象は aruaru-llm を呼ぶ箇所のみ。本リポジトリ自身のAPI・他サービス(aruaru-search 等)の呼び出しは別途。
- 未検証: ビルド/実サーバー往復(aruaru-llm の `connect-migration` ブランチが VPS 未反映のため)。aruaru-llm の main マージ・VPS反映と同時に反映すること。
