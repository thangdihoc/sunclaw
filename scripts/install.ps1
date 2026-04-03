# Sunclaw Installer for Windows (PowerShell)
# Cài đặt Sunclaw AI Agent Runtime

$VERSION = "latest"
$REPO = "thangdihoc/sunclaw"
$BINARY_NAME = "sunclaw.exe"
$INSTALL_DIR = "$HOME\.sunclaw\bin"

if (-not (Test-Path -Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR | Out-Null
}

$ARCH = $env:PROCESSOR_ARCHITECTURE
$OS = "pc-windows-msvc"

if ($ARCH -eq "AMD64") {
    $ARCH = "x86_64"
} elseif ($ARCH -eq "ARM64") {
    $ARCH = "aarch64"
} else {
    Write-Error "Kiến trúc $ARCH chưa được hỗ trợ."
    exit 1
}

$ASSET_NAME = "sunclaw-$ARCH-$OS.zip"
$DOWNLOAD_URL = "https://github.com/$REPO/releases/download/v0.1.0/$ASSET_NAME"

Write-Host "🚀 Đang tải Sunclaw $VERSION cho $ARCH-pc-windows..." -ForegroundColor Cyan

# Thực hiện tải file từ GitHub
Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile "sunclaw.zip"
Expand-Archive -Path "sunclaw.zip" -DestinationPath "$INSTALL_DIR" -Force
Remove-Item "sunclaw.zip"

Write-Host "✅ Đã tải và giải nén Sunclaw!" -ForegroundColor Green

# Add to PATH for current session
if ($env:Path -notlike "*$INSTALL_DIR*") {
    $env:Path = "$INSTALL_DIR;" + $env:Path
    # Permanent add for user
    [System.Environment]::SetEnvironmentVariable("Path", $env:Path + ";$INSTALL_DIR", [System.EnvironmentVariableTarget]::User)
    Write-Host "✅ Đã thêm $INSTALL_DIR vào PATH!" -ForegroundColor Green
}

Write-Host "`n🎉 Chúc mừng! Cài đặt hoàn tất. Hãy chạy 'sunclaw --setup' trong terminal mới." -ForegroundColor Magenta
