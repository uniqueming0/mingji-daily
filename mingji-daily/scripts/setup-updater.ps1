# =============================================================
# 自动更新签名密钥初始化
# 作用：生成签名密钥对，并把公钥写入 tauri.conf.json
# 运行：powershell -NoProfile -ExecutionPolicy Bypass -File scripts\setup-updater.ps1
# 注意：每次运行都会重新生成密钥（发布过一次之后请勿重跑，否则旧用户无法验证更新）
# =============================================================
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$keyPath = Join-Path $env:USERPROFILE ".tauri\mingji.key"
$keyPassword = if ($env:MINGJI_SIGN_PASSWORD) { $env:MINGJI_SIGN_PASSWORD } else { "mingji-daily" }

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

# 新版 CLI 会把公钥写入单独的 .pub 文件；旧版则在输出里打印
$pubFile = "$keyPath.pub"
$pub = $null
if (Test-Path $pubFile) {
    $pub = (Get-Content $pubFile -Raw).Trim()
}
if (-not $pub) {
    # 兼容旧版 CLI：从输出中提取（先去除 ANSI 颜色码）
    $clean = $out -replace "\x1b\[[0-9;]*[A-Za-z]", ""
    $m = [regex]::Match($clean, '(?im)public(?:\s+key)?[:\s=]+([A-Za-z0-9+/=]{40,})')
    if ($m.Success) { $pub = $m.Groups[1].Value.Trim() }
    if (-not $pub) {
        $m2 = [regex]::Match($clean, '(?m)^([A-Za-z0-9+/=]{40,})\s*$')
        if ($m2.Success) { $pub = $m2.Groups[1].Value.Trim() }
    }
}
if (-not $pub) {
    # 原始输出落盘，方便粘贴给开发者排查
    $dbg = "$root\scripts\pubkey-debug.txt"
    [System.IO.File]::WriteAllText($dbg, $out, (New-Object System.Text.UTF8Encoding($false)))
    Write-Host "公钥提取失败。原始输出已保存到：$dbg"
    throw "请把 $dbg 的内容发给开发者"
}
# 成功后清理调试文件
Remove-Item "$root\scripts\pubkey-debug.txt" -ErrorAction SilentlyContinue

$confPath = "$root\src-tauri\tauri.conf.json"
$conf = Get-Content $confPath -Raw -Encoding UTF8 | ConvertFrom-Json
$conf.plugins.updater.pubkey = $pub
# 注意：不能写 BOM（Rust 的 JSON 解析器会报错），用无 BOM UTF-8 写回
$json = $conf | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText($confPath, $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Host "公钥已写入 tauri.conf.json" -ForegroundColor Green
Write-Host ""
Write-Host "初始化完成！后续发布直接运行：npm run publish"
