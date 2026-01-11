# Linux 数据库环境搭建指南

本文档提供在 Linux 上**完全规避容器技术**（Docker/Podman）的数据库安装方案，优先使用**官方二进制包**，其次使用包管理器。

支持的数据库：MongoDB、MySQL、Redis、Neo4j、Qdrant、SeekDB、SurrealDB

适配架构：AMD64 / ARM64

适配系统：Ubuntu 20.04+、CentOS 7+、Debian 11+

## 一、前置准备

### 1.1 系统要求

- Linux 发行版：Ubuntu 20.04+/CentOS 7+/Debian 11+（64位）
- 架构支持：AMD64（x86_64）/ ARM64（aarch64）

### 1.2 安装基础工具

```bash
# Ubuntu/Debian 系统
sudo apt update
sudo apt install -y wget curl tar unzip vim libssl-dev

# CentOS/RHEL 系统
sudo yum install -y wget curl tar unzip vim openssl-devel
```

### 1.3 统一目录规划

```bash
# 创建核心目录（所有二进制/配置/数据统一存放）
mkdir -p /opt/.local-db/{bin,config,data,logs}

# 赋予当前用户权限（避免后续操作权限不足）
chown -R $USER:$USER /opt/.local-db

# 配置环境变量（让命令全局可用）
echo 'export PATH=$PATH:/opt/.local-db/bin' >> ~/.bashrc
source ~/.bashrc
```

## 二、数据库安装配置

### 2.1 Redis（包管理器方式）

```bash
# Ubuntu/Debian 系统
sudo apt update && sudo apt install -y redis-server

# CentOS/RHEL 系统
sudo yum install -y epel-release && sudo yum install -y redis

# 创建数据和日志目录
mkdir -p /opt/.local-db/data/redis /opt/.local-db/logs/redis

# 备份并移动配置文件到统一目录
sudo cp /etc/redis/redis.conf /opt/.local-db/config/redis.conf

# 修改配置文件
sudo sed -i 's|^dir .*|dir /opt/.local-db/data/redis|g' /opt/.local-db/config/redis.conf
sudo sed -i 's|^logfile .*|logfile /opt/.local-db/logs/redis/redis.log|g' /opt/.local-db/config/redis.conf
sudo sed -i 's|^bind .*|bind 127.0.0.1|g' /opt/.local-db/config/redis.conf
sudo sed -i 's|^port .*|port 6379|g' /opt/.local-db/config/redis.conf
sudo sed -i 's|^daemonize .*|daemonize yes|g' /opt/.local-db/config/redis.conf
sudo sed -i 's|^protected-mode .*|protected-mode no|g' /opt/.local-db/config/redis.conf

# 创建软链接（保持系统默认配置路径关联）
sudo ln -sf /opt/.local-db/config/redis.conf /etc/redis/redis.conf

# 授权目录
sudo chown -R redis:redis /opt/.local-db/data/redis /opt/.local-db/logs/redis
```

**配置信息：**

- 配置文件：`/opt/.local-db/config/redis.conf`
- 数据目录：`/opt/.local-db/data/redis`
- 日志目录：`/opt/.local-db/logs/redis`
- 默认端口：6379

### 2.2 MySQL（二进制方式）

### 2.2 MySQL（二进制方式）

```bash
# 1. 下载二进制包（根据架构选择）
cd /tmp

# AMD64 架构
wget https://cdn.mysql.com/Downloads/MySQL-8.0/mysql-8.0.36-linux-glibc2.28-x86_64.tar.xz

# ARM64 架构
# wget https://cdn.mysql.com/Downloads/MySQL-8.0/mysql-8.0.36-linux-glibc2.28-aarch64.tar.xz

# 2. 解压到统一目录
tar -xvf mysql-8.0.36-linux-glibc2.28-*.tar.xz -C /opt/.local-db/bin/
mv /opt/.local-db/bin/mysql-8.0.36-linux-glibc2.28-* /opt/.local-db/bin/mysql
rm -f mysql-8.0.36-linux-glibc2.28-*.tar.xz

# 3. 创建 MySQL 专用用户和组
sudo groupadd mysql
sudo useradd -r -g mysql -s /sbin/nologin mysql

# 4. 创建数据和日志目录并授权
mkdir -p /opt/.local-db/data/mysql /opt/.local-db/logs/mysql
sudo chown -R mysql:mysql /opt/.local-db/data/mysql /opt/.local-db/logs/mysql
chmod 750 /opt/.local-db/data/mysql

# 5. 初始化 MySQL（生成临时密码，记录下来）
/opt/.local-db/bin/mysql/bin/mysqld --initialize --user=mysql --datadir=/opt/.local-db/data/mysql --basedir=/opt/.local-db/bin/mysql

# 6. 编写配置文件
cat > /opt/.local-db/config/my.cnf << EOF
[mysqld]
datadir=/opt/.local-db/data/mysql
basedir=/opt/.local-db/bin/mysql
socket=/tmp/mysql.sock
port=3306
character-set-server=utf8mb4
collation-server=utf8mb4_unicode_ci
log-error=/opt/.local-db/logs/mysql/mysqld.log
pid-file=/opt/.local-db/data/mysql/mysqld.pid

[client]
socket=/tmp/mysql.sock
default-character-set=utf8mb4
EOF
```

