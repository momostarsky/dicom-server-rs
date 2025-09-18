use openssl::x509::X509;
use x509_parser::nom::AsBytes;

static CA_CONTENT: &str = "-----BEGIN CERTIFICATE-----
MIIGJzCCBA+gAwIBAgIBATANBgkqhkiG9w0BAQsFADCBwjELMAkGA1UEBhMCQ04x
ETAPBgNVBAgMCFpoZWppYW5nMREwDwYDVQQHDAhIYW5nemhvdTEXMBUGA1UECgwO
TGljZW5zZSBTZXJ2ZXIxFTATBgNVBAMMDGRpY29tLm9yZy5jbjEfMB0GCSqGSIb3
DQEJARYQNDExNTkyMTQ4QHFxLmNvbTEbMBkGCgmSJomT8ixkAQEMCzE1OTY3MTMy
MTcyMQwwCgYDVQQEDANkYWkxETAPBgNVBCoMCGhhbnpoYW5nMB4XDTI1MDkwOTEx
MTcwMloXDTM1MDkwNzExMTcwMlowgcIxCzAJBgNVBAYTAkNOMREwDwYDVQQIDAha
aGVqaWFuZzERMA8GA1UEBwwISGFuZ3pob3UxFzAVBgNVBAoMDkxpY2Vuc2UgU2Vy
dmVyMRUwEwYDVQQDDAxkaWNvbS5vcmcuY24xHzAdBgkqhkiG9w0BCQEWEDQxMTU5
MjE0OEBxcS5jb20xGzAZBgoJkiaJk/IsZAEBDAsxNTk2NzEzMjE3MjEMMAoGA1UE
BAwDZGFpMREwDwYDVQQqDAhoYW56aGFuZzCCAiIwDQYJKoZIhvcNAQEBBQADggIP
ADCCAgoCggIBALjxg98Vg000kcFCw0Af1RgniFHh7b3RvaQd/5moZyWZVMvaofA+
qnPf5UhIxvW2wjQgWyPeCHsnmHb4kTtNtlJ4VvmnJx9NaqcRLGRIghmmNoJjDkbI
SGCq7B4L1cYWLUX7WWVOTA5233l4d/tmEfcisVv5/BaagvnC9V9KTixucN/dsUSW
UnzB1YWd4S5TxW0bbRVBpD6YGcfU0CFhl1csA39BzvLl/VSO1e7S3XZJ193lVKju
/ZMyL9nJH8RGWrMj2FmbU9Fxb2feEz6l1JkyxyUjgZGgOzk2zIAjk/i4JnAXvtSm
zNaVtQNjbpF6ua4i2RxOtYFqlAHzHv8yBj0f8ZDlyzMhaUuKLppFr5wRqv1bkts5
978v2kePKfAwosr52aV1eqhbBQsS32Hy2O1Y5mhrlBieZKsIhwsff1pgetFfA1+i
+6UZs26V49GMQq8uUoSvqVVWLJFqfZnNhni0nJmvwilHTEQHrAU1ARwa2NSIW0tv
lwpz0goWiFfj/DGfnx14/VZVjlTTv9OYbPkXElwbSVf0d8OIV5D7Htd6nQm1DytA
wz/NJq7Bb0tgA/w5K9bEOy3IqWZrokLI1lr3atRP6Dl3UtKk2btelKpSEtbWTBFD
+zQE5pEYMwVJ5OeFJJ9s939+x/viYLJ4m7lKDQ5e4Gnn/cb+423L3IzDAgMBAAGj
JjAkMBIGA1UdEwEB/wQIMAYBAf8CAQIwDgYDVR0PAQH/BAQDAgEGMA0GCSqGSIb3
DQEBCwUAA4ICAQBZDYMV54eWBeymSN5OTEGJ2B2mHQmUrK8LnOi8IN3tt/Gjg+ci
o2P73WnNOYvm+BJ9BnHsneANlYxC4SBJi/GNPcSuJrLc/JwiIu0U4wTFA+J/1AjR
qSEH6VATMtDq95gSm636dLEH9K823z8PyO46In2gLVio2Cu103voRFe4H+eURMEW
NU/bFg6WAHN2747GNCh7uspD3bApO3JhWKMN8E2vhZA7t+BG+QmiQ+99gnB0o5Hr
zY9n6nIXjJ0MrTLsOvrh/+0AtI7DN6Ped4ulB5jBDN6lval3nArqxvoJ7yX4JPeH
8DpG/7T5D8mLZNWOG2vPCntOCd33a4qrPGFfdaJ6rIoLZ3PZk6JlW5/4r0eBbYIp
upjoLKPRjDfzA6yBiwcg/vhL+uOuhEuzL3q8HnfMdPIBhP9udKXxcPhSwk41144M
QlrAMdFnhfZ9SySAd+BGXLM8Mc1zhehP159ml2giliXfRbpomeqjP1VvqhVZztHd
J7JISgvbTH3c8t3KvXAmJZkgIJQ36roL0IygaS2pbhEIegNnZeXHsVCzNBF0MyY7
tkBoDRNky61Xm6+x/ud+VBS0c7FRnuvCL5gFMXGqJkZ1Vrc4+QN6VGEiydqLU65/
17jnqrNlmpiR3IhKfXdZMMIhe1R4Y8ZXb9wSEse0HVWtSimCiZOhyluJJg==
-----END CERTIFICATE-----\n";

pub async fn load_ca_certificate() -> Result<X509, Box<dyn std::error::Error>> {
    X509::from_pem(CA_CONTENT.as_bytes()).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_ca_certificate() {
        let result = load_ca_certificate().await;
        assert!(result.is_ok(), "Failed to load CA certificate: {:?}", result.err());

        let cert = result.unwrap();
        let subject = cert.subject_name();
        // 验证证书主题包含预期的信息
        let common_name = subject.entries_by_nid(openssl::nid::Nid::COMMONNAME)
            .next()
            .expect("Certificate should have a common name");

        let common_name_str = String::from_utf8(common_name.data().as_slice().to_vec()).unwrap();
        assert_eq!(common_name_str, "dicom.org.cn");
    }

    #[tokio::test]
    async fn test_ca_certificate_pem_format() {
        // 验证CA证书内容是有效的PEM格式
        assert!(CA_CONTENT.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(CA_CONTENT.ends_with("-----END CERTIFICATE-----\n"));

        // 验证能够成功解析PEM内容
        let result = load_ca_certificate().await;
        assert!(result.is_ok(), "CA certificate should be valid PEM format");
    }

    #[tokio::test]
    async fn test_ca_certificate_not_empty() {
        // 验证CA证书内容不为空
        assert!(!CA_CONTENT.is_empty());
        assert!(CA_CONTENT.len() > 100); // 合理的最小证书大小
    }
}
