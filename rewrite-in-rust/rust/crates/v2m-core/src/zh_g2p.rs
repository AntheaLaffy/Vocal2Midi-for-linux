//! Chinese dictionary G2P compatibility helpers.
//!
//! This module mirrors deterministic behavior from
//! `inference/LyricFA/tools/ZhG2p.py`. Python remains the runtime owner for
//! bundled dictionary file loading, language processors, LyricMatcher/lfa_api
//! orchestration, Japanese G2P, model execution, GUI/Web/CLI callers, and
//! production routing.

use std::collections::HashMap;

/// In-memory dictionaries used by the Chinese G2P converter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZhG2pDictionaries {
    /// The phrases map.
    pub phrases_map: HashMap<String, String>,
    /// The trans dict.
    pub trans_dict: HashMap<String, String>,
    /// The word dict.
    pub word_dict: HashMap<String, Vec<String>>,
    /// The phrases dict.
    pub phrases_dict: HashMap<String, Vec<String>>,
}

/// Dictionary-backed converter matching legacy `ZhG2p` control flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZhG2p {
    dictionaries: ZhG2pDictionaries,
}

impl ZhG2p {
    /// Creates a converter from already loaded dictionary maps.
    pub fn new(dictionaries: ZhG2pDictionaries) -> Self {
        Self { dictionaries }
    }

    /// Splits and converts text using legacy `split_string_no_regex`.
    pub fn convert(&self, input_text: &str, include_tone: bool, convert_number: bool) -> String {
        self.convert_list(
            &split_string_no_regex(input_text),
            include_tone,
            convert_number,
        )
    }

    /// Converts a pre-tokenized list and restores converted tokens into the
    /// original token positions.
    pub fn convert_list(
        &self,
        input_list: &[String],
        include_tone: bool,
        convert_number: bool,
    ) -> String {
        let mut processed_input = Vec::new();
        let mut input_positions = Vec::new();
        self.zh_position(
            input_list,
            &mut processed_input,
            &mut input_positions,
            convert_number,
        );

        let mut result = Vec::new();
        let mut cursor = 0_usize;
        while cursor < processed_input.len() {
            let raw_current_char = &processed_input[cursor];
            let current_char = self.traditional_to_simplified(raw_current_char);

            if !self.dictionaries.word_dict.contains_key(&current_char) {
                result.push(current_char);
                cursor += 1;
                continue;
            }

            if !self.is_polyphonic(&current_char) {
                result.push(
                    self.get_default_pinyin(&current_char)
                        .unwrap_or(current_char),
                );
                cursor += 1;
                continue;
            }

            let mut found = false;
            for length in (2..=4).rev() {
                if cursor + length <= processed_input.len() {
                    let sub_phrase = join_tokens(&processed_input[cursor..cursor + length]);
                    if let Some(phrase) = self.dictionaries.phrases_dict.get(&sub_phrase) {
                        result.extend(phrase.iter().cloned());
                        cursor += length;
                        found = true;
                    }

                    if cursor >= 1 && !found {
                        let start = cursor - 1;
                        let end = cursor + length - 1;
                        let sub_phrase = join_tokens(&processed_input[start..end]);
                        if let Some(phrase) = self.dictionaries.phrases_dict.get(&sub_phrase) {
                            result.pop();
                            result.extend(phrase.iter().cloned());
                            cursor += length - 1;
                            found = true;
                        }
                    }
                }

                if cursor + 1 >= length && !found && cursor < processed_input.len() {
                    let start = cursor + 1 - length;
                    if start < cursor + 1 {
                        let sub_phrase = join_tokens(&processed_input[start..cursor + 1]);
                        if let Some(phrase) = self.dictionaries.phrases_dict.get(&sub_phrase) {
                            remove_elements(&mut result, start, length - 1);
                            result.extend(phrase.iter().cloned());
                            cursor += 1;
                            found = true;
                        }
                    }
                }

                if cursor + 2 >= length && !found && cursor < processed_input.len() {
                    let start = cursor + 2 - length;
                    if start < cursor + 2 && cursor + 2 <= processed_input.len() {
                        let sub_phrase = join_tokens(&processed_input[start..cursor + 2]);
                        if let Some(phrase) = self.dictionaries.phrases_dict.get(&sub_phrase) {
                            remove_elements(&mut result, start, length - 1);
                            result.extend(phrase.iter().cloned());
                            cursor += 2;
                            found = true;
                        }
                    }
                }
            }

            if !found {
                result.push(
                    self.get_default_pinyin(&current_char)
                        .unwrap_or(current_char),
                );
                cursor += 1;
            }
        }

        if !include_tone {
            result = result
                .into_iter()
                .map(|value| strip_trailing_ascii_digit(&value))
                .collect();
        }

        let result = result
            .iter()
            .map(|value| tone_to_normal(value, false))
            .collect::<Vec<_>>();
        reset_zh(input_list, &result, &input_positions)
    }

