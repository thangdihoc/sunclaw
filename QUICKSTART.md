# Sunclaw v0.1 Quickstart

Chào mừng bạn đến với **Sunclaw v0.1**. Đây là nền tảng AI Agent hiệu năng cao được xây dựng bằng Rust.

## 🚀 Cài đặt nhanh

### 1. Cấu hình môi trường
Tạo tệp `.env` tại thư mục gốc:
```env
OPENROUTER_API_KEY=your_openrouter_key
TAVILY_API_KEY=your_tavily_key
SUNCLAW_API_KEY=my_secret_key
```

### 2. Chạy ứng dụng CLI
Sử dụng CLI để tương tác trực tiếp:
```bash
cargo run -- "Lập kế hoạch du lịch Đà Lạt 3 ngày 2 đêm"
```

### 3. Chạy API Server
Khởi động máy chủ REST API:
```bash
cargo run -p sunclaw-server
```
Server sẽ chạy tại: `http://localhost:8080`

---

## 📡 Các API chính

### 1. Kiểm tra trạng thái
```bash
curl http://localhost:8080/api/v1/health \
  -H "Authorization: Bearer my_secret_key"
```

### 2. Gửi lệnh chat
```bash
curl -X POST http://localhost:8080/api/v1/chat \
  -H "Authorization: Bearer my_secret_key" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Kiểm tra giá Bitcoin hiện tại bằng công cụ tìm kiếm",
    "model_profile": "reasoning"
  }'
```

---

## 🛡️ Tính năng nổi bật đã tích hợp
- **Persistence:** Lưu lịch sử vào SQLite (`sunclaw.db`).
- **Reliability:** Tự động retry khi API lỗi.
- **Security:** Sandbox cơ bản cho Tool và giới hạn Timeout.
- **Context Management:** Tự động cắt tỉa tin nhắn khi vượt giới hạn Token.

---
*Ghi chú: Bản v0.1 MVP tập trung vào sự ổn định và hiệu năng hạt nhân.*
