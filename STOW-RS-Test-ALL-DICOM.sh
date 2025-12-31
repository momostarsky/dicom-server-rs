#!/bin/bash

BOUNDARY="DICOM_BOUNDARY"
TEMP_FILE="multipart_request_all_dicom.tmp"

# 1. 写入 JSON 数据的分隔符和头部
printf -- "--%s\r\n" "$BOUNDARY" > "$TEMP_FILE"
# shellcheck disable=SC2129
printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"
# 2. 附加 dcm1.dcm 的内容
cat dcm1.dcm >> "$TEMP_FILE"

# 3. 写入第二个 DICOM 文件的分隔符和头部
printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

# 4. 附加 dcm2.dcm 的内容
cat dcm2.dcm >> "$TEMP_FILE"

# 5. 写入第三个 DICOM 文件的分隔符和头部
printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

# 6. 附加 dcm3.dcm 的内容
cat dcm3.dcm >> "$TEMP_FILE"

# 7. 写入请求体的结束分隔符
printf -- "\r\n--%s--\r\n" "$BOUNDARY" >> "$TEMP_FILE"

# 8. 使用单个 --data-binary 发送合并后的临时文件
curl -X POST http://localhost:9999/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=$BOUNDARY; type=application/dicom" \
     -H "x-tenant: 1234567890" \
     -H "Accept: application/json" \
     --data-binary @"$TEMP_FILE"

# 9. 清理临时文件
rm "$TEMP_FILE"