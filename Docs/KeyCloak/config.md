#### KeyCload  在oauth2 中的认证说明

```json
{
  "wado_oauth2": {
    "issuer_url": "https://keycloak.medical.org:8443/realms/dicom-org-cn",
    "audience": "wado-rs-api",
    "jwks_url": "https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/certs",
    "realm_roles": [
      "role_patients"
    ],
    "resource_id": [
      "wado-rs-api"
    ],
    "resource_roles": [
      "image_reader"
    ]
  }
}
```

*realm_roles* : 用户角色.
*resource_id* : 资源id. 属于可选的参数.设为空,则表示所有资源.
*resource_roles* : 资源内的权限.
配置样例如下(主要参考[Keycloak 认证](https://www.keycloak.org/docs/latest/securing_apps/index.html#_jwt))
```json
{
        "exp": 1762764709,
        "iat": 1762764409,
        "jti": "onrtro:4239f53d-7af1-4319-f531-da6384756652",
        "iss": "https://keycloak.medical.org:8443/realms/dicom-org-cn",
        "aud": [
                "wado-rs-api",
                "account"
        ],
        "sub": "76b3541b-e039-453c-b1cb-961a74ad85b6",
        "typ": "Bearer",
        "azp": "wado-rs-api",
        "sid": "4cc2e688-7cf8-b882-bc49-11380678465c",
        "acr": "1",
        "allowed-origins": [
            "/*"
        ],
        "realm_access": {
                "roles": [
                        "role_patients",
                        "offline_access",
                        "default-roles-dicom-org-cn",
                        "uma_authorization"
                ]
        },
        "resource_access": {
                "wado-rs-api": {
                        "roles": [
                            "image_reader",
                            "study_viewer",
                            "image_viewer",
                            "series_viewer"
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
        "scope": "openid profile email",
        "email_verified": false,
        "name": "LIN Yanliang",
        "preferred_username": "pat001",
        "given_name": "LIN",
        "family_name": "Yanliang",
        "email": "hzx_000x932@qq.com"
}
```