### 单机版本部署说明.

整体系统可以运行在单机环境.也可以部署在多台机器上.
wado-storescp,wado-consumer, wado-server 三个服务均可以通过NGINX做TCP代理或是LVS_DR做负载均衡.
****  集群部署的时候建议采用NAS作为共享文件存储 ****
**** 后续为增加 wado-archive 服务,用于归档存储DICOM文件, 支持:阿里云OSS, 华为云OBS, 天翼云SW3 协议 ****

####  支持的操作系统
- Ubuntu 22.04 LTS
####  Redis
```docker-compose.yml
redis:
    image: redis
    restart: always
    volumes:
      - ./redata:/data
    ports:
      - "6379:6379" 
```
####  Apache Doris  版本
- Doris 单机启动命令, 建议使用最新版本.
```bash
./Doris3.X/3.1.0/fe/bin/start_fe.sh --daemon
./Doris3.X/3.1.0/be/bin/start_be.sh --daemon
```

####  MySQL  8.0 版本  
```docker-compose.yml
version: '3.3'

services:
  db:
    image: mysql:8.0
    container_name: mysql-dicom
    environment:
      MYSQL_ROOT_PASSWORD: 'XjtDcpSjq!12dx0y'
      MYSQL_DATABASE: 'dicomdb'
      MYSQL_USER: 'dicomstore'
      MYSQL_PASSWORD: 'HzS$jox32Pwd!'
      TZ: 'Asia/Shanghai'  # 设置时区为上海（北京时间）
    ports:
      - "3306:3306"
    restart: unless-stopped
    volumes:
      - ./mysql-data:/var/lib/mysql  # 数据持久化
      - ./my.cnf:/etc/mysql/conf.d/my.cnf:ro
      - /etc/localtime:/etc/localtime:ro  # 挂载宿主机时区文件

```
```my.cnf
[mysqld]
default-time-zone = '+08:00'                               
```

####  PgSQL   PgVector-15.0 版本
```docker-compose.yml
version: '3'

services:
  pgdb:
    image: ankane/pgvector:latest
    container_name: pgappx  # 你写的是 container:pgappx，应为 container_name
    restart: always
    environment:
      POSTGRES_PASSWORD: "HzX1Sjq!12dx0y"
      POSTGRES_USER: "dicomstore"
      PGTZ: "Asia/Shanghai"
    volumes:
      - ./pgdata:/var/lib/postgresql/data
      - ./pg_hba.conf:/var/lib/postgresql/data/pg_hba.conf  # ✅ 正确路径
    ports:
      - "5432:5432"
```
```pg_hba.conf
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
host    all             all             192.168.1.0/24          scram-sha-256
```

#### 消息队列  RedPandan /  Apache Kafka
- Redpanda    参考官方文档.
- Kafka       参考官方文档.
- 创建4个消息队列实例. 
  - dicom_image_queue 用于存储 dicom 图像数据
  - dicom_state_queue 用于存储 dicom 状态数据
  - log_queue   用于存储StoreSCP接收的DICOM对象数据, 方便对后续的收图性能及文件占用空间进行评估.
  - storage_queue   用于提取DICOM的序列层级信息并写入Pg或是MySQL数据库,方便后续进行检索.

- 创建队列
```bash 
rpk topic create dicom_image_queue  --partitions 1 --replicas 1
rpk topic create dicom_state_queue  --partitions 1 --replicas 1
rpk topic create log_queue          --partitions 1 --replicas 1
rpk topic create storage_queue      --partitions 1 --replicas 1
```

- 清空队列
```bash
rpk topic trim-prefix dicom_image_queue  -p 0 --offset end --no-confirm
rpk topic trim-prefix dicom_state_queue  -p 0 --offset end --no-confirm
rpk topic trim-prefix log_queue          -p 0 --offset end --no-confirm
rpk topic trim-prefix storage_queue      -p 0 --offset end --no-confirm
```