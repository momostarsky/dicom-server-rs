# 1. 生成私钥

openssl genrsa -out x-dicom-private.key 4096

# 2. 生成证书请求（包含完整企业信息）

openssl req -new -key x-dicom-private.key -out x-dicom-request.csr -subj "
/C=CN/ST=Zhejiang/L=Hangzhou/O=startsky.technology/CN=x-dicom-encrypt.jianpeicn.com"

# 3. 生成自签名证书

openssl x509 -req -in x-dicom-request.csr -signkey x-dicom-private.key -out certificate.crt -days 3650

# 4. 生成公钥

openssl rsa -in x-dicom-private.key -pubout -out x-dicom-public.key

# 5. 对字符串进行加密

```bash

# echo -n "838FbMzv^orl0Aol" | openssl pkeyutl -encrypt -pubin -inkey x-dicom-public.key -out cipher-key.bin

echo -n "838FbMzv^orl0Aol" | openssl pkeyutl -encrypt -pubin -inkey x-dicom-public.key -pkeyopt rsa_padding_mode:pkcs1 -out cipher-key.bin
```

```bash
# echo -n "oLZphu61s%zguTjS" | openssl pkeyutl -encrypt -pubin -inkey x-dicom-public.key -out cipher-iv.bin 
 
echo -n "oLZphu61s%zguTjS" | openssl pkeyutl -encrypt -pubin -inkey x-dicom-public.key -pkeyopt rsa_padding_mode:pkcs1 -out cipher-iv.bin 
```

# 6. 对字符串进行解密
```bash
openssl pkeyutl -decrypt -inkey x-dicom-private.key -in cipher-key.bin
openssl pkeyutl -decrypt -inkey x-dicom-private.key -in cipher-iv.bin
```
