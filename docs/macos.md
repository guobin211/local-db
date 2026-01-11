# macOS 数据库环境搭建指南

本文档提供在 macOS 上**完全规避容器技术**（Docker/Podman）的数据库安装方案，优先使用**官方二进制包**，其次使用 Homebrew Services。

支持的数据库：MongoDB、MySQL、Redis、Neo4j、Qdrant、SeekDB、SurrealDB

适配架构：Intel / Apple Silicon（M1/M2/M3）

## 一、前置准备

### 1.1 安装 Homebrew

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew update && brew upgrade
```

### 1.2 统一目录规划

```bash
# 创建核心目录（所有二进制/配置/数据统一存放）
mkdir -p ~/.local-db/{bin,config,data,logs}
# 加入环境变量（永久生效）
echo 'export PATH="$HOME/.local-db/bin:$PATH"' >> ~/.zshrc # 若用bash则改~/.bashrc
source ~/.zshrc
```

## 二、数据库安装配置

### 2.1 Redis（Homebrew 方式）

```bash
# 安装
brew install redis

# 配置(自定义数据/日志目录,避免默认路径权限问题)
sed -i '' 's#dir /usr/local/var/db/redis#dir ~/.local-db/data/redis#g' $(brew --prefix)/etc/redis.conf
sed -i '' 's#logfile ""#logfile ~/.local-db/logs/redis.log#g' $(brew --prefix)/etc/redis.conf
# 启动（开机自启）
brew services start redis

# 验证
redis-cli ping # 返回 PONG 则正常
```

**配置信息：**

- 配置文件：`$(brew --prefix)/etc/redis.conf`
- 数据目录：`~/.local-db/data/redis`
- 日志文件：`~/.local-db/logs/redis.log`
- 默认端口：6379

### 2.2 MySQL（Homebrew 方式）

```bash
# 安装 MySQL 8.0
brew install mysql

# 配置自定义目录
mkdir -p ~/.local-db/data/mysql ~/.local-db/logs/mysql
# 修改配置文件
cat > $(brew --prefix)/etc/my.cnf << EOF
[mysqld]
datadir = ~/.local-db/data/mysql
socket = ~/.local-db/data/mysql/mysql.sock
log-error = ~/.local-db/logs/mysql/mysqld.log
pid-file = ~/.local-db/data/mysql/mysqld.pid
port = 3306
user = $(whoami)
[client]
socket = ~/.local-db/data/mysql/mysql.sock
port = 3306
EOF

# 初始化(首次安装需执行)
mysqld --initialize-insecure --user=$(whoami) --datadir=~/.local-db/data/mysql
# 启动（开机自启）
brew services start mysql

# 安全配置（设置root密码）
mysql_secure_installation

# 验证
mysql -u root -p # 输入密码后执行 SELECT VERSION(); 正常返回版本则成功
```

**配置信息：**

- 配置文件：`$(brew --prefix)/etc/my.cnf`
- 数据目录：`~/.local-db/data/mysql`
- 日志目录：`~/.local-db/logs/mysql`
- 默认端口：3306

### 2.3 MongoDB（二进制方式）

```bash
# 1. 下载适配macOS的二进制包（Apple Silicon/Intel自动识别）
curl -O https://fastdl.mongodb.org/osx/mongodb-macos-x86_64-7.0.11.tgz # Intel
# Apple Silicon 替换为：curl -O https://fastdl.mongodb.org/osx/mongodb-macos-arm64-7.0.11.tgz

