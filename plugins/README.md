# 🛡️ Sunclaw WASM Plugins

Thư mục này chứa các plugin chạy trong môi trường **Sandbox WASM** bảo mật.

## 🚀 Cách tạo Plugin mới

1. **Rust**: Sử dụng crate `extism` dành cho Rust.
2. **Build**: Build bằng target `wasm32-unknown-unknown` hoặc `wasm32-wasi`.
3. **Deploy**: Copy file `.wasm` vào thư mục này. Sunclaw sẽ tự động nhận diện và đăng ký nó thành một Tool cho Agent.

## 🧪 Ví dụ (Plugin 'Echo')

Mã nguồn mẫu (`src/lib.rs` trong dự án WASM riêng):

```rust
use extism_pdk::*;

#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    Ok(format!("WASM Sandbox Echo: {}", input))
}
```

Build lệnh:
```bash
cargo build --target wasm32-unknown-unknown --release
```

Copy `target/wasm32-unknown-unknown/release/echo.wasm` vào `plugins/echo.wasm`.
Agent sẽ thấy tool tên là `echo` và có thể gọi nó an toàn.
