### 安装本地开发用的 HTTPS  证书

```bash
dotnet dev-certs https -ep ./localhost.pfx -p "StarKy1233!" --format PFX
openssl pkcs12 -in localhost.pfx -clcerts -nokeys -out localhost.crt -password pass:"StarKy1233!"
openssl pkcs12 -in localhost.pfx -nocerts -nodes -out localhost.key -password pass:"StarKy1233!"
sudo cp localhost.crt /usr/local/share/ca-certificates/aspnet-https.crt
sudo update-ca-certificates
```


#### 测试上传文件

```bash
 curl -X POST http://localhost:9000/stow-rs/v1/studies \
  -H "Content-Type: multipart/related; type=\"application/dicom\"" \
  --data-binary @test.dcm
```

```bash
 curl -X POST http://localhost:9000/stow-rs/v1/studies/1232323329902039230923092309 \
  -H "Content-Type: multipart/related; type=\"application/dicom\"" \
  --data-binary @test.dcm
```

```bash
curl -X POST http://localhost:9000/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=DICOM_BOUNDARY; type=application/json" \
     -H "Accept: application/json" \
     --data-binary $'--DICOM_BOUNDARY\r\nContent-Type: application/json\r\n\r\n' \
     --data-binary @metadata.json \
     --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
     --data-binary @dcm1.dcm \
     --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
     --data-binary @dcm2.dcm \
     --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
     --data-binary @dcm3.dcm \
     --data-binary $'\r\n--DICOM_BOUNDARY--\r\n'
```