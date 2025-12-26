#### Installation

how to deploy to test ?

在KeyCloak中添加Client wado-rs-api  stow-rs-api 
和用户  docker_lily , Pat001.
通过curl 命令获取的JWT 能输出如下格式:
```json
{
  "exp": 1766655234,
  "iat": 1766654934,
  "jti": "onrtro:0b268443-a59d-6367-8e6d-9d5201e0b54d",
  "iss": "http://localhost:8080/realms/xdicom",
  "aud": [
    "stow-rs-api",
    "XDICOM",
    "wado-rs-api",
    "account"
  ],
  "sub": "ea63af23-62a9-405a-96f3-feb55fc07b9d",
  "typ": "Bearer",
  "azp": "stow-rs-api",
  "sid": "e368deaa-caf4-b639-81d1-deb04ee74b50",
  "acr": "1",
  "realm_access": {
    "roles": [
      "role_patients",
      "offline_access",
      "default-roles-xdicom",
      "uma_authorization",
      "role_doctor"
    ]
  },
  "resource_access": {
    "stow-rs-api": {
      "roles": [
        "image_writer"
      ]
    },
    "wado-rs-api": {
      "roles": [
        "image_reader"
      ]
    },
    "account": {
      "roles": [
        "manage-account",
        "manage-account-links",
        "view-profile"
      ]
    }
  },
  "scope": "openid profile xdicom:full email",
  "email_verified": true,
  "name": "Yong FAN",
  "preferred_username": "doctor_lily",
  "given_name": "Yong",
  "family_name": "FAN",
  "email": "lily@xdicom.com"
}
```