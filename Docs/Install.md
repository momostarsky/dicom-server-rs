#### 安装部署说明.


我将帮助您为这三个应用程序创建Ubuntu服务，并配置环境变量。以下是具体步骤：

## 1. 创建 systemd 服务文件

### wado-consumer 服务

```bash
sudo tee /etc/systemd/system/wado-consumer.service << EOF
[Unit]
Description=WADO Consumer Service
After=network.target

[Service]
Type=simple
User=$(whoami)
WorkingDirectory=$(pwd)
Environment=APP_ENV=prod
ExecStart=$(pwd)/wado-consumer
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
```


### wado-server 服务

```bash
sudo tee /etc/systemd/system/wado-server.service << EOF
[Unit]
Description=WADO Server Service
After=network.target

[Service]
Type=simple
User=$(whoami)
WorkingDirectory=$(pwd)
Environment=APP_ENV=prod
ExecStart=$(pwd)/wado-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
```


### wado-storescp 服务

```bash
sudo tee /etc/systemd/system/wado-storescp.service << EOF
[Unit]
Description=WADO StoreSCP Service
After=network.target

[Service]
Type=simple
User=$(whoami)
WorkingDirectory=$(pwd)
Environment=APP_ENV=prod
ExecStart=$(pwd)/wado-storescp
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
```


## 2. 创建或更新 .env 文件

```bash
echo "APP_ENV=prod" > .env
```


## 3. 重新加载 systemd 配置

```bash
sudo systemctl daemon-reload
```


## 4. 启用并启动服务

```bash
# 启用服务（开机自启）
sudo systemctl enable wado-consumer.service
sudo systemctl enable wado-server.service
sudo systemctl enable wado-storescp.service

# 启动服务
sudo systemctl start wado-consumer.service
sudo systemctl start wado-server.service
sudo systemctl start wado-storescp.service
```


## 5. 验证服务状态

```bash
sudo systemctl status wado-consumer.service
sudo systemctl status wado-server.service
sudo systemctl status wado-storescp.service
```


## 服务管理命令

- **启动服务**: `sudo systemctl start service-name`
- **停止服务**: `sudo systemctl stop service-name`
- **重启服务**: `sudo systemctl restart service-name`
- **查看状态**: `sudo systemctl status service-name`
- **查看日志**: `sudo journalctl -u service-name -f`

这些服务将会：
- 在系统启动时自动运行
- 在意外终止时自动重启
- 使用当前目录作为工作目录
- 设置 `APP_ENV=prod` 环境变量
- 以当前用户身份运行