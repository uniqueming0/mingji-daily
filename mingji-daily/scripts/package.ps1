# =============================================================
# 铭记日常 双产物打包脚本
# 产物：release/铭记日常-便携版-vX.Y.Z-win-x64.zip（解压即用）
#       release/铭记日常-Setup-X.Y.Z.exe（NSIS 安装向导）
#       release/sha256sums.txt（校验值）
# =============================================================
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

Write-Host "[1/4] 构建 Tauri（NSIS 安装器 + 主程序）..." -ForegroundColor Cyan
Push-Location $root
try {
    npm run tauri build
    if ($LASTEXITCODE -ne 0) { throw "tauri build 失败" }
}
finally {
    Pop-Location
}

$conf = Get-Content "$root\src-tauri\tauri.conf.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$version = [string]$conf.version
$productName = [string]$conf.productName
if (-not $version -or -not $productName) { throw "读取版本号/产品名失败，请检查 tauri.conf.json" }

$releaseDir = Join-Path $root "release"
$nsisDir = Join-Path $root "src-tauri\target\release\bundle\nsis"
$exeDir = Join-Path $root "src-tauri\target\release"

$setupExe = Join-Path $nsisDir "${productName}_${version}_x64-setup.exe"
$rawExe = Join-Path $exeDir "mingji-daily.exe"

if (-not (Test-Path $setupExe)) { throw "未找到安装器：$setupExe" }
if (-not (Test-Path $rawExe)) { throw "未找到主程序：$rawExe（Cargo 包名变化时请同步修改本脚本）" }

New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null

Write-Host "[2/4] 组装便携版目录..." -ForegroundColor Cyan
$portableDir = Join-Path $releaseDir "${productName}-便携版"
if (Test-Path $portableDir) { Remove-Item $portableDir -Recurse -Force }
New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
Copy-Item $rawExe (Join-Path $portableDir "${productName}.exe")
Copy-Item "$root\docs\使用说明.txt" (Join-Path $portableDir "使用说明.txt")
Copy-Item "$root\docs\更新日志.txt" (Join-Path $portableDir "更新日志.txt")
$portableCount = @(Get-ChildItem -Path $portableDir).Count
if ($portableCount -eq 0) { throw "便携版目录为空，打包中止" }

Write-Host "[3/4] 压缩便携 zip..." -ForegroundColor Cyan
$zipPath = Join-Path $releaseDir "${productName}-便携版-v${version}-win-x64.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($portableDir, $zipPath)
if (-not (Test-Path $zipPath)) { throw "便携 zip 生成失败：$zipPath" }

Write-Host "[4/4] 复制安装版并生成校验值..." -ForegroundColor Cyan
$setupOut = Join-Path $releaseDir "${productName}-Setup-${version}.exe"
Copy-Item $setupExe $setupOut -Force
if (-not (Test-Path $setupOut)) { throw "安装版复制失败：$setupOut" }

$hashFile = Join-Path $releaseDir "sha256sums.txt"
$zipHash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash
$setupHash = (Get-FileHash -Path $setupOut -Algorithm SHA256).Hash
"$zipHash  $(Split-Path $zipPath -Leaf)" | Set-Content $hashFile -Encoding UTF8
"$setupHash  $(Split-Path $setupOut -Leaf)" | Add-Content $hashFile -Encoding UTF8

Remove-Item $portableDir -Recurse -Force

Write-Host ""
Write-Host "打包完成，产物位于 release/ 目录：" -ForegroundColor Green
Get-ChildItem $releaseDir | ForEach-Object {
    Write-Host ("  " + $_.Name + "  (" + [math]::Round($_.Length / 1MB, 2) + " MB)")
}
