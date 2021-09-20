#define AppName "Test"

[Setup]
AppName={#AppName}
AppVersion=0.1
DefaultDirName={autopf}\{#AppName}
OutputBaseFilename=testfile.innosetup
OutputDir=..\good

[Files]
Source: "testfile.txt"; DestDir: "{app}"
