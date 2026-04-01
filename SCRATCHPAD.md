# 📝 Bảng Nháp Dự Án Sunclaw (Scratchpad)

## Trạng thái hiện tại: **Giai đoạn 4 Hoàn tất (MVP v0.1.0)**
Sunclaw hiện đã là một runtime AI Agent thực thụ, có khả năng kết nối API, bảo mật công cụ và làm việc nhóm.

### ✅ Những gì đã làm được:
- **Kết nối API Real-time:** Tích hợp OpenAI/OpenRouter (DeepSeek, Grok, Gemini).
- **Bảo mật (Security):** Allowlist cho công cụ và Denylist cho tham số (chặn `rm`, `delete`, `format`).
- **Công cụ (Tools):** Thêm kỹ năng Tìm kiếm Web (Web Search) qua Tavily.
- **Điều phối (Orchestration):** Luồng làm việc nhóm Planner -> Executor -> Reviewer.
- **CLI:** Hỗ trợ chế độ chạy đơn và chạy nhóm (`--team`).

### 🏁 Các mốc quan trọng (Milestones):
- [x] **Sprint 1:** Cấu trúc hạt nhân và CLI cơ bản.
- [x] **Sprint 2:** Kết nối Provider thực tế và nạp `.env`.
- [x] **Sprint 3:** Bảo mật và Công cụ tìm kiếm.
- [x] **Sprint 4:** Điều phối đa tác nhân (Multi-agent Team).
- [ ] **Sprint 5 (TIẾP THEO):** Lưu trữ bộ nhớ lâu dài bằng SQLite.

### 📌 Ghi chú kỹ thuật:
- **Routing:** Planner mặc định dùng profile `reasoning` để có kết quả lập kế hoạch tốt nhất.
- **Performance:** Đã tối ưu hóa việc truyền ngữ cảnh vai trò (AgentRole) qua từng bước.

---
*Cập nhật lần cuối: 2026-04-01*
