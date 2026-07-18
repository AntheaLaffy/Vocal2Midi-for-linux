//! Chinese ASR inverse text normalization helpers.
//!
//! This module mirrors the fixture-backed public behavior of
//! `inference/qwen3asr_dml/chinese_itn.py::chinese_to_num`. It is intentionally
//! not wired into the Python runtime.

const UNITS: &[(&str, Option<&str>)] = &[
    ("千米每小时", Some("km/h")),
    ("千克", Some("kg")),
    ("个", None),
    ("只", None),
    ("分", None),
    ("万", None),
    ("亿", None),
    ("秒", None),
    ("年", None),
    ("月", None),
    ("日", None),
    ("天", None),
    ("时", None),
    ("钟", None),
    ("人", None),
    ("层", None),
    ("楼", None),
    ("倍", None),
    ("块", None),
    ("次", None),
    ("克", Some("g")),
    ("米", Some("米")),
    ("千米", Some("千米")),
];

const IDIOMS: &[&str] = &[
    "正经八百",
    "五零二落",
    "五零四散",
    "五十步笑百步",
    "乌七八糟",
    "污七八糟",
    "四百四病",
    "思绪万千",
    "十有八九",
    "十之八九",
    "三十而立",
    "三十六策",
    "三十六计",
    "三十六行",
    "三五成群",
    "三百六十行",
    "三六九等",
    "七老八十",
    "七零八落",
    "七零八碎",
    "七七八八",
    "乱七八遭",
    "乱七八糟",
    "略知一二",
    "零零星星",
    "零七八碎",
    "九九归一",
    "二三其德",
    "二三其意",
    "无银三百两",
    "八九不离十",
    "百分之百",
    "年三十",
    "烂七八糟",
    "一点一滴",
    "路易十六",
    "九三学社",
    "五四运动",
    "入木三分",
    "九九八十一",
    "三七二十一",
    "十二五",
    "十三五",
    "十四五",
    "十五五",
    "十六五",
    "十七五",
    "十八五",
];

/// Converts Chinese spoken-form numbers in `input` to Arabic numerals.
///
/// # Panics
///
/// Panics only if the internal UTF-8 character-boundary table becomes
/// inconsistent with `input`.
pub fn chinese_to_num(input: &str) -> String {
    let mut output = String::new();
    let starts = char_starts(input);
    let mut index = 0;

    while index < starts.len() - 1 {
        if let Some(idiom) = idiom_at(input, starts[index]) {
            let after_idiom = byte_to_char_index(&starts, starts[index] + idiom.len());
            let end_index = idiom_noop_end(input, after_idiom, &starts).unwrap_or(after_idiom);
            let end_byte = if end_index < starts.len() {
                starts[end_index]
            } else {
                input.len()
            };
            output.push_str(&input[starts[index]..end_byte]);
            index = end_index;
            continue;
        }

        let byte = starts[index];
        let ch = input[byte..].chars().next().unwrap();
        if !can_start_candidate(input, index, &starts, ch) {
            output.push(ch);
            index += 1;
            continue;
        }

        let max_index = candidate_end(input, index, &starts);
        if ch == '几' {
            let end_byte = if max_index < starts.len() {
                starts[max_index]
            } else {
                input.len()
            };
            output.push_str(&input[byte..end_byte]);
            index = max_index;
            continue;
        }

        let end_byte = if max_index < starts.len() {
            starts[max_index]
        } else {
            input.len()
        };
        let candidate = &input[byte..end_byte];
        let converted = replace_candidate(input, byte, end_byte, candidate)
            .unwrap_or_else(|| candidate.to_string());
        output.push_str(&converted);
        index = max_index;
    }

    output
}

