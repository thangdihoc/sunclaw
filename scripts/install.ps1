# Sunclaw Installation Script for Windows
# Usage: powershell -c "irm https://raw.githubusercontent.com/thangdihoc/sunclaw/main/scripts/install.ps1 | iex"

$ErrorActionPreference = "Stop"

function Write-Host-Color($msg, $color) {
    Write-Host ">>> $msg" -ForegroundColor $color
}

Write-Host-Color "Chào mừng bạn đến với Sunclaw - AI Agent Hiệu năng cao 🦀" "Cyan"
Write-Host-Color "--------------------------------------------------------" "Gray"

# 1. Kiểm tra yêu cầu hệ thống
Write-Host-Color "Đang kiểm tra môi trường..." "Cyan"

if (!(Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host-Color "Lỗi: Chưa cài đặt Git. Vui lòng cài đặt Git từ https://git-scm.com/" "Red"
    exit 1
}

if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host-Color "Lỗi: Chưa cài đặt Rust. Vui lòng cài đặt từ https://rustup.rs/" "Red"
    exit 1
}

# 2. Cấu hình thư mục
$SUNCLAW_HOME = Join-Path $HOME ".sunclaw"
$SUNCLAW_BIN = Join-Path $SUNCLAW_HOME "bin"
$SUNCLAW_SRC = Join-Path $SUNCLAW_HOME "src"

if (!(Test-Path $SUNCLAW_BIN)) { New-Item -ItemType Directory -Path $SUNCLAW_BIN -Force | Out-Null }
if (!(Test-Path $SUNCLAW_SRC)) { New-Item -ItemType Directory -Path $SUNCLAW_SRC -Force | Out-Null }

# 3. Tải mã nguồn
Write-Host-Color "Đang chuẩn bị mã nguồn..." "Cyan"
$REPO_URL = "https://github.com/thangdihoc/sunclaw.git"

Set-Location $SUNCLAW_SRC
if (Test-Path (Join-Path $SUNCLAW_SRC ".git")) {
    Write-Host-Color "Đã tìm thấy mã nguồn, đang cập nhật..." "Gray"
    git pull
} else {
    Write-Host-Color "Đang tải mã nguồn từ GitHub..." "Gray"
    git clone $REPO_URL .
}

# 4. Build dự án
Write-Host-Color "Đang biên dịch Sunclaw (Chế độ Release)..." "Cyan"
Write-Host-Color "Lưu ý: Quá trình này có thể mất vài phút cho lần đầu tiên." "Yellow"

cargo build --release --workspace

# 5. Cài đặt Binary
Write-Host-Color "Đang cài đặt binary vào hệ thống..." "Cyan"
$BINARY_PATH = "target/release/sunclaw.exe"
if (Test-Path $BINARY_PATH) {
    Copy-Item $BINARY_PATH $SUNCLAW_BIN -Force
    Write-Host-Color "Đã cài đặt: $SUNCLAW_BIN\sunclaw.exe" "Green"
} else {
    Write-Host-Color "Lỗi: Không tìm thấy file biên dịch." "Red"
    exit 1
}

# 6. Cập nhật PATH (Dùng cho phiên làm việc sau)
Write-Host-Color "Đang cập nhật biến môi trường PATH..." "Cyan"
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$SUNCLAW_BIN*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$SUNCLAW_BIN", "User")
    Write-Host-Color "Đã thêm $SUNCLAW_BIN vào PATH người dùng." "Green"
}

# 7. Hoàn tất
Write-Host-Color "--------------------------------------------------------" "Gray"
Write-Host-Color "CHÚC MỪNG! SUNCLAW ĐÃ SẴN SÀNG." "Green"
Write-Host-Color "Vui lòng mở lại Terminal hoặc gõ: `$env:Path = [System.Environment]::GetEnvironmentVariable('Path','User')`" "Yellow"
Write-Host-Color "Sau đó chạy lệnh: sunclaw onboard" "Cyan"
Write-Host-Color "--------------------------------------------------------" "Gray"
