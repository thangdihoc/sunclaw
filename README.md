# 🦂 Sunclaw

**Hệ thống AI Agent Hiệu năng cao (Rust-first) với thiết kế tinh gọn và chuyên nghiệp.**

Sunclaw được xây dựng để trở thành "bộ óc" điều phối đa tác nhân, tối ưu hóa cho tốc độ phản hồi và sự an toàn trong thực thi công cụ.

---

## 🚀 Cài đặt nhanh (One-Liner)

Chúng tôi đã đơn giản hóa quy trình cài đặt để bạn có thể bắt đầu trong nháy mắt.

**Đối với Windows (PowerShell):**

```powershell
iwr -useb raw.githubusercontent.com/SunclawTeam/sunclaw/main/scripts/install.ps1 | iex
```

**Sau khi cài đặt, hãy khởi chạy trình hướng dẫn thiết lập:**

```bash
sunclaw onboard
```

---

## ✨ Điểm nổi bật

- **Plugin-based architecture**: Dễ dàng mở rộng với Plugin SDK.
- **Runtime guardrails**: Giới hạn số lượt và công cụ để đảm bảo an toàn.
- **Multi-agent coordination**: Hỗ trợ mô hình Hierarchical (Phân cấp) và Sequential (Nối tiếp).
- **Multi-model routing**: OpenRouter, Gemini, OpenAI tích hợp sẵn.
- **Lightweight**: Chỉ tiêu tốn vài MB RAM (PicoClaw-style).

---

## 🛠️ Quy trình làm việc (Workspace Crates)

Dự án được phân chia thành các module chuyên biệt:

- `sunclaw-core`: Các hợp đồng lõi và kiểu dữ liệu chung.
- `sunclaw-provider`: Điều phối Model và Fallback.
- `sunclaw-runtime`: Vòng lặp thực thi (Policy/Tool/Memory/Audit).
- `sunclaw-orchestrator`: Điều phối đa tác nhân (TeamFlow).
- `sunclaw-skills`: Định nghĩa kịch bản Agent (Manifest).

---

## 🤖 Điều phối Đa tác nhân (Orchestration)

Sunclaw hỗ trợ các quy trình Agentic phức tạp:

- **Hierarchical**: Một Supervisor Agent làm trưởng nhóm, phân công cho các Worker Agent.
- **Sequential**: Chuỗi Agent nối tiếp (TeamFlow), kết quả của người này là đầu vào của người kia.

---

## 🧪 Phát triển & Kiểm thử

Bạn vẫn có thể chạy trực tiếp từ mã nguồn nếu muốn đóng góp:

```bash
cargo run -p sunclaw-cli -- chat
cargo run -p sunclaw-cli -- onboard
```

---
*Cảm ơn bạn đã tin dùng Sunclaw - Agent của tương lai.*
