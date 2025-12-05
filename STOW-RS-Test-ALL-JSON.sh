#!/bin/bash

BOUNDARY="DICOM_BOUNDARY"
TEMP_FILE="multipart_request_only_json.tmp"

# 1. 写入 JSON 数据的分隔符和头部
printf -- "--%s\r\n" "$BOUNDARY" > "$TEMP_FILE"
printf -- "Content-Type: application/json\r\n\r\n" >> "$TEMP_FILE"

# 2. 附加 metadata.json 的内容
cat metadata.json >> "$TEMP_FILE"

# 3. 写入请求体的结束分隔符
printf -- "\r\n--%s--\r\n" "$BOUNDARY" >> "$TEMP_FILE"

# 4. 使用单个 --data-binary 发送合并后的临时文件
curl -X POST http://localhost:9000/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=$BOUNDARY; type=application/json" \
     -H "Accept: application/json" \
     --data-binary @"$TEMP_FILE"

# 5. 清理临时文件
rm "$TEMP_FILE"