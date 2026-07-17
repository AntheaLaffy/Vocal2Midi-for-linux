//! Lyric language processor compatibility helpers.
//!
//! This module mirrors deterministic behavior from
//! `inference/LyricFA/tools/language_processors.py`. Python remains the runtime
//! owner for bundled dictionary file loading, OpenJTalk frontend analysis,
//! LyricMatcher file/state/JSON orchestration, model execution, GUI/Web/CLI
//! callers, and production routing.

use std::error::Error;
use std::fmt;

use crate::ja_g2p::JaG2p;
use crate::zh_g2p::{self, ZhG2p, ZhG2pDictionaries};

/// Processed lyric data produced by a language processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricData {
    pub text_list: Vec<String>,
    pub phonetic_list: Vec<String>,
    pub raw_text: String,
}

/// Error returned when a language code has no processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedLanguageError {
    language_code: String,
}

impl UnsupportedLanguageError {
    /// Returns the rejected language code as supplied by the caller.
    pub fn language_code(&self) -> &str {
        &self.language_code
    }
}

impl fmt::Display for UnsupportedLanguageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Unsupported language: {}", self.language_code)
    }
}

impl Error for UnsupportedLanguageError {}

/// Factory for legacy lyric language processors.
pub struct ProcessorFactory;

impl ProcessorFactory {
    /// Creates a processor using empty injected Chinese dictionaries.
    ///
    /// Production dictionary file IO remains Python-owned; writer fixtures that
    /// need Chinese phonetics should call `create_processor_with_zh_dictionaries`.
    ///
    /// # Errors
    ///
    /// Returns `UnsupportedLanguageError` when `language_code` is not `zh`,
    /// `en`, or `ja` after lowercasing.
    pub fn create_processor(language_code: &str) -> Result<Processor, UnsupportedLanguageError> {
        Self::create_processor_with_zh_dictionaries(language_code, ZhG2pDictionaries::default())
    }

    /// Creates a processor with caller-supplied Chinese dictionaries.
    ///
    /// # Errors
    ///
    /// Returns `UnsupportedLanguageError` when `language_code` is not `zh`,
    /// `en`, or `ja` after lowercasing.
    pub fn create_processor_with_zh_dictionaries(
        language_code: &str,
        zh_dictionaries: ZhG2pDictionaries,
    ) -> Result<Processor, UnsupportedLanguageError> {
        match language_code.to_lowercase().as_str() {
            "zh" => Ok(Processor::Chinese(ChineseProcessor::new(zh_dictionaries))),
            "en" => Ok(Processor::English(EnglishProcessor::new())),
            "ja" => Ok(Processor::Japanese(JapaneseProcessor::new())),
            _ => Err(UnsupportedLanguageError {
                language_code: language_code.to_string(),
            }),
        }
    }

    /// Returns supported language codes in legacy insertion order.
    pub fn get_supported_languages() -> Vec<&'static str> {
        vec!["zh", "en", "ja"]
    }
}

/// One of the legacy language processors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Processor {
    Chinese(ChineseProcessor),
    English(EnglishProcessor),
    Japanese(JapaneseProcessor),
}

impl Processor {
    /// Returns the legacy processor type name.
    pub const fn processor_type(&self) -> &'static str {
        match self {
            Self::Chinese(_) => "ChineseProcessor",
            Self::English(_) => "EnglishProcessor",
            Self::Japanese(_) => "JapaneseProcessor",
        }
    }

    /// Returns the normalized legacy language code.
    pub const fn language_code(&self) -> &'static str {
        match self {
            Self::Chinese(processor) => processor.language_code(),
            Self::English(processor) => processor.language_code(),
            Self::Japanese(processor) => processor.language_code(),
        }
    }

    /// Cleans text using the selected processor.
    pub fn clean_text(&self, text: &str) -> String {
        match self {
            Self::Chinese(processor) => processor.clean_text(text),
            Self::English(processor) => processor.clean_text(text),
            Self::Japanese(processor) => processor.clean_text(text),
        }
    }

    /// Splits cleaned text into display tokens.
    pub fn split_text(&self, text: &str) -> Vec<String> {
        match self {
            Self::Chinese(processor) => processor.split_text(text),
            Self::English(processor) => processor.split_text(text),
            Self::Japanese(processor) => processor.split_text(text),
        }
    }

    /// Converts display tokens into phonetic tokens.
    pub fn get_phonetic_list(&self, text_list: &[String]) -> Vec<String> {
        match self {
            Self::Chinese(processor) => processor.get_phonetic_list(text_list),
            Self::English(processor) => processor.get_phonetic_list(text_list),
            Self::Japanese(processor) => processor.get_phonetic_list(text_list),
        }
    }

    /// Runs clean, split, and phonetic conversion as `LyricMatcher` does for
    /// ordinary ASR content.
    pub fn process_text(&self, raw_text: &str) -> LyricData {
        let cleaned = self.clean_text(raw_text);
        let text_list = self.split_text(&cleaned);
        let phonetic_list = self.get_phonetic_list(&text_list);
        LyricData {
            text_list,
            phonetic_list,
            raw_text: cleaned,
        }
    }

    /// Runs the Japanese reference lyric path when available, otherwise the
    /// ordinary split/phonetic flow.
    pub fn process_reference_lyric(&self, raw_text: &str) -> LyricData {
        let cleaned = self.clean_text(raw_text);
        if let Self::Japanese(processor) = self {
            let (text_list, phonetic_list) = processor.build_reference_lyric(&cleaned);
            return LyricData {
                text_list,
                phonetic_list,
                raw_text: cleaned,
            };
        }
        let text_list = self.split_text(&cleaned);
        let phonetic_list = self.get_phonetic_list(&text_list);
        LyricData {
            text_list,
            phonetic_list,
            raw_text: cleaned,
        }
    }
}