    fn zh_position(
        &self,
        input_list: &[String],
        result: &mut Vec<String>,
        positions: &mut Vec<usize>,
        convert_number: bool,
    ) {
        for (index, value) in input_list.iter().enumerate() {
            if self.dictionaries.word_dict.contains_key(value)
                || self.dictionaries.trans_dict.contains_key(value)
            {
                result.push(value.clone());
                positions.push(index);
            } else if convert_number && let Some(number) = number_map(value) {
                result.push(number.to_string());
                positions.push(index);
            }
        }
    }

    fn is_polyphonic(&self, text: &str) -> bool {
        self.dictionaries.phrases_map.contains_key(text)
    }

    fn traditional_to_simplified(&self, text: &str) -> String {
        self.dictionaries
            .trans_dict
            .get(text)
            .cloned()
            .unwrap_or_else(|| text.to_string())
    }

    fn get_default_pinyin(&self, text: &str) -> Option<String> {
        let simplified = self.traditional_to_simplified(text);
        self.dictionaries
            .word_dict
            .get(&simplified)
            .and_then(|values| values.first())
            .cloned()
    }
}

/// Normalizes tone marks to ASCII pinyin letters, optionally restoring `ü`.
pub fn tone_to_normal(pinyin: &str, v_to_u: bool) -> String {
    let mut result = String::new();
    for character in pinyin.chars() {
        if character.is_ascii_lowercase() {
            result.push(character);
        } else if let Some(mapped) = tone_base(character) {
            result.push(mapped);
        } else {
            result.push(character);
        }
    }
    if v_to_u {
        result.replace('v', "ü")
    } else {
        result
    }
}

/// Splits text using the module-level legacy tokenizer.
pub fn split_string(input: &str) -> Vec<String> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut position = 0_usize;
    while position < chars.len() {
        let current_char = chars[position];
        if is_letter(current_char) || is_special_letter(current_char) {
            let start = position;
            while position < chars.len()
                && (is_letter(chars[position]) || is_special_letter(chars[position]))
            {
                position += 1;
            }
            result.push(chars[start..position].iter().collect());
        } else if is_hanzi(current_char) || is_digit_like(current_char) {
            result.push(current_char.to_string());
            position += 1;
        } else if is_kana(current_char) {
            let length = if position + 1 < chars.len() && is_special_kana(chars[position + 1]) {
                2
            } else {
                1
            };
            result.push(chars[position..position + length].iter().collect());
            position += length;
        } else {
            position += 1;
        }
    }
    result
}

/// Splits text using `ZhG2p.split_string_no_regex` behavior.
pub fn split_string_no_regex(input: &str) -> Vec<String> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut position = 0_usize;
    while position < chars.len() {
        let current_char = chars[position];
        if is_letter(current_char) {
            let start = position;
            while position < chars.len() && is_letter(chars[position]) {
                position += 1;
            }
            result.push(chars[start..position].iter().collect());
        } else if is_hanzi(current_char) || is_digit_like(current_char) {
            result.push(current_char.to_string());
            position += 1;
        } else if is_kana(current_char) {
            let length = if position + 1 < chars.len() && is_special_kana(chars[position + 1]) {
                2
            } else {
                1
            };
            result.push(chars[position..position + length].iter().collect());
            position += length;
        } else {
            position += 1;
        }
    }
    result
}

fn tone_base(character: char) -> Option<char> {
    match character {
        'ā' | 'á' | 'ǎ' | 'à' => Some('a'),
        'ō' | 'ó' | 'ǒ' | 'ò' => Some('o'),
        'ē' | 'é' | 'ě' | 'è' => Some('e'),
        'ī' | 'í' | 'ǐ' | 'ì' => Some('i'),
        'ū' | 'ú' | 'ǔ' | 'ù' => Some('u'),
        'ḿ' => Some('m'),
        'ǹ' => Some('n'),
        'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => Some('v'),
        _ => None,
    }
}

