package tokyo.aon.openbackup;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;

import java.io.File;

/**
 * バックアップのエンジン(Rust の `open-backup serve`)を、前面サービスとして起動し続ける。
 * エンジンは lib*.so として同梱してあり、アプリのネイティブライブラリの置き場所から直接実行する。
 * 設定・トークンは、このアプリ専用の領域(filesDir)にだけ置く(OPEN_BACKUP_HOME)。
 */
public class EngineService extends Service {
    static final int PORT = 4620;
    private static final String CHANNEL = "engine";
    private volatile boolean stopping = false;
    private Process process;
    private Thread watcher;

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        startAsForeground();
        if (watcher == null || !watcher.isAlive()) {
            watcher = new Thread(this::superviseEngine, "engine-supervisor");
            watcher.setDaemon(true);
            watcher.start();
        }
        return START_STICKY;
    }

    private void startAsForeground() {
        NotificationManager nm = getSystemService(NotificationManager.class);
        if (Build.VERSION.SDK_INT >= 26) {
            nm.createNotificationChannel(new NotificationChannel(CHANNEL, "open-backup", NotificationManager.IMPORTANCE_LOW));
        }
        Intent open = new Intent(this, MainActivity.class);
        PendingIntent pi = PendingIntent.getActivity(this, 0, open, PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT);
        Notification n = new Notification.Builder(this, CHANNEL)
                .setSmallIcon(android.R.drawable.ic_menu_save)
                .setContentTitle("open-backup")
                .setContentText("バックアップのエンジンが動いています / The backup engine is running")
                .setContentIntent(pi)
                .setOngoing(true)
                .build();
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(1, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC);
        } else {
            startForeground(1, n);
        }
    }

    /** エンジンが終了したら、少し待って起動し直す(待ち時間は最大1分まで伸ばす)。 */
    private void superviseEngine() {
        long backoff = 2000;
        while (!stopping) {
            long started = System.currentTimeMillis();
            try {
                File exe = new File(getApplicationInfo().nativeLibraryDir, "libopen_backup.so");
                File home = new File(getFilesDir(), "ob");
                home.mkdirs();
                ProcessBuilder pb = new ProcessBuilder(exe.getAbsolutePath(), "serve", "--port", String.valueOf(PORT));
                pb.environment().put("OPEN_BACKUP_HOME", home.getAbsolutePath());
                pb.environment().put("HOME", home.getAbsolutePath());
                pb.redirectErrorStream(true);
                pb.redirectOutput(new File(getFilesDir(), "engine.log"));
                process = pb.start();
                process.waitFor();
            } catch (InterruptedException e) {
                return;
            } catch (Exception e) {
                android.util.Log.e("open-backup", "engine start failed", e);
            }
            if (stopping) {
                return;
            }
            // 長く動いてから落ちたなら、すぐに再起動。すぐ落ちるなら、待ち時間を延ばす
            backoff = (System.currentTimeMillis() - started > 60000) ? 2000 : Math.min(backoff * 2, 60000);
            try {
                Thread.sleep(backoff);
            } catch (InterruptedException e) {
                return;
            }
        }
    }

    @Override
    public void onDestroy() {
        stopping = true;
        if (process != null) {
            process.destroy();
        }
        super.onDestroy();
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }
}
