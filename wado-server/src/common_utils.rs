use std::collections::HashMap;

// 解析查询字符串，支持重复键
pub(crate) fn parse_query_string_case_insensitive(query: &str) -> HashMap<String, Vec<String>> {
    let mut params: HashMap<String, Vec<String>> = HashMap::new();

    for pair in query.split('&') {
        // 跳过空的键值对
        if pair.is_empty() {
            continue;
        }

        if let Some((key, value)) = pair.split_once('=') {
            let key = urlencoding::decode(key).unwrap_or_else(|_| key.into()).into_owned();
            let value = urlencoding::decode(value).unwrap_or_else(|_| value.into()).into_owned();
            // 去除参数值的首尾空格以提高兼容性
            let trimmed_value = value.trim().to_string();
            params.entry(key.to_lowercase()).or_insert_with(Vec::new).push(trimmed_value);
        }
    }

    params
}

// 不区分大小写的参数获取
pub(crate) fn get_param_case_insensitive<'a>(
    params: &'a HashMap<String, Vec<String>>,
    key: &str
) -> Option<&'a Vec<String>> {
    params.get(&key.to_lowercase())
}