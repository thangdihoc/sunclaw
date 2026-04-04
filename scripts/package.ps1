# 🦂 Sunclaw Single-Binary Packaging Script (Windows)

$distDir = "dist"
if (!(Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir
}

Write-Host "🚀 Đang biên dịch Sunclaw (Release)..." -ForegroundColor Cyan
cargo build --release -p sunclaw-cli

if ($LASTEXITCODE -eq 0) {
    Copy-Item "target/release/sunclaw.exe" "$distDir/sunclaw.exe"
    Write-Host "✅ Đã đóng gói thành công: $distDir/sunclaw.exe" -ForegroundColor Green
    Write-Host "ℹ️  Đây là binary duy nhất chứa tất cả UI và Logic." -ForegroundColor Gray
} else {
    Write-Host "❌ Lỗi biên dịch!" -ForegroundColor Red
    exit 1
}
