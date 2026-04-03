# Sunclaw Professional Installer for Windows
# Phỏng theo phong cách của PicoClaw - Nhanh chóng, Tinh gọn, Chuyên nghiệp.

$ErrorActionPreference = 'Stop'

$INSTALL_DIR = "$HOME\.sunclaw\bin"
$BINARY_NAME = "sunclaw.exe"
$GITHUB_REPO = "SunclawTeam/sunclaw"

function Show-SunclawBanner {
    Write-Host @"
   _____ _    _ _   _  _____ _          __          __
  / ____| |  | | \ | |/ ____| |         \ \        / /
 | (___ | |  | |  \| | |    | |   __ _   \ \  /\  / / 
  \___ \| |  | | . ` | |    | |  / _` |   \ \/  \/ /  
  ____) | |__| | |\  | |____| | | (_| |    \  /\  /   
 |_____/ \____/|_| \_|\_____|_|  \__,_|     \/  \/    
"@ -ForegroundColor Yellow
    Write-Host "`n>>> Đang cài đặt Sunclaw v0.1 - AI Agent Hiệu năng cao <<<`n" -ForegroundColor Cyan
}

function New-SunclawDirectory {
    if (!(Test-Path $INSTALL_DIR)) {
        Write-Host "[-] Tạo thư mục cài đặt: $INSTALL_DIR" -ForegroundColor Gray
        New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
    }
}

function Invoke-BinaryDownload {
    Write-Host "[*] Đang kiểm tra phiên bản mới nhất từ $GITHUB_REPO..." -ForegroundColor White
    # Trong môi trường thực tế, lệnh này sẽ tải từ GitHub Release:
    # Invoke-WebRequest -Uri "https://github.com/$GITHUB_REPO/releases/latest/download/sunclaw-windows-amd64.zip" -OutFile "$TEMP\sunclaw.zip"
    
    Write-Host "[!] Đang giả lập việc tải xuống binary..." -ForegroundColor Gray
    # Giả định binary đã được build sẵn trong target/release
    $SourcePath = "target\release\sunclaw-cli.exe"
    if (Test-Path $SourcePath) {
        Copy-Item -Path $SourcePath -Destination "$INSTALL_DIR\$BINARY_NAME" -Force
        Write-Host "[+] Đã cài đặt binary vào $INSTALL_DIR" -ForegroundColor Green
    } else {
        Write-Host "[?] Lưu ý: Không tìm thấy bản build sẵn. Vui lòng chạy 'cargo build --release' trước." -ForegroundColor Yellow
    }
}

function Add-SunclawToPath {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$INSTALL_DIR*") {
        Write-Host "[*] Đang thêm Sunclaw vào biến môi trường PATH..." -ForegroundColor White
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$INSTALL_DIR", "User")
        $env:Path += ";$INSTALL_DIR"
        Write-Host "[+] Đã cập nhật PATH thành công." -ForegroundColor Green
    } else {
        Write-Host "[ok] Sunclaw đã có trong PATH." -ForegroundColor Gray
    }
}

function Complete-Installation {
    Write-Host "`n✨ CÀI ĐẶT HOÀN TẤT! ✨" -ForegroundColor Magenta -NoNewline
    Write-Host " Hãy khởi động lại Terminal của bạn.`n"
    Write-Host "Sau đó, chạy lệnh sau để thiết lập Agent:" -ForegroundColor White
    Write-Host "  sunclaw onboard" -ForegroundColor BrightYellow
    Write-Host "`nChúc bạn có trải nghiệm tuyệt vời với Sunclaw!`n"
}

# --- Execution ---
Show-SunclawBanner
New-SunclawDirectory
Invoke-BinaryDownload
Add-SunclawToPath
Complete-Installation
