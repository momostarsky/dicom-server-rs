#!/bin/bash

echo "APP_ENV=prod" > .env

echo "Create wado-consumer.servervice file..."
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

echo "Create wado-server.servervice file..."
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

echo "Create wado-storescp.servervice file..."
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

echo "Reload systemd..."
sudo systemctl daemon-reload

# 启用服务（开机自启）
echo "Enable services..."
sudo systemctl enable wado-consumer.service
sudo systemctl enable wado-server.service
sudo systemctl enable wado-storescp.service

# 启动服务
echo "Start services..."
sudo systemctl start wado-consumer.service
sudo systemctl start wado-server.service
sudo systemctl start wado-storescp.service

