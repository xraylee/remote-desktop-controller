# ICE 跨架构测试 - 快速参考

## 🚀 30 分钟测试流程

### Intel Mac 操作（5 分钟）

```bash
cd /path/to/remote-desktop-controller
git pull
./scripts/ice-test-server.sh
```

等待并复制 OFFER JSON → 发送到 Apple Silicon Mac

---

### Apple Silicon Mac 操作（5 分钟）

```bash
cd /Users/lc/Development/source/remote-desktop-controller
./scripts/ice-test-client.sh
```

1. 粘贴 OFFER JSON
2. 按 `Ctrl+D`
3. 复制 ANSWER JSON → 发送回 Intel Mac

---

### Intel Mac 继续（5 分钟）

1. 粘贴 ANSWER JSON
2. 按 `Ctrl+D`
3. 等待连接建立

---

## ✅ 成功标志

两边都显示：
```
========================================
✅ ICE CONNECTION ESTABLISHED!
========================================
```

---

## 📊 记录数据

### 候选数量
- Intel Mac: ____ 个候选
- Apple Silicon Mac: ____ 个候选

### 时间
- 候选收集: ____ 秒
- 连接建立: ____ 秒

### 候选类型
- [ ] Host (本地)
- [ ] Srflx (STUN)
- [ ] Relay (TURN)

---

## ❌ 问题排查

| 问题 | 解决方法 |
|------|---------|
| cargo not found | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| 编译失败 | `cargo clean && cargo build -p rdcs-connection` |
| 连接超时 | 检查防火墙和网络连接 |
| JSON 错误 | 确保复制完整 JSON，不包含日志 |

---

## 📝 测试后

1. 填写测试报告模板
2. 记录问题和改进建议
3. 更新文档
4. 提交 git commit

完整文档: [docs/testing/CROSS_ARCHITECTURE_TEST.md](../docs/testing/CROSS_ARCHITECTURE_TEST.md)
