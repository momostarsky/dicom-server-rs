#!/bin/bash

BOUNDARY="DICOM_BOUNDARY"
TEMP_FILE="multipart_request.tmp"

# 1. 写入 JSON 数据的分隔符和头部
printf -- "--%s\r\n" "$BOUNDARY" > "$TEMP_FILE"
printf -- "Content-Type: application/json\r\n\r\n" >> "$TEMP_FILE"

# 2. 附加 metadata.json 的内容
cat metadata.json >> "$TEMP_FILE"

# 3. 写入第一个 DICOM 文件的分隔符和头部
printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

# 4. 附加 dcm1.dcm 的内容
cat dcm1.dcm >> "$TEMP_FILE"

# 5. 写入第二个 DICOM 文件的分隔符和头部
printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

# 6. 附加 dcm2.dcm 的内容
cat dcm2.dcm >> "$TEMP_FILE"

# 7. 写入第三个 DICOM 文件的分隔符和头部
printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

# 8. 附加 dcm3.dcm 的内容
cat dcm3.dcm >> "$TEMP_FILE"

# 9. 写入请求体的结束分隔符
printf -- "\r\n--%s--\r\n" "$BOUNDARY" >> "$TEMP_FILE"

# 10. 使用单个 --data-binary 发送合并后的临时文件
curl -X POST http://localhost:9000/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=$BOUNDARY; type=application/json" \
     -H "Accept: application/json" \
     --data-binary @"$TEMP_FILE"

# 11. 清理临时文件
rm "$TEMP_FILE"