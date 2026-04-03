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
Sử dụng CLI để tương tác trực diện:
```bash
# Chạy agent đơn lẻ
cargo run -- "Lập kế hoạch du lịch Đà Lạt 3 ngày 2 đêm"

# Chạy chế độ Multi-Agent Team (Đội ngũ đa tác nhân)
cargo run -- --team "Nghiên cứu về Rust 1.80 và viết bài tóm tắt"
```

## 🛠️ Phát triển Tool mới (Plugin SDK)

Việc mở rộng Sunclaw cực kỳ đơn giản với macro `sunclaw_tool!`. Bạn không cần viết JSON Schema thủ công:

```rust
#[derive(Deserialize, JsonSchema)]
pub struct MyArgs {
    pub query: String,
}

sunclaw_tool!(
    MyTool, MyArgs, "tool_name", "Mô tả công cụ...", 
    self_obj, args, {
        // Logic thực thi của bạn ở đây
        Ok(ToolResult { output: format!("Kết quả: {}", args.query) })
    }
);
```

## 🤖 Điều phối Đa tác nhân (Orchestration)

Sunclaw hỗ trợ hai mô hình điều phối chính:
1.  **Hierarchical (Phân cấp)**: Một Supervisor Agent làm trưởng nhóm, phân chia công việc cho các Worker Agent.
2.  **Sequential (Nối tiếp)**: Các Agent làm việc theo chuỗi, kết quả của Agent này là đầu vào của Agent kia.

## 📡 API Server

Khởi động máy chủ REST API:
```bash
cargo run -p sunclaw-server
```

**Ví dụ gửi lệnh chat:**
```bash
curl -X POST http://localhost:8080/api/v1/chat \
  -H "Authorization: Bearer my_secret_key" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Kiểm tra giá Bitcoin hiện tại",
    "model_profile": "reasoning"
  }'
```

---
*Ghi chú: Bản v0.1 tập trung vào sự ổn định và khả năng mở rộng thông qua Plugin SDK.*