fn is_letter(character: char) -> bool {
    character.is_ascii_alphabetic()
}

fn is_special_letter(character: char) -> bool {
    matches!(character, '\'' | '-' | '’')
}

fn is_hanzi(character: char) -> bool {
    ('\u{4e00}'..='\u{9fa5}').contains(&character)
}

fn is_digit_like(character: char) -> bool {
    let code = character as u32;
    // Python `str.isdigit()` is narrower than Rust `char::is_numeric()`.
    matches!(
        code,
        0x0030..=0x0039
            | 0x00B2..=0x00B3
            | 0x00B9
            | 0x0660..=0x0669
            | 0x06F0..=0x06F9
            | 0x07C0..=0x07C9
            | 0x0966..=0x096F
            | 0x09E6..=0x09EF
            | 0x0A66..=0x0A6F
            | 0x0AE6..=0x0AEF
            | 0x0B66..=0x0B6F
            | 0x0BE6..=0x0BEF
            | 0x0C66..=0x0C6F
            | 0x0CE6..=0x0CEF
            | 0x0D66..=0x0D6F
            | 0x0DE6..=0x0DEF
            | 0x0E50..=0x0E59
            | 0x0ED0..=0x0ED9
            | 0x0F20..=0x0F29
            | 0x1040..=0x1049
            | 0x1090..=0x1099
            | 0x1369..=0x1371
            | 0x17E0..=0x17E9
            | 0x1810..=0x1819
            | 0x1946..=0x194F
            | 0x19D0..=0x19DA
            | 0x1A80..=0x1A89
            | 0x1A90..=0x1A99
            | 0x1B50..=0x1B59
            | 0x1BB0..=0x1BB9
            | 0x1C40..=0x1C49
            | 0x1C50..=0x1C59
            | 0x2070
            | 0x2074..=0x2079
            | 0x2080..=0x2089
            | 0x2460..=0x2468
            | 0x2474..=0x247C
            | 0x2488..=0x2490
            | 0x24EA
            | 0x24F5..=0x24FD
            | 0x24FF
            | 0x2776..=0x277E
            | 0x2780..=0x2788
            | 0x278A..=0x2792
            | 0xA620..=0xA629
            | 0xA8D0..=0xA8D9
            | 0xA900..=0xA909
            | 0xA9D0..=0xA9D9
            | 0xA9F0..=0xA9F9
            | 0xAA50..=0xAA59
            | 0xABF0..=0xABF9
            | 0xFF10..=0xFF19
            | 0x104A0..=0x104A9
            | 0x10A40..=0x10A43
            | 0x10D30..=0x10D39
            | 0x10E60..=0x10E68
            | 0x11052..=0x1105A
            | 0x11066..=0x1106F
            | 0x110F0..=0x110F9
            | 0x11136..=0x1113F
            | 0x111D0..=0x111D9
            | 0x112F0..=0x112F9
            | 0x11450..=0x11459
            | 0x114D0..=0x114D9
            | 0x11650..=0x11659
            | 0x116C0..=0x116C9
            | 0x11730..=0x11739
            | 0x118E0..=0x118E9
            | 0x11950..=0x11959
            | 0x11C50..=0x11C59
            | 0x11D50..=0x11D59
            | 0x11DA0..=0x11DA9
            | 0x11F50..=0x11F59
            | 0x16A60..=0x16A69
            | 0x16AC0..=0x16AC9
            | 0x16B50..=0x16B59
            | 0x1D7CE..=0x1D7FF
            | 0x1E140..=0x1E149
            | 0x1E2F0..=0x1E2F9
            | 0x1E4F0..=0x1E4F9
            | 0x1E950..=0x1E959
            | 0x1F100..=0x1F10A
            | 0x1FBF0..=0x1FBF9
    )
}

fn is_kana(character: char) -> bool {
    ('\u{3040}'..='\u{309f}').contains(&character) || ('\u{30a0}'..='\u{30ff}').contains(&character)
}

fn is_special_kana(character: char) -> bool {
    "ャュョゃゅょァィゥェォぁぃぅぇぉ".contains(character)
}

