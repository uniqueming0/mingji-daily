# =============================================================
# 清理「铭记日常」旧版本安装残留（手动删除程序文件夹导致）
# 现象：安装新版本时提示"检测到旧版本/无法卸载"
# 运行：powershell -NoProfile -ExecutionPolicy Bypass -File scripts\fix-old-install.ps1
# 作用：删除注册表中"铭记日常"的卸载信息与安装标记，随后即可正常安装
# =============================================================
$ErrorActionPreference = "Continue"

$roots = @(
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall",
    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"
)

$found = @()
foreach ($root in $roots) {
    if (-not (Test-Path $root)) { continue }
    Get-ChildItem $root -ErrorAction SilentlyContinue | ForEach-Object {
        $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        if ($props.DisplayName -like "*铭记日常*") { $found += $_.PSPath }
    }
}

# 额外清理可能的安装标记键（Tauri NSIS 可能写入）
foreach ($extra in @("HKCU:\Software\com.mingji.daily", "HKCU:\Software\铭记日常")) {
    if (Test-Path $extra) { $found += $extra }
}

if ($found.Count -eq 0) {
    Write-Host "未找到旧版本残留，可直接重新运行安装程序。"
    exit 0
}

Write-Host "找到残留注册表项："
$found | ForEach-Object { Write-Host ("  " + $_) }

foreach ($p in $found) {
    try {
        Remove-Item -Path $p -Recurse -Force -ErrorAction Stop
        Write-Host ("已删除: " + $p)
    } catch {
        Write-Host ("删除失败（可能需要管理员权限）: " + $p)
        Write-Host ("  → 请右键 PowerShell 选择「以管理员身份运行」后重试本脚本")
    }
}

Write-Host ""
Write-Host "清理完成，请重新运行铭记日常安装程序。"
