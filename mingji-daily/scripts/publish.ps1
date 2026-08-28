# =============================================================
# 一键发布：打包 + 生成更新清单 + 发布到 GitHub Releases
# 前置：已运行 scripts\setup-updater.ps1（签名密钥）
#       （可选）已安装 GitHub CLI：winget install GitHub.cli 并 gh auth login
# 运行：npm run publish
# =============================================================
param([string]$Notes = "例行更新")
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

$keyPath = Join-Path $env:USERPROFILE ".tauri\mingji.key"
if (-not (Test-Path $keyPath)) { throw "未找到签名密钥，请先运行 scripts\setup-updater.ps1" }
$env:TAURI_SIGNING_PRIVATE_KEY = $keyPath
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "mingji-daily"

# 1) 打包（自动对安装包签名，生成 .sig）
Write-Host "[1/3] 打包（含签名）..." -ForegroundColor Cyan
Push-Location $root
try {
    npm run package
    if ($LASTEXITCODE -ne 0) { throw "打包失败" }
} finally {
    Pop-Location
}

$conf = Get-Content "$root\src-tauri\tauri.conf.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$version = [string]$conf.version
$productName = [string]$conf.productName

$nsisDir = Join-Path $root "src-tauri\target\release\bundle\nsis"
$setupSrc = Join-Path $nsisDir "${productName}_${version}_x64-setup.exe"
$sigFile = "$setupSrc.sig"
if (-not (Test-Path $sigFile)) { throw "未找到签名文件：$sigFile（请确认已运行 setup-updater.ps1）" }

# 2) 生成更新清单 latest.json（v2 格式）
Write-Host "[2/3] 生成更新清单 latest.json..." -ForegroundColor Cyan
$releaseDir = Join-Path $root "release"
$asciiSetup = Join-Path $releaseDir "mingji-daily-Setup-${version}.exe"
Copy-Item $setupSrc $asciiSetup -Force

$sig = (Get-Content $sigFile -Raw).Trim()
$url = "https://github.com/uniqueming0/mingji-daily/releases/download/v${version}/mingji-daily-Setup-${version}.exe"
$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$latest = @{
    version   = $version
    notes     = $Notes
    pub_date  = $pubDate
    platforms = @{
        "windows-x86_64" = @{
            signature = $sig
            url       = $url
        }
    }
} | ConvertTo-Json -Depth 6
$latestPath = Join-Path $releaseDir "latest.json"
[System.IO.File]::WriteAllText($latestPath, $latest, (New-Object System.Text.UTF8Encoding($false)))

# 3) 发布到 GitHub Releases
Write-Host "[3/3] 发布到 GitHub Releases..." -ForegroundColor Cyan
$zipPath = Join-Path $releaseDir "${productName}-便携版-v${version}-win-x64.zip"
$gh = Get-Command gh -ErrorAction SilentlyContinue
if ($gh) {
    gh release create "v$version" $zipPath $asciiSetup $latestPath --title "铭记日常 v$version" --notes $Notes
    if ($LASTEXITCODE -ne 0) { throw "gh 发布失败，请检查登录状态（gh auth login）" }
    Write-Host ""
    Write-Host "发布完成！用户端将自动收到更新提示。" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "未安装 GitHub CLI，请手动发布（5 分钟）：" -ForegroundColor Yellow
    Write-Host "1. 打开 https://github.com/uniqueming0/mingji-daily/releases/new"
    Write-Host "2. Tag 填：v$version"
    Write-Host "3. 上传 release\ 目录下这 3 个文件："
    Write-Host ("   - " + (Split-Path $zipPath -Leaf))
    Write-Host ("   - " + (Split-Path $asciiSetup -Leaf))
    Write-Host "   - latest.json"
    Write-Host "4. 点 Publish release（保持 Set as the latest release 勾选）"
    Write-Host ""
    Write-Host "（建议安装 GitHub CLI 一劳永逸：winget install GitHub.cli，然后 gh auth login）"
}
