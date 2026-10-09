# Build a Windows zip. Run from the repo root in PowerShell:  powershell -File tools/package_windows.ps1
$ErrorActionPreference = "Stop"
cargo run -q --release -p u67_assetgen
cargo build --release -p u67_game --target x86_64-pc-windows-msvc
$out = "dist/windows/Ultima67"
Remove-Item -Recurse -Force $out -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item target/x86_64-pc-windows-msvc/release/ultima67.exe $out
Copy-Item -Recurse assets "$out/assets" -Exclude original
Remove-Item -Recurse -Force "$out/assets/original","$out/assets/maps" -ErrorAction SilentlyContinue
Copy-Item README.md,LICENSE,commands.txt,ASSETS.md $out
Compress-Archive -Force -Path $out -DestinationPath dist/windows/Ultima67-windows-x64.zip
Write-Host "Built dist/windows/Ultima67-windows-x64.zip"
