#!/bin/bash

export TLS_INSECURE=true 
export NO_PROXY=medical.org,.medical.org,keycloak.medical.org,127.0.0.1,localhost
# 获取访问令牌并存储到环境变量
# ACCESS_TOKEN=$(curl -k --cert ./certs/tls.crt --key ./certs/tls.key -X POST \
#   https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/token \
#   -d "client_id=wado-rs-api" \
#   -d "client_secret=TJibFrPe5xv67fEUA681pEQBmRbrrhNl" \
#   -d "grant_type=client_credentials" \
#   -d "audience=wado-rs-api" | jq -r '.access_token')

ACCESS_TOKEN=$(curl -X POST \
  https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/token \
  -d "client_id=wado-rs-api" \
  -d "client_secret=TJibFrPe5xv67fEUA681pEQBmRbrrhNl" \
  -d "grant_type=client_credentials" \
  -d "audience=wado-rs-api" | jq -r '.access_token')

# 使用访问令牌调用API
curl -X 'GET' \
  'http://localhost:9000/wado-rs/v1/studies/1.3.12.2.1107.5.2.12.21149.2021013008195914768758/subseries' \
  -H 'accept: application/dicom+json' \
  -H 'x-tenant: 1234567890' \
  -H "Authorization: Bearer $ACCESS_TOKEN"
