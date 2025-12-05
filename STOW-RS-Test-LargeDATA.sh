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

# 1. 初始化文件（不包含JSON部分）
> "$TEMP_FILE"

# 2. 循环处理所有DICOM文件（第一个文件不需要前置分隔符）
for i in "${!DICOM_FILES[@]}"; do
    dicom_file="${DICOM_FILES[$i]}"

    # 除了第一个文件，其他文件都需要前置分隔符
    if [ $i -gt 0 ]; then
        printf -- "\r\n--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
    else
        # 第一个文件需要起始分隔符
        printf -- "--%s\r\n" "$BOUNDARY" >> "$TEMP_FILE"
    fi

    printf -- "Content-Type: application/dicom\r\n\r\n" >> "$TEMP_FILE"

    # 附加 DICOM 文件的内容
    cat "$dicom_file" >> "$TEMP_FILE"

    echo "Added file: $(basename "$dicom_file")"
done

# 3. 写入请求体的结束分隔符
printf -- "\r\n--%s--\r\n" "$BOUNDARY" >> "$TEMP_FILE"

# 4. 计算文件大小
CONTENT_LENGTH=$(wc -c < "$TEMP_FILE" | tr -d ' ')

echo "Total content length: $CONTENT_LENGTH bytes"

# 5. 发送请求
curl -X POST http://localhost:9000/stow-rs/v1/studies \
     -H "Content-Type: multipart/related; boundary=$BOUNDARY; type=application/dicom" \
     -H "Accept: application/json" \
     -H "x-tenant: 1234567890" \
     -H "Content-Length: $CONTENT_LENGTH" \
     --data-binary @"$TEMP_FILE"

# 6. 清理临时文件
rm "$TEMP_FILE"

echo "Upload completed"
