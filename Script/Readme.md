# Standalone Deployment Instructions

The entire system can run in a standalone environment. ***It can also be deployed across multiple machines***.
The three services `wado-storescp` and `wado-server` can all use NGINX for TCP proxy or LVS_DR for load balancing.
**** When deploying in a cluster, it is recommended to use NAS as shared file storage ****
**** A `wado-archive` service will be added later for archiving DICOM files, supporting: Alibaba Cloud OSS, Huawei Cloud OBS, Tianyi Cloud SW3 protocol ****

#### Alternative Software for Cluster or Standalone Deployment
- **MySQL**: OceanBase Community Edition or Enterprise Edition, Unsupported Now
- PgSQL: openGauss LTS Version
- ClickHouse: No alternative needed
- RedPanda: Kafka cluster or RedPanda Enterprise Edition
- Redis: Tencent Tendis Middleware

#### Supported Operating Systems
- Ubuntu 22.04.5 LTS

#### Redis
```docker-compose.yml
redis:
    image: redis
    restart: always
    volumes:
      - ./redata:/data
    ports:
      - "6379:6379" 
```


#### Apache Doris Version
- Doris standalone startup commands, recommended to use the latest version.
```bash
./Doris3.X/3.1.0/fe/bin/start_fe.sh --daemon
./Doris3.X/3.1.0/be/bin/start_be.sh --daemon
```


#### MySQL 8.0 Version
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
      TZ: 'Asia/Shanghai'  # Set timezone to Shanghai (Beijing Time)
    ports:
      - "3306:3306"
    restart: unless-stopped
    volumes:
      - ./mysql-data:/var/lib/mysql  # Data persistence
      - ./my.cnf:/etc/mysql/conf.d/my.cnf:ro
      - /etc/localtime:/etc/localtime:ro  # Mount host timezone file

```
[mysqld]
default-time-zone = '+08:00'


#### PgSQL PgVector-15.0 Version
```docker-compose.yml
version: '3'

services:
  pgdb:
    image: ankane/pgvector:latest
    container_name: pgappx  # You wrote container:pgappx, should be container_name
    restart: always
    environment:
      POSTGRES_PASSWORD: "HzX1Sjq!12dx0y"
      POSTGRES_USER: "xdicomstore"
      PGTZ: "Asia/Shanghai"
    volumes:
      - ./pgdata:/var/lib/postgresql/data
      - ./pg_hba.conf:/var/lib/postgresql/data/pg_hba.conf  # ✅ Correct path
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


#### Message Queue RedPanda / Apache Kafka
- Redpanda: Refer to official documentation.
- Kafka: Refer to official documentation.
- Create 4 message queue instances.

  - `log_queue`: Used to store DICOM object information received by StoreSCP, facilitating subsequent evaluation of image receiving performance and file space usage.
    Data from this queue is written to  ClickHouse or Doris `dicom_object_meta` table

  - `storage_queue`: Used to extract sequence-level information from DICOM and write to Pg or MySQL database, facilitating subsequent retrieval.

    ** wado-consumer   consumes data from this queue and publishes to the other two queues.**

    - `dicom_image_queue`: Used to store DICOM image data, written to ClickHouse or Doris  `dicom_image_meta` table
    - `dicom_state_queue`: Used to store DICOM status data, written to  ClickHouse or Doris `dicom_state_meta` table

- Create topics
```bash
 
rpk topic create dicom_image_queue        --partitions 1 --replicas 1
rpk topic create dicom_state_queue        --partitions 1 --replicas 1
rpk topic create log_queue                --partitions 1 --replicas 1
rpk topic create storage_queue            --partitions 1 --replicas 1
rpk topic create webapi_access_queue      --partitions 1 --replicas 1
```


- Clear topics
```bash
rpk topic trim-prefix dicom_image_queue         -p 0 --offset end --no-confirm
rpk topic trim-prefix dicom_state_queue         -p 0 --offset end --no-confirm
rpk topic trim-prefix log_queue                 -p 0 --offset end --no-confirm
rpk topic trim-prefix storage_queue             -p 0 --offset end --no-confirm
rpk topic trim-prefix webapi_access_queue       -p 0 --offset end --no-confirm
```
