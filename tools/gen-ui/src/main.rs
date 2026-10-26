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
.rlist{max-height:260px;overflow:auto;border:1px solid var(--line);border-radius:10px;margin-top:8px}.rlist .row{cursor:pointer;padding:8px 10px}.rlist .row small{color:#8b949e;white-space:nowrap}.chk{display:flex;gap:8px;align-items:center;min-height:44px}.chk input{width:22px;min-height:22px;height:22px}#rmsg.ng{color:var(--ng)}#rmsg.ok{color:var(--ok)}#off{display:none;border:1px solid var(--ng);border-radius:10px;padding:10px 12px;margin:12px 0}"#;

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

const R={};
async function post(p,b){const r=await fetch(BASE+p,{method:"POST",body:JSON.stringify(b)});const j=await r.json().catch(()=>({}));if(!r.ok){const e=new Error(j.error||r.status);e.need=!!j.need_pass;throw e;}return j;}
function fmt(n){return n<1024?n+" B":n<1048576?(n/1024).toFixed(1)+" KB":n<1073741824?(n/1048576).toFixed(1)+" MB":(n/1073741824).toFixed(2)+" GB";}
function rbody(x){return Object.assign({gen:Number($("rgen").value)||1,github:$("rgh").checked,pass:$("rpass").value},x||{});}
function rmsg(t,c){const m=$("rmsg");m.className=c||"";m.textContent=t;}
function mkrow(main,sub,onclick){const d=document.createElement("div");d.className="row";const a=document.createElement("span");a.textContent=main;const b=document.createElement("small");b.textContent=sub||"";d.appendChild(a);d.appendChild(b);d.onclick=onclick;return d;}
$("rload").onclick=async()=>{
  rmsg("一覧を読み込み中… / Loading…");$("rlist").textContent="";
  try{
    const r=await post("/restore/list",rbody({q:$("rq").value.trim()}));
    rmsg((r.encrypted?"🔒 暗号化されたバックアップ / Encrypted — ":"")+r.total+" 件・"+fmt(r.total_bytes)+"(クリックで復元対象を選択 / click to choose)","ok");
    const L=$("rlist");
    r.labels.forEach(l=>L.appendChild(mkrow("📁 "+l.label+"/(すべて / all)",l.count+" 件・"+fmt(l.bytes),()=>{$("ronly").value=l.label+"/";})));
    r.items.forEach(i=>L.appendChild(mkrow(i.rel,fmt(i.size)+(i.topic?" · "+i.topic:""),()=>{$("ronly").value=i.rel;})));
  }catch(e){rmsg(e.message,"ng");if(e.need)$("rpass").focus();}
};
$("rq").addEventListener("keydown",e=>{if(e.key==="Enter")$("rload").click();});
async function rpoll(){
  try{
    const j=await api("/restore/status");
    if(j.running){rmsg("復元中… / Restoring… → "+j.dest);setTimeout(rpoll,1000);return;}
    if(j.done){
      rmsg(j.msg+"("+fmt(j.bytes)+")→ "+j.dest,j.ok?"ok":"ng");
      const L=$("rlist");L.textContent="";(j.failed||[]).forEach(f=>L.appendChild(mkrow("⚠ "+f,"",()=>{})));
      $("rrun").disabled=false;
    }
  }catch(e){rmsg(e.message,"ng");$("rrun").disabled=false;}
}
$("rrun").onclick=async()=>{
  const orig=$("rorig").checked,dest=$("rdest").value.trim();
  if(!orig&&!dest){rmsg("復元先のフォルダを入力するか、「元の場所へ」を選んでください / Enter a destination or choose original location","ng");return;}
  $("rrun").disabled=true;rmsg("開始します… / Starting…");
  try{await post("/restore/run",rbody({dest:dest,only:$("ronly").value.trim(),overwrite:$("rover").checked,original:orig}));setTimeout(rpoll,500);}
  catch(e){rmsg(e.message,"ng");$("rrun").disabled=false;if(e.need)$("rpass").focus();}
};
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
        .child(
            h("div").attr("class", "card").attr("id", "restorecard")
                .child(h("h2").child(txt("復元 / Restore")).build())
                .child(h("p").attr("class", "sub").child(txt("保存したデータを、好きな場所へ戻します。暗号化したバックアップは自動で復号します。既存のファイルは、指定しない限り上書きしません。 / Restore your data to any folder. Encrypted backups are decrypted automatically. Existing files are not overwritten unless you ask.")).build())
                .child(
                    h("div").attr("class", "add")
                        .child(h("input").attr("id", "rq").attr("placeholder", "ファイル名・分類で検索(空欄で全部) / Search (blank = all)").attr("autocomplete", "off").attr("spellcheck", "false").build())
                        .child(h("button").attr("id", "rload").child(txt("一覧 / List")).build())
                        .build(),
                )
                .child(
                    h("div").attr("class", "add")
                        .child(h("input").attr("id", "rgen").attr("type", "number").attr("min", "1").attr("value", "1").attr("title", "リポジトリの世代番号 / repository generation").build())
                        .child(h("label").attr("class", "chk").child(h("input").attr("id", "rgh").attr("type", "checkbox").build()).child(txt("GitHubから取得 / Fetch from GitHub")).build())
                        .build(),
                )
                .child(h("div").attr("id", "rlist").attr("class", "rlist").build())
                .child(h("div").attr("class", "add").child(h("input").attr("id", "ronly").attr("placeholder", "復元する範囲(上の一覧をクリック。空欄で全部) / What to restore (click above; blank = all)").attr("autocomplete", "off").attr("spellcheck", "false").build()).build())
                .child(h("div").attr("class", "add").child(h("input").attr("id", "rdest").attr("placeholder", "復元先のフォルダ(例: D:\\復元) / Destination folder").attr("autocomplete", "off").attr("spellcheck", "false").build()).build())
                .child(h("div").attr("class", "add").child(h("input").attr("id", "rpass").attr("type", "password").attr("placeholder", "パスフレーズ(暗号化の場合。保存済みなら空欄でOK) / Passphrase (encrypted only)").attr("autocomplete", "off").build()).build())
                .child(h("label").attr("class", "chk").child(h("input").attr("id", "rorig").attr("type", "checkbox").build()).child(txt("元の場所へ戻す / Restore to original locations")).build())
                .child(h("label").attr("class", "chk").child(h("input").attr("id", "rover").attr("type", "checkbox").build()).child(txt("既存のファイルを上書きする / Overwrite existing files")).build())
                .child(h("button").attr("id", "rrun").child(txt("復元する / Restore")).build())
                .child(h("div").attr("id", "rmsg").build())
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
