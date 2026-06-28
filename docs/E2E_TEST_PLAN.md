# 端到端测试方案（E2E Test Plan）

**制定日期**: 2026-06-28  
**测试框架**: Superpowers 垂直切片原则  
**机器角色**: Apple Silicon Mac（主控端）→ Intel Mac（被控端）

---

## 🎯 测试目标

### MVP 目标
**用户能够从 Apple Silicon Mac 远程查看和控制 Intel Mac 的桌面**

### 验证的核心功能
1. ✅ 屏幕捕获（被控端）
2. ✅ 视频编码（被控端）
3. ✅ 网络传输（P2P/Relay）
4. ✅ 视频解码（主控端）
5. ✅ 视频显示（主控端）
6. ✅ 输入控制（键盘/鼠标）
7. ✅ 跨架构兼容性

---

## 📊 测试金字塔

### Level 1: 单元测试（Unit Tests）
**已有覆盖**：
- ✅ Mock 编解码器测试
- ✅ 加密模块测试
- ⚠️ VideoToolbox 编解码测试（有崩溃）

**缺失**：
- ❌ 网络模块单元测试
- ❌ 输入处理单元测试

### Level 2: 组件测试（Component Tests）
**已有覆盖**：
- ✅ ICE 连接测试
- ✅ OpenH264 集成测试
- ✅ TCP 传输测试

**缺失**：
- ❌ 完整的编解码器测试（真实视频帧）
- ❌ RTP 打包/解包测试

### Level 3: 集成测试（Integration Tests）
**已有覆盖**：
- ✅ 本地回环测试（Mock）

**缺失**：
- ❌ 跨进程集成测试
- ❌ 跨网络集成测试

### Level 4: 端到端测试（E2E Tests） ⭐ 本方案重点
**缺失**：
- ❌ 完整的远程桌面会话测试
- ❌ 跨架构兼容性测试
- ❌ 真实场景用户测试

---

## 🎬 E2E 测试场景

### 场景 1: 同一局域网下的远程桌面 ⭐ MVP

**测试目标**: 验证基础远程桌面功能

**前置条件**：
- Apple Silicon Mac 和 Intel Mac 在同一局域网
- 两台机器都安装了最新代码
- 网络没有严格防火墙限制

**测试步骤**：

#### Step 1: 启动被控端（Intel Mac）
```bash
# 在 Intel Mac 上
cd /path/to/remote-desktop-controller
cargo run -p rdcs-agent --release
```

**预期输出**：
```
[INFO] RDCS Agent started
[INFO] Device ID: intel-mac-001
[INFO] Listening for connections...
[INFO] Screen capture initialized
[INFO] Encoder: VideoToolbox H.264
```

#### Step 2: 启动主控端（Apple Silicon Mac）
```bash
# 在 Apple Silicon Mac 上
cd /Users/lc/Development/source/remote-desktop-controller
cargo run -p rdcs-controller --release -- connect intel-mac-001
```

**预期输出**：
```
[INFO] RDCS Controller started
[INFO] Discovering devices...
[INFO] Found device: intel-mac-001 (192.168.1.x)
[INFO] Initiating connection...
```

#### Step 3: 建立连接
**自动流程**：
1. mDNS 发现设备
2. 交换 ICE 候选
3. 建立 P2P 连接
4. DTLS 握手
5. 开始视频流传输

**预期日志**：
```
[INFO] ICE connection established
[INFO] DTLS handshake complete
[INFO] Receiving video stream: 1920x1080@30fps
[INFO] Latency: ~50ms
```

#### Step 4: 验证视频显示
**手动验证**：
- [ ] 主控端窗口显示被控端屏幕
- [ ] 画面流畅，无明显卡顿
- [ ] 延迟可接受（< 200ms）
- [ ] 画质清晰

**自动验证**（如果可能）：
```bash
# 截图并比较
screenshot_controller > controller.png
screenshot_agent > agent.png
compare_images controller.png agent.png  # 相似度 > 90%
```

#### Step 5: 验证输入控制
**鼠标控制**：
- [ ] 在主控端移动鼠标 → 被控端光标同步移动
- [ ] 点击 → 被控端响应点击
- [ ] 拖拽 → 被控端正确处理拖拽

**键盘控制**：
- [ ] 打开文本编辑器
- [ ] 在主控端输入文字 → 被控端正确显示
- [ ] 快捷键（Cmd+C/V）正确工作