/// Chinese lyric processor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChineseProcessor {
    g2p: ZhG2p,
}

impl ChineseProcessor {
    pub fn new(dictionaries: ZhG2pDictionaries) -> Self {
        Self {
            g2p: ZhG2p::new(dictionaries),
        }
    }

    pub const fn language_code(&self) -> &'static str {
        "zh"
    }

    pub fn clean_text(&self, text: &str) -> String {
        collapse_whitespace(&clean_chinese_legacy_regex(text))
    }

    pub fn split_text(&self, text: &str) -> Vec<String> {
        zh_g2p::split_string(text)
    }

    pub fn get_phonetic_list(&self, text_list: &[String]) -> Vec<String> {
        split_on_ascii_space_preserving_empty(&self.g2p.convert_list(text_list, false, false))
    }
}

/// English lyric processor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnglishProcessor;

impl EnglishProcessor {
    pub const fn new() -> Self {
        Self
    }

    pub const fn language_code(&self) -> &'static str {
        "en"
    }

    pub fn clean_text(&self, text: &str) -> String {
        let filtered = text
            .chars()
            .filter(|character| {
                character.is_ascii_alphanumeric()
                    || is_python_regex_whitespace(*character)
                    || matches!(
                        character,
                        '.' | ',' | '!' | '?' | ';' | ':' | '"' | '\'' | '-'
                    )
            })
            .collect::<String>();
        collapse_whitespace(&filtered)
    }

    pub fn split_text(&self, text: &str) -> Vec<String> {
        zh_g2p::split_string(&text.to_lowercase())
    }

    pub fn get_phonetic_list(&self, text_list: &[String]) -> Vec<String> {
        text_list.to_vec()
    }
}

/// Japanese lyric processor for the pyopenjtalk-absent fallback path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JapaneseProcessor {
    g2p: JaG2p,
}

impl JapaneseProcessor {
    pub fn new() -> Self {
        Self { g2p: JaG2p::new() }
    }

    pub const fn language_code(&self) -> &'static str {
        "ja"
    }

    pub fn clean_text(&self, text: &str) -> String {
        collapse_whitespace(text)
    }

    pub fn split_text(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            Vec::new()
        } else {
            self.g2p.split_kana_no_regex(text)
        }
    }

    pub fn get_phonetic_list(&self, text_list: &[String]) -> Vec<String> {
        if text_list.is_empty() {
            return Vec::new();
        }
        self.g2p
            .convert_list(text_list, false, true)
            .split_whitespace()
            .map(str::to_string)
            .collect()
    }

    pub fn build_reference_lyric(&self, text: &str) -> (Vec<String>, Vec<String>) {
        if text.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let kana_moras = self
            .g2p
            .split_kana_no_regex(text)
            .into_iter()
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        if kana_moras.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let romaji_moras = self
            .g2p
            .convert_list(&kana_moras, false, false)
            .split_whitespace()
            .filter(|token| !token.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        (kana_moras, romaji_moras)
    }
}

fn collapse_whitespace(text: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    for character in text.chars() {
        if is_python_regex_whitespace(character) {
            pending_space = true;
        } else {
            if pending_space && !output.is_empty() {
                output.push(' ');
            }
            output.push(character);
            pending_space = false;
        }
    }
    output
}

fn clean_chinese_legacy_regex(text: &str) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let mut output = String::new();
    let mut index = 0_usize;
    while index < chars.len() {
        if index + 1 < chars.len()
            && chars[index + 1] == ']'
            && !is_chinese_clean_allowed(chars[index])
        {
            index += 2;
        } else {
            output.push(chars[index]);
            index += 1;
        }
    }
    output
}

fn is_chinese_clean_allowed(character: char) -> bool {
    character == '[' || (('\u{4e00}'..='\u{9fa5}').contains(&character))
}

