; open-backup Windows installer (Inno Setup 6) / per-user, no admin required
; Build: powershell -File scripts\build-installer.ps1 -Version 0.1.1
#ifndef MyAppVersion
  #define MyAppVersion "0.1.1"
#endif
#define BuildDir "..\target\release"

[Setup]
AppId={{3D8B6F52-7A1C-4E0B-9C47-5B2E1A90D6F3}
AppName=open-backup
AppVersion={#MyAppVersion}
AppPublisher=aon-co-jp
AppPublisherURL=https://github.com/aon-co-jp/open-backup
DefaultDirName={autopf}\open-backup
DefaultGroupName=open-backup
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
OutputDir=..\dist
OutputBaseFilename=open-backup-{#MyAppVersion}-windows-x86_64-setup
Compression=lzma2
SolidCompression=yes
LicenseFile=..\LICENSE-MIT
UninstallDisplayIcon={app}\open-backup.exe
CloseApplications=no

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "autostart"; Description: "ログイン時に自動で起動する / Start automatically at sign-in"

[Files]
Source: "{#BuildDir}\open-backup.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; 常駐エンジン(画面は http://127.0.0.1:4620/ui)
Name: "{group}\open-backup (engine)"; Filename: "{app}\open-backup.exe"; Parameters: "serve"; WorkingDir: "{app}"; Flags: runminimized
Name: "{group}\open-backup (screen)"; Filename: "http://127.0.0.1:4620/ui"
Name: "{userstartup}\open-backup"; Filename: "{app}\open-backup.exe"; Parameters: "serve"; WorkingDir: "{app}"; Flags: runminimized; Tasks: autostart

[Run]
Filename: "{app}\open-backup.exe"; Parameters: "serve"; WorkingDir: "{app}"; Flags: nowait runminimized postinstall skipifsilent; Description: "エンジンを起動する / Start the engine"

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /T /IM open-backup.exe"; Flags: runhidden; RunOnceId: "KillEngine"