fn number_map(value: &str) -> Option<&'static str> {
    match value {
        "0" => Some("零"),
        "1" => Some("一"),
        "2" => Some("二"),
        "3" => Some("三"),
        "4" => Some("四"),
        "5" => Some("五"),
        "6" => Some("六"),
        "7" => Some("七"),
        "8" => Some("八"),
        "9" => Some("九"),
        _ => None,
    }
}

fn reset_zh(input_list: &[String], result: &[String], positions: &[usize]) -> String {
    let mut final_result = input_list.to_vec();
    for (index, position) in positions.iter().copied().enumerate() {
        if let Some(slot) = final_result.get_mut(position)
            && let Some(value) = result.get(index)
        {
            *slot = value.clone();
        }
    }
    final_result.join(" ")
}

fn remove_elements(list_to_modify: &mut Vec<String>, start_index: usize, count: usize) {
    if start_index < list_to_modify.len() && count > 0 {
        let end = (start_index + count).min(list_to_modify.len());
        list_to_modify.drain(start_index..end);
    }
}

fn strip_trailing_ascii_digit(value: &str) -> String {
    if value
        .chars()
        .last()
        .is_some_and(|character| character.is_ascii_digit())
    {
        let mut output = value.to_string();
        output.pop();
        output
    } else {
        value.to_string()
    }
}

fn join_tokens(tokens: &[String]) -> String {
    tokens.concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/zh_g2p_dictionary_core.jsonl");

    fn parse_string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn parse_string_map(value: &Value) -> HashMap<String, String> {
        value
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
            .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_string()))
            .collect()
    }

    fn parse_vec_map(value: &Value) -> HashMap<String, Vec<String>> {
        value
            .as_object()
            .unwrap_or(&Map::new())
            .iter()
            .map(|(key, value)| (key.clone(), parse_string_vec(value)))
            .collect()
    }

    fn parse_dicts(value: &Value) -> ZhG2pDictionaries {
        ZhG2pDictionaries {
            phrases_map: parse_string_map(&value["phrases_map"]),
            trans_dict: parse_string_map(&value["trans_dict"]),
            word_dict: parse_vec_map(&value["word_dict"]),
            phrases_dict: parse_vec_map(&value["phrases_dict"]),
        }
    }

    fn assert_json_close(actual: &Value, expected: &Value, context: &str) {
        match (actual, expected) {
            (Value::Array(left), Value::Array(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: array lengths differ");
                for (index, (left_item, right_item)) in left.iter().zip(right).enumerate() {
                    assert_json_close(left_item, right_item, &format!("{context}[{index}]"));
                }
            }
            (Value::Object(left), Value::Object(right)) => {
                assert_eq!(left.len(), right.len(), "{context}: object lengths differ");
                for (key, right_value) in right {
                    let left_value = left
                        .get(key)
                        .unwrap_or_else(|| panic!("{context}: missing {key}"));
                    assert_json_close(left_value, right_value, &format!("{context}.{key}"));
                }
            }
            _ => assert_eq!(actual, expected, "{context}"),
        }
    }

    #[test]
    fn zh_g2p_dictionary_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "tone_to_normal" => json!({
                    "value": tone_to_normal(
                        case["pinyin"].as_str().unwrap(),
                        case["v_to_u"].as_bool().unwrap_or(false),
                    ),
                }),
                "split_string" => json!({
                    "tokens": split_string(case["text"].as_str().unwrap()),
                }),
                "split_no_regex" => json!({
                    "tokens": split_string_no_regex(case["text"].as_str().unwrap()),
                }),
                "convert_list" => {
                    let g2p = ZhG2p::new(parse_dicts(&case["dicts"]));
                    json!({
                        "value": g2p.convert_list(
                            &parse_string_vec(&case["input_list"]),
                            case["include_tone"].as_bool().unwrap_or(false),
                            case["convert_number"].as_bool().unwrap_or(false),
                        ),
                    })
                }
                "convert_text" => {
                    let g2p = ZhG2p::new(parse_dicts(&case["dicts"]));
                    json!({
                        "value": g2p.convert(
                            case["text"].as_str().unwrap(),
                            case["include_tone"].as_bool().unwrap_or(false),
                            case["convert_number"].as_bool().unwrap_or(false),
                        ),
                    })
                }
                other => panic!("unknown fixture kind {other}"),
            };

            assert_json_close(
                &actual,
                &case["expect"],
                &format!(
                    "{} fixture line {}",
                    case["case_id"].as_str().unwrap(),
                    line_index + 1
                ),
            );
        }
    }
}