**配置信息：**

- 配置文件：`/opt/.local-db/config/my.cnf`
- 数据目录：`/opt/.local-db/data/mysql`
- 日志目录：`/opt/.local-db/logs/mysql`
- 默认端口：3306
- 临时密码查看：`grep 'temporary password' /opt/.local-db/logs/mysql/mysqld.log`

### 2.3 MongoDB（二进制方式）

### 2.3 MongoDB（二进制方式）

```bash
# 1. 下载二进制包（根据架构选择）
cd /tmp

# AMD64 架构
wget https://fastdl.mongodb.org/linux/mongodb-linux-x86_64-ubuntu2004-6.0.13.tgz

# ARM64 架构
# wget https://fastdl.mongodb.org/linux/mongodb-linux-aarch64-ubuntu2004-6.0.13.tgz

# 2. 解压到统一目录
tar -zxvf mongodb-linux-*-6.0.13.tgz -C /opt/.local-db/bin/
mv /opt/.local-db/bin/mongodb-linux-*-6.0.13 /opt/.local-db/bin/mongodb
rm -f mongodb-linux-*-6.0.13.tgz

# 3. 创建数据和日志目录
mkdir -p /opt/.local-db/data/mongodb /opt/.local-db/logs/mongodb

# 4. 编写配置文件
cat > /opt/.local-db/config/mongod.conf << EOF
systemLog:
  destination: file
  path: "/opt/.local-db/logs/mongodb/mongod.log"
  logAppend: true
storage:
  dbPath: "/opt/.local-db/data/mongodb"
  engine: wiredTiger
net:
  port: 27017
  bindIp: 127.0.0.1
processManagement:
  fork: true
  pidFilePath: /opt/.local-db/data/mongodb/mongod.pid
EOF
```

**配置信息：**

- 配置文件：`/opt/.local-db/config/mongod.conf`
- 数据目录：`/opt/.local-db/data/mongodb`
- 日志目录：`/opt/.local-db/logs/mongodb`
- 默认端口：27017

### 2.4 Qdrant（二进制方式）

### 2.4 Qdrant（二进制方式）

```bash
# 1. 下载二进制包（根据架构选择）
cd /tmp

# AMD64 架构
wget https://github.com/qdrant/qdrant/releases/download/v1.8.3/qdrant-v1.8.3-x86_64-unknown-linux-gnu.tar.gz

# ARM64 架构
# wget https://github.com/qdrant/qdrant/releases/download/v1.8.3/qdrant-v1.8.3-aarch64-unknown-linux-gnu.tar.gz

# 2. 解压到统一目录
tar -zxvf qdrant-v1.8.3-*.tar.gz -C /opt/.local-db/bin/
rm -f qdrant-v1.8.3-*.tar.gz

# 3. 创建数据和日志目录
mkdir -p /opt/.local-db/data/qdrant /opt/.local-db/logs/qdrant

# 4. 编写配置文件
cat > /opt/.local-db/config/qdrant-config.yaml << EOF
service:
  port: 6333
  grpc_port: 6334
storage:
  path: /opt/.local-db/data/qdrant
logging:
  level: INFO
  file_path: /opt/.local-db/logs/qdrant/qdrant.log
api_key: admin888
EOF
```

**配置信息：**

- 配置文件：`/opt/.local-db/config/qdrant-config.yaml`
- 数据目录：`/opt/.local-db/data/qdrant`
- 日志目录：`/opt/.local-db/logs/qdrant`
- 默认端口：6333（HTTP）、6334（gRPC）
- API密钥：admin888

### 2.5 Neo4j（二进制方式）

```bash
# 1. 下载二进制包
cd /tmp
wget https://dist.neo4j.org/neo4j-community-5.18.1-unix.tar.gz

# 2. 解压到统一目录
tar -zxvf neo4j-community-5.18.1-unix.tar.gz -C /opt/.local-db/bin/
mv /opt/.local-db/bin/neo4j-community-5.18.1 /opt/.local-db/bin/neo4j
rm -f neo4j-community-5.18.1-unix.tar.gz

# 3. 创建数据和日志目录
mkdir -p /opt/.local-db/data/neo4j /opt/.local-db/logs/neo4j

# 4. 编写配置文件
cat > /opt/.local-db/config/neo4j.conf << EOF
# 数据目录
dbms.directories.data=/opt/.local-db/data/neo4j

# 日志目录
dbms.directories.logs=/opt/.local-db/logs/neo4j

# 允许本地访问
dbms.connector.http.listen_address=127.0.0.1:7474
dbms.connector.bolt.listen_address=127.0.0.1:7687

# 禁用远程访问（开发环境安全）
dbms.default_listen_address=127.0.0.1

# 启用密码认证
dbms.security.auth_enabled=true
EOF

# 5. 创建软链接（替换默认配置）
rm -f /opt/.local-db/bin/neo4j/conf/neo4j.conf
ln -s /opt/.local-db/config/neo4j.conf /opt/.local-db/bin/neo4j/conf/neo4j.conf
```

