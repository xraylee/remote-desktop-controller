# STUN/TURN 服务器部署指南

**目的**: 为 RDCS NAT 穿透测试提供真实的 STUN/TURN 服务器环境

---

## 1. STUN 服务器部署

### 使用 coturn (推荐)

```bash
# Docker 部署 STUN 服务器
docker run -d \
  --name rdcs-stun-server \
  --restart=always \
  -p 3478:3478/udp \
  -p 3478:3478/tcp \
  coturn/coturn \
  -n \
  --log-file=stdout \
  --listening-ip=0.0.0.0 \
  --external-ip=$(curl -s ifconfig.me) \
  --fingerprint \
  --lt-cred-mech \
  --realm=rdcs.io

# 验证 STUN 服务器
stunclient <server-ip>:3478
```

### 配置文件 (`turnserver.conf`)

```ini
# STUN 基本配置
listening-port=3478
fingerprint
lt-cred-mech
realm=rdcs.io

# 日志配置
log-file=/var/log/turnserver.log
verbose

# 不需要认证（STUN only）
no-auth
```

---

## 2. TURN 中继服务器部署

### 多区域部署策略

```
区域         服务器位置            延迟目标
---------------------------------------------
us-west      美国西海岸 (俄勒冈)   <50ms (北美西部)
us-east      美国东海岸 (弗吉尼亚) <50ms (北美东部)
eu-central   欧洲中部 (法兰克福)   <80ms (欧洲)
ap-southeast 亚太东南 (新加坡)     <100ms (亚洲)
```

### 单节点 TURN 部署

```bash
# Docker 部署 TURN 服务器（带认证）
docker run -d \
  --name rdcs-turn-server \
  --restart=always \
  -p 3478:3478/udp \
  -p 3478:3478/tcp \
  -p 49152-65535:49152-65535/udp \
  -e REALM=rdcs.io \
  -e USERNAME=rdcs-user \
  -e PASSWORD=<strong-password> \
  -e MIN_PORT=49152 \
  -e MAX_PORT=65535 \
  coturn/coturn
```

### 完整配置文件

```ini
# /etc/turnserver.conf

# 监听端口
listening-port=3478
alt-listening-port=3479

# 中继端口范围
min-port=49152
max-port=65535

# 认证配置
lt-cred-mech
realm=rdcs.io
user=rdcs-user:rdcs-password

# 外部 IP（公网 IP）
external-ip=<server-public-ip>

# 带宽限制（免费版）
max-bps=2000000          # 2 Mbps
bps-capacity=4000000     # 4 Mbps burst

# 会话限制
user-quota=10            # 每用户最多 10 个会话
total-quota=100          # 服务器总计 100 个会话

# 日志
log-file=/var/log/turnserver.log
verbose

# 安全配置
no-tlsv1
no-tlsv1_1
fingerprint
```

---

## 3. 防火墙配置

### STUN 端口

```bash
# UDP 3478 (STUN)
sudo ufw allow 3478/udp
sudo ufw allow 3478/tcp
```

### TURN 端口

```bash
# UDP 3478 (TURN 信令)
sudo ufw allow 3478/udp
sudo ufw allow 3478/tcp

# UDP 49152-65535 (中继数据)
sudo ufw allow 49152:65535/udp
```

---

## 4. 客户端配置

### Rust 配置

```rust
// crates/rdcs-signaling/src/ice_config.rs

pub struct IceServerConfig {
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<TurnServerConfig>,
}

pub struct TurnServerConfig {
    pub urls: Vec<String>,
    pub username: String,
    pub credential: String,
    pub region: String,
}

impl Default for IceServerConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.rdcs.io:3478".to_string(),
                "stun:stun.l.google.com:19302".to_string(), // 备用
            ],
            turn_servers: vec![
                TurnServerConfig {
                    urls: vec![
                        "turn:turn-us-west.rdcs.io:3478?transport=udp".to_string(),
                        "turn:turn-us-west.rdcs.io:3478?transport=tcp".to_string(),
                    ],
                    username: "rdcs-user".to_string(),
                    credential: "<password>".to_string(),
                    region: "us-west".to_string(),
                },
                TurnServerConfig {
                    urls: vec![
                        "turn:turn-us-east.rdcs.io:3478?transport=udp".to_string(),
                    ],
                    username: "rdcs-user".to_string(),
                    credential: "<password>".to_string(),
                    region: "us-east".to_string(),
                },
            ],
        }
    }
}
```

### 信令服务器集成

```go
// backend/signaling/ice_config.go

type IceConfiguration struct {
    StunServers []string      `json:"stun_servers"`
    TurnServers []TurnServer  `json:"turn_servers"`
}

type TurnServer struct {
    URLs       []string `json:"urls"`
    Username   string   `json:"username"`
    Credential string   `json:"credential"`
    Region     string   `json:"region"`
}

func GetIceConfiguration(clientRegion string) *IceConfiguration {
    return &IceConfiguration{
        StunServers: []string{
            "stun:stun.rdcs.io:3478",
            "stun:stun.l.google.com:19302",
        },
        TurnServers: selectNearestTurnServers(clientRegion),
    }
}
```