fn replace_candidate(context: &str, start: usize, end: usize, candidate: &str) -> Option<String> {
    let (head, original) = split_ascii_head(candidate);
    let group_start = start + head.len();
    if idiom_overlaps(context, lookback_start_byte(context, group_start, 2), end)
        || candidate.contains('几')
    {
        return None;
    }
    if original.is_empty() {
        return None;
    }

    let final_text = if is_range_expression(original) {
        convert_range_expression(original)?
    } else if is_time_value(original) {
        convert_time_value(original)?
    } else if is_pure_num(strip_trailing_unit(original)) {
        convert_pure_num(original, false)?
    } else if is_consecutive_value(original) {
        split_consecutive_value(original)?
    } else if is_value_num(strip_trailing_unit(original)) {
        convert_value_num(original)?
    } else if original.starts_with("百分之") && is_value_num(&original["百分之".len()..]) {
        convert_percent_value(original)?
    } else if let Some((denominator, numerator)) = original.split_once("分之") {
        if is_value_num(denominator) && is_value_num(numerator) {
            convert_fraction_value(original)?
        } else {
            return None;
        }
    } else if let Some((left, right)) = original.split_once('比') {
        if is_value_num(left) && is_value_num(right) {
            convert_ratio_value(original)?
        } else {
            return None;
        }
    } else if is_date_value(original) {
        convert_date_value(original)?
    } else {
        return None;
    };

    Some(format!("{head}{final_text}"))
}

fn split_ascii_head(candidate: &str) -> (&str, &str) {
    let mut chars = candidate.char_indices();
    let Some((_, first)) = chars.next() else {
        return ("", candidate);
    };
    if !first.is_ascii_lowercase() {
        return ("", candidate);
    }

    let mut split = first.len_utf8();
    for (byte, ch) in candidate[split..].char_indices() {
        if ch == ' ' {
            split += byte + ch.len_utf8();
            continue;
        }
        break;
    }

    let rest = &candidate[split..];
    if rest.chars().next().is_some_and(is_numeric_start) {
        (&candidate[..split], rest)
    } else {
        ("", candidate)
    }
}

fn char_starts(input: &str) -> Vec<usize> {
    input
        .char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(input.len()))
        .collect::<Vec<_>>()
}

fn byte_to_char_index(starts: &[usize], byte: usize) -> usize {
    starts
        .iter()
        .position(|candidate| *candidate >= byte)
        .unwrap_or(starts.len() - 1)
}

fn lookback_start_byte(input: &str, byte: usize, chars_back: usize) -> usize {
    let positions = input
        .char_indices()
        .map(|(position, _)| position)
        .take_while(|position| *position < byte)
        .collect::<Vec<_>>();
    if positions.len() <= chars_back {
        0
    } else {
        positions[positions.len() - chars_back]
    }
}

fn idiom_noop_end(input: &str, index: usize, starts: &[usize]) -> Option<usize> {
    let mut cursor = index;
    while cursor < starts.len() - 1 {
        let ch = input[starts[cursor]..].chars().next().unwrap();
        if ch == ' ' {
            cursor += 1;
        } else {
            break;
        }
    }
    if cursor >= starts.len() - 1 {
        return None;
    }
    let ch = input[starts[cursor]..].chars().next().unwrap();
    if is_numeric_start(ch) {
        Some(candidate_end(input, cursor, starts))
    } else {
        None
    }
}

fn can_start_candidate(input: &str, index: usize, starts: &[usize], ch: char) -> bool {
    if is_numeric_start(ch) {
        return true;
    }
    if ch.is_ascii_lowercase() {
        let mut cursor = index + 1;
        while cursor < starts.len() - 1 {
            let next = input[starts[cursor]..].chars().next().unwrap();
            if next == ' ' {
                cursor += 1;
                continue;
            }
            return is_numeric_start(next);
        }
    }
    false
}

fn candidate_end(input: &str, index: usize, starts: &[usize]) -> usize {
    let mut cursor = index;
    if cursor < starts.len() - 1 {
        let first = input[starts[cursor]..].chars().next().unwrap();
        if first.is_ascii_lowercase() {
            cursor += 1;
            while cursor < starts.len() - 1 {
                let ch = input[starts[cursor]..].chars().next().unwrap();
                if ch == ' ' {
                    cursor += 1;
                } else {
                    break;
                }
            }
        }
    }

    while cursor < starts.len() - 1 {
        let ch = input[starts[cursor]..].chars().next().unwrap();
        if is_numeric_body_char(ch) {
            cursor += 1;
        } else {
            break;
        }
    }

    cursor + suffix_char_len(input, cursor, starts)
}

