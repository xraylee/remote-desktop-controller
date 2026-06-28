# 🌏 国内镜像源最佳实践配置

根据社区反馈和实际测试，以下是各个工具的最佳国内镜像源配置。

---

## 📊 推荐镜像源（2026 年最新）

### 🦀 Rust / Cargo

**最佳选择**: **rsproxy.cn** (字节跳动)

**为什么？**
- ✅ 专为 Rust 优化
- ✅ 支持 sparse 协议（最快）
- ✅ 国内访问速度最快
- ✅ 稳定性高

**配置**:
```bash
# Rustup 安装
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# Cargo 依赖（项目中 .cargo/config.toml）
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
```

**备选**:
- USTC (中科大): `https://mirrors.ustc.edu.cn/crates.io-index`
- TUNA (清华): `https://mirrors.tuna.tsinghua.edu.cn/crates.io-index`

---

### 🐹 Go

**最佳选择**: **goproxy.cn** (七牛云)

**为什么？**
- ✅ 官方推荐
- ✅ 速度快，稳定
- ✅ 支持所有 Go 模块

**配置**:
```bash
go env -w GOPROXY=https://goproxy.cn,direct
go env -w GOSUMDB=sum.golang.google.cn
```

**备选**:
- goproxy.io: `https://goproxy.io`
- 阿里云: `https://mirrors.aliyun.com/goproxy`

---

### 🐦 Flutter

**最佳选择**: **pub.flutter-io.cn** + **storage.flutter-io.cn** (Flutter 中国社区)

**为什么？**
- ✅ Flutter 官方中国镜像
- ✅ 完整支持
- ✅ 实时同步

**配置**:
```bash
export PUB_HOSTED_URL="https://pub.flutter-io.cn"
export FLUTTER_STORAGE_BASE_URL="https://storage.flutter-io.cn"

# 永久配置
cat >> ~/.zshrc << 'EOL'
export PUB_HOSTED_URL="https://pub.flutter-io.cn"
export FLUTTER_STORAGE_BASE_URL="https://storage.flutter-io.cn"
EOL
```

**Flutter SDK 下载源**（按推荐顺序）:

1. **Homebrew** (macOS 最佳):
   ```bash
   brew install flutter
   ```

2. **清华大学 TUNA** (预编译包):
   ```bash
   # macOS Apple Silicon
   https://mirrors.tuna.tsinghua.edu.cn/flutter/flutter_infra_release/releases/stable/macos/flutter_macos_arm64_3.24.5-stable.zip
   
   # macOS Intel
   https://mirrors.tuna.tsinghua.edu.cn/flutter/flutter_infra_release/releases/stable/macos/flutter_macos_3.24.5-stable.zip
   
   # Linux
   https://mirrors.tuna.tsinghua.edu.cn/flutter/flutter_infra_release/releases/stable/linux/flutter_linux_3.24.5-stable.tar.xz
   ```

3. **Gitee** (Git 克隆):
   ```bash
   git clone https://gitee.com/mirrors/flutter.git -b stable --depth 1
   ```

**备选**:
- 上海交大 SJTUG: `https://mirrors.sjtug.sjtu.edu.cn/flutter`

---

### 📦 npm

**最佳选择**: **registry.npmmirror.com** (阿里云，原淘宝镜像)

**为什么？**
- ✅ 国内最大的 npm 镜像
- ✅ 实时同步
- ✅ 稳定可靠

**配置**:
```bash
npm config set registry https://registry.npmmirror.com
```

**备选**:
- 腾讯云: `https://mirrors.cloud.tencent.com/npm`
- 华为云: `https://mirrors.huaweicloud.com/repository/npm`

---

### 🍺 Homebrew (macOS)

**最佳选择**: **mirrors.tuna.tsinghua.edu.cn** (清华大学)

**为什么？**
- ✅ 教育网出口，速度快
- ✅ 完整同步
- ✅ 稳定性好

**配置**:
```bash
export HOMEBREW_API_DOMAIN="https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles/api"
export HOMEBREW_BOTTLE_DOMAIN="https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles"
export HOMEBREW_BREW_GIT_REMOTE="https://mirrors.tuna.tsinghua.edu.cn/git/homebrew/brew.git"
export HOMEBREW_CORE_GIT_REMOTE="https://mirrors.tuna.tsinghua.edu.cn/git/homebrew/homebrew-core.git"
```

