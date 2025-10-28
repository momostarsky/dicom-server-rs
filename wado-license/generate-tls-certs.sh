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
  -subj "/C=CN/ST=Zhejiang/L=Hangzhou/O=DicomOrg/CN=$CA_NAME"

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
ST = Zhejiang
L  = Hangzhou
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

# =============================================================================
# 8. 生成用于应用层加解密的 RSA 公私钥对（非 TLS 用途）
#    - 公钥 (encrypt-public.pem)：用于加密数据（可公开）
#    - 私钥 (encrypt-private.key)：用于解密数据（必须保密！）
#    用途示例：License 文件加密、配置加密、API 安全传输等
# =============================================================================
echo "🔐 正在生成用于加解密的 RSA 公私钥对..."

# 生成私钥（4096 位，更高安全性）
openssl genrsa -out encrypt-private.key 4096

# 从私钥提取公钥（PEM 格式，标准公钥）
openssl rsa -in encrypt-private.key -pubout -out encrypt-public.pem

# 设置私钥权限（仅所有者可读写）
chmod 600 encrypt-private.key
chmod 644 encrypt-public.pem

echo "✅ 加解密密钥对已生成:"
echo "   - 私钥（解密用）: encrypt-private.key"
echo "   - 公钥（加密用）: encrypt-public.pem"

# =============================================================================
# 9. （可选）演示：如何用这对密钥加解密一段文本
# =============================================================================
echo ""
echo "🧪 示例：使用公钥加密、私钥解密一段文本（test.txt）..."

echo "This is a secret message for wado-license." > test.txt

# 使用公钥加密（注意：RSA 只能加密小于密钥长度的数据，通常用于加密对称密钥）
openssl rsautl -encrypt -inkey encrypt-public.pem -pubin -in test.txt -out test.txt.enc

# 使用私钥解密
openssl rsautl -decrypt -inkey encrypt-private.key -in test.txt.enc -out test.txt.dec

# 验证是否一致
if cmp -s test.txt test.txt.dec; then
    echo "✅ 加解密成功：原始文件与解密文件一致！"
else
    echo "❌ 加解密失败！"
fi

# 清理测试文件（可选）
rm -f test.txt test.txt.enc test.txt.dec

# =============================================================================
# 10. （更新版）使用 pkeyutl 进行 RSA 公钥加密 / 私钥解密（兼容 OpenSSL 3.0+）
# =============================================================================
echo ""
echo "🧪 示例：使用 pkeyutl（OpenSSL 3.0+ 推荐）进行加解密..."

echo "This is a secret message for wado-license." > test.txt

# 🔒 使用公钥加密
openssl pkeyutl -encrypt \
  -in test.txt \
  -inkey encrypt-public.pem -pkeyopt rsa_padding_mode:pkcs1\
  -pubin \
  -out test.txt.enc

# 🔓 使用私钥解密
openssl pkeyutl -decrypt \
  -in test.txt.enc \
  -inkey encrypt-private.key -pkeyopt rsa_padding_mode:pkcs1\
  -out test.txt.dec

# 验证是否一致
if cmp -s test.txt test.txt.dec; then
    echo "✅ 加解密成功：原始文件与解密文件一致！"
else
    echo "❌ 加解密失败！"
fi
echo ""
echo "🧪 示例：oaep SHA256 进行加解密..."
echo "This is a secret message for wado-license." > plaintext.txt
# 加密
openssl pkeyutl -encrypt \
  -in plaintext.txt \
  -inkey encrypt-public.pem -pubin \
  -pkeyopt rsa_padding_mode:oaep \
  -pkeyopt rsa_oaep_md:sha256 \
  -out ciphertext.bin

# 解密（需相同参数）
openssl pkeyutl -decrypt \
  -in ciphertext.bin \
  -inkey encrypt-private.key \
  -pkeyopt rsa_padding_mode:oaep \
  -pkeyopt rsa_oaep_md:sha256 \
  -out plaintext.dec

# 验证是否一致
if cmp -s plaintext.txt plaintext.dec; then
     echo "✅ 加解密成功：原始文件与解密文件一致！"
else
     echo "❌ 加解密失败！"
fi

# 清理测试文件
rm -f test.txt test.txt.enc test.txt.dec   plaintext.txt plaintext.dec   ciphertext.bin
echo ""
echo "📌 使用说明："
echo "   - 在客户端/前端：使用 encrypt-public.pem 对敏感数据加密后传输"
echo "   - 在服务端（wado-license）：使用 encrypt-private.key 解密数据"
echo "   - 注意：RSA 不适合直接加密大文件，建议结合 AES（混合加密）"
echo ""
echo ""
echo ""
echo "  注意：你之前生成的 encrypt-private.key / encrypt-public.pem 更适合加密，但 License 验证推荐用签名（sign/verify），而非加密/解密 "
echo ""
echo ""
echo ""
echo "📌 生成用于 License 签名的密钥对（与 TLS 证书分离）："
echo "   - license-sign-private.pem：仅服务端持有，用于签发 License"
echo "   - license-sign-public.pem：可打包进客户端或公开分发，用于验证"
echo "   - license-sign-public.der: license-sign-public.pem 转为 DER 格式嵌入客户端 在 Rust 中用 include_bytes! 嵌入"
openssl genrsa -out  license-sign-private.pem 4096
openssl rsa -in license-sign-private.pem -pubout -out  license-sign-public.pem
openssl rsa -in license-sign-public.pem  -pubin -outform DER -out license-sign-public.der