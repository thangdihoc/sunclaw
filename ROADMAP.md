# 📋 KẾ HOẠCH PHÁT TRIỂN SUNCLAW (v0.1 MVP)

Dựa trên tình hình thực tế, đây là bảng theo dõi tiến độ chi tiết. **Lưu ý: Không xóa hoặc sửa nội dung gốc, chỉ tích dấu [x] khi hoàn thành.**

## 🏁 TIẾN ĐỘ TỔNG THỂ
**Hiện tại: ~85% (Cập nhật ngày 02/04/2026)**

- [x] Giai Đoạn 1 (01-14/04): 45% → 60% [Tích Hợp Provider]
- [x] Giai Đoạn 2 (14-28/04): 60% → 75% [Lưu Trữ SQLite]
- [x] Giai Đoạn 3 (28/04-12/05): 75% → 85% [HTTP API Server & Bảo Mật]
- [x] Giai Đoạn 4 (12-26/05): 85% → 95% [Tối Ưu Context & Token]
- [ ] Giai Đoạn 5 (26-31/05): 95% → 100% [Phát Hành v0.1]

---

## 📅 CHI TIẾT HÀNG TUẦN

### Tuần 1 (01-07 Tháng 4) - HOÀN THÀNH
- [x] OpenRouter HTTP client hoạt động (Dùng reqwest)
- [x] Config loader từ .env/TOML (Dùng dotenvy)
- [x] Provider routing tests (Chạy ổn định)
- [x] Cơ chế Multi-agent Planner -> Executor -> Reviewer

### Tuần 2 (08-14 Tháng 4) - HOÀN THÀNH
- [x] Gemini + xAI clients (Cấu hình OpenAI compatibility)
- [x] Phân loại lỗi + retry logic nâng cao
- [x] Real provider integration tests

### Tuần 3 (15-21 Tháng 4) - HOÀN THÀNH
- [x] SQLite schema + migrations (Dùng sqlx)
- [x] Audit trail logging (Lưu vào DB)
- [x] Session persistence (Lưu lịch sử qua các lần chạy)

### Tuần 4 (22-28 Tháng 4) - HOÀN THÀNH
- [x] Memory context optimization (Đếm Token với tiktoken)
- [x] Conversation history pruning (Tự động cắt tỉa tin nhắn)
- [x] Cleanup policies (Giữ lại System Prompt khi đầy bộ nhớ)

### Tuần 5 (29 Tháng 4 - 05 Tháng 5) - HOÀN THÀNH
- [x] REST API server (Crate sunclaw-server với Axum)
- [x] WebSocket support (Dùng cho streaming)
- [x] Authentication/API keys (Bảo vệ API)

### Tuần 6 (06-12 Tháng 5) - TIẾP THEO
- [ ] Tool sandbox wrapper (Cô lập tool execution)
- [ ] Resource limits (Giới hạn CPU/RAM cho tool)
- [x] Security hardening (Đã xong phần Denylist/Allowlist)

### Tuần 7 (13-19 Tháng 5)
- [/] Full test coverage (Đã có một số unit test)
- [x] API documentation (Đã có QUICKSTART.md)
- [ ] Examples + guides nâng cao

### Tuần 8 (20-26 Tháng 5)
- [ ] Load testing (Kiểm tra chịu tải)
- [ ] Performance optimization (Tối ưu hóa async)
- [ ] Bug fixes

### Tuần 9 (27 Tháng 5 - 02 Tháng 6)
- [ ] Final polishing (Chỉnh sửa cuối cùng)
- [ ] v0.1 release (Gắn tag GitHub)
- [ ] crates.io publish (Đưa lên cộng đồng Rust)

---

## ⚠️ RỦI RO & GIẢI PHÁP

| Rủi Ro | Tác Động | Giải Pháp |
| :--- | :--- | :--- |
| Giới hạn API quota | Provider không khả dụng | Triển khai Fallback + Queueing (Đã xong) |
| SQLite deadlocks | Hỏng dữ liệu | Sử dụng WAL mode + connection pooling (Đã tích hợp) |
| Tool sandbox escape | Lỗ hổng bảo mật | Sử dụng OS-level sandboxing (Sẽ triển khai cơ bản) |
| Bottleneck hiệu năng | Response chậm | Caching layer + async optimization (Đang tối ưu) |

---
*Ghi chú: Bản kế hoạch được cập nhật lần cuối vào ngày 02/04/2026.*
