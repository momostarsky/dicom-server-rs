#!/bin/bash

# 生成医疗系统专用证书
openssl req -x509 -newkey rsa:4096 -sha512 -days 365 \
  -keyout certs/tls.key \
  -out certs/tls.crt \
  -subj "/C=CN/ST=Medical Province/L=Hospital District/O=Medical Organization/CN=keycloak.medical.org" \
  -addext "subjectAltName = DNS:keycloak.medical.org,DNS:auth.medical.org"