**配置信息：**

- 配置文件：`/opt/.local-db/config/neo4j.conf`
- 数据目录：`/opt/.local-db/data/neo4j`
- 日志目录：`/opt/.local-db/logs/neo4j`
- 默认端口：7474（HTTP）、7687（Bolt）
- 默认账号：neo4j/neo4j（首次登录需修改密码）

### 2.6 SurrealDB（二进制方式）

### 2.6 SurrealDB（二进制方式）

```bash
# 1. 执行官方安装脚本
curl -sSf https://install.surrealdb.com | sh

# 2. 移动二进制文件到统一目录
mv ~/.surrealdb/bin/surreal /opt/.local-db/bin/
rm -rf ~/.surrealdb

# 3. 创建数据和日志目录
mkdir -p /opt/.local-db/data/surrealdb /opt/.local-db/logs/surrealdb
```

**配置信息：**

- 数据目录：`/opt/.local-db/data/surrealdb`
- 日志目录：`/opt/.local-db/logs/surrealdb`
- 默认端口：8000
- 默认账号：root/root

### 2.7 SeekDB（二进制方式）

```bash
# 1. 下载二进制包（根据架构选择）
cd /tmp

# AMD64 架构
wget https://github.com/seekdb/seekdb/releases/download/v0.1.0/seekdb_0.1.0_linux_amd64.tar.gz

# ARM64 架构
# wget https://github.com/seekdb/seekdb/releases/download/v0.1.0/seekdb_0.1.0_linux_arm64.tar.gz

# 2. 解压到统一目录
tar -zxvf seekdb_0.1.0_linux_*.tar.gz -C /opt/.local-db/bin/
rm -f seekdb_0.1.0_linux_*.tar.gz

# 3. 创建数据和日志目录
mkdir -p /opt/.local-db/data/seekdb /opt/.local-db/logs/seekdb

# 4. 编写配置文件
cat > /opt/.local-db/config/seekdb.conf << EOF
[server]
port = 8080
host = 127.0.0.1

[data]
dir = /opt/.local-db/data/seekdb

[logging]
level = info
file = /opt/.local-db/logs/seekdb/seekdb.log
EOF
```

**配置信息：**

- 配置文件：`/opt/.local-db/config/seekdb.conf`
- 数据目录：`/opt/.local-db/data/seekdb`
- 日志目录：`/opt/.local-db/logs/seekdb`
- 默认端口：8080

## 三、总结

## 三、总结

### 3.1 核心信息速查表

| 数据库    | 安装方式 | 默认端口  | 数据目录                      | 配置文件                                 |
| --------- | -------- | --------- | ----------------------------- | ---------------------------------------- |
| Redis     | 包管理器 | 6379      | /opt/.local-db/data/redis     | /opt/.local-db/config/redis.conf         |
| MySQL     | 二进制   | 3306      | /opt/.local-db/data/mysql     | /opt/.local-db/config/my.cnf             |
| MongoDB   | 二进制   | 27017     | /opt/.local-db/data/mongodb   | /opt/.local-db/config/mongod.conf        |
| Neo4j     | 二进制   | 7474/7687 | /opt/.local-db/data/neo4j     | /opt/.local-db/config/neo4j.conf         |
| Qdrant    | 二进制   | 6333/6334 | /opt/.local-db/data/qdrant    | /opt/.local-db/config/qdrant-config.yaml |
| SurrealDB | 二进制   | 8000      | /opt/.local-db/data/surrealdb | -                                        |
| SeekDB    | 二进制   | 8080      | /opt/.local-db/data/seekdb    | /opt/.local-db/config/seekdb.conf        |

### 3.2 关键注意事项

#### 3.2.1 架构适配

- 下载二进制包时务必根据服务器架构（AMD64/ARM64）选择对应版本
- 查询架构命令：`uname -m`（x86_64 为 AMD64，aarch64 为 ARM64）

#### 3.2.2 权限问题

- 所有操作建议使用 `sudo` 或切换到 root 用户（`sudo -i`）
- 统一目录 `/opt/.local-db` 需确保对应服务用户有读写权限
- MySQL 使用 mysql 用户，其他服务使用当前用户

#### 3.2.3 依赖库缺失

- 若启动服务时提示「error while loading shared libraries」，说明缺失依赖库
- Ubuntu/Debian：`sudo apt install libssl-dev`
- CentOS/RHEL：`sudo yum install openssl-devel`

#### 3.2.4 端口占用检查

```bash
# 查询指定端口占用进程（以 6379 为例）
netstat -tulnp | grep :6379
# 或
lsof -i :6379

# 关闭占用进程（替换为实际 PID）
kill -9 <PID>
```

#### 3.2.5 方案特点

- 完全规避容器技术，所有服务均通过二进制/包管理器安装
- 目录统一管理，便于维护和备份
- 适配 Linux 开发/测试场景，数据持久化、配置可定制
- 支持 AMD64/ARM64 多架构部署