fn suffix_char_len(input: &str, cursor: usize, starts: &[usize]) -> usize {
    if cursor >= starts.len() - 1 {
        return 0;
    }
    let prefix = &input[..starts[cursor]];
    let rest = &input[starts[cursor]..];
    if prefix.ends_with(['秒', '分', '年', '月', '日', '号']) {
        return 0;
    }
    if prefix.ends_with('千') {
        for suffix in ["米每小时", "米", "克"] {
            if rest.starts_with(suffix) {
                return suffix.chars().count();
            }
        }
    }
    for (unit, _) in sorted_units() {
        if rest.starts_with(unit) {
            return unit.chars().count();
        }
    }
    rest.chars()
        .take_while(|ch| ch.is_ascii_alphabetic())
        .count()
}

fn is_numeric_body_char(ch: char) -> bool {
    matches!(
        ch,
        '几' | '零'
            | '幺'
            | '一'
            | '二'
            | '两'
            | '三'
            | '四'
            | '五'
            | '六'
            | '七'
            | '八'
            | '九'
            | '十'
            | '百'
            | '千'
            | '万'
            | '亿'
            | '点'
            | '比'
            | '分'
            | '之'
            | '年'
            | '月'
            | '日'
            | '号'
            | '秒'
            | ' '
    )
}

fn is_numeric_start(ch: char) -> bool {
    matches!(
        ch,
        '几' | '零'
            | '幺'
            | '一'
            | '二'
            | '两'
            | '三'
            | '四'
            | '五'
            | '六'
            | '七'
            | '八'
            | '九'
            | '十'
            | '百'
            | '点'
    )
}

fn idiom_at(input: &str, byte: usize) -> Option<&'static str> {
    IDIOMS
        .iter()
        .copied()
        .find(|idiom| input[byte..].starts_with(idiom))
}

fn idiom_overlaps(input: &str, start: usize, end: usize) -> bool {
    IDIOMS.iter().copied().any(|idiom| {
        let mut search_from = 0;
        while let Some(offset) = input[search_from..].find(idiom) {
            let idiom_start = search_from + offset;
            let idiom_end = idiom_start + idiom.len();
            if idiom_start < end && idiom_end > start {
                return true;
            }
            search_from = idiom_start + idiom.len();
        }
        false
    })
}

