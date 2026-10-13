//! open-backup の画面を RPoem(open-runo-crossui)で生成する。
//! 使い方: `cargo run` → ../../ui/ に全OS共通のページ(index.html)と OS 別の起動ファイルを書く。
use open_runo_crossui::{generate, AppSpec, Bridge, Target, ALL_TARGETS};
use open_runo_view::{h, VNode};

const CSS: &str = r#":root{--bg:#fff;--fg:#14181f;--card:#f3f5f8;--acc:#1f6feb;--ok:#1a7f37;--ng:#c62828;--line:#d0d7de}
@media (prefers-color-scheme:dark){:root{--bg:#0d1117;--fg:#e6edf3;--card:#161b22;--acc:#4a9eff;--ok:#3fb950;--ng:#ff7b72;--line:#30363d}}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.5 system-ui,sans-serif}
main{max-width:720px;margin:0 auto;padding:16px 16px 112px}
h1{font-size:1.3rem;margin:0 0 4px}
.sub{color:#8b949e;margin:0 0 12px;font-size:.9rem}
.warn{border:1px solid var(--ng);border-radius:10px;padding:10px 12px;margin:12px 0;font-size:.9rem}
.card{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:12px 14px;margin:12px 0}
.card h2{font-size:1rem;margin:0 0 8px}
.row{display:flex;gap:8px;align-items:center;justify-content:space-between;padding:8px 0;border-top:1px solid var(--line)}
.row:first-of-type{border-top:0}
.row span{overflow-wrap:anywhere;min-width:0}
button{font:inherit;min-height:44px;padding:0 16px;border-radius:10px;border:1px solid var(--line);background:var(--card);color:var(--fg);cursor:pointer}
button.x{min-width:44px;padding:0 10px}
input{font:inherit;min-height:44px;width:100%;border-radius:10px;border:1px solid var(--line);background:var(--bg);color:var(--fg);padding:0 12px}
.add{display:flex;gap:8px;margin-top:8px}
.add button{flex:none}
#ok{color:var(--ok)}#ng{color:var(--ng)}
#bar{position:fixed;left:0;right:0;bottom:0;padding:12px 16px calc(12px + env(safe-area-inset-bottom));background:var(--bg);border-top:1px solid var(--line)}
#bar button{display:block;width:100%;max-width:720px;margin:0 auto;min-height:52px;background:var(--acc);border-color:var(--acc);color:#fff;font-weight:600}
#bar button:disabled{opacity:.6}
.guide{margin-top:10px;font-size:.9rem}.guide summary{cursor:pointer;font-weight:600;min-height:32px}.guide ol{padding-left:1.2rem}.guide li{margin:6px 0}.guide a{color:var(--acc);overflow-wrap:anywhere}.guide code{background:var(--bg);padding:1px 6px;border-radius:6px;overflow-wrap:anywhere}
.chips{display:flex;flex-wrap:wrap;gap:8px;margin-top:8px}.chips button{min-height:40px;font-size:.9rem}
#off{display:none;border:1px solid var(--ng);border-radius:10px;padding:10px 12px;margin:12px 0}"#;

const JS: &str = r#"(()=>{
const $=i=>document.getElementById(i);
const BASE="";
async function api(p,b){const r=await fetch(BASE+p,b===undefined?{}:{method:"POST",body:JSON.stringify(b)});if(!r.ok)throw new Error((await r.json().catch(()=>({}))).error||r.status);return r.json();}
function ago(t){if(!t)return"まだ実行していません / Not run yet";const s=Math.max(0,Math.floor(Date.now()/1000-t));return s<60?s+"秒前":s<3600?Math.floor(s/60)+"分前":Math.floor(s/3600)+"時間前";}
function render(s){
  $("off").style.display="none";
  $("ver").textContent="v"+s.version;
  $("login").textContent=s.login?("GitHub: "+s.login):"GitHub: 未ログイン / not logged in (open-backup login)";
  $("loginbox").hidden=!!s.login;
  const sg=$("sugg");sg.textContent="";
  (s.suggested||[]).forEach(f=>{const b=document.createElement("button");b.textContent="+ "+f.split(/[\\/]/).pop();b.title=f;b.onclick=async()=>{try{render(await api("/add",{folder:f}));}catch(e){err(e);}};sg.appendChild(b);});
  $("repo").textContent=s.repo+"(非公開 / private)";$("gname").textContent=s.repo;
  const l=$("folders");l.textContent="";
  if(!s.folders.length){const d=document.createElement("div");d.className="row";d.textContent="対象フォルダがありません / No folders yet";l.appendChild(d);}
  s.folders.forEach(f=>{const d=document.createElement("div");d.className="row";const sp=document.createElement("span");sp.textContent=f;const b=document.createElement("button");b.className="x";b.textContent="×";b.setAttribute("aria-label","削除 / remove");b.onclick=async()=>{try{render(await api("/remove",{folder:f}));}catch(e){err(e);}};d.append(sp,b);l.appendChild(d);});
  const r=$("res");r.className=s.last.at?(s.last.ok?"ok":"ng"):"";r.textContent=(s.running?"実行中… / running… ":"")+(s.last.at?(ago(s.last.at)+": "+s.last.msg):ago(0));
  $("run").disabled=s.running||!s.folders.length||!s.login;
  $("run").textContent=s.running?"バックアップ中… / Backing up…":"今すぐバックアップ / Back up now";
}
function err(e){$("res").className="ng";$("res").textContent=String(e.message||e);}
async function tick(){try{render(await api("/status"));}catch(e){$("off").style.display="block";}}
$("run").onclick=async()=>{try{await api("/run",{});tick();}catch(e){err(e);}};
$("addb").onclick=async()=>{const v=$("addf").value.trim();if(!v)return;try{render(await api("/add",{folder:v}));$("addf").value="";}catch(e){err(e);}};
$("addf").addEventListener("keydown",e=>{if(e.key==="Enter")$("addb").click();});
$("loginb").onclick=async()=>{const v=$("tok").value.trim();if(!v)return;try{render(await api("/login",{token:v}));$("tok").value="";}catch(e){err(e);}};
$("tok").addEventListener("keydown",e=>{if(e.key==="Enter")$("loginb").click();});
tick();setInterval(tick,3000);
})();"#;

fn txt(s: &str) -> VNode {
    VNode::Text(s.into())
}

fn root() -> VNode {
    h("main")
        .child(h("h1").child(txt("open-backup ")).child(h("small").attr("id", "ver").build()).build())
        .child(h("p").attr("class", "sub").child(txt("端末のデータを、あなたのGitHub非公開リポジトリへ自動バックアップ / Auto-backup to your private GitHub repository")).build())
        .child(h("div").attr("id", "off").child(txt("エンジンに接続できません。open-backup serve を起動してください。 / Cannot reach the engine. Start `open-backup serve`.")).build())
        .child(h("div").attr("class", "warn").child(txt("⚠ 保存先のGitHubリポジトリは必ず非公開(Private)にしてください。公開すると全データが世界中に見えます。 / Keep the GitHub repository PRIVATE. Making it public exposes all your data.")).build())
        .child(
            h("div").attr("class", "card").attr("id", "loginbox").attr("hidden", "")
                .child(h("h2").child(txt("GitHub にログイン / Log in to GitHub")).build())
                .child(h("p").attr("class", "sub").child(txt("最小権限のトークン(保存先の非公開リポジトリ1つだけに「Contents: 読み書き」)を、ご自身で作って貼り付けてください。作り方は README を参照。 / Create a minimal-permission token yourself (Contents: read & write on the one private repository) and paste it here. See the README.")).build())
                .child(
                    h("div").attr("class", "add")
                        .child(h("input").attr("id", "tok").attr("type", "password").attr("placeholder", "ghp_… / github_pat_…").attr("autocomplete", "off").attr("spellcheck", "false").build())
                        .child(h("button").attr("id", "loginb").child(txt("ログイン / Log in")).build())
                        .build(),
                )
                .child(
                    h("details").attr("class", "guide")
                        .child(h("summary").child(txt("はじめての方へ(GitHub のアカウントが無い方も) / New here? (no GitHub account yet)")).build())
                        .child(
                            h("ol")
                                .child(h("li")
                                    .child(txt("GitHub のアカウントが無い場合は、無料で作ります。アカウントの作成とメールアドレスの確認は、ご自身で GitHub の画面で行います(このアプリは代わりに作れません)。 / If you have no GitHub account, create a free one. You sign up and confirm your email yourself on GitHub; this app cannot do it for you. "))
                                    .child(h("a").attr("href", "https://github.com/signup").attr("target", "_blank").attr("rel", "noopener").child(txt("github.com/signup")).build())
                                    .build())
                                .child(h("li")
                                    .child(txt("保存先の非公開(Private)リポジトリを作ります。名前は、下の「アカウント・保存先」に出ている名前にしてください: / Create a private repository named exactly as shown under \"Account & destination\" below: "))
                                    .child(h("code").attr("id", "gname").build())
                                    .child(txt(" (容量がいっぱいになったとき自動で切り替えるため、末尾が -0002 のリポジトリも作っておくと安心です / also create the -0002 one so it can switch automatically when the first fills up) "))
                                    .child(h("a").attr("href", "https://github.com/new").attr("target", "_blank").attr("rel", "noopener").child(txt("github.com/new")).build())
                                    .build())
                                .child(h("li")
                                    .child(txt("そのリポジトリだけに「Contents: 読み書き(Read and write)」を許可した Fine-grained トークンを作ります(有効期限は短めに)。 / Create a fine-grained token limited to those repositories with \"Contents: Read and write\" only (short expiry). "))
                                    .child(h("a").attr("href", "https://github.com/settings/personal-access-tokens/new").attr("target", "_blank").attr("rel", "noopener").child(txt("github.com/settings/personal-access-tokens/new")).build())
                                    .build())
                                .child(h("li").child(txt("作ったトークンを上の欄に貼り付けて「ログイン」。トークンは他人に見せないでください。 / Paste the token above and press Log in. Never share it.")).build())
                                .build(),
                        )
                        .child(h("p").attr("class", "sub").child(txt("GitHub は大容量のバックアップ向けではありません。1ファイルは 100MB(このアプリは 40MB)まで、リポジトリ全体は数GBまでが目安です。 / GitHub is not meant for huge backups: files up to 100 MB (this app: 40 MB) and a few GB per repository.")).build())
                        .build(),
                )
                .build(),
        )
        .child(
            h("div").attr("class", "card")
                .child(h("h2").child(txt("アカウント・保存先 / Account & destination")).build())
                .child(h("div").attr("class", "row").child(h("span").attr("id", "login").build()).build())
                .child(h("div").attr("class", "row").child(h("span").attr("id", "repo").build()).build())
                .build(),
        )
        .child(
            h("div").attr("class", "card")
                .child(h("h2").child(txt("バックアップするフォルダ / Folders")).build())
                .child(h("div").attr("id", "folders").build())
                .child(h("div").attr("id", "sugg").attr("class", "chips").build())
                .child(
                    h("div").attr("class", "add")
                        .child(h("input").attr("id", "addf").attr("placeholder", "フォルダのパス / Folder path").attr("autocomplete", "off").attr("spellcheck", "false").build())
                        .child(h("button").attr("id", "addb").child(txt("追加 / Add")).build())
                        .build(),
                )
                .build(),
        )
        .child(
            h("div").attr("class", "card")
                .child(h("h2").child(txt("直近の結果 / Last result")).build())
                .child(h("div").attr("id", "res").build())
                .build(),
        )
        .child(h("div").attr("id", "bar").child(h("button").attr("id", "run").child(txt("今すぐバックアップ / Back up now")).build()).build())
        .build()
}

fn main() -> std::io::Result<()> {
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui");
    let app = AppSpec {
        id: "open-backup".into(),
        name: "open-backup".into(),
        version: "0.1.0".into(),
        engine_url: "http://127.0.0.1:4620".into(),
        bridge: Bridge::default(),
        root: root(),
        css: CSS.into(),
        script: Some(JS.into()),
        daemon: "open-backup".into(),
    };
    let files = generate(&app, ALL_TARGETS, &out)?;
    // CLI が埋め込む共通ページ(同一オリジン用)
    std::fs::copy(out.join(Target::Web.dir()).join("index.html"), out.join("index.html"))?;
    for f in files {
        println!("{}", f.display());
    }
    Ok(())
}
