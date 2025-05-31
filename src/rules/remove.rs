use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub enum RemovePosition {
    /// 删除所有出现的文本
    All,
    /// 删除第一个出现的文本
    First,
    /// 删除最后一个出现的文本
    Last,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveRule {
    /// 要移除的文本
    pub text: String,
    /// 要操作的位置
    pub remove_position: RemovePosition,
    /// 区分大小写
    pub case_sensitive: bool,
    /// 忽略扩展名
    pub ignore_extension: bool,
}

pub fn remove(old_text: &str, rule: RemoveRule) -> String {
    if rule.text.is_empty() {
        return old_text.to_string(); // 没有要移除的内容
    }

    // 1. 根据 ignore_extension 拆分 old_text 为 "要处理的部分" 和 "要追加的扩展名"
    let (name_to_process, extension_to_append) = if rule.ignore_extension {
        let path_obj = Path::new(old_text);

        // file_stem() 获取文件名中最后一个点之前的部分。
        // 例如: "archive.tar.gz" -> "archive.tar"
        //       ".bashrc" -> ".bashrc" (因为没有被识别为传统意义的扩展名)
        //       "nodot" -> "nodot"
        //       "" -> None
        //       "/" -> None
        let base_name = path_obj
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| old_text.to_string()); // 如果没有 stem (如 "" 或 "/"), 则处理整个 old_text

        // 只有当 Path 对象能同时识别出 stem 和 extension 时，我们才分离扩展名
        // 例如，对于 ".bashrc"，file_stem() 是 ".bashrc"，extension() 是 None。
        // 我们不希望将其错误地拆分为 name="" 和 ext=".bashrc"
        let ext_suffix = if path_obj.file_stem().is_some() && path_obj.extension().is_some() {
            path_obj
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default() // 理论上如果外层条件满足，这里总是 Some
        } else {
            String::new() // 没有可分离的扩展名，或者不应分离
        };
        (base_name, ext_suffix)
    } else {
        // 不忽略扩展名，则整个 old_text 都是要处理的部分
        (old_text.to_string(), String::new())
    };

    // 如果要处理的部分是空的，并且要移除的文本非空，则无法移除。
    // 例如：old_text="", rule.text="a" -> ""
    // 例如：old_text=".txt", ignore_extension=true -> name_to_process=".txt", extension_to_append="" (Path::extension on ".txt" is None)
    // 如果 name_to_process 是空字符串（比如 old_text 本身是空），则直接返回 extension_to_append (通常也是空)
    if name_to_process.is_empty() && !rule.text.is_empty() {
        return extension_to_append; // 通常是返回空字符串
    }

    // 2. 在 "要处理的部分" (name_to_process) 上执行移除操作
    let mut final_processed_name_part = name_to_process.clone(); // 克隆一份用于修改

    match rule.remove_position {
        RemovePosition::First => {
            todo!()
        }
        RemovePosition::Last => {
            todo!()
        }
        RemovePosition::All => {
            // 移除所有匹配项
            if rule.case_sensitive {
                final_processed_name_part = name_to_process.replace(&rule.text, "");
            } else {
                let escaped_text = regex::escape(&rule.text);
                if let Ok(re) = RegexBuilder::new(&escaped_text)
                    .case_insensitive(true)
                    .build()
                {
                    final_processed_name_part = re.replace_all(&name_to_process, "").into_owned();
                }
                // 如果 regex 构建失败，final_processed_name_part 保持原样
            }
        }
    }

    // 3. 将处理后的部分与之前分离的扩展名（如果适用）重新组合
    format!("{}{}", final_processed_name_part, extension_to_append)
}

pub fn removes(old_text: &str, rules: Vec<RemoveRule>) -> String {
    rules.into_iter().fold(
        old_text.to_string(),                             /*初始值*/
        |current_text, rule| remove(&current_text, rule), // 对每个rule应用remove函数
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // 测试样例：删除全部 + 区分大小写 + 忽略扩展名
    #[rstest]
    #[case("a.txt", "a", ".txt")]
    #[case("aa.txt", "a", ".txt")]
    #[case("aa.txt", "A", "aa.txt")]
    #[case("abcda.txt", "a", "bcd.txt")]
    fn test_remove_all_case_sensitive_ignore_extension(
        #[case] input: &str,
        #[case] text: &str,
        #[case] expected: &str,
    ) {
        let rule = RemoveRule {
            text: text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: true,
            ignore_extension: true,
        };

        let result = remove(input, rule);
        assert_eq!(result, expected);
    }

    // 测试样例 : 删除全部 + 区分大小写 + 不忽略扩展名
    #[rstest]
    #[case("a.txt", "a", ".txt")]
    #[case("aa.txt", "a.txt", "a")]
    #[case("abcda.txt", "a.txt", "abcd")]
    #[case("abcda.tat", "a", "bcd.tt")]
    fn test_remove_all_case_sensitive_not_ignore_extension(
        #[case] input: &str,
        #[case] text: &str,
        #[case] expected: &str,
    ) {
        let rule = RemoveRule {
            text: text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: true,
            ignore_extension: false,
        };

        let result = remove(input, rule);
        assert_eq!(result, expected);
    }

    // 测试样例 : 删除全部 + 不区分大小写 + 不忽略扩展名
    #[rstest]
    #[case("a.txt", "A", ".txt")]
    #[case("aa.txt", "A.txt", "a")]
    #[case("abcda.txt", "A.txt", "abcd")]
    #[case("abcda.tat", "A", "bcd.tt")]
    fn test_remove_all_case_insensitive_not_ignore_extension(
        #[case] input: &str,
        #[case] text: &str,
        #[case] expected: &str,
    ) {
        let rule = RemoveRule {
            text: text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: false,
            ignore_extension: false,
        };

        let result = remove(input, rule);
        assert_eq!(result, expected);
    }

    // 测试样例 : 删除全部 + 不区分大小写 + 忽略扩展名
    #[rstest]
    #[case("a.txt", "A", ".txt")]
    #[case("aa.txt", "A.txt", "aa.txt")]
    #[case("abcda.txt", "A.txt", "abcda.txt")]
    #[case("abcda.tat", "A", "bcd.tat")]
    fn test_remove_all_case_insensitive_ignore_extension(
        #[case] input: &str,
        #[case] text: &str,
        #[case] expected: &str,
    ) {
        let rule = RemoveRule {
            text: text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: false,
            ignore_extension: true,
        };

        let result = remove(input, rule);
        assert_eq!(result, expected);
    }
}
