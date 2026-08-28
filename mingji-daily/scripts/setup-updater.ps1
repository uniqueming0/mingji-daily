# =============================================================
# 自动更新签名密钥初始化（一次性）
# 作用：生成签名密钥对（不存在时），并把公钥写入 tauri.conf.json
# 运行：powershell -NoProfile -ExecutionPolicy Bypass -File scripts\setup-updater.ps1
# 之后每次发布用 scripts\publish.ps1（自动带密钥签名）
# =============================================================
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$keyPath = Join-Path $env:USERPROFILE ".tauri\mingji.key"
$keyPassword = "mingji-daily"

if (-not (Test-Path $keyPath)) {
    Write-Host "生成签名密钥..." -ForegroundColor Cyan
    Push-Location $root
    try {
        npm run tauri signer generate -- --password $keyPassword -w $keyPath
        if ($LASTEXITCODE -ne 0) { throw "密钥生成失败" }
    } finally {
        Pop-Location
    }
    Write-Host "密钥已生成：$keyPath"
} else {
    Write-Host "密钥已存在：$keyPath"
}

Write-Host "读取公钥..." -ForegroundColor Cyan
Push-Location $root
try {
    $pub = (npm run tauri signer pubkey -- -w $keyPath 2>&1 | Out-String).Trim()
} finally {
    Pop-Location
}
if (-not $pub -or $pub -notmatch "^[A-Za-z0-9+/=]+$") {
    Write-Host "公钥输出异常：$pub"
    throw "公钥读取失败"
}

$confPath = "$root\src-tauri\tauri.conf.json"
$conf = Get-Content $confPath -Raw -Encoding UTF8 | ConvertFrom-Json
$conf.plugins.updater.pubkey = $pub
$conf | ConvertTo-Json -Depth 10 | Set-Content $confPath -Encoding UTF8
Write-Host "公钥已写入 tauri.conf.json" -ForegroundColor Green
Write-Host ""
Write-Host "初始化完成！后续发布直接运行：npm run publish"
