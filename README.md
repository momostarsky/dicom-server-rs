### 总体架构

1. Apache-kafka 作为消息队列.
2. Apache-Doris 作为数据仓库.提供数据分析功能.
3. MySQL8 作为数据库.提供数据存储功能.及检查索引功能.
4. Redis 作为缓存.提供数据缓存功能.
5. Nginx 作为反向代理服务器.提供负载均衡和静态文件服务.

收图文件服务接收的文件,先存储到本地,再通过 kafka 发送到消息队列.
消息体包括以下内容:

{
TransferSynatx, SopInstancheUID, StudyInstanceUID,SeriesInstanceUID, PatientID, FileName, FileSize, FilePath
}
消息分发到多个队列:
1. 存储队列: 存储文件信息,文件存储路径,文件大小.
2. 索引队列: 提取文件TAG信息, 包括PatientInfomation, StudyInformation, SeriesInformation, ImageInformation.并写入Doris库
3. 转换队列: 对于部分传输语法,因为Cornerstone3D无法解析,需要转换成CornerstoneJS能够解析的格式.转换失败的写入Doris转换记录表.
### 需要安装 dicom-org-cn.pem 文件到证书目录.
```bash
curl https://dicom.org.cn:8443/ca  >>  ~/dicom-org-cn.crt
sudo cp ~/dicom-org-cn.crt  /usr/local/share/ca-certificates/dicom-org-cn.crt  
sudo update-ca-certificates
```
