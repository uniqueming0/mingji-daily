# =============================================================
# 铭记日常 占位图标生成脚本（账本 + 字母 M）
# 说明：这是开发期占位图标；正式图标请设计师出图后执行：
#   npm run tauri icon <你的图标.png>   （自动生成全套多尺寸）
# =============================================================
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$iconsDir = Join-Path (Split-Path -Parent $PSScriptRoot) "src-tauri\icons"
New-Item -ItemType Directory -Force -Path $iconsDir | Out-Null

function New-IconBitmap([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap($size, $size)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.Clear([System.Drawing.Color]::Transparent)

    # 账本主体：绿色圆角矩形
    $margin = [int]($size * 0.10)
    $w = $size - 2 * $margin
    $h = [int]($size * 0.62)
    $y = [int]($size * 0.30)
    $radius = [int]($size * 0.06)

    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $radius * 2
    $path.AddArc($margin, $y, $d, $d, 180, 90)
    $path.AddArc($margin + $w - $d, $y, $d, $d, 270, 90)
    $path.AddArc($margin + $w - $d, $y + $h - $d, $d, $d, 0, 90)
    $path.AddArc($margin, $y + $h - $d, $d, $d, 90, 90)
    $path.CloseFigure()

    $brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 0, 181, 120))
    $g.FillPath($brush, $path)

    # 书脊：左侧深绿色条
    $spineW = [int]($size * 0.06)
    $spineBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 0, 154, 102))
    $g.FillRectangle($spineBrush, $margin, $y, $spineW, $h)

    # 字母 M
    $fontSize = [float]($size * 0.34)
    $font = New-Object System.Drawing.Font("Segoe UI", $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rect = New-Object System.Drawing.RectangleF($margin, $y, $w, $h)
    $g.DrawString("M", $font, [System.Drawing.Brushes]::White, $rect, $sf)

    $g.Dispose()
    return $bmp
}

# 1024 源图（正式图标替换后，用 `npm run tauri icon app-icon.png` 生成全套多尺寸）
$src = New-IconBitmap 1024
$src.Save((Join-Path $iconsDir "app-icon.png"), [System.Drawing.Imaging.ImageFormat]::Png)
$src.Dispose()
Write-Host "已生成 app-icon.png（1024 源图）"

foreach ($s in @(32, 128, 256)) {
    $bmp = New-IconBitmap $s
    $name = if ($s -eq 256) { "128x128@2x.png" } else { "${s}x${s}.png" }
    $bmp.Save((Join-Path $iconsDir $name), [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
    Write-Host "已生成 $name"
}

# icon.ico（256 单帧，NSIS 打包用）
$bmp256 = New-IconBitmap 256
$hIcon = $bmp256.GetHicon()
$icon = [System.Drawing.Icon]::FromHandle($hIcon)
$fs = [System.IO.File]::Create((Join-Path $iconsDir "icon.ico"))
$icon.Save($fs)
$fs.Close()
$bmp256.Dispose()
Write-Host "已生成 icon.ico"

Write-Host "占位图标已生成到 $iconsDir"
