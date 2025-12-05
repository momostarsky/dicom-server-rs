#!/bin/bash

# 检查参数
if [ $# -eq 0 ]; then
    echo "Usage: $0 <dicom_directory>"
    echo "Example: $0 /home/dhz/amprData"
    exit 1
fi

DICOM_DIR="$1"

# 检查目录是否存在
if [ ! -d "$DICOM_DIR" ]; then
    echo "Error: Directory '$DICOM_DIR' does not exist"
    exit 1
fi

BOUNDARY="DICOM_BOUNDARY"
TEMP_FILE="multipart_request_largdata.tmp"

# 检查是否有DICOM文件
DICOM_FILES=($(find "$DICOM_DIR" -type f -name "*.dcm"))
if [ ${#DICOM_FILES[@]} -eq 0 ]; then
    echo "Warning: No DICOM files found in '$DICOM_DIR'"
    exit 0
fi

echo "Found ${#DICOM_FILES[@]} DICOM files"

# 1. 写入 JSON 数据的分隔符和头部
printf -- "--%s\r\n" "$BOUNDARY" > "$TEMP_FILE"
printf -- "Content-Type: application/json\r\n\r\n" >> "$TEMP_FILE"

# 2. 创建基本的metadata.json内容（简化版）
printf -- "{\n" >> "$TEMP_FILE"
printf -- "  \"TransactionUID\": \"%s\",\n" "$(uuidgen)" >> "$TEMP_FILE"
printf -- "  \"Description\": \"Batch upload of %d DICOM files\"\n" ${#DICOM_FILES[@]} >> "$TEMP_FILE"
printf -- "}\n" >> "$TEMP_FILE"

# 3. 循环处理所有DICOM文件
for i in "${!DICOM_FILES[@]}"; do
    dicom_file="${DICOM_FILES[$i]}"

    # 写入当前 DICOM 文件的分隔符和头部
    printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
    printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

    # 附加 DICOM 文件的内容
    cat "$dicom_file" >> "$TEMP_FILE"

    echo "Added file: $(basename "$dicom_file")"
done

# 4. 写入请求体的结束分隔符
printf -- "\r\n--%s--\r\n" "$BOUNDARY" >> "$TEMP_FILE"

# 5. 计算文件大小
CONTENT_LENGTH=$(wc -c < "$TEMP_FILE" | tr -d ' ')

echo "Total content length: $CONTENT_LENGTH bytes"

# 6. 发送请求
curl -X POST http://localhost:9000/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=$BOUNDARY; type=application/dicom+json" \
     -H "Accept: application/json" \
     -H "Content-Length: $CONTENT_LENGTH" \
     --data-binary @"$TEMP_FILE"

# 7. 清理临时文件
rm "$TEMP_FILE"

echo "Upload completed"
