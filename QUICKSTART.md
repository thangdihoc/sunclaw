# Sunclaw AI Runtime - Hướng dẫn sử dụng nhanh

Chào mừng bạn đến với **Sunclaw**, framework AI Agent mạnh mẽ được xây dựng bằng Rust.

## 🚀 Cài đặt nhanh

1. **Cấu hình môi trường:**
   Sao chép tệp `.env.example` thành `.env` và điền các API Key của bạn:
   ```bash
   cp .env.example .env
   ```
   Các biến quan trọng:
   - `OPENROUTER_API_KEY`: Dùng để kết nối với các mô hình như DeepSeek, Grok.
   - `TAVILY_API_KEY`: Dùng cho công cụ tìm kiếm web.

2. **Build dự án:**
   ```bash
   cargo build --workspace
   ```

## 🛠️ Cách sử dụng

### 1. Chế độ Agent đơn lẻ (Single Agent)
Chạy một câu hỏi đơn giản:
```bash
cargo run --package sunclaw-cli -- "Hỏi gì đó ở đây"
```

### 2. Chế độ Làm việc nhóm (Team Mode)
Kích hoạt luồng phối hợp **Planner -> Executor -> Reviewer**:
```bash
cargo run --package sunclaw-cli -- --team "Lập kế hoạch và thực hiện tìm kiếm về Rust 2024"
```

### 3. Chọn model profile
Sử dụng các profile đã cấu hình sẵn (`default`, `reasoning`, `cheap`):
```bash
cargo run --package sunclaw-cli -- --profile reasoning "Giải bài toán khó này..."
```

## 🛡️ Tính năng bảo mật
Hệ thống đã tích hợp sẵn:
- **Allowlist:** Chỉ cho phép các công cụ đã đăng ký.
- **Input Checking:** Tự động chặn các lệnh nguy hiểm (ví dụ: `rm`, `delete`).

---
Dự án được phát triển bởi Sunclaw Team. Chúc bạn có những trải nghiệm tuyệt vời!
