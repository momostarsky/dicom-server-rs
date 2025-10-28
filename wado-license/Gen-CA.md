#### 为wado-license 服务生成自定义CA证书及自签名证书, 同时生成 为Caddy 代理服务器生成TLS使用的证书 server.crt 和 server.key
```bash
#!/bin/bash

# =============================================================================
# 为 wado-license 服务生成自定义 CA 和 TLS 证书
# 支持域名：
#   - dicom.org.cn
#   - *.dicom.org.cn（泛域名）
#   - localhost
#   - 127.0.0.1
# 生成文件：
#   - ca.crt       : 自定义 CA 证书（用于客户端信任）
#   - ca.key       : 自定义 CA 私钥（务必保密！）
#   - server.key   : 服务器私钥（Caddy 使用）
#   - server.crt   : 服务器证书（由 CA 签发，Caddy 使用）
#   - server.conf  : OpenSSL 配置（含 SAN 扩展）
#   - server.csr   : 证书签名请求（临时文件，可删除）
#   - ca.srl       : CA 序列号文件（OpenSSL 自动生成）
# =============================================================================

set -e  # 遇错立即退出

OUTPUT_DIR="tls-certs"
CA_NAME="DicomOrg Root CA"
DOMAIN="dicom.org.cn"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"
cd "$OUTPUT_DIR"

echo "📁 工作目录: $(pwd)"

# =============================================================================
# 1. 生成 CA 私钥和自签名证书
# =============================================================================
echo "🔐 正在生成自定义 CA 私钥和证书..."

# 生成 CA 私钥（4096 位，无密码，便于自动化；生产环境建议加密码并妥善保管）
openssl genrsa -out ca.key 4096

# 生成自签名 CA 证书，有效期 10 年（3650 天）
openssl req -x509 -new -nodes \
  -key ca.key \
  -sha256 \
  -days 3650 \
  -out ca.crt \
  -subj "/C=CN/ST=Beijing/L=Beijing/O=DicomOrg/CN=$CA_NAME"

echo "✅ CA 证书已生成: ca.crt"

# =============================================================================
# 2. 生成服务器私钥
# =============================================================================
echo "🔑 正在生成服务器私钥..."

openssl genrsa -out server.key 2048

echo "✅ 服务器私钥已生成: server.key"

# =============================================================================
# 3. 创建 OpenSSL 配置文件（含 SAN 扩展）
# =============================================================================
echo "📝 正在创建 OpenSSL 配置文件 (server.conf)..."

cat > server.conf <<EOF
[ req ]
default_bits       = 2048
distinguished_name = req_distinguished_name
req_extensions     = req_ext
prompt             = no

[ req_distinguished_name ]
C  = CN
ST = Beijing
L  = Beijing
O  = DicomOrg
CN = $DOMAIN

[ req_ext ]
subjectAltName = @alt_names

[ v3_ext ]
basicConstraints = CA:FALSE
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[ alt_names ]
DNS.1 = $DOMAIN
DNS.2 = *.$DOMAIN
DNS.3 = localhost
IP.1  = 127.0.0.1
# DNS.4 = wado-license  # 如需服务名解析，可取消注释并在 /etc/hosts 添加
EOF

echo "✅ OpenSSL 配置已创建: server.conf"

# =============================================================================
# 4. 生成证书签名请求 (CSR)
# =============================================================================
echo "📬 正在生成证书签名请求 (CSR)..."

openssl req -new \
  -key server.key \
  -out server.csr \
  -config server.conf

echo "✅ CSR 已生成: server.csr"

# =============================================================================
# 5. 使用自定义 CA 签发服务器证书
# =============================================================================
echo "✍️  正在使用 CA 签发服务器证书..."

openssl x509 -req \
  -in server.csr \
  -CA ca.crt \
  -CAkey ca.key \
  -CAcreateserial \
  -out server.crt \
  -days 365 \
  -sha256 \
  -extfile server.conf \
  -extensions v3_ext

echo "✅ 服务器证书已生成: server.crt"

# =============================================================================
# 6. 清理临时文件（可选）
# =============================================================================
# 保留 server.conf 便于审计，删除 CSR（非必需）
rm -f server.csr

# =============================================================================
# 7. 验证证书内容（可选输出）
# =============================================================================
echo ""
echo "🔍 证书 SAN（Subject Alternative Name）信息："
openssl x509 -in server.crt -text -noout | grep -A1 "Subject Alternative Name"

echo ""
echo "🎉 证书生成完成！"
echo ""
echo "📁 输出文件位于: $(pwd)"
echo "   - CA 证书（用于客户端信任）: ca.crt"
echo "   - 服务器证书（Caddy 使用）   : server.crt"
echo "   - 服务器私钥（Caddy 使用）   : server.key"
echo ""
echo "💡 使用提示："
echo "   - 将 ca.crt 导入操作系统或浏览器的「受信任根证书颁发机构」"
echo "   - Caddy 配置示例："
echo "        tls /path/to/server.crt /path/to/server.key"
echo ""
```
