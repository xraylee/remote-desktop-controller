# 🔧 Cargo Feature 配置修复

**日期**: 2026-06-29  
**问题**: `cargo run --example local_loopback_test --features software-encoder` 报错

---

## ❌ 问题

```
error: the package 'rdcs-ffi' does not contain this feature: software-encoder
help: packages with the missing feature: rdcs-codec, rdcs-connection, rdcs-transport
```

---

## ✅ 解决方案

### 修改 `crates/rdcs-ffi/Cargo.toml`

**之前**:
```toml
[dependencies]
rdcs-codec = { path = "../rdcs-codec", features = ["software-encoder"] }

# 无 [features] 部分
```

**之后**:
```toml
[dependencies]
rdcs-codec = { path = "../rdcs-codec" }

[features]
default = ["software-encoder"]
software-encoder = ["rdcs-codec/software-encoder"]
```

---

## 📝 原因

1. `rdcs-codec` 的 `software-encoder` feature 被硬编码在依赖中
2. `rdcs-ffi` 没有定义自己的 feature 来传递这个 feature
3. 用户无法通过 `--features` 控制是否启用

---

## 🎯 改进

### 现在的用法

```bash
# 默认启用软件编码器
cargo build
cargo run --example local_loopback_test

# 禁用软件编码器（使用硬件编码器）
cargo build --no-default-features

# 显式启用
cargo build --features software-encoder
```

---

## 📚 Feature 层次结构

```
rdcs-ffi (default = ["software-encoder"])
    ↓
rdcs-ffi (software-encoder feature)
    ↓
rdcs-codec (software-encoder feature)
    ↓
OpenH264 软件编解码器
```

---

## ✅ 验证

```bash
cd crates/rdcs-ffi

# 检查 feature 定义
cargo metadata --format-version 1 | jq '.packages[] | select(.name=="rdcs-ffi") | .features'

# 构建测试
cargo build
cargo run --example local_loopback_test
```

---

**修复日期**: 2026-06-29  
**影响文件**:
- `crates/rdcs-ffi/Cargo.toml`
- `QUICK_TEST_GUIDE.md`
- `docs/implementation/TASK_45_LOCAL_LOOPBACK_IMPLEMENTATION.md`
