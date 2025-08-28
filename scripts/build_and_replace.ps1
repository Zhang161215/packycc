Param(
    [switch]$SkipKill,
    [string]$Dest = "$env:USERPROFILE\.claude\ccline\statusline.exe",
    [int]$ShowSecs = 4,
    [int]$CooldownSecs = 60
)

$ErrorActionPreference = 'Stop'

Write-Host "==> CCometixLine: Build and Replace (Release)" -ForegroundColor Cyan

# Move to project root (this script sits in packycc/scripts)
Set-Location (Join-Path $PSScriptRoot '..')

# Check cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "未检测到 Cargo，请先安装 Rust（rustup）后重试。" -ForegroundColor Yellow
    Write-Host "安装链接: https://rustup.rs/ 或使用 winget 安装 rustup" -ForegroundColor DarkGray
    exit 1
}

# Optional: configure banner timings for this process
$env:PACKYCC_SPEED_BANNER_SHOW_SECS = "$ShowSecs"
$env:PACKYCC_SPEED_BANNER_COOLDOWN_SECS = "$CooldownSecs"

Write-Host "[1/4] 构建 release..." -ForegroundColor Cyan
cargo build --release

$src = Join-Path (Join-Path (Resolve-Path '.') 'target\release') 'statusline.exe'
if (-not (Test-Path $src)) {
    Write-Host "未找到构建产物: $src" -ForegroundColor Red
    exit 1
}

Write-Host "[2/4] 结束可能运行中的 statusline.exe..." -ForegroundColor Cyan
if (-not $SkipKill) {
    try {
        taskkill /IM statusline.exe /F | Out-Null
    } catch {
        # ignore
    }
}

Write-Host "[3/4] 复制到: $Dest" -ForegroundColor Cyan
$destDir = Split-Path $Dest -Parent
if (-not (Test-Path $destDir)) { New-Item -ItemType Directory -Path $destDir -Force | Out-Null }
if (Test-Path $Dest) { Copy-Item $Dest "$Dest.bak" -Force; Write-Host "已备份到 $Dest.bak" -ForegroundColor DarkGray }
Copy-Item $src $Dest -Force

Write-Host "[4/4] 快速验证（无管道输入，短暂显示后自动清除）..." -ForegroundColor Cyan
try {
    & $Dest | Out-Null
} catch {
    Write-Host "验证运行失败: $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host "完成 ✅ 已替换: $Dest" -ForegroundColor Green

