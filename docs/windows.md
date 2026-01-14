# Windows 数据库环境搭建指南

本文档提供在 Windows 上**完全规避容器技术**（Docker/Podman）的数据库安装方案，优先使用**官方二进制包**，其次使用 MSI 安装包。

支持的数据库：MongoDB、MySQL、Redis、Qdrant、SurrealDB

适配架构：AMD64（x86_64）/ ARM64（aarch64）

适配系统：Windows 10/11（64位）

## 一、前置准备

### 1.1 系统要求

- 操作系统：Windows 10/11（64位）
- 架构支持：AMD64（x86_64）/ ARM64（aarch64，如 Surface Pro X）

### 1.2 安装基础工具

- 解压工具：用于解压二进制包
- 命令行工具：Windows 终端/PowerShell（管理员模式运行）
- 文本编辑器：Notepad++/VS Code（编辑配置文件，避免记事本乱码）

### 1.3 统一目录规划

在 D 盘创建统一目录（避免 C 盘空间不足/权限限制）：

```powershell
# 以管理员身份打开 PowerShell，执行以下命令
md D:\.local-db\bin          # 存放所有数据库二进制文件
md D:\.local-db\config       # 存放所有数据库配置文件
md D:\.local-db\data         # 存放所有数据库数据
md D:\.local-db\logs         # 存放所有数据库日志
```

### 1.4 配置环境变量

**关键步骤（让命令全局可用）：**

1. 右键「此电脑」→「属性」→「高级系统设置」→「环境变量」
2. 在「系统变量」中找到「Path」→「编辑」→「新建」，添加 `D:\.local-db\bin`
3. 点击「确定」保存，关闭所有命令行窗口重新打开（环境变量生效）

## 二、数据库安装配置

### 2.1 Redis（二进制方式）

```powershell
# 1. 下载二进制包
# 访问 Redis Windows 官方镜像，下载最新稳定版（如 Redis-x64-5.0.14.1.zip）
# 下载地址：https://github.com/tporadowski/redis/releases

# 2. 解压到统一目录
# 将下载的 zip 包解压到 D:\.local-db\bin\redis

# 3. 创建数据和日志目录
md D:\.local-db\data\redis
md D:\.local-db\logs\redis

# 4. 复制并编辑配置文件
# 复制 D:\.local-db\bin\redis\redis.windows.conf 到 D:\.local-db\config\redis.conf
copy D:\.local-db\bin\redis\redis.windows.conf D:\.local-db\config\redis.conf
```

**编辑配置文件 `D:\.local-db\config\redis.conf`：**

```conf
# 数据目录
dir D:/.local-db/data/redis

# 日志文件
logfile "D:/.local-db/logs/redis/redis.log"

# 绑定本地IP
bind 127.0.0.1

# 端口
port 6379

# 守护进程模式
daemonize yes
```

**配置信息：**

- 配置文件：`D:\.local-db\config\redis.conf`
- 数据目录：`D:\.local-db\data\redis`
- 日志目录：`D:\.local-db\logs\redis`
- 默认端口：6379

### 2.2 MySQL（MSI 安装包方式）

```powershell
# 1. 下载 MSI 安装包
# 访问 MySQL 官方下载页，下载「MySQL Installer for Windows」（推荐社区版 8.0）
# 下载地址：https://dev.mysql.com/downloads/installer/

# 2. 安装步骤
# - 双击 MSI 文件，选择「Custom」自定义安装
# - 勾选「MySQL Server 8.0」，点击「Next」
# - 安装路径改为 D:\.local-db\bin\mysql

# 3. 配置关键步骤
# - Type and Networking：选择「Standalone MySQL Server」，端口保持 3306
# - Authentication Method：选择「Use Legacy Authentication Method」
# - Accounts and Roles：设置 root 密码（记住，后续登录用）
# - Windows Service：勾选「Configure MySQL as a Windows Service」，服务名默认「MySQL80」

# 4. 环境变量配置
# 将 D:\.local-db\bin\mysql\bin 添加到系统环境变量「Path」中
# 重新打开命令行窗口，验证：
mysql --version  # 显示版本则正常
```

**配置信息：**

