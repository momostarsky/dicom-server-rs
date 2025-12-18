#!/bin/bash

BOUNDARY="DICOM_BOUNDARY"
TEMP_FILE="multipart_request.tmp"

# 1. 写入 JSON 数据的分隔符和头部
printf -- "--%s\r\n" "$BOUNDARY" > "$TEMP_FILE"
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

# 10. 计算文件大小
CONTENT_LENGTH=$(wc -c < "$TEMP_FILE" | tr -d ' ')

# 10. 使用单个 --data-binary 发送合并后的临时文件
curl -X POST http://localhost:9999/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=$BOUNDARY; type=application/dicom" \
     -H "Accept: application/json" \
     -H "x-tenant: 1234567890" \
     -H "Content-Length: $CONTENT_LENGTH" \
     --data-binary @"$TEMP_FILE"

# 11. 清理临时文件
#rm "$TEMP_FILE"