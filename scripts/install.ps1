# Sunclaw Installation Script for Windows
# Usage: powershell -c "irm https://raw.githubusercontent.com/thangdihoc/sunclaw/main/scripts/install.ps1 | iex"

$ErrorActionPreference = "Stop"

function Write-Host-Color($msg, $color) {
    Write-Host ">>> $msg" -ForegroundColor $color
}

Write-Host-Color "Chào mừng bạn đến với Sunclaw - AI Agent Hiệu năng cao 🦀" "Cyan"
Write-Host-Color "--------------------------------------------------------" "Gray"

# 1. Cấu hình thư mục
$SUNCLAW_HOME = Join-Path $HOME ".sunclaw"
$SUNCLAW_BIN = Join-Path $SUNCLAW_HOME "bin"
$SUNCLAW_SRC = Join-Path $SUNCLAW_HOME "src"

if (!(Test-Path $SUNCLAW_BIN)) { New-Item -ItemType Directory -Path $SUNCLAW_BIN -Force | Out-Null }

# 2. Thử tải Binary từ GitHub Releases (Dành cho người không có Rust)
Write-Host-Color "Đang kiểm tra bản cài đặt sẵn (Binary)..." "Cyan"
$RELEASE_URL = "https://github.com/thangdihoc/sunclaw/releases/latest/download/sunclaw-x86_64-pc-windows-msvc.zip"
$ZIP_PATH = Join-Path $env:TEMP "sunclaw.zip"

$INSTALL_SUCCESS = $false

try {
    Write-Host-Color "Đang tải từ: $RELEASE_URL" "Gray"
    Invoke-WebRequest -Uri $RELEASE_URL -OutFile $ZIP_PATH -ErrorAction Stop
    Write-Host-Color "Đã tìm thấy bản build sẵn. Đang giải nén..." "Green"
    Expand-Archive -Path $ZIP_PATH -DestinationPath $SUNCLAW_BIN -Force
    Remove-Item $ZIP_PATH
    $INSTALL_SUCCESS = $true
} catch {
    Write-Host-Color "Không tìm thấy bản build sẵn hoặc lỗi tải về. Chuyển sang cài đặt từ mã nguồn..." "Yellow"
}

# 3. Cài đặt từ mã nguồn (Nếu tải binary thất bại)
if (!$INSTALL_SUCCESS) {
    Write-Host-Color "Bắt đầu quy trình cài đặt từ mã nguồn (Yêu cầu Git và Rust)..." "Cyan"

    if (!(Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Host-Color "Lỗi: Chưa cài đặt Git. Vui lòng cài đặt từ https://git-scm.com/" "Red"
        exit 1
    }

    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host-Color "Lỗi: Chưa cài đặt Rust. Vui lòng cài đặt từ https://rustup.rs/" "Red"
        exit 1
    }

    if (!(Test-Path $SUNCLAW_SRC)) { New-Item -ItemType Directory -Path $SUNCLAW_SRC -Force | Out-Null }
    
    Set-Location $SUNCLAW_SRC
    $REPO_URL = "https://github.com/thangdihoc/sunclaw.git"
    if (Test-Path (Join-Path $SUNCLAW_SRC ".git")) {
        Write-Host-Color "Cập nhật mã nguồn..." "Gray"
        git pull
    } else {
        Write-Host-Color "Tải mã nguồn..." "Gray"
        git clone $REPO_URL .
    }

    Write-Host-Color "Đang biên dịch (Release mode)..." "Cyan"
    cargo build --release --workspace
    
    $BUILD_BIN = "target/release/sunclaw.exe"
    if (Test-Path $BUILD_BIN) {
        Copy-Item $BUILD_BIN $SUNCLAW_BIN -Force
        $INSTALL_SUCCESS = $true
    }
}

if (!$INSTALL_SUCCESS) {
    Write-Host-Color "Lỗi: Cài đặt thất bại." "Red"
    exit 1
}

# 4. Cập nhật PATH
Write-Host-Color "Cập nhật biến môi trường PATH..." "Cyan"
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$SUNCLAW_BIN*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$SUNCLAW_BIN", "User")
    Write-Host-Color "Đã thêm $SUNCLAW_BIN vào PATH người dùng." "Green"
}

# 5. Hoàn tất
Write-Host-Color "--------------------------------------------------------" "Gray"
Write-Host-Color "CHÚC MỪNG! SUNCLAW ĐÃ ĐƯỢC CÀI ĐẶT THÀNH CÔNG." "Green"
Write-Host-Color "Ghi chú: Nếu đây là lần đầu cài đặt, vui lòng MỞ LẠI TERMINAL." "Yellow"
Write-Host-Color "Sau đó chạy lệnh: sunclaw onboard" "Cyan"
Write-Host-Color "--------------------------------------------------------" "Gray"