- 安装目录：`D:\.local-db\bin\mysql`
- 数据目录：`D:\.local-db\bin\mysql\data`（由安装程序自动创建）
- 日志目录：`D:\.local-db\bin\mysql\data`
- 默认端口：3306
- 服务名称：MySQL80

### 2.3 MongoDB（二进制方式）

```powershell
# 1. 下载二进制包
# 访问 MongoDB 官方下载页，选择「Windows x64」版本（zip 包）
# 下载地址：https://www.mongodb.com/try/download/community
# 下载后解压到 D:\.local-db\bin\mongodb

# 2. 创建数据和日志目录
md D:\.local-db\data\mongodb
md D:\.local-db\logs\mongodb

# 3. 创建配置文件 D:\.local-db\config\mongodb.conf
```

**编辑配置文件 `D:\.local-db\config\mongodb.conf`：**

```yaml
systemLog:
  destination: file
  path: 'D:/.local-db/logs/mongodb/mongod.log'
  logAppend: true
storage:
  dbPath: 'D:/.local-db/data/mongodb'
  engine: wiredTiger
net:
  port: 27017
  bindIp: 127.0.0.1
```

**配置信息：**

- 配置文件：`D:\.local-db\config\mongodb.conf`
- 数据目录：`D:\.local-db\data\mongodb`
- 日志目录：`D:\.local-db\logs\mongodb`
- 默认端口：27017

### 2.4 Neo4j（二进制方式）

```powershell
# 1. 下载二进制包
# 访问 Neo4j 官方下载页，选择「Windows」版本（zip 包）
# 下载地址：https://neo4j.com/download-center/
# 下载后解压到 D:\.local-db\bin\neo4j

# 2. 创建数据和日志目录
md D:\.local-db\data\neo4j
md D:\.local-db\logs\neo4j

# 3. 复制并编辑配置文件
copy D:\.local-db\bin\neo4j\conf\neo4j.conf D:\.local-db\config\neo4j.conf
```

**编辑配置文件 `D:\.local-db\config\neo4j.conf`：**

```conf
# 数据目录
dbms.directories.data=D:/.local-db/data/neo4j

# 日志目录
dbms.directories.logs=D:/.local-db/logs/neo4j

# 允许本地访问
dbms.connector.http.listen_address=127.0.0.1:7474
dbms.connector.bolt.listen_address=127.0.0.1:7687

# 禁用远程访问（开发环境安全）
dbms.default_listen_address=127.0.0.1
```

**配置信息：**

- 配置文件：`D:\.local-db\config\neo4j.conf`
- 数据目录：`D:\.local-db\data\neo4j`
- 日志目录：`D:\.local-db\logs\neo4j`
- 默认端口：7474（HTTP）、7687（Bolt）
- 默认账号：neo4j/neo4j（首次登录需修改密码）

### 2.5 Qdrant（二进制方式）

```powershell
# 1. 下载二进制包（根据架构选择）
# 访问 Qdrant 官方 releases：https://github.com/qdrant/qdrant/releases
# - AMD64 架构：qdrant-v1.8.3-x86_64-pc-windows-msvc.zip
# - ARM64 架构（如 Surface Pro X）：qdrant-v1.8.3-aarch64-pc-windows-msvc.zip
# 下载后解压到 D:\.local-db\bin\qdrant

# 2. 创建数据和日志目录
md D:\.local-db\data\qdrant
md D:\.local-db\logs\qdrant

# 3. 创建配置文件 D:\.local-db\config\qdrant-config.yaml
```

**编辑配置文件 `D:\.local-db\config\qdrant-config.yaml`：**

```yaml
service:
  port: 6333
  grpc_port: 6334
storage:
  path: D:/.local-db/data/qdrant
logging:
  level: INFO
  file_path: D:/.local-db/logs/qdrant/qdrant.log
api_key: admin888
```

**配置信息：**

- 配置文件：`D:\.local-db\config\qdrant-config.yaml`
- 数据目录：`D:\.local-db\data\qdrant`
- 日志目录：`D:\.local-db\logs\qdrant`
- 默认端口：6333（HTTP）、6334（gRPC）
- API密钥：admin888

### 2.6 SurrealDB（二进制方式）

