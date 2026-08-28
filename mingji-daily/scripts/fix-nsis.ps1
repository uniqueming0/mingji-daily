# =============================================================
# 修复 NSIS 工具下载超时（国内网络访问 GitHub 慢/被墙时使用）
# 作用：下载 nsis-3.zip 与 nsis-plugins-3.zip 并解压到
#       %LOCALAPPDATA%\tauri\NSIS\（Tauri CLI 的缓存目录）
# 运行：powershell -NoProfile -ExecutionPolicy Bypass -File scripts\fix-nsis.ps1
# =============================================================
$ErrorActionPreference = "Stop"

$target = Join-Path $env:LOCALAPPDATA "tauri\NSIS"
New-Item -ItemType Directory -Force -Path $target | Out-Null
if (Test-Path (Join-Path $target "makensis.exe")) {
    Write-Host "NSIS 已就绪，无需处理：$(Join-Path $target 'makensis.exe')"
    exit 0
}

$github = "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3"
$mirrors = @(
    $github,
    "https://ghproxy.net/$github",
    "https://gh-proxy.com/$github",
    "https://ghfast.top/$github"
)

function Download-WithMirrors([string]$name, [string]$out) {
    foreach ($base in $mirrors) {
        $url = "$base/$name"
        Write-Host "尝试下载: $url"
        try {
            Invoke-WebRequest -Uri $url -OutFile $out -TimeoutSec 300 -UseBasicParsing
            Write-Host "下载成功: $name"
            return
        } catch {
            Write-Host ("  失败: " + $_.Exception.Message)
        }
    }
    throw "下载失败（所有镜像源均不可用）：$name，请挂代理后重试"
}

# 1) NSIS 主程序
$tmp = Join-Path $env:TEMP "nsis-3.zip"
Download-WithMirrors "nsis-3.zip" $tmp
$work = Join-Path $env:TEMP "nsis-extract"
Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $work | Out-Null
Expand-Archive -Path $tmp -DestinationPath $work -Force
$mk = Get-ChildItem -Path $work -Recurse -Filter "makensis.exe" | Select-Object -First 1
if (-not $mk) { throw "解压后未找到 makensis.exe，压缩包布局可能已变化，请反馈" }
# 平铺：把含 makensis.exe 的目录内容复制到 NSIS 根
Copy-Item -Path (Join-Path $mk.DirectoryName "*") -Destination $target -Recurse -Force
if (-not (Test-Path (Join-Path $target "makensis.exe"))) { throw "平铺后仍缺少 makensis.exe" }
Write-Host "NSIS 主程序就绪"

# 2) 插件包（含 Plugins\x86-unicode\*.dll）
$ptmp = Join-Path $env:TEMP "nsis-plugins-3.zip"
try {
    Download-WithMirrors "nsis-plugins-3.zip" $ptmp
    $pwork = Join-Path $env:TEMP "nsis-plugins-extract"
    Remove-Item -Recurse -Force $pwork -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $pwork | Out-Null
    Expand-Archive -Path $ptmp -DestinationPath $pwork -Force
    $pd = Get-ChildItem -Path $pwork -Recurse -Directory -Filter "x86-unicode" | Select-Object -First 1
    if ($pd) {
        $pluginsRoot = $pd.Parent
        New-Item -ItemType Directory -Force -Path (Join-Path $target "Plugins") | Out-Null
        Copy-Item -Path (Join-Path $pluginsRoot "*") -Destination (Join-Path $target "Plugins") -Recurse -Force
        Write-Host "NSIS 插件就绪"
    } else {
        Write-Host "警告：插件包中未找到 x86-unicode 目录（可能影响部分安装功能）"
    }
} catch {
    Write-Host "警告：插件包下载失败（$($_.Exception.Message)），可稍后重试本脚本补齐"
}

Write-Host ""
Write-Host "全部完成！NSIS 工具位于：$target"
Write-Host "请重新执行：npm run package"
