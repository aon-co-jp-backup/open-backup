package tokyo.aon.openbackup;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Environment;
import android.os.Handler;
import android.os.Looper;
import android.provider.Settings;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

/**
 * 画面は、端末の中のエンジン(http://127.0.0.1:4620/ui)が配る共通のページ(RPoem の crossui で生成)を、WebView で表示するだけ。
 * 起動時に、エンジンの前面サービスを起動し、「すべてのファイルへのアクセス」(端末内のフォルダをパスで読むため)を案内する。
 */
public class MainActivity extends Activity {
    private static final String URL = "http://127.0.0.1:" + EngineService.PORT + "/ui";
    private WebView web;
    private final Handler ui = new Handler(Looper.getMainLooper());
    private boolean askedStorage = false;

    @Override
    protected void onCreate(Bundle b) {
        super.onCreate(b);
        if (Build.VERSION.SDK_INT >= 33) {
            requestPermissions(new String[]{Manifest.permission.POST_NOTIFICATIONS}, 1);
        }
        startForegroundService(new Intent(this, EngineService.class));

        web = new WebView(this);
        WebSettings s = web.getSettings();
        s.setJavaScriptEnabled(true);
        s.setDomStorageEnabled(false);
        s.setAllowFileAccess(false);
        s.setAllowContentAccess(false);
        web.setWebViewClient(new WebViewClient() {
            @Override
            public void onReceivedError(WebView v, WebResourceRequest r, WebResourceError e) {
                // エンジンの起動待ち: 少し待って読み直す
                if (r.isForMainFrame()) {
                    ui.postDelayed(() -> web.loadUrl(URL), 1500);
                }
            }

            @Override
            public boolean shouldOverrideUrlLoading(WebView v, WebResourceRequest r) {
                // 端末の中のエンジン以外(GitHub の案内リンクなど)は、WebView ではなく、端末のブラウザで開く
                if ("127.0.0.1".equals(r.getUrl().getHost())) {
                    return false;
                }
                String scheme = r.getUrl().getScheme();
                if ("https".equals(scheme)) {
                    try {
                        startActivity(new Intent(Intent.ACTION_VIEW, r.getUrl()));
                    } catch (Exception ignored) {
                    }
                }
                return true;
            }
        });
        setContentView(web);
        web.loadDataWithBaseURL(null, "<html><body style='font-family:sans-serif;padding:24px'>open-backup を起動しています… / Starting…</body></html>", "text/html", "utf-8", null);
        web.loadUrl(URL);
    }

    @Override
    protected void onResume() {
        super.onResume();
        // 端末内のフォルダ(写真・文書)を読むための許可。拒否されたままなら、この起動中は1回だけ案内する
        if (Build.VERSION.SDK_INT >= 30 && !Environment.isExternalStorageManager() && !askedStorage) {
            askedStorage = true;
            Intent i = new Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:" + getPackageName()));
            startActivity(i);
        }
    }

    @Override
    public void onBackPressed() {
        if (web != null && web.canGoBack()) {
            web.goBack();
        } else {
            super.onBackPressed();
        }
    }
}