# 2. 解压并移动到自定义目录
tar -zxvf mongodb-macos-*.tgz
cp -r mongodb-macos-*/bin/* ~/.local-db/bin/
rm -rf mongodb-macos-*.tgz mongodb-macos-*

# 3. 创建配置/数据/日志目录
mkdir -p ~/.local-db/config/mongodb ~/.local-db/data/mongodb ~/.local-db/logs/mongodb

# 4. 编写配置文件
cat > ~/.local-db/config/mongodb/mongod.conf << EOF
systemLog:
  destination: file
  path: ~/.local-db/logs/mongodb/mongod.log
  logAppend: true
storage:
  dbPath: ~/.local-db/data/mongodb
  engine: wiredTiger
net:
  port: 27017
  bindIp: 127.0.0.1
processManagement:
  fork: true
  pidFilePath: ~/.local-db/data/mongodb/mongod.pid
EOF

# 5. 启动(配置开机自启见下方补充)
mongod --config ~/.local-db/config/mongodb/mongod.conf
# 6. 验证
mongosh --port 27017 # 执行 db.version() 正常返回版本则成功
```

**启动脚本（简化命令）：**

```bash
# 创建启动脚本
cat > ~/.local-db/bin/start-mongodb.sh << EOF
#!/bin/bash
mongod --config ~/.local-db/config/mongodb/mongod.conf
EOF
chmod +x ~/.local-db/bin/start-mongodb.sh
# 启动：start-mongodb.sh
# 停止：mongod --config ~/.local-db/config/mongodb/mongod.conf --shutdown
```

**开机自启：** 将启动脚本添加到 `~/Library/LaunchAgents/com.mongodb.plist`（见第三章）

### 2.4 Qdrant（二进制方式）

```bash
# 1. 下载适配macOS的二进制包
# Apple Silicon（M1/M2/M3）
curl -L https://github.com/qdrant/qdrant/releases/download/v1.11.1/qdrant-v1.11.1-aarch64-apple-darwin.tar.gz -o qdrant.tar.gz
# Intel
# curl -L https://github.com/qdrant/qdrant/releases/download/v1.11.1/qdrant-v1.11.1-x86_64-apple-darwin.tar.gz -o qdrant.tar.gz

# 2. 解压并移动到自定义目录
tar -zxvf qdrant.tar.gz
cp qdrant ~/.local-db/bin/
rm -rf qdrant.tar.gz qdrant

# 3. 创建配置/数据/日志目录
mkdir -p ~/.local-db/config/qdrant ~/.local-db/data/qdrant ~/.local-db/logs/qdrant

# 4. 编写配置文件
cat > ~/.local-db/config/qdrant/config.yaml << EOF
service:
  port: 6333
  grpc_port: 6334
storage:
  path: ~/.local-db/data/qdrant
logging:
  level: INFO
  file_path: ~/.local-db/logs/qdrant/qdrant.log
api_key: admin888
EOF

# 5. 启动(后台运行)
nohup qdrant --config ~/.local-db/config/qdrant/config.yaml > ~/.local-db/logs/qdrant/nohup.log 2>&1 &
# 6. 验证
curl http://localhost:6333/health?api_key=admin888 # 返回 {"status":"ok"} 则正常
```

**管理命令：**

- 停止：`pkill qdrant`
- 启动脚本：
  ```bash
  cat > ~/.local-db/bin/start-qdrant.sh << EOF
  #!/bin/bash
  nohup qdrant --config ~/.local-db/config/qdrant/config.yaml > ~/.local-db/logs/qdrant/nohup.log 2>&1 &
  EOF
  chmod +x ~/.local-db/bin/start-qdrant.sh
  ```

### 2.5 Neo4j（二进制方式）

```bash
# 1. 下载二进制包（5.18.0稳定版）
# Apple Silicon
curl -L https://neo4j.com/dist/neo4j-community-5.18.0-unix.tar.gz -o neo4j.tar.gz
# Intel 同上述链接（Neo4j二进制包适配全架构）

# 2. 解压并移动
tar -zxvf neo4j.tar.gz
mv neo4j-community-5.18.0 ~/.local-db/neo4j
rm -rf neo4j.tar.gz

# 3. 配置自定义目录
mkdir -p ~/.local-db/data/neo4j ~/.local-db/logs/neo4j ~/.local-db/config/neo4j
cp ~/.local-db/neo4j/conf/neo4j.conf ~/.local-db/config/neo4j/
# 修改配置文件
sed -i '' 's#dbms.directories.data=data#dbms.directories.data=~/.local-db/data/neo4j#g' ~/.local-db/config/neo4j/neo4j.conf
sed -i '' 's#dbms.directories.logs=logs#dbms.directories.logs=~/.local-db/logs/neo4j#g' ~/.local-db/config/neo4j/neo4j.conf
sed -i '' 's#dbms.connector.bolt.listen_address=:7687#dbms.connector.bolt.listen_address=:7687#g' ~/.local-db/config/neo4j/neo4j.conf
sed -i '' 's#dbms.connector.http.listen_address=:7474#dbms.connector.http.listen_address=:7474#g' ~/.local-db/config/neo4j/neo4j.conf

# 4. 启动
~/.local-db/neo4j/bin/neo4j start --config ~/.local-db/config/neo4j/neo4j.conf

# 5. 验证
curl http://localhost:7474 # 浏览器访问http://localhost:7474，默认账号neo4j/neo4j，修改密码后正常则成功
```

**管理命令：**

- 停止：`~/.local-db/neo4j/bin/neo4j stop`
- 状态：`~/.local-db/neo4j/bin/neo4j status`

### 2.6 SurrealDB（二进制方式）

```bash
# 1. 下载二进制包（自动适配架构）
curl -sSf https://install.surrealdb.com | sh

# 2. 移动到自定义目录
mv ~/.surreal/bin/surreal ~/.local-db/bin/
rm -rf ~/.surreal

# 3. 创建数据/日志目录
mkdir -p ~/.local-db/data/surrealdb ~/.local-db/logs/surrealdb

# 4. 启动(后台运行 + 认证 + 持久化)
nohup surreal start --log debug --user root --pass root file://~/.local-db/data/surrealdb > ~/.local-db/logs/surrealdb/nohup.log 2>&1 &
# 5. 验证
surreal sql --conn http://localhost:8000 --user root --pass root --ns test --db test
# 执行 SELECT * FROM test; 无报错则正常
```

**管理命令：**

- 停止：`pkill surreal`

### 2.7 SeekDB（二进制方式）

```bash
# 1. 下载macOS二进制包（需从官方获取最新版，示例为0.1.0）
curl -L https://github.com/seekdb/seekdb/releases/download/v0.1.0/seekdb_0.1.0_darwin_amd64.tar.gz -o seekdb.tar.gz # Intel
# Apple Silicon 需自行编译或等待官方发布，临时方案：用Rosetta兼容运行Intel版本

# 2. 解压并移动
tar -zxvf seekdb.tar.gz
cp seekdb ~/.local-db/bin/
rm -rf seekdb.tar.gz seekdb

# 3. 创建数据/日志/配置目录
mkdir -p ~/.local-db/data/seekdb ~/.local-db/logs/seekdb ~/.local-db/config/seekdb

# 4. 编写配置文件
cat > ~/.local-db/config/seekdb/seekdb.conf << EOF
[server]
port = 8080
host = 127.0.0.1
[data]
dir = ~/.local-db/data/seekdb
[logging]
level = info
file = ~/.local-db/logs/seekdb/seekdb.log
EOF

# 5. 启动(后台运行)
nohup seekdb server --config ~/.local-db/config/seekdb/seekdb.conf > ~/.local-db/logs/seekdb/nohup.log 2>&1 &
# 6. 验证
curl http://localhost:8080/health # 返回 {"status":"healthy"} 则正常
```

**管理命令：**

- 停止：`pkill seekdb`

**注意：** Apple Silicon 需自行编译或等待官方发布，临时方案可使用 Rosetta 兼容运行 Intel 版本

## 三、总结

### 3.1 核心信息速查表

| 数据库    | 安装方式 | 启动方式                  | 停止方式                       | 默认端口  | 核心目录                   |
| --------- | -------- | ------------------------- | ------------------------------ | --------- | -------------------------- |
| Redis     | Homebrew | brew services start redis | brew services stop redis       | 6379      | ~/.local-db/data/redis     |
| MySQL     | Homebrew | brew services start mysql | brew services stop mysql       | 3306      | ~/.local-db/data/mysql     |
| MongoDB   | 二进制   | start-mongodb.sh          | mongod --config ... --shutdown | 27017     | ~/.local-db/data/mongodb   |
| Neo4j     | 二进制   | neo4j start --config ...  | neo4j stop                     | 7474/7687 | ~/.local-db/data/neo4j     |
| Qdrant    | 二进制   | start-qdrant.sh           | pkill qdrant                   | 6333/6334 | ~/.local-db/data/qdrant    |
| SeekDB    | 二进制   | start-seekdb.sh           | pkill seekdb                   | 8080      | ~/.local-db/data/seekdb    |
| SurrealDB | 二进制   | start-surrealdb.sh        | pkill surreal                  | 8000      | ~/.local-db/data/surrealdb |

### 3.2 关键注意事项

#### 3.2.1 Apple Silicon 适配

- Qdrant/SeekDB 优先下载 ARM 架构二进制包，无则用 Rosetta 运行 Intel 版本（`arch -x86_64 ./seekdb`）
- Neo4j 二进制包已原生适配 ARM，无需额外配置

#### 3.2.2 权限问题

- 所有自定义目录均为当前用户所属，无需 sudo 运行，避免权限冲突
- 若启动报错「Permission denied」，执行 `chmod -R 755 ~/.local-db`

#### 3.2.3 日志排查

- 所有服务日志均集中在 `~/.local-db/logs`，启动失败时优先查看对应日志文件

#### 3.2.4 方案特点

- 完全规避容器技术，所有服务均通过二进制/Homebrew 安装
- 目录统一管理，支持一键启停和开机自启
- 适配 macOS 开发/测试场景，数据持久化、配置可定制
- 维护成本极低
