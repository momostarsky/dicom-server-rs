#!/bin/bash
# deploy-medical-keycloak.sh

set -e

echo "🚀 部署医疗系统Keycloak 26.4.2..."
 
# 1. 环境变量检查
if [ -z "$KC_ADMIN_PASSWORD" ] || [ -z "$KC_DB_PASSWORD" ]; then
  echo "❌ 错误：请设置KC_ADMIN_PASSWORD和KC_DB_PASSWORD环境变量"
  exit 1
fi

# 2. 创建必要目录
mkdir -p certs logs extensions medical-theme/css

# 3. 生成自签名证书（生产环境应使用正式证书）
if [ ! -f "certs/tls.crt" ]; then
  echo "🔧 生成TLS证书..."
  openssl req -x509 -newkey rsa:4096 -sha512 -days 365 \
    -nodes -keyout certs/tls.key \
    -out certs/tls.crt \
    -subj "/C=CN/ST=Medical/L=Hospital/O=MedicalOrg/CN=keycloak.medical.org" \
    -addext "subjectAltName = DNS:keycloak.medical.org"
fi

# 4. 设置文件权限（医疗系统严格权限）
chmod 600 certs/*
chmod 700 logs

# 5. 启动服务
echo "🐳 启动Keycloak医疗系统..."
docker compose -f docker-compose.yml up -d

# 6. 等待服务就绪
# Replace the health check section with:
echo "⏳ 等待Keycloak初始化..."
timeout=120
while [ $timeout -gt 0 ]; do
  # Try multiple approaches for checking readiness
  if curl -s -k -f "https://localhost:8443" > /dev/null || \
     curl -s -k -f "https://localhost:8443/realms/master/.well-known/openid-configuration" > /dev/null; then
    echo "✅ Keycloak服务已就绪"
    break
  fi
  sleep 5
  timeout=$((timeout - 5))
done

if [ $timeout -eq 0 ]; then
  echo "❌ Keycloak服务启动超时"
  exit 1
fi
 echo "🚀 Keycloak服务启动完成..."