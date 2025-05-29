use std::path::Path;
use serde::{Deserialize, Serialize};
use regex::RegexBuilder;

#[derive(Debug,Serialize,Deserialize)]
pub enum RemovePosition{
    /// 删除所有出现的文本
    All,
    /// 删除第一个出现的文本
    First,
    /// 删除最后一个出现的文本
    Last,
}
#[derive(Debug,Serialize,Deserialize)]
pub struct RemoveRule{
    /// 要移除的文本
    pub text: String,
    /// 要操作的位置
    pub remove_position: RemovePosition,
    /// 区分大小写
    pub case_sensitive: bool,
    /// 忽略扩展名
    pub ignore_extension:bool,
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
        let base_name = path_obj.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| old_text.to_string()); // 如果没有 stem (如 "" 或 "/"), 则处理整个 old_text

        // 只有当 Path 对象能同时识别出 stem 和 extension 时，我们才分离扩展名
        // 例如，对于 ".bashrc"，file_stem() 是 ".bashrc"，extension() 是 None。
        // 我们不希望将其错误地拆分为 name="" 和 ext=".bashrc"
        let ext_suffix = if path_obj.file_stem().is_some() && path_obj.extension().is_some() {
            path_obj.extension()
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
        RemovePosition::First => { // 移除第一个匹配项
            if rule.case_sensitive {
                // replacen 会替换第n个匹配项，这里n=1
                final_processed_name_part = name_to_process.replacen(&rule.text, "", 1);
            } else {
                let escaped_text = regex::escape(&rule.text);
                if let Ok(re) = RegexBuilder::new(&escaped_text).case_insensitive(true).build() {
                    // re.replace 只替换第一个匹配项
                    final_processed_name_part = re.replace(&name_to_process, "").into_owned();
                }
                // 如果 regex 构建失败，则 final_processed_name_part 保持为 name_to_process 的克隆值
            }
        }
        RemovePosition::Last => { // 移除最后一个匹配项
            if rule.case_sensitive {
                if let Some(index) = name_to_process.rfind(&rule.text) {
                    let mut temp_string = String::with_capacity(name_to_process.len().saturating_sub(rule.text.len()));
                    temp_string.push_str(&name_to_process[..index]);
                    temp_string.push_str(&name_to_process[index + rule.text.len()..]);
                    final_processed_name_part = temp_string;
                }
                // 如果未找到，final_processed_name_part 保持原样
            } else {
                let escaped_text = regex::escape(&rule.text);
                if let Ok(re) = RegexBuilder::new(&escaped_text).case_insensitive(true).build() {
                    let matches: Vec<regex::Match> = re.find_iter(&name_to_process).collect();
                    if let Some(last_match) = matches.last() {
                        let mut temp_string = String::with_capacity(name_to_process.len().saturating_sub(last_match.as_str().len()));
                        temp_string.push_str(&name_to_process[..last_match.start()]);
                        temp_string.push_str(&name_to_process[last_match.end()..]);
                        final_processed_name_part = temp_string;
                    }
                    // 如果未找到或 regex 构建失败，final_processed_name_part 保持原样
                }
            }
        }
        RemovePosition::All => { // 移除所有匹配项
            if rule.case_sensitive {
                final_processed_name_part = name_to_process.replace(&rule.text, "");
            } else {
                let escaped_text = regex::escape(&rule.text);
                if let Ok(re) = RegexBuilder::new(&escaped_text).case_insensitive(true).build() {
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
    rules.into_iter().fold(old_text.to_string(), /*初始值*/
       |current_text,rule| remove(&current_text, rule) // 对每个rule应用remove函数
    )
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use super::*;

    #[rstest]
    #[case("123abc456.txt",&["1","2","3","4","5","6"],"abc.txt")]
    #[case("abc.txt",&["ab","C","txt"],"c.txt")]
    fn test_removes(#[case] input: &str, #[case] remove_texts:&[&str], #[case] expected:&str) {
        let mut rules = vec![];
        for r in remove_texts.iter(){
            let rule = RemoveRule{
                text: r.to_string(),
                remove_position: RemovePosition::All,
                case_sensitive: true,
                ignore_extension: true,
            };
            rules.push(rule);
        }
        let result = removes(input, rules);
        assert_eq!(result, expected);
    }




    #[rstest]
    #[case("abc_abc_ABC.txt","abc","_abc_ABC.txt")]
    #[case("abc_abc_ABC.txt","ABC","abc_abc_.txt")]
    #[case("abc_abc_ABC.txt","ABC_abc","abc_abc_ABC.txt")]
    #[case("abc_abc_ABC.txt","ABC.txt","abc_abc_ABC.txt")]
    #[case("abc_abc_ABC.txt","txt","abc_abc_ABC.txt")]
    #[case("abc_abc_ABC.txt","TXT","abc_abc_ABC.txt")]
    fn test_remove_first_sensitive_ignore_ext(#[case] input: &str, #[case] remove_text:&str,#[case] expected:&str) {
        println!("测试: 删除首个, 区分大小写, 忽略后缀名");
        let rule = RemoveRule{
            text: remove_text.to_string(),
            remove_position: RemovePosition::First,
            case_sensitive: true,
            ignore_extension: true,
        };
        let removed = remove(&input.to_string(), rule);
        assert_eq!(removed, expected);
    }

    #[rstest]
    #[case("abc_123_abc.txt","abc","abc_123_.txt")]
    #[case("abc_123_abc.txt","ABC","abc_123_abc.txt")]
    #[case("abc_123_abc.txt","ABC_abc","abc_123_abc.txt")]
    #[case("abc_123_abc.txt","ABC.txt","abc_123_abc.txt")]
    #[case("abc_123_abc.txt","txt","abc_123_abc.txt")]
    #[case("abc_123_abc.txt","TXT","abc_123_abc.txt")]
    fn test_remove_last_sensitive_ignore_ext(#[case] input: &str, #[case] remove_text:&str,#[case] expected:&str) {
        println!("测试: 删除最后一个, 区分大小写, 忽略后缀名");
        let rule = RemoveRule{
            text: remove_text.to_string(),
            remove_position: RemovePosition::Last,
            case_sensitive: true,
            ignore_extension: true,
        };
        let removed = remove(&input.to_string(), rule);
        assert_eq!(removed, expected);
    }

    #[rstest]
    #[case("abc_ABC.txt","abc","_ABC.txt")]
    #[case("abc_ABC.txt","ABC","abc_.txt")]
    #[case("abc_ABC.txt","ABC_abc","abc_ABC.txt")]
    #[case("abc_ABC.txt","abc.txt","abc_ABC.txt")]
    #[case("abc_ABC.txt","ABC.txt","abc_ABC.txt")]
    #[case("abc_ABC.txt","txt","abc_ABC.txt")]
    #[case("abc_ABC.txt","TXT","abc_ABC.txt")]
    fn test_remove_any_sensitive_ignore_ext(#[case] input: &str, #[case] remove_text:&str,#[case] expected:&str) {
        println!("测试: 删除所有, 区分大小写, 忽略后缀名");
        let rule = RemoveRule{
            text: remove_text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: true,
            ignore_extension: true,
        };
        let removed = remove(&input.to_string(), rule);
        assert_eq!(removed, expected);
    }




    #[rstest]
    #[case("abc.txt","a","bc.txt")]
    #[case("abca.txt","a","bc.txt")]
    #[case("aaaa.txt","a",".txt")]
    #[case("aaaa.txt","txt","aaaa.")]
    #[case("123.txt",".txt","123")]
    #[case("abc.txt",".TXT","abc")]
    #[case("abc.txt","ABC",".txt")]
    fn test_remove_any(#[case] input: &str, #[case] remove_text:&str,#[case] expected:&str) {
        println!("测试: 删除所有, 不区分大小写, 不忽略后缀名");
        let rule = RemoveRule{
            text: remove_text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: false,
            ignore_extension: false,
        };
        let removed = remove(&input.to_string(), rule);
        assert_eq!(removed, expected);
    }
    #[rstest]
    #[case("abc.txt","a","bc.txt")]
    #[case("abca.txt","bC","abca.txt")]
    #[case("aaaa.txt","A","aaaa.txt")]
    #[case("abc.txt",".TXT","abc.txt")]
    #[case("abc.txt","ABC","abc.txt")]
    fn test_remove_any_sensitive(#[case] input: &str, #[case] remove_text:&str,#[case] expected:&str) {
        println!("测试: 删除所有, 区分大小写, 不忽略后缀名");
        let rule = RemoveRule{
            text: remove_text.to_string(),
            remove_position: RemovePosition::All,
            case_sensitive: true,
            ignore_extension: false,
        };
        let removed = remove(&input.to_string(), rule);
        assert_eq!(removed, expected);
    }
}
