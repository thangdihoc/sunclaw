# Sunclaw Progress Tracker

> File này cập nhật trạng thái thực tế của dự án Sunclaw.

## 🏁 Ảnh chụp Tiến độ

- **Hoàn thành tổng thể (v0.1 MVP):** **95%**
- **Trọng tâm hiện tại:** **Đóng gói và Phát hành (v0.1)**
- **Cột mốc mới nhất:** **Multi-Agent Orchestrator & Plugin SDK hoàn tất.**

## ✅ Các hạng mục đã hoàn thành

- **Multi-Agent Engine:** Hỗ trợ mô hình Supervisor-Worker và TeamFlow.
- **Plugin SDK:** Macro `sunclaw_tool!` tự động sinh JSON Schema, tối giản hóa việc tạo Tool.
- **Provider Routing:** Hỗ trợ OpenRouter, Gemini, xAI với cơ chế Fallback.
- **Persistence:** Lưu trữ hội thoại và Audit trail bằng SQLite.
- **Security:** Guardrails (turn/tool limits) và Allowlist/Denylist cho tham số.

## 🚀 Kế hoạch Tiếp theo (Roadmap Ngắn hạn)

1. **Bridge Adapters:** Triển khai kết nối với Discord, Zalo và Telegram.
2. **Tool Sandboxing:** Nghiên cứu cô lập thực thi công cụ bằng WebAssembly (WASM).
3. **Performance:** Tối ưu hóa bộ đếm Token và bộ nhớ đệm (Caching).
4. **v0.1 Release:** Gắn tag phiên bản và đăng tải lên crates.io.

## ⚠️ Rủi ro & Chặn

- **Quản lý Token:** Cần thuật toán cắt tỉa ngữ cảnh thông minh hơn khi hội thoại quá dài.
- **Bảo mật Tool:** Thực thi các tool hệ thống cần được cô lập triệt để hơn (Sandbox).

---
*Cập nhật lần cuối: 03/04/2026*
