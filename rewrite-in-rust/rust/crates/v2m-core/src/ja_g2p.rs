//! Japanese fallback G2P compatibility helpers.
//!
//! This module mirrors deterministic fallback behavior from
//! `inference/LyricFA/tools/JaG2p.py` with `pyopenjtalk` unavailable. Python
//! remains the runtime owner for OpenJTalk frontend analysis, language
//! processors, LyricMatcher/lfa_api orchestration, model execution,
//! GUI/Web/CLI callers, and production routing.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Stateless Japanese fallback grapheme-to-phoneme converter.
pub struct JaG2p;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AnalysisEntry {
    moras: Vec<String>,
    kana_moras: Vec<String>,
}

impl JaG2p {
    /// Creates a Japanese fallback converter.
    pub fn new() -> Self {
        Self
    }

    /// Converts text to romaji moras using legacy fallback tokenization.
    pub fn convert(&self, text: &str, include_tone: bool, convert_number: bool) -> String {
        let tokens = split_input_string_no_regex(text);
        self.convert_list(&tokens, include_tone, convert_number)
    }

    /// Converts pre-tokenized text to romaji moras.
    ///
    /// `include_tone` is accepted for signature compatibility. The legacy
    /// Japanese implementation ignores it.
    pub fn convert_list(
        &self,
        input_list: &[String],
        _include_tone: bool,
        convert_number: bool,
    ) -> String {
        let mut analysis = Vec::new();
        for token in input_list {
            analysis.extend(self.analyze_token(token, convert_number));
        }

        analysis
            .into_iter()
            .flat_map(|entry| entry.moras)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns romaji moras as display tokens.
    pub fn split_string_no_regex(&self, text: &str) -> Vec<String> {
        self.get_analysis(text)
            .into_iter()
            .flat_map(|entry| entry.moras)
            .collect()
    }

    /// Returns kana moras as display tokens.
    pub fn split_kana_no_regex(&self, text: &str) -> Vec<String> {
        self.get_analysis(text)
            .into_iter()
            .flat_map(|entry| entry.kana_moras)
            .collect()
    }

    fn get_analysis(&self, text: &str) -> Vec<AnalysisEntry> {
        let normalized_text = normalize_text(text);
        let mut analysis = Vec::new();
        for token in split_input_string_no_regex(&normalized_text) {
            analysis.extend(self.analyze_token(&token, true));
        }
        analysis
    }

    fn analyze_token(&self, token: &str, convert_number: bool) -> Vec<AnalysisEntry> {
        let normalized_token = normalize_text(token);
        if normalized_token.is_empty() {
            return Vec::new();
        }

        if normalized_token
            .chars()
            .all(|character| number_map(character).is_some())
        {
            if convert_number {
                let mapped = normalized_token
                    .chars()
                    .filter_map(number_map)
                    .collect::<String>();
                return self.analyze_japanese_segment(&mapped);
            }
            return fallback_entry(&normalized_token);
        }

        if normalized_token
            .chars()
            .all(|character| is_letter(character) || is_special_letter(character))
        {
            return fallback_entry(&normalized_token);
        }

        if normalized_token.chars().any(is_japanese_char) {
            return self.analyze_japanese_segment(&normalized_token);
        }

        fallback_entry(&normalized_token)
    }

    fn analyze_japanese_segment(&self, segment: &str) -> Vec<AnalysisEntry> {
        let normalized_segment = normalize_text(segment);
        if normalized_segment.is_empty() {
            return Vec::new();
        }

        let smaller_tokens = split_japanese_segment(&normalized_segment);
        if smaller_tokens.len() > 1 {
            let mut fallback_analysis = Vec::new();
            for token in smaller_tokens {
                fallback_analysis.extend(self.analyze_token(&token, false));
            }
            if !fallback_analysis.is_empty() {
                return fallback_analysis;
            }
        }

        if normalized_segment.chars().any(is_kana) {
            let direct_entry = self.parse_pron_to_entry(&normalized_segment, &normalized_segment);
            if !direct_entry.is_empty() {
                return direct_entry;
            }
        }

        fallback_entry(&normalized_segment)
    }

    fn parse_pron_to_entry(&self, original: &str, pron: &str) -> Vec<AnalysisEntry> {
        let cleaned_pron = normalize_text(&pron.replace('’', ""));
        if cleaned_pron.is_empty() {
            return Vec::new();
        }

        let kata_pron = hiragana_to_katakana(&cleaned_pron);
        let moras = kata_to_moras(&kata_pron);
        let kana_moras = kata_to_kana_moras(&kata_pron);
        if moras.is_empty() {
            return Vec::new();
        }

        let _ = original;
        vec![AnalysisEntry { moras, kana_moras }]
    }
}

/// Collapses Unicode whitespace to single ASCII spaces and trims the result.
pub fn normalize_text(text: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    for character in text.chars() {
        if character.is_whitespace() {
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

/// Splits input text using legacy no-regex Japanese tokenization.
pub fn split_input_string_no_regex(input: &str) -> Vec<String> {
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
        } else if is_numeric_like(current_char) {
            let start = position;
            while position < chars.len() && is_numeric_like(chars[position]) {
                position += 1;
            }
            result.push(chars[start..position].iter().collect());
        } else if is_japanese_char(current_char) {
            let start = position;
            while position < chars.len() && is_japanese_char(chars[position]) {
                position += 1;
            }
            result.push(chars[start..position].iter().collect());
        } else {
            position += 1;
        }
    }
    result
}

/// Splits a Japanese segment into kana contraction/long-vowel groups.
pub fn split_japanese_segment(segment: &str) -> Vec<String> {
    let chars = segment.chars().collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut position = 0_usize;
    while position < chars.len() {
        let current_char = chars[position];
        if is_kana(current_char) {
            let mut length = if position + 1 < chars.len() && is_special_kana(chars[position + 1]) {
                2
            } else {
                1
            };
            if position + length < chars.len() && chars[position + length] == 'ー' {
                length += 1;
            }
            result.push(chars[position..position + length].iter().collect());
            position += length;
        } else {
            result.push(current_char.to_string());
            position += 1;
        }
    }
    result
}

/// Returns true for ASCII letters accepted by the legacy tokenizer.
pub fn is_letter(character: char) -> bool {
    character.is_ascii_alphabetic()
}

/// Returns true for special word-joining characters accepted as letters.
pub fn is_special_letter(character: char) -> bool {
    matches!(character, '\'' | '-' | '’')
}

/// Returns true for characters where Python 3.12 `str.isdigit()` is true.
pub fn is_digit(character: char) -> bool {
    let code = character as u32;
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

/// Returns true for digit-like tokens, including Japanese zero `〇`.
pub fn is_numeric_like(character: char) -> bool {
    is_digit(character) || character == '〇'
}

/// Returns true for CJK Unified Ideographs accepted by the fallback path.
pub fn is_kanji(character: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&character)
}

/// Returns true for Hiragana or Katakana codepoints.
pub fn is_kana(character: char) -> bool {
    ('\u{3040}'..='\u{309f}').contains(&character) || ('\u{30a0}'..='\u{30ff}').contains(&character)
}

/// Returns true for small kana that joins with the previous kana.
pub fn is_special_kana(character: char) -> bool {
    "ャュョゃゅょァィゥェォぁぃぅぇぉ".contains(character)
}

/// Returns true for Japanese symbols preserved by the tokenizer.
pub fn is_japanese_symbol(character: char) -> bool {
    matches!(character, '々' | '〆' | 'ヶ' | 'ヵ' | 'ー' | '〇')
}

/// Returns true for kanji, kana, or supported Japanese symbols.
pub fn is_japanese_char(character: char) -> bool {
    is_kanji(character) || is_kana(character) || is_japanese_symbol(character)
}

fn fallback_entry(token: &str) -> Vec<AnalysisEntry> {
    let normalized = normalize_text(token);
    if normalized.is_empty() {
        return Vec::new();
    }

    if normalized
        .chars()
        .all(|character| is_letter(character) || is_special_letter(character))
    {
        let lowered = normalized.to_lowercase();
        return vec![AnalysisEntry {
            moras: vec![lowered.clone()],
            kana_moras: vec![lowered],
        }];
    }

    let kana_token = if normalized.chars().any(is_kana) {
        katakana_to_hiragana(&normalized)
    } else {
        normalized.clone()
    };
    vec![AnalysisEntry {
        moras: vec![normalized],
        kana_moras: vec![kana_token],
    }]
}

fn katakana_to_hiragana(text: &str) -> String {
    text.chars()
        .map(|character| {
            let code = character as u32;
            if (0x30A1..=0x30F6).contains(&code) {
                char::from_u32(code - 0x60).unwrap_or(character)
            } else {
                character
            }
        })
        .collect()
}

fn hiragana_to_katakana(text: &str) -> String {
    text.chars()
        .map(|character| {
            let code = character as u32;
            if (0x3041..=0x3096).contains(&code) {
                char::from_u32(code + 0x60).unwrap_or(character)
            } else {
                character
            }
        })
        .collect()
}

fn kata_to_moras(kata: &str) -> Vec<String> {
    kata_to_mora_pairs(kata)
        .into_iter()
        .map(|(_, romaji)| romaji)
        .collect()
}

fn kata_to_kana_moras(kata: &str) -> Vec<String> {
    kata_to_mora_pairs(kata)
        .into_iter()
        .map(|(kana, _)| katakana_to_hiragana(&kana))
        .collect()
}

fn kata_to_mora_pairs(kata: &str) -> Vec<(String, String)> {
    let chars = kata.chars().collect::<Vec<_>>();
    let mut pairs = Vec::new();
    let mut index = 0_usize;
    while index < chars.len() {
        if index + 1 < chars.len() {
            let token = chars[index..index + 2].iter().collect::<String>();
            if let Some(romaji) = kata_to_romaji(&token) {
                pairs.push((token, romaji.to_string()));
                index += 2;
                continue;
            }
        }

        let token = chars[index].to_string();
        if let Some(romaji) = kata_to_romaji(&token) {
            pairs.push((token, romaji.to_string()));
            index += 1;
        } else if chars[index] == 'ー' {
            if let Some((_, last_mora)) = pairs.last()
                && last_mora != "cl"
                && last_mora != "n"
                && let Some(vowel) = last_mora.chars().last()
            {
                pairs.push((
                    vowel_to_kata(vowel).unwrap_or("ー").to_string(),
                    vowel.to_string(),
                ));
            }
            index += 1;
        } else {
            let character = chars[index];
            if character.is_alphabetic() {
                let lowered = character.to_lowercase().collect::<String>();
                pairs.push((lowered.clone(), lowered));
            }
            index += 1;
        }
    }
    pairs
}

fn number_map(character: char) -> Option<char> {
    match character {
        '0' | '０' | '〇' => Some('零'),
        '1' | '１' => Some('一'),
        '2' | '２' => Some('二'),
        '3' | '３' => Some('三'),
        '4' | '４' => Some('四'),
        '5' | '５' => Some('五'),
        '6' | '６' => Some('六'),
        '7' | '７' => Some('七'),
        '8' | '８' => Some('八'),
        '9' | '９' => Some('九'),
        _ => None,
    }
}

fn vowel_to_kata(character: char) -> Option<&'static str> {
    match character {
        'a' => Some("ア"),
        'i' => Some("イ"),
        'u' => Some("ウ"),
        'e' => Some("エ"),
        'o' => Some("オ"),
        _ => None,
    }
}

fn kata_to_romaji(token: &str) -> Option<&'static str> {
    match token {
        "ア" => Some("a"),
        "イ" => Some("i"),
        "ウ" => Some("u"),
        "エ" => Some("e"),
        "オ" => Some("o"),
        "カ" => Some("ka"),
        "キ" => Some("ki"),
        "ク" => Some("ku"),
        "ケ" => Some("ke"),
        "コ" => Some("ko"),
        "サ" => Some("sa"),
        "シ" => Some("shi"),
        "ス" => Some("su"),
        "セ" => Some("se"),
        "ソ" => Some("so"),
        "タ" => Some("ta"),
        "チ" => Some("chi"),
        "ツ" => Some("tsu"),
        "テ" => Some("te"),
        "ト" => Some("to"),
        "ナ" => Some("na"),
        "ニ" => Some("ni"),
        "ヌ" => Some("nu"),
        "ネ" => Some("ne"),
        "ノ" => Some("no"),
        "ハ" => Some("ha"),
        "ヒ" => Some("hi"),
        "フ" => Some("fu"),
        "ヘ" => Some("he"),
        "ホ" => Some("ho"),
        "マ" => Some("ma"),
        "ミ" => Some("mi"),
        "ム" => Some("mu"),
        "メ" => Some("me"),
        "モ" => Some("mo"),
        "ヤ" => Some("ya"),
        "ユ" => Some("yu"),
        "ヨ" => Some("yo"),
        "ラ" => Some("ra"),
        "リ" => Some("ri"),
        "ル" => Some("ru"),
        "レ" => Some("re"),
        "ロ" => Some("ro"),
        "ワ" => Some("wa"),
        "ヲ" => Some("o"),
        "ン" => Some("n"),
        "ガ" => Some("ga"),
        "ギ" => Some("gi"),
        "グ" => Some("gu"),
        "ゲ" => Some("ge"),
        "ゴ" => Some("go"),
        "ザ" => Some("za"),
        "ジ" => Some("ji"),
        "ズ" => Some("zu"),
        "ゼ" => Some("ze"),
        "ゾ" => Some("zo"),
        "ダ" => Some("da"),
        "ヂ" => Some("ji"),
        "ヅ" => Some("zu"),
        "デ" => Some("de"),
        "ド" => Some("do"),
        "バ" => Some("ba"),
        "ビ" => Some("bi"),
        "ブ" => Some("bu"),
        "ベ" => Some("be"),
        "ボ" => Some("bo"),
        "パ" => Some("pa"),
        "ピ" => Some("pi"),
        "プ" => Some("pu"),
        "ペ" => Some("pe"),
        "ポ" => Some("po"),
        "キャ" => Some("kya"),
        "キュ" => Some("kyu"),
        "キョ" => Some("kyo"),
        "シャ" => Some("sha"),
        "シュ" => Some("shu"),
        "ショ" => Some("sho"),
        "チャ" => Some("cha"),
        "チュ" => Some("chu"),
        "チョ" => Some("cho"),
        "ニャ" => Some("nya"),
        "ニュ" => Some("nyu"),
        "ニョ" => Some("nyo"),
        "ヒャ" => Some("hya"),
        "ヒュ" => Some("hyu"),
        "ヒョ" => Some("hyo"),
        "ミャ" => Some("mya"),
        "ミュ" => Some("myu"),
        "ミョ" => Some("myo"),
        "リャ" => Some("rya"),
        "リュ" => Some("ryu"),
        "リョ" => Some("ryo"),
        "ギャ" => Some("gya"),
        "ギュ" => Some("gyu"),
        "ギョ" => Some("gyo"),
        "ジャ" => Some("ja"),
        "ジュ" => Some("ju"),
        "ジョ" => Some("jo"),
        "ビャ" => Some("bya"),
        "ビュ" => Some("byu"),
        "ビョ" => Some("byo"),
        "ピャ" => Some("pya"),
        "ピュ" => Some("pyu"),
        "ピョ" => Some("pyo"),
        "ファ" => Some("fa"),
        "フィ" => Some("fi"),
        "フェ" => Some("fe"),
        "フォ" => Some("fo"),
        "ヴァ" => Some("va"),
        "ヴィ" => Some("vi"),
        "ヴ" => Some("vu"),
        "ヴェ" => Some("ve"),
        "ヴォ" => Some("vo"),
        "ティ" => Some("ti"),
        "ディ" => Some("di"),
        "トゥ" => Some("tu"),
        "ドゥ" => Some("du"),
        "チェ" => Some("che"),
        "ジェ" => Some("je"),
        "シェ" => Some("she"),
        "ウィ" => Some("wi"),
        "ウェ" => Some("we"),
        "ウォ" => Some("wo"),
        "クァ" => Some("kwa"),
        "グァ" => Some("gwa"),
        "ッ" => Some("cl"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/ja_g2p_fallback_core.jsonl");

    fn parse_string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn single_char(value: &str) -> char {
        let mut chars = value.chars();
        let character = chars.next().unwrap();
        assert!(chars.next().is_none(), "{value:?} is not one char");
        character
    }

    fn classify_char(value: &str) -> Value {
        let character = single_char(value);
        json!({
            "char": value,
            "letter": is_letter(character),
            "special_letter": is_special_letter(character),
            "digit": is_digit(character),
            "numeric_like": is_numeric_like(character),
            "kanji": is_kanji(character),
            "kana": is_kana(character),
            "special_kana": is_special_kana(character),
            "japanese_symbol": is_japanese_symbol(character),
            "japanese_char": is_japanese_char(character),
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
    fn ja_g2p_fallback_follows_parity_fixture_table() {
        let g2p = JaG2p::new();
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "classify" => json!({
                    "items": parse_string_vec(&case["chars"])
                        .iter()
                        .map(|value| classify_char(value))
                        .collect::<Vec<_>>(),
                }),
                "normalize_text" => json!({
                    "value": normalize_text(case["text"].as_str().unwrap()),
                }),
                "split_input" => json!({
                    "tokens": split_input_string_no_regex(case["text"].as_str().unwrap()),
                }),
                "split_japanese_segment" => json!({
                    "tokens": split_japanese_segment(case["text"].as_str().unwrap()),
                }),
                "kata_moras" => json!({
                    "moras": kata_to_moras(case["text"].as_str().unwrap()),
                    "kana_moras": kata_to_kana_moras(case["text"].as_str().unwrap()),
                }),
                "convert_text" => json!({
                    "value": g2p.convert(
                        case["text"].as_str().unwrap(),
                        case["include_tone"].as_bool().unwrap_or(false),
                        case["convert_number"].as_bool().unwrap_or(true),
                    ),
                }),
                "convert_list" => json!({
                    "value": g2p.convert_list(
                        &parse_string_vec(&case["input_list"]),
                        case["include_tone"].as_bool().unwrap_or(false),
                        case["convert_number"].as_bool().unwrap_or(true),
                    ),
                }),
                "split_romaji" => json!({
                    "tokens": g2p.split_string_no_regex(case["text"].as_str().unwrap()),
                }),
                "split_kana" => json!({
                    "tokens": g2p.split_kana_no_regex(case["text"].as_str().unwrap()),
                }),
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