#### Step 6: 压力测试
**场景**：
- [ ] 播放视频 → 高动态画面传输正常
- [ ] 快速移动窗口 → 画面更新及时
- [ ] 打开多个应用 → 性能稳定

#### Step 7: 断线重连
**测试**：
1. 断开网络（拔网线或禁用 Wi-Fi）
2. 等待 5 秒
3. 恢复网络

**预期**：
- [ ] 自动检测断线
- [ ] 自动尝试重连
- [ ] 30 秒内恢复连接
- [ ] 画面继续正常显示

#### Step 8: 正常退出
```bash
# 主控端按 Ctrl+C
```

**预期**：
- [ ] 优雅关闭连接
- [ ] 清理资源
- [ ] 被控端回到等待状态

---

### 场景 2: 跨网络远程桌面（通过 Relay）

**测试目标**: 验证 NAT 穿透和 Relay 功能

**前置条件**：
- 两台机器在不同网络（如：家庭网络 vs 公司网络）
- 有可用的 TURN Relay 服务器

**测试步骤**：类似场景 1，但需验证：
- [ ] ICE 候选包含 Relay 候选
- [ ] 使用 Relay 候选建立连接
- [ ] 连接稳定性

**性能预期**：
- 延迟: < 500ms（通过中继）
- 带宽: 足够支持 720p@30fps

---

### 场景 3: 真实用户场景

**测试目标**: 验证实际使用场景

**任务列表**：
1. **浏览网页**
   - [ ] 打开浏览器
   - [ ] 输入网址
   - [ ] 滚动页面
   - [ ] 点击链接

2. **编辑文档**
   - [ ] 打开 Word/Pages
   - [ ] 输入文字
   - [ ] 格式化文本
   - [ ] 保存文件

3. **观看视频**
   - [ ] 打开 YouTube
   - [ ] 播放视频
   - [ ] 全屏观看
   - [ ] 声音同步（如支持）

4. **使用开发工具**
   - [ ] 打开 VS Code
   - [ ] 编辑代码
   - [ ] 运行终端命令
   - [ ] 查看输出

---

## 🧪 测试环境配置

### Apple Silicon Mac（主控端）
```yaml
硬件:
  CPU: M1/M2/M3
  内存: >= 8GB
  网络: Wi-Fi 或有线

软件:
  OS: macOS 13+
  Rust: 1.75+
  工具: cargo, git

角色: Controller/Viewer
功能:
  - 视频解码和显示
  - 输入捕获和发送
  - 连接管理
```

### Intel Mac（被控端）
```yaml
硬件:
  CPU: Intel Core i5+
  内存: >= 8GB
  网络: Wi-Fi 或有线

软件:
  OS: macOS 12+
  Rust: 1.75+
  工具: cargo, git

角色: Agent/Host
功能:
  - 屏幕捕获
  - 视频编码
  - 输入处理
```

### 网络环境
```yaml
本地网络:
  类型: 同一局域网
  延迟: < 5ms
  带宽: >= 100Mbps

远程网络:
  类型: 跨公网
  延迟: < 100ms
  带宽: >= 10Mbps
  NAT: 支持 STUN/TURN
```

---

## 📏 成功标准

### 功能性指标
- ✅ 连接建立成功率 >= 95%
- ✅ 视频流传输成功率 >= 99%
- ✅ 输入响应准确率 = 100%
- ✅ 断线重连成功率 >= 90%

### 性能指标
| 指标 | 本地网络 | 远程网络 |
|------|---------|---------|
| 连接建立时间 | < 5s | < 10s |
| 端到端延迟 | < 100ms | < 300ms |
| 视频帧率 | >= 30fps | >= 24fps |
| 视频分辨率 | 1920x1080 | 1280x720 |
| CPU 使用率 | < 50% | < 60% |
| 内存使用 | < 500MB | < 600MB |

### 兼容性指标
- ✅ ARM ↔ Intel 字节序正确
- ✅ 不同 macOS 版本兼容
- ✅ 不同网络环境稳定

---

## 🔧 测试工具

### 自动化测试脚本
```bash
scripts/
├── e2e/
│   ├── run-all-tests.sh          # 运行所有 E2E 测试
│   ├── test-local-network.sh     # 本地网络测试
│   ├── test-remote-network.sh    # 远程网络测试
│   ├── test-performance.sh       # 性能测试
│   └── test-stability.sh         # 稳定性测试（长时间）
```