fn digit_value(ch: char) -> Option<i64> {
    match ch {
        '零' => Some(0),
        '一' | '幺' => Some(1),
        '二' | '两' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

fn value_of(ch: char) -> Option<i64> {
    match ch {
        '零' => Some(0),
        '一' | '幺' => Some(1),
        '二' | '两' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        '十' => Some(10),
        '百' => Some(100),
        '千' => Some(1000),
        '万' => Some(10000),
        '亿' => Some(100000000),
        _ => None,
    }
}

fn arabic_digit(ch: char) -> Option<char> {
    match ch {
        '零' => Some('0'),
        '一' | '幺' => Some('1'),
        '二' | '两' => Some('2'),
        '三' => Some('3'),
        '四' => Some('4'),
        '五' => Some('5'),
        '六' => Some('6'),
        '七' => Some('7'),
        '八' => Some('8'),
        '九' => Some('9'),
        '点' => Some('.'),
        _ => None,
    }
}

fn strip_trailing_unit(text: &str) -> &str {
    if let Some((unit, _)) = unit_suffix(text) {
        return &text[..text.len() - unit.len()];
    }
    let trimmed = text.trim_end_matches(|ch: char| ch.is_ascii_alphabetic());
    if trimmed.len() != text.len() {
        trimmed
    } else {
        text
    }
}

fn strip_unit(original: &str) -> (&str, String) {
    if let Some((unit_cn, mapped)) = unit_suffix(original) {
        let stripped = &original[..original.len() - unit_cn.len()];
        let unit = mapped.unwrap_or(unit_cn).to_string();
        return (stripped.trim(), unit);
    }

    let mut split = original.len();
    for (byte, ch) in original.char_indices().rev() {
        if ch.is_ascii_alphabetic() {
            split = byte;
        } else {
            break;
        }
    }
    if split != original.len() {
        (&original[..split], original[split..].to_string())
    } else {
        (original, String::new())
    }
}

fn unit_suffix(text: &str) -> Option<(&'static str, Option<&'static str>)> {
    let mut units = UNITS.to_vec();
    units.sort_by_key(|(unit, _)| std::cmp::Reverse(unit.len()));
    units.into_iter().find(|(unit, _)| text.ends_with(unit))
}

fn convert_pure_num(original: &str, strict: bool) -> Option<String> {
    let (stripped, unit) = strip_unit(original);
    if stripped == "一" && !strict {
        return Some(original.to_string());
    }
    let mut converted = String::new();
    for ch in stripped.chars() {
        converted.push(arabic_digit(ch)?);
    }
    converted.push_str(&unit);
    Some(converted)
}

fn convert_value_num(original: &str) -> Option<String> {
    let (stripped, unit) = strip_unit(original);
    let normalized;
    let stripped = if stripped.contains('点') {
        stripped
    } else {
        normalized = format!("{stripped}点");
        &normalized
    };

    let (int_part, decimal_part) = stripped.split_once('点')?;
    if int_part.is_empty() {
        return Some(original.to_string());
    }

    let mut value = 0_i64;
    let mut temp = 0_i64;
    let mut base = 1_i64;
    for ch in int_part.chars() {
        match ch {
            '十' => {
                temp = if temp == 0 { 10 } else { value_of(ch)? * temp };
                base = 1;
            }
            '零' => base = 1,
            '一' | '二' | '两' | '三' | '四' | '五' | '六' | '七' | '八' | '九' => {
                temp += value_of(ch)?;
            }
            '万' => {
                value += temp;
                value *= value_of(ch)?;
                base = value_of(ch)? / 10;
                temp = 0;
            }
            '百' | '千' => {
                value += temp * value_of(ch)?;
                base = value_of(ch)? / 10;
                temp = 0;
            }
            _ => return None,
        }
    }
    value += temp * base;

    let mut final_text = value.to_string();
    let decimal = convert_pure_num(decimal_part, true)?;
    if !decimal.is_empty() {
        final_text.push('.');
        final_text.push_str(&decimal);
    }
    final_text.push_str(&unit);
    Some(final_text)
}

fn convert_fraction_value(original: &str) -> Option<String> {
    let (denominator, numerator) = original.split_once("分之")?;
    Some(format!(
        "{}/{}",
        convert_value_num(numerator)?,
        convert_value_num(denominator)?
    ))
}

fn convert_percent_value(original: &str) -> Option<String> {
    Some(format!(
        "{}%",
        convert_value_num(&original["百分之".len()..])?
    ))
}

fn convert_ratio_value(original: &str) -> Option<String> {
    let (left, right) = original.split_once('比')?;
    Some(format!(
        "{}:{}",
        convert_value_num(left)?,
        convert_value_num(right)?
    ))
}

fn convert_time_value(original: &str) -> Option<String> {
    let parts = original
        .split(['点', '分', '秒'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let mut final_text = zfill(&convert_value_num(parts[0])?, 2);
    final_text.push(':');
    final_text.push_str(&zfill(&convert_value_num(parts[1])?, 2));
    if parts.len() > 2 {
        final_text.push(':');
        final_text.push_str(&zfill(&convert_value_num(parts[2])?, 2));
    }
    if parts.len() > 3 {
        final_text.push('.');
        final_text.push_str(&convert_pure_num(parts[3], false)?);
    }
    Some(final_text)
}

fn zfill(value: &str, width: usize) -> String {
    if value.len() >= width {
        value.to_string()
    } else {
        format!("{}{}", "0".repeat(width - value.len()), value)
    }
}

fn convert_date_value(original: &str) -> Option<String> {
    let mut rest = original;
    let mut final_text = String::new();
    if let Some((year, tail)) = rest.split_once('年') {
        final_text.push_str(&convert_pure_num(year, false)?);
        final_text.push('年');
        rest = tail;
    }
    if let Some((month, tail)) = rest.split_once('月') {
        final_text.push_str(&convert_value_num(month)?);
        final_text.push('月');
        rest = tail;
    }
    if let Some((day, _)) = rest.split_once('日') {
        final_text.push_str(&convert_value_num(day)?);
        final_text.push('日');
    } else if let Some((day, _)) = rest.split_once('号') {
        final_text.push_str(&convert_value_num(day)?);
        final_text.push('号');
    }
    if final_text.is_empty() {
        None
    } else {
        Some(final_text)
    }
}

fn is_pure_num(text: &str) -> bool {
    let stripped = text.trim_end();
    if stripped.is_empty() {
        return false;
    }
    for segment in stripped.split('点') {
        if segment.is_empty() {
            return false;
        }
        if !segment.chars().all(|ch| {
            matches!(
                ch,
                '零' | '幺' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
            )
        }) {
            return false;
        }
    }
    true
}

fn is_value_num(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }
    if let Some((integer, decimal)) = text.split_once('点') {
        if integer.is_empty() || decimal.is_empty() || decimal.contains('点') {
            return false;
        }
        if !decimal.chars().all(|ch| {
            matches!(
                ch,
                '零' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
            )
        }) {
            return false;
        }
    }
    let mut seen = false;
    let mut dot_count = 0;
    for ch in text.chars() {
        if matches!(
            ch,
            '零' | '一'
                | '二'
                | '两'
                | '三'
                | '四'
                | '五'
                | '六'
                | '七'
                | '八'
                | '九'
                | '十'
                | '百'
                | '千'
                | '万'
        ) {
            seen = true;
        } else if ch == '点' {
            dot_count += 1;
            if dot_count > 1 {
                return false;
            }
        } else {
            return false;
        }
    }
    seen
}

fn is_time_value(text: &str) -> bool {
    text.contains('点') && text.contains('分') && text.ends_with(['分', '秒'])
}

fn is_date_value(text: &str) -> bool {
    (text.contains('年') || text.contains('月') || text.contains('日') || text.contains('号'))
        && text.chars().all(|ch| {
            matches!(
                ch,
                '零' | '幺'
                    | '一'
                    | '二'
                    | '两'
                    | '三'
                    | '四'
                    | '五'
                    | '六'
                    | '七'
                    | '八'
                    | '九'
                    | '十'
                    | '年'
                    | '月'
                    | '日'
                    | '号'
            )
        })
}

fn is_consecutive_value(text: &str) -> bool {
    let (stripped, _) = strip_one_unit_char(text);
    is_repeated_tens(stripped) || is_repeated_hundreds(stripped)
}

fn split_consecutive_value(text: &str) -> Option<String> {
    let (stripped, unit) = strip_one_unit_char(text);
    if is_repeated_tens(stripped) {
        let chars = stripped.chars().collect::<Vec<_>>();
        let mut values = Vec::new();
        for chunk in chars.chunks(2) {
            let phrase = chunk.iter().collect::<String>();
            values.push(convert_value_num(&phrase)?);
        }
        return Some(format!("{}{}", values.join(" "), unit));
    }
    if is_repeated_hundreds(stripped) {
        let chars = stripped.chars().collect::<Vec<_>>();
        let mut values = Vec::new();
        let mut cursor = 0;
        while cursor + 2 < chars.len() {
            let mut end = cursor + 3;
            if chars.get(cursor + 2) == Some(&'零') {
                end = cursor + 4;
            }
            let phrase = chars[cursor..end].iter().collect::<String>();
            values.push(convert_value_num(&phrase)?);
            cursor = end;
        }
        return Some(format!("{}{}", values.join(" "), unit));
    }
    Some(format!("{stripped}{unit}"))
}

fn strip_one_unit_char(text: &str) -> (&str, String) {
    if let Some((byte, ch)) = text.char_indices().last()
        && unit_chars().contains(ch)
    {
        return (&text[..byte], ch.to_string());
    }
    (text, String::new())
}

fn unit_chars() -> &'static str {
    "千米每小时|千克|个|只|分|万|亿|秒|年|月|日|天|时|钟|人|层|楼|倍|块|次|克|米|千米"
}

fn is_repeated_tens(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    !chars.is_empty()
        && chars.len() % 2 == 0
        && chars.chunks(2).all(|chunk| {
            chunk[0] == '十'
                && matches!(
                    chunk[1],
                    '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
                )
        })
}

fn is_repeated_hundreds(text: &str) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    let mut cursor = 0;
    let mut count = 0;
    while cursor < chars.len() {
        if cursor + 2 >= chars.len()
            || !matches!(
                chars[cursor],
                '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
            )
            || chars[cursor + 1] != '百'
        {
            return false;
        }
        if chars[cursor + 2] == '零' {
            if cursor + 3 >= chars.len()
                || !matches!(
                    chars[cursor + 3],
                    '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
                )
            {
                return false;
            }
            cursor += 4;
        } else if matches!(
            chars[cursor + 2],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        ) {
            cursor += 3;
        } else {
            return false;
        }
        count += 1;
    }
    count > 0
}

fn is_range_expression(text: &str) -> bool {
    if text.contains('点') {
        return false;
    }
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() >= 3
        && matches!(
            chars[0],
            '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(
            chars[1],
            '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(chars[2], '十' | '百' | '千' | '万' | '亿')
    {
        return true;
    }
    if chars.len() >= 3
        && chars[0] == '十'
        && matches!(
            chars[1],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(
            chars[2],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
    {
        return true;
    }
    chars.len() >= 5
        && matches!(
            chars[0],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(chars[1], '百' | '千')
        && matches!(
            chars[2],
            '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(
            chars[3],
            '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && chars[4] == '十'
        || is_large_place_two_digit_range(text)
}

fn is_large_place_two_digit_range(text: &str) -> bool {
    let core = strip_range_optional_unit(text);
    let chars = core.chars().collect::<Vec<_>>();
    if chars.len() < 4 {
        return false;
    }
    let prefix_len = chars.len() - 2;
    matches!(chars[prefix_len - 1], '万' | '千' | '百')
        && chars[..prefix_len].iter().all(|ch| {
            matches!(
                ch,
                '一' | '二'
                    | '三'
                    | '四'
                    | '五'
                    | '六'
                    | '七'
                    | '八'
                    | '九'
                    | '十'
                    | '百'
                    | '千'
                    | '万'
            )
        })
        && matches!(
            chars[prefix_len],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(
            chars[prefix_len + 1],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
}

fn strip_range_optional_unit(text: &str) -> &str {
    for (unit_cn, _) in sorted_units() {
        if matches!(unit_cn, "万" | "亿" | "千" | "百" | "十") {
            continue;
        }
        if let Some(stripped) = text.strip_suffix(unit_cn) {
            return stripped;
        }
    }
    text
}

fn convert_range_expression(text: &str) -> Option<String> {
    let mut stripped = text;
    let mut mapped_unit = String::new();
    for (unit_cn, mapped) in sorted_units() {
        if matches!(unit_cn, "万" | "亿" | "千" | "百" | "十") {
            continue;
        }
        if let Some(without_unit) = text.strip_suffix(unit_cn) {
            stripped = without_unit;
            mapped_unit = mapped.unwrap_or(unit_cn).to_string();
            break;
        }
    }

    if let Some(converted) = convert_range_pattern_2(stripped) {
        return Some(format!("{converted}{mapped_unit}"));
    }
    if let Some(converted) = convert_range_pattern_1(stripped) {
        return Some(format!("{converted}{mapped_unit}"));
    }
    if let Some(converted) = convert_range_pattern_3(stripped) {
        return Some(format!("{converted}{mapped_unit}"));
    }
    Some(text.to_string())
}

fn sorted_units() -> Vec<(&'static str, Option<&'static str>)> {
    let mut units = UNITS.to_vec();
    units.sort_by_key(|(unit, _)| std::cmp::Reverse(unit.len()));
    units
}

fn convert_range_pattern_1(text: &str) -> Option<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() < 3 {
        return None;
    }
    let d1 = chars[0];
    let d2 = chars[1];
    let unit = chars[2];
    if !matches!(d1, '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九')
        || !matches!(d2, '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九')
        || !matches!(unit, '十' | '百' | '千' | '万' | '亿')
    {
        return None;
    }
    let suffix = chars.get(3).copied().unwrap_or('\0');
    let suffix_text = if matches!(suffix, '万' | '千' | '百' | '亿') {
        suffix.to_string()
    } else {
        String::new()
    };
    let v1 = digit_value(d1)?;
    let v2 = digit_value(d2)?;
    if unit == '十' {
        Some(format!("{}~{}{}", v1 * 10, v2 * 10, suffix_text))
    } else if matches!(unit, '万' | '亿') || (unit == '千' && !suffix_text.is_empty()) {
        Some(format!("{v1}~{v2}{unit}{suffix_text}"))
    } else {
        Some(format!(
            "{}~{}{}",
            v1 * value_of(unit)?,
            v2 * value_of(unit)?,
            suffix_text
        ))
    }
}

fn convert_range_pattern_2(text: &str) -> Option<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() < 3 {
        return None;
    }
    for base_len in (1..=(chars.len().saturating_sub(2))).rev() {
        let base = &chars[..base_len];
        let d1 = chars[base_len];
        let d2 = chars[base_len + 1];
        if !matches!(
            d1,
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        ) || !matches!(
            d2,
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        ) {
            continue;
        }
        if !(base == ['十']
            || base
                .last()
                .is_some_and(|ch| matches!(ch, '十' | '百' | '千' | '万')))
        {
            continue;
        }
        if !base.iter().all(|ch| {
            matches!(
                ch,
                '一' | '二'
                    | '三'
                    | '四'
                    | '五'
                    | '六'
                    | '七'
                    | '八'
                    | '九'
                    | '十'
                    | '百'
                    | '千'
                    | '万'
            )
        }) {
            continue;
        }
        let unit = chars.get(base_len + 2).copied().unwrap_or('\0');
        let unit_text = if matches!(unit, '万' | '千' | '亿') {
            unit.to_string()
        } else {
            String::new()
        };
        let last = *base.last()?;
        let base_value = if last == '十' {
            if base.len() == 1 {
                10
            } else {
                range_leading_value(base[0])? * 10
            }
        } else if value_of(last).is_some() {
            if base.len() > 1 {
                range_leading_value(base[0])? * value_of(last)?
            } else {
                value_of(last)?
            }
        } else {
            return None;
        };
        let multiplier = value_of(last).unwrap_or(10) / 10;
        return Some(format!(
            "{}~{}{}",
            base_value + digit_value(d1)? * multiplier,
            base_value + digit_value(d2)? * multiplier,
            unit_text
        ));
    }
    None
}

fn range_leading_value(ch: char) -> Option<i64> {
    if ch == '十' {
        Some(10)
    } else {
        digit_value(ch)
    }
}

fn convert_range_pattern_3(text: &str) -> Option<String> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() == 2
        && matches!(
            chars[0],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
        && matches!(
            chars[1],
            '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
        )
    {
        Some(format!(
            "{}~{}",
            digit_value(chars[0])?,
            digit_value(chars[1])?
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    const FIXTURES: &str = include_str!("../../../../fixtures/asr_chinese_itn_core.jsonl");

    #[test]
    fn asr_chinese_itn_core_fixture_parity() {
        for (index, line) in FIXTURES.lines().enumerate() {
            let case: Value = serde_json::from_str(line).unwrap();
            let input = case["input"].as_str().unwrap();
            let expected = case["expected"].as_str().unwrap();
            assert_eq!(
                chinese_to_num(input),
                expected,
                "case {} {}",
                index + 1,
                case["category"].as_str().unwrap()
            );
        }
    }
}