fn is_python_regex_whitespace(character: char) -> bool {
    matches!(character, '\u{001c}'..='\u{001f}') || character.is_whitespace()
}

fn split_on_ascii_space_preserving_empty(value: &str) -> Vec<String> {
    value.split(' ').map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::collections::HashMap;

    const FIXTURES: &str =
        include_str!("../../../../fixtures/lyric_language_processor_contract.jsonl");

    fn parse_string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn parse_string_map(value: Option<&Value>) -> HashMap<String, String> {
        value
            .and_then(Value::as_object)
            .map(|items| {
                items
                    .iter()
                    .map(|(key, value)| (key.clone(), value.as_str().unwrap().to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_string_vec_map(value: Option<&Value>) -> HashMap<String, Vec<String>> {
        value
            .and_then(Value::as_object)
            .map(|items| {
                items
                    .iter()
                    .map(|(key, value)| (key.clone(), parse_string_vec(value)))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_zh_dictionaries(value: Option<&Value>) -> ZhG2pDictionaries {
        let Some(value) = value else {
            return ZhG2pDictionaries::default();
        };
        ZhG2pDictionaries {
            phrases_map: parse_string_map(value.get("phrases_map")),
            trans_dict: parse_string_map(value.get("trans_dict")),
            word_dict: parse_string_vec_map(value.get("word_dict")),
            phrases_dict: parse_string_vec_map(value.get("phrases_dict")),
        }
    }

    fn processor_for_case(case: &Value) -> Result<Processor, UnsupportedLanguageError> {
        ProcessorFactory::create_processor_with_zh_dictionaries(
            case["language"].as_str().unwrap(),
            parse_zh_dictionaries(case.get("dicts")),
        )
    }

    fn encode_processor_flow(case: &Value) -> Value {
        let processor = processor_for_case(case).unwrap();
        let cleaned = processor.clean_text(case["text"].as_str().unwrap());
        let text_list = processor.split_text(&cleaned);
        let phonetic_list = processor.get_phonetic_list(&text_list);
        json!({
            "processor_type": processor.processor_type(),
            "language_code": processor.language_code(),
            "cleaned": cleaned,
            "text_list": text_list,
            "phonetic_list": phonetic_list,
        })
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
    fn lyric_language_processor_contract_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let case_id = case["case_id"].as_str().unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "factory" => {
                    let created = parse_string_vec(&case["codes"])
                        .iter()
                        .map(|code| {
                            let processor = ProcessorFactory::create_processor(code).unwrap();
                            json!({
                                "code": code,
                                "type": processor.processor_type(),
                                "language_code": processor.language_code(),
                            })
                        })
                        .collect::<Vec<_>>();
                    let errors = parse_string_vec(&case["unsupported"])
                        .iter()
                        .map(|code| {
                            let error = ProcessorFactory::create_processor(code).unwrap_err();
                            json!({
                                "code": code,
                                "type": "ValueError",
                                "message": error.to_string(),
                            })
                        })
                        .collect::<Vec<_>>();
                    json!({
                        "supported": ProcessorFactory::get_supported_languages(),
                        "created": created,
                        "errors": errors,
                    })
                }
                "processor_flow" => encode_processor_flow(&case),
                "lyric_data_shape" => {
                    let processor = processor_for_case(&case).unwrap();
                    let lyric_data = processor.process_text(case["text"].as_str().unwrap());
                    json!({
                        "text_list": lyric_data.text_list,
                        "phonetic_list": lyric_data.phonetic_list,
                        "raw_text": lyric_data.raw_text,
                    })
                }
                "reference_lyric_data_shape" => {
                    let processor = processor_for_case(&case).unwrap();
                    let lyric_data =
                        processor.process_reference_lyric(case["text"].as_str().unwrap());
                    json!({
                        "text_list": lyric_data.text_list,
                        "phonetic_list": lyric_data.phonetic_list,
                        "raw_text": lyric_data.raw_text,
                    })
                }
                "japanese_reference_flow" => {
                    let processor = JapaneseProcessor::new();
                    let cleaned = processor.clean_text(case["text"].as_str().unwrap());
                    let text_list = processor.split_text(&cleaned);
                    let phonetic_list = processor.get_phonetic_list(&text_list);
                    let (reference_text, reference_phonetic) =
                        processor.build_reference_lyric(&cleaned);
                    json!({
                        "cleaned": cleaned,
                        "text_list": text_list,
                        "phonetic_list": phonetic_list,
                        "reference_text": reference_text,
                        "reference_phonetic": reference_phonetic,
                    })
                }
                other => panic!("unknown fixture kind {other}"),
            };

            assert_json_close(
                &actual,
                &case["expect"],
                &format!("{} fixture line {}", case_id, line_index + 1),
            );
        }
    }
}