**备选**:
- 中科大 USTC: `https://mirrors.ustc.edu.cn/homebrew-bottles`
- 阿里云: `https://mirrors.aliyun.com/homebrew`

---

## 🎯 完整配置脚本

将以下内容添加到 `~/.zshrc` 或 `~/.bashrc`:

```bash
# ============================================
# 国内镜像源配置（2026 最佳实践）
# ============================================

# Rust (rsproxy.cn)
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# Go (goproxy.cn)
export GOPROXY="https://goproxy.cn,direct"
export GOSUMDB="sum.golang.google.cn"

# Flutter (flutter-io.cn)
export PUB_HOSTED_URL="https://pub.flutter-io.cn"
export FLUTTER_STORAGE_BASE_URL="https://storage.flutter-io.cn"

# Homebrew (TUNA)
export HOMEBREW_API_DOMAIN="https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles/api"
export HOMEBREW_BOTTLE_DOMAIN="https://mirrors.tuna.tsinghua.edu.cn/homebrew-bottles"
export HOMEBREW_BREW_GIT_REMOTE="https://mirrors.tuna.tsinghua.edu.cn/git/homebrew/brew.git"
export HOMEBREW_CORE_GIT_REMOTE="https://mirrors.tuna.tsinghua.edu.cn/git/homebrew/homebrew-core.git"
```

---

## 📈 各镜像源对比

### Rust 镜像

| 镜像源 | 提供方 | 速度 | 稳定性 | 推荐度 |
|--------|--------|------|--------|--------|
| **rsproxy.cn** | 字节跳动 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| mirrors.ustc.edu.cn | 中科大 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| mirrors.tuna.tsinghua.edu.cn | 清华 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

### Go 代理

| 镜像源 | 提供方 | 速度 | 稳定性 | 推荐度 |
|--------|--------|------|--------|--------|
| **goproxy.cn** | 七牛云 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| goproxy.io | 国际 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| mirrors.aliyun.com/goproxy | 阿里云 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

### Flutter 镜像

| 镜像源 | 提供方 | 速度 | 稳定性 | 推荐度 |
|--------|--------|------|--------|--------|
| **pub.flutter-io.cn** | Flutter CN | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| mirrors.tuna.tsinghua.edu.cn | 清华 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| mirrors.sjtug.sjtu.edu.cn | 上交 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

### npm 镜像

| 镜像源 | 提供方 | 速度 | 稳定性 | 推荐度 |
|--------|--------|------|--------|--------|
| **registry.npmmirror.com** | 阿里云 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| mirrors.cloud.tencent.com | 腾讯云 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| mirrors.huaweicloud.com | 华为云 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

### Homebrew 镜像

| 镜像源 | 提供方 | 速度 | 稳定性 | 推荐度 |
|--------|--------|------|--------|--------|
| **mirrors.tuna.tsinghua.edu.cn** | 清华 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| mirrors.ustc.edu.cn | 中科大 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| mirrors.aliyun.com | 阿里云 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

---

## 🔄 切换镜像源

如果某个镜像源访问慢或出现问题，可以切换到备选源：

### Rust 切换

编辑 `.cargo/config.toml`:
```toml
[source.crates-io]
replace-with = 'ustc'  # 或 'tuna'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"

[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"
```

### Go 切换

```bash
go env -w GOPROXY=https://goproxy.io,https://goproxy.cn,direct
```

### npm 切换

```bash
npm config set registry https://mirrors.cloud.tencent.com/npm
```

---

## 💡 最佳实践建议

1. **优先使用 Homebrew** (macOS): 最简单，自动管理依赖
2. **配置所有镜像环境变量**: 一次配置，终身受益
3. **使用浅克隆**: Git 项目用 `--depth 1` 大幅减少下载量
4. **定期更新镜像配置**: 镜像源可能会变化
5. **备用方案**: 准备 2-3 个镜像源，以防主源失效

---

## 📚 参考资源

- [RsProxy 文档](https://rsproxy.cn)
- [Go Proxy 文档](https://goproxy.cn)
- [Flutter 中国镜像](https://flutter.cn)
- [清华大学开源镜像站](https://mirrors.tuna.tsinghua.edu.cn)
- [中科大开源镜像站](https://mirrors.ustc.edu.cn)

---

**更新时间**: 2026-06-27  
**适用范围**: 中国大陆网络环境