---

## 5. 健康检查和监控

### STUN 健康检查

```bash
#!/bin/bash
# scripts/check-stun.sh

STUN_SERVER="stun.rdcs.io:3478"

stunclient $STUN_SERVER > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "STUN server is healthy"
    exit 0
else
    echo "STUN server is down"
    exit 1
fi
```

### TURN 健康检查

```bash
#!/bin/bash
# scripts/check-turn.sh

TURN_SERVER="turn.rdcs.io:3478"
USERNAME="rdcs-user"
PASSWORD="<password>"

turnutils_uclient -v -u $USERNAME -w $PASSWORD $TURN_SERVER > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "TURN server is healthy"
    exit 0
else
    echo "TURN server is down"
    exit 1
fi
```

### 监控指标

```
1. 服务可用性
   - STUN 响应率 >99.9%
   - TURN 响应率 >99.5%

2. 性能指标
   - STUN 响应延迟 <50ms
   - TURN 建立时间 <500ms
   - 中继延迟开销 <20ms

3. 容量指标
   - 并发会话数
   - 带宽使用量
   - 每用户配额使用率

4. 成本指标
   - 中继使用率（目标 <40%）
   - 流量成本
   - 服务器成本
```

---

## 6. 测试验证

### 本地测试

```bash
# 测试 STUN 响应
cargo test --test nat_traversal_test test_stun_server_reachability

# 测试 TURN 分配
cargo test --test nat_traversal_test test_turn_allocation

# 测试完整 ICE 流程
cargo test --test nat_traversal_test test_ice_with_real_servers
```

### 跨网络测试

```bash
# 家庭网络 vs 家庭网络
cargo test --test nat_traversal_test test_home_to_home_connection

# 家庭网络 vs 企业网络
cargo test --test nat_traversal_test test_home_to_corporate_connection

# 移动网络 vs Wi-Fi
cargo test --test nat_traversal_test test_mobile_to_wifi_connection
```

---

## 7. 成本估算

### 服务器成本（每月）

```
服务类型         实例规格        数量    月成本
------------------------------------------------
STUN 服务器      t3.micro        1       $8
TURN 中继节点    t3.small        4       $68
负载均衡器       ALB             1       $20
监控和日志       CloudWatch      -       $10
------------------------------------------------
总计                                     $106/月
```

### 流量成本

```
用户规模     中继使用率    平均流量      月流量成本
-----------------------------------------------------
100 用户     30%          5 GB/用户     $50
500 用户     35%          5 GB/用户     $250
1000 用户    40%          5 GB/用户     $500
```

### 优化策略

1. **优化 P2P 成功率**: 目标 >60%，减少中继使用
2. **就近路由**: 智能选择最近的中继节点
3. **带宽限流**: 免费版限制 2 Mbps
4. **会话限制**: 每用户最多 10 个并发会话

---

## 8. 部署清单

### Phase 1: 基础部署（1-2天）

- [ ] 部署 1 个 STUN 服务器
- [ ] 部署 1 个 TURN 测试节点（us-west）
- [ ] 配置防火墙规则
- [ ] 更新客户端配置
- [ ] 验证基本连接

### Phase 2: 扩展部署（3-5天）

- [ ] 部署 3 个额外 TURN 节点（us-east, eu-central, ap-southeast）
- [ ] 配置负载均衡和健康检查
- [ ] 实现就近节点选择
- [ ] 配置带宽限流
- [ ] 部署监控和告警

### Phase 3: 优化（1周）

- [ ] 性能调优
- [ ] 成本优化
- [ ] 容量规划
- [ ] 灾难恢复

---

## 9. 故障排查

### 常见问题

#### STUN 服务器无响应

```bash
# 检查端口
netstat -tunlp | grep 3478

# 检查防火墙
sudo ufw status

# 查看日志
docker logs rdcs-stun-server
```

#### TURN 分配失败

```bash
# 检查认证配置
cat /etc/turnserver.conf | grep -A 5 "lt-cred-mech"

# 检查端口范围
netstat -tunlp | grep 49152

# 测试手动分配
turnutils_uclient -v -u rdcs-user -w <password> turn.rdcs.io:3478
```

#### 高延迟

```bash
# 检查网络路径
traceroute turn.rdcs.io

# 检查服务器负载
top
htop

# 检查带宽使用
iftop
```

---

## 10. 参考资料

- [RFC 5389 - STUN](https://tools.ietf.org/html/rfc5389)
- [RFC 5766 - TURN](https://tools.ietf.org/html/rfc5766)
- [RFC 8445 - ICE](https://tools.ietf.org/html/rfc8445)
- [coturn Documentation](https://github.com/coturn/coturn)
- [WebRTC Samples](https://webrtc.github.io/samples/)