```powershell
# 1. 执行官方安装脚本
# 以「管理员身份」打开 PowerShell，执行：
iwr -useb https://install.surrealdb.com | iex

# 2. 移动二进制文件到统一目录
mv ~\AppData\Roaming\surreal\bin\surreal.exe D:\.local-db\bin\
rm -rf ~\AppData\Roaming\surreal

# 3. 创建数据和日志目录
md D:\.local-db\data\surrealdb
md D:\.local-db\logs\surrealdb
```

**配置信息：**

- 数据目录：`D:\.local-db\data\surrealdb`
- 日志目录：`D:\.local-db\logs\surrealdb`
- 默认端口：8000
- 默认账号：root/root

### 2.7 SeekDB（二进制方式）

```powershell
# 1. 下载二进制包
# 访问 SeekDB 官方 releases：https://github.com/seekdb/seekdb/releases
# 下载 Windows 版本（如 seekdb_0.1.0_windows_amd64.zip）
# 下载后解压到 D:\.local-db\bin\seekdb

# 2. 创建数据和日志目录
md D:\.local-db\data\seekdb
md D:\.local-db\logs\seekdb

# 3. 创建配置文件 D:\.local-db\config\seekdb.conf
```

**编辑配置文件 `D:\.local-db\config\seekdb.conf`：**

```conf
[server]
port = 8080
host = 127.0.0.1

[data]
dir = D:/.local-db/data/seekdb

[logging]
level = info
file = D:/.local-db/logs/seekdb/seekdb.log
```

**配置信息：**

- 配置文件：`D:\.local-db\config\seekdb.conf`
- 数据目录：`D:\.local-db\data\seekdb`
- 日志目录：`D:\.local-db\logs\seekdb`
- 默认端口：8080

## 三、总结

### 3.1 核心信息速查表

| 数据库    | 安装方式  | 默认端口  | 数据目录                    | 配置文件                               |
| --------- | --------- | --------- | --------------------------- | -------------------------------------- |
| Redis     | 二进制    | 6379      | D:\.local-db\data\redis     | D:\.local-db\config\redis.conf         |
| MySQL     | MSI安装包 | 3306      | D:\.local-db\bin\mysql\data | D:\.local-db\bin\mysql\my.ini          |
| MongoDB   | 二进制    | 27017     | D:\.local-db\data\mongodb   | D:\.local-db\config\mongodb.conf       |
| Neo4j     | 二进制    | 7474/7687 | D:\.local-db\data\neo4j     | D:\.local-db\config\neo4j.conf         |
| Qdrant    | 二进制    | 6333/6334 | D:\.local-db\data\qdrant    | D:\.local-db\config\qdrant-config.yaml |
| SurrealDB | 二进制    | 8000      | D:\.local-db\data\surrealdb | -                                      |
| SeekDB    | 二进制    | 8080      | D:\.local-db\data\seekdb    | D:\.local-db\config\seekdb.conf        |

### 3.2 关键注意事项

#### 3.2.1 架构适配

- 下载二进制包时务必根据系统架构（AMD64/ARM64）选择对应版本
- Windows 10/11 推荐 64 位版本，否则会提示「不是有效的 Win32 应用程序」

#### 3.2.2 权限问题

- 所有命令行/脚本务必以「管理员身份」运行，否则无法创建系统服务、写入目录
- 统一目录 `D:\.local-db` 需确保当前用户有「读写权限」（右键目录→「属性」→「安全」→「编辑」，赋予完全控制权限）

#### 3.2.3 端口占用检查

```powershell
# 查询指定端口占用进程（以 6379 为例）
netstat -ano | findstr :6379
# 或
Get-NetTCPConnection -LocalPort 6379

# 关闭占用进程（替换为实际 PID）
taskkill /f /pid <PID>
```

#### 3.2.4 配置文件路径问题

Windows 路径分隔符需用「`/`」或「`\\`」，避免直接用「`\`」（会被解析为转义字符）

**正确示例：**

- `D:/.local-db/data/redis`
- `D:\\.local-db\\data\\redis`

**错误示例：**

- `D:\.local-db\data\redis`

#### 3.2.5 方案特点

- 完全规避容器技术，所有服务均通过二进制/MSI 安装包安装
- 目录统一管理，便于维护和备份
- 适配 Windows 开发/测试场景，数据持久化、配置可定制
- 支持 AMD64/ARM64 多架构部署
