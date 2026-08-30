# =============================================================
# 自动更新签名密钥初始化
# 作用：生成签名密钥对，并把公钥写入 tauri.conf.json
# 运行：powershell -NoProfile -ExecutionPolicy Bypass -File scripts\setup-updater.ps1
# 注意：每次运行都会重新生成密钥（发布过一次之后请勿重跑，否则旧用户无法验证更新）
# =============================================================
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$keyPath = Join-Path $env:USERPROFILE ".tauri\mingji.key"
$keyPassword = "mingji-daily"

if (Test-Path $keyPath) {
    Write-Host "检测到已存在的密钥（可能未成功写入公钥），删除后重新生成..." -ForegroundColor Yellow
    Remove-Item $keyPath -Force
}

Write-Host "生成签名密钥..." -ForegroundColor Cyan
Push-Location $root
try {
    $out = npm run tauri signer generate -- --password $keyPassword -w $keyPath 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        Write-Host $out
        throw "密钥生成失败"
    }
} finally {
    Pop-Location
}
Write-Host "密钥已生成：$keyPath"

# 公钥在 generate 的输出中打印。先去除 ANSI 颜色码（终端捕获时常见干扰），再提取
$clean = $out -replace "\x1b\[[0-9;]*[A-Za-z]", ""
$pub = $null
$m = [regex]::Match($clean, '(?im)public(?:\s+key)?[:\s=]+([A-Za-z0-9+/=]{40,})')
if ($m.Success) { $pub = $m.Groups[1].Value.Trim() }
if (-not $pub) {
    $m2 = [regex]::Match($clean, '(?m)^([A-Za-z0-9+/=]{40,})\s*$')
    if ($m2.Success) { $pub = $m2.Groups[1].Value.Trim() }
}
if (-not $pub) {
    # 原始输出落盘，方便粘贴给开发者排查
    $dbg = "$root\scripts\pubkey-debug.txt"
    [System.IO.File]::WriteAllText($dbg, $out, (New-Object System.Text.UTF8Encoding($false)))
    Write-Host "公钥提取失败。原始输出已保存到：$dbg"
    throw "请把 $dbg 的内容发给开发者"
}

$confPath = "$root\src-tauri\tauri.conf.json"
$conf = Get-Content $confPath -Raw -Encoding UTF8 | ConvertFrom-Json
$conf.plugins.updater.pubkey = $pub
$conf | ConvertTo-Json -Depth 10 | Set-Content $confPath -Encoding UTF8
Write-Host "公钥已写入 tauri.conf.json" -ForegroundColor Green
Write-Host ""
Write-Host "初始化完成！后续发布直接运行：npm run publish"
