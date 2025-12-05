// 导入 memchr 库中的 memmem 模块，用于查找子切片（子序列）
use memchr::memmem;
/// 使用 memchr 库中的 memmem::find_iter 来高效地分割 u8 数组。
///
/// 适用于在大型字节流（如 HTTP multipart 数据）中查找一个较长的分隔符。
///
/// # 参数
/// * `data`: 要被分割的 u8 数组（字节切片）。
/// * `delimiter`: 用于分割的 u8 数组（分隔符）。
///
/// # 返回
/// 一个 Vec<&[u8]>，包含分割后的所有数据块的切片。
#[allow(dead_code)]
fn split_multipart_data<'a>(data: &'a [u8], delimiter: &[u8]) -> Vec<&'a [u8]> {
    let mut result = Vec::new();
    let mut current_start = 0;
    let delim_len = delimiter.len();

    // memmem::find_iter 返回一个迭代器，它生成分隔符在数据中所有匹配的起始索引
    // 注意参数顺序：在 data 中查找 delimiter
    for match_index in memmem::find_iter(data, delimiter) {
        // 1. 提取当前块：从上一个起始点到当前分隔符的起始索引
        let current_slice = &data[current_start..match_index];
        result.push(current_slice);

        // 2. 更新下一个块的起始位置：跳过分隔符
        current_start = match_index + delim_len;
    }

    // 3. 将最后一个块（从最后一个分隔符之后到数据末尾）添加到结果中
    result.push(&data[current_start..]);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_multipart_data() {
        // 使用与 main 函数中相同的分隔符
        let delimiter: &[u8] = b"\r\n-89893289328328238\r\n";

        // 构造测试数据
        let part1: &[u8] = b"Content-Type: text/plain\r\n\r\nThis is part 1 data.";
        let part2: &[u8] =
            b"Content-Type: application/json\r\n\r\n{\"id\": 123, \"status\": \"ok\"}";
        let part3: &[u8] = b"Content-Type: image/jpeg\r\n\r\n[...large binary image data...]";

        // 拼接成完整的字节流
        let mut data_vec = Vec::new();
        data_vec.extend_from_slice(part1);
        data_vec.extend_from_slice(delimiter);
        data_vec.extend_from_slice(part2);
        data_vec.extend_from_slice(delimiter);
        data_vec.extend_from_slice(part3);

        let data: &[u8] = &data_vec;

        // 调用 split_multipart_data 函数
        let parts = split_multipart_data(data, delimiter);

        // 验证分割后的块数 - 应该是3个块，不是4个
        assert_eq!(parts.len(), 3); // 3个数据块

        // 验证每个块的内容
        assert_eq!(parts[0], part1);
        assert_eq!(parts[1], part2);
        assert_eq!(parts[2], part3);
    }

    #[test]
    fn test_split_multipart_data_2() {

        let delimiter: &[u8] = b"--89893289328328238\r\n";
        let end_delimiter: &[u8] = b"--89893289328328238--\r\n";

        let part1: &[u8] = b"Content-Type: text/plain\r\n\r\nThis is part 1 data.";
        let part2: &[u8] =
            b"Content-Type: application/json\r\n\r\n{\"id\": 123, \"status\": \"ok\"}";
        let part3: &[u8] = b"Content-Type: image/jpeg\r\n\r\n[...large binary image data...]";

        // 正确构造 multipart 数据流
        let mut data_vec = Vec::new();
        data_vec.extend_from_slice(delimiter); // 起始分隔符
        data_vec.extend_from_slice(part1);
        data_vec.extend_from_slice(b"\r\n");

        data_vec.extend_from_slice(delimiter); // 中间分隔符
        data_vec.extend_from_slice(part2);
        data_vec.extend_from_slice(b"\r\n");

        data_vec.extend_from_slice(delimiter); // 最后分隔符
        data_vec.extend_from_slice(part3);
        data_vec.extend_from_slice(b"\r\n");
        data_vec.extend_from_slice(end_delimiter); // 结束分隔符

        let data: &[u8] = &data_vec;

        let parts = split_multipart_data(data, delimiter);

        // 应该产生 4 个块：
        // 1. "" (在第一个分隔符之前)
        // 2. "Content-Type: text/plain\r\n\r\nThis is part 1 data.\r\n" (第一个部分及换行)
        // 3. "Content-Type: application/json\r\n\r\n{\"id\": 123, \"status\": \"ok\"}\r\n" (第二个部分及换行)
        // 4. "Content-Type: image/jpeg\r\n\r\n[...large binary image data...]\r\n--89893289328328238--\r\n" (第三个部分及换行和结束分隔符)
        assert_eq!(parts.len(), 4);

        // 验证每个分割块的内容
        assert_eq!(parts[0], b""); // 第一个分隔符之前的内容为空

        // 第一个部分包括其后添加的 \r\n
        let mut expected_part1 = part1.to_vec();
        expected_part1.extend_from_slice(b"\r\n");
        assert_eq!(parts[1], &expected_part1[..]);

        // 第二个部分包括其后添加的 \r\n
        let mut expected_part2 = part2.to_vec();
        expected_part2.extend_from_slice(b"\r\n");
        assert_eq!(parts[2], &expected_part2[..]);

        // 第三个部分包括其后添加的 \r\n 和结束分隔符
        let mut expected_part3 = part3.to_vec();
        expected_part3.extend_from_slice(b"\r\n");
        expected_part3.extend_from_slice(end_delimiter);
        assert_eq!(parts[3], &expected_part3[..]);
    }

    #[test]
    fn test_split_multipart_data_no_delimiter() {
        let delimiter: &[u8] = b"\r\n--boundary--\r\n";
        let data: &[u8] = b"Content-Type: text/plain\r\n\r\nThis is single part data.";

        let parts = split_multipart_data(data, delimiter);

        // 没有找到分隔符时，应该返回包含整个数据的单个块
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], data);
    }

    #[test]
    fn test_split_multipart_data_empty_data() {
        let delimiter: &[u8] = b"\r\n--boundary--\r\n";
        let data: &[u8] = b"";

        let parts = split_multipart_data(data, delimiter);

        // 空数据应该返回包含一个空块的向量
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0], b"");
    }

    #[test]
    fn test_split_multipart_data_delimiter_at_beginning() {
        let delimiter: &[u8] = b"--boundary";
        let data: &[u8] = b"--boundarySome content after boundary";

        let parts = split_multipart_data(data, delimiter);

        // 分隔符在开头时，第一个块应该是空的
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], b"");
        assert_eq!(parts[1], b"Some content after boundary");
    }
}
