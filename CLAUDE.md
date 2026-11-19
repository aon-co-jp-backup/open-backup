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

## HANDOFF(2026-11-19) v0.2.0 公開・実機導入 / v0.2.0 released, installed on a device

**日本語**: v0.2.0 を公開(暗号化・復元・Windowsインストーラー・Linux・Android APK、`SHA256SUMS-0.2.0.txt`)。紹介ページ(VPS `/root/easy-web.tokyo/open-backup`)も同版へ。確認済み: Windowsインストーラーの無人インストール→`/ui` 200→アンインストール、Android x86_64 エミュレーターと**実機(A401OP / Android 15、前回記録のmoto g53yとは別の端末)**で起動・画面・復元カード・`/status`、アプリ内の版表記 0.2.0。**未了**: 実機でのログイン以降(トークン入力が要るため利用者が行う)、iPhone、macOS の実行、Android の署名、GitHub 以外の保存先、`open-easy-web` による `aruaru-llm` 管理。**注意**: `build-release.ps1` は WSL の失敗で止まる(以前は古い Linux 版が黙って使われた)。Android の版は `android/app/build.gradle` の `versionName`/`versionCode` も上げる。
**English**: v0.2.0 is released (encryption, restore, Windows installer, Linux, Android APK, `SHA256SUMS-0.2.0.txt`) and the intro page on the VPS matches. Verified: silent install -> `/ui` 200 -> uninstall of the Windows installer; start, UI, restore card and `/status` on the x86_64 emulator and on a **real device (A401OP / Android 15, not the moto g53y of the earlier notes)**; in-app version 0.2.0. **Open**: login onward on a real device (needs a token, so the owner does it), iPhone, running on macOS, Android signing, non-GitHub destinations, managing `aruaru-llm` via `open-easy-web`. **Notes**: `build-release.ps1` now stops on a WSL failure (it used to reuse a stale Linux binary silently); also bump `versionName`/`versionCode` in `android/app/build.gradle` for Android.

## HANDOFF(2026-10-14 時点) / State

- **実装済み / Done**: コア(認証・非公開リポジトリの確認/作成・保存・世代の切替・索引+AI の分類)、**2 つの保存方法**(`git` の経路 `sync.rs`、GitHub Data API だけの経路 `apisync.rs`。`backend="auto"`)、CLI(`login` `add` `remove` `status` `once` `run` `serve` `index`)、常駐エンジン `serve`(127.0.0.1:4620、画面 `/ui`、`/login` `/logout` を含む)、画面(RPoem で生成、ログイン欄・おすすめフォルダ・「はじめての方へ」の日英の案内)、Android アプリ(`android/`)。
- **確認済み / Verified**: Windows の git 経路・API 経路(実 GitHub)、画面(ブラウザ)、Android x86_64 エミュレーターで起動→ログイン→フォルダ追加→実行→GitHub に保存、Android 実機(moto g53y 5G、Android 14)にアプリを導入してエンジンが起動、Linux(WSL)のビルドとテスト、macOS は `cargo check` のみ。
- **未了 / Open**: 実機でログイン以降の通し確認(ユーザーが実機で試す)、iPhone、macOS の実行、3 OS の CI(GitHub の支払い設定)、Android の署名、`open-easy-web` による `aruaru-llm` の管理、GitHub 以外の保存先。(Windows インストーラーは 2026-10-16 完了: `installer/open-backup.iss`、`scripts/build-installer.ps1`、未公開=リリースへの添付はユーザー確認後)詳細は PORTING.md の「未了」。
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

## HANDOFF(2026-10-26) 暗号化・圧縮・並列・復元 / Encryption, compression, parallelism, restore

- **実装**: `crates/open-backup-core/src/crypt.rs`(暗号化・復号・圧縮・目録・鍵)、`restore.rs`(復元・一覧・元の場所へ)、`sync.rs`(`encrypt=true` なら平文を作業コピーに置かず `crypt::sync_into`)、CLI(`crypt-init` / `restore`)、エンジン(`/restore/list` `/restore/run` `/restore/status`、127.0.0.1専用)、画面(`tools/gen-ui` に「復元」カード。再生成は `cd tools/gen-ui && cargo run`)。
- **形式 v2**: `OBE2` | nonce(19) | { 長さ(u32 LE) | 暗号文 }。チャンクの nonce = ファイルnonce | 番号(u32 BE) | 最後なら1(STREAM と同じ構成。チャンク単位で並列に処理でき、切り詰めは「最後」の印で検出)。チャンクの中身は [モード 0=そのまま/1=zstd] + データ。AAD = `<ラベル>/<パス>`(ファイルの入れ替えを検出)。ファイルnonce = BLAKE3 鍵つきハッシュ(パス + 内容)=決定的(変更無しなら暗号文も同一)。
- **試験**: `cargo test -p open-backup-core --release`(単体9 + 通し1: 暗号化→復元→内容一致・名前が平文で出ない・変更無しで差分0・改ざん1件だけ失敗・平文リポジトリへの書き込み拒否)。ベンチ: `cargo test -p open-backup-core --release bench_throughput -- --ignored --nocapture`。
- **未了 / Open**: (1) ~~GitHub への実 push を通した通し確認~~ **済み(2026-10-26)**: `tests/e2e_github.rs`(手動・`#[ignore]`)で git 方式・API 方式とも、暗号化→実 GitHub→取得→復元→一致、非公開、平文の名前なしを確認。テスト用リポジトリ `open-backup-e2e-{git,api}-<時刻>-000{1,2}` が `aon-co-jp-backup` に残る(**削除は利用者が行う**)。(2) ~~API 方式対応~~ **済み**(暗号化したステージング `data_dir/api-enc/<repo>` を API で送る)。この確認で、API 方式の既存の不具合(空のリポジトリの初期化の URL に `contents/` が無く、常に 404)を見つけて直した。(3) 画面(復元カード)のブラウザでの目視確認(エンジンの窓口は curl で確認済み)。(4) 実機のオフィス運用向け: 大きいファイルの分割、保存期間のルール、失敗の通知、複数台の一括管理、起動後に遅らせて実行。
- **注意**: 暗号化はパスフレーズを失うと復元できない。`crypt-init` は新しい salt を作るので、既に暗号化済みのリポジトリへ別の salt では書き込まない(拒否する)。

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