### 测试辅助工具
```bash
# 网络模拟
sudo pfctl -E  # 模拟丢包
tc qdisc add dev eth0 root netem delay 100ms  # 模拟延迟

# 性能监控
top -pid $(pgrep rdcs-agent)
iostat -w 1
networkQuality  # macOS 网络质量测试

# 截图比对
screencapture -x screenshot.png
compare -metric RMSE screenshot1.png screenshot2.png diff.png
```

---

## 📋 测试检查清单

### 测试前准备
- [ ] 两台 Mac 都安装最新代码
- [ ] 网络连通性测试
- [ ] 防火墙配置正确
- [ ] STUN 服务器可访问
- [ ] 测试环境记录清楚

### 场景 1: 本地网络测试
- [ ] 被控端启动成功
- [ ] 主控端发现设备
- [ ] 连接建立成功
- [ ] 视频流显示正常
- [ ] 鼠标控制正常
- [ ] 键盘输入正常
- [ ] 性能指标达标
- [ ] 断线重连成功
- [ ] 正常退出

### 场景 2: 远程网络测试
- [ ] Relay 候选生成
- [ ] 通过 Relay 连接
- [ ] 视频流稳定
- [ ] 性能可接受

### 场景 3: 真实用户场景
- [ ] 浏览网页流畅
- [ ] 编辑文档正常
- [ ] 视频播放流畅
- [ ] 开发工具可用

### 测试后验证
- [ ] 记录测试结果
- [ ] 截图和日志保存
- [ ] 性能数据记录
- [ ] 问题跟踪创建

---

## 📊 测试报告模板

### 执行测试后填写

```markdown
# E2E 测试报告

**测试日期**: YYYY-MM-DD  
**测试人员**: [Name]  
**版本**: [Git commit hash]

## 测试环境

**主控端**:
- 设备: Apple Silicon Mac (M1/M2/M3)
- OS: macOS [version]
- IP: [IP address]

**被控端**:
- 设备: Intel Mac
- OS: macOS [version]
- IP: [IP address]

**网络**:
- 类型: [本地网络/远程网络]
- 延迟: [X]ms
- 带宽: [X]Mbps

## 测试结果

### 场景 1: 本地网络远程桌面
- 连接建立: [✅/❌] ([X]秒)
- 视频显示: [✅/❌]
- 输入控制: [✅/❌]
- 性能指标:
  - 延迟: [X]ms
  - 帧率: [X]fps
  - CPU: [X]%
  - 内存: [X]MB

### 场景 2: 远程网络测试
- 状态: [✅/❌/⏭️ 未测试]
- ...

### 场景 3: 真实用户场景
- 浏览网页: [✅/❌]
- 编辑文档: [✅/❌]
- 观看视频: [✅/❌]
- 开发工具: [✅/❌]

## 问题记录

1. [问题描述]
   - 严重性: [高/中/低]
   - 复现步骤: ...
   - 日志: ...

## 结论

- [ ] 测试通过，可以进入下一阶段
- [ ] 测试部分通过，有[X]个问题需要修复
- [ ] 测试失败，需要重大修复

## 下一步

1. ...
2. ...
```

---

## 🚀 执行计划

### 第一周: 准备阶段
- [ ] Day 1: 完善测试脚本
- [ ] Day 2-3: 修复已知问题（VideoToolbox）
- [ ] Day 4-5: 搭建测试环境

### 第二周: 执行测试
- [ ] Day 1-2: 场景 1 测试（本地网络）
- [ ] Day 3: 场景 2 测试（远程网络）
- [ ] Day 4: 场景 3 测试（真实场景）
- [ ] Day 5: 记录和修复

### 第三周: 优化迭代
- [ ] 修复发现的问题
- [ ] 重新测试
- [ ] 性能优化
- [ ] 文档更新

---

## 🎯 成功里程碑

### Milestone 1: 基础连接 ✅
- ICE 连接建立成功
- DTLS 加密工作
- 网络传输稳定

### Milestone 2: 视频传输 🔄 当前
- 屏幕捕获工作
- 编码/解码正常
- 视频显示流畅

### Milestone 3: 输入控制 📋 计划
- 鼠标控制
- 键盘输入
- 快捷键支持

### Milestone 4: 跨架构验证 📋 计划
- ARM ↔ Intel 兼容
- 端到端测试通过
- MVP 完成 🎉

---

**制定人**: AI Assistant  
**制定日期**: 2026-06-28  
**版本**: v1.0  
**状态**: ✅ 待执行
