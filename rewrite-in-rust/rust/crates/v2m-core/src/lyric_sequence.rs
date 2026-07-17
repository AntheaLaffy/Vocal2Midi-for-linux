//! Lyric sequence alignment compatibility helpers.
//!
//! This module mirrors the deterministic behavior in
//! `inference/LyricFA/tools/sequence_aligner.py`. Python remains the runtime
//! owner for language processors, G2P dictionaries, file IO, lab/JSON
//! persistence, model execution, GUI/Web/CLI callers, and production routing.

use std::collections::HashMap;

const OVERLAP_THRESHOLD: f64 = 0.3;
const GAP: &str = "-";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditOperation {
    Match,
    Substitute,
    Delete,
    Insert,
}

/// Edit-cost configuration used by legacy `SequenceAligner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceAligner {
    pub deletion_cost: i64,
    pub insertion_cost: i64,
    pub substitution_cost: i64,
}

impl Default for SequenceAligner {
    fn default() -> Self {
        Self {
            deletion_cost: 1,
            insertion_cost: 1,
            substitution_cost: 1,
        }
    }
}

impl SequenceAligner {
    /// Creates an aligner with caller-provided edit costs.
    pub const fn new(deletion_cost: i64, insertion_cost: i64, substitution_cost: i64) -> Self {
        Self {
            deletion_cost,
            insertion_cost,
            substitution_cost,
        }
    }

    /// Computes edit distance and aligned token lists.
    pub fn compute_alignment(&self, seq1: &[String], seq2: &[String]) -> AlignmentResult {
        let len1 = seq1.len();
        let len2 = seq2.len();
        let mut dp = vec![vec![0_i64; len2 + 1]; len1 + 1];
        let mut bt = vec![vec![EditOperation::Match; len2 + 1]; len1 + 1];

        for i in 1..=len1 {
            dp[i][0] = i as i64 * self.deletion_cost;
            bt[i][0] = EditOperation::Delete;
        }
        for j in 1..=len2 {
            dp[0][j] = j as i64 * self.insertion_cost;
            bt[0][j] = EditOperation::Insert;
        }

        for i in 1..=len1 {
            let s1_char = &seq1[i - 1];
            for j in 1..=len2 {
                let s2_char = &seq2[j - 1];
                if s1_char == s2_char {
                    dp[i][j] = dp[i - 1][j - 1];
                    bt[i][j] = EditOperation::Match;
                } else {
                    let sub_cost = dp[i - 1][j - 1] + self.substitution_cost;
                    let del_cost = dp[i - 1][j] + self.deletion_cost;
                    let ins_cost = dp[i][j - 1] + self.insertion_cost;

                    let mut min_cost = sub_cost;
                    let mut op = EditOperation::Substitute;
                    if del_cost < min_cost {
                        min_cost = del_cost;
                        op = EditOperation::Delete;
                    }
                    if ins_cost < min_cost {
                        min_cost = ins_cost;
                        op = EditOperation::Insert;
                    }

                    dp[i][j] = min_cost;
                    bt[i][j] = op;
                }
            }
        }

        let (aligned1, aligned2) = backtrack(seq1, seq2, &bt);
        AlignmentResult {
            distance: dp[len1][len2],
            aligned1,
            aligned2,
        }
    }

    /// Computes legacy LCS length using the memory-swapping DP.
    pub fn compute_lcs_length(seq1: &[String], seq2: &[String]) -> usize {
        let (seq1, seq2) = if seq1.len() < seq2.len() {
            (seq2, seq1)
        } else {
            (seq1, seq2)
        };
        let m = seq1.len();
        let n = seq2.len();
        let mut prev = vec![0_usize; n + 1];
        let mut curr = vec![0_usize; n + 1];

        for i in 1..=m {
            let c1 = &seq1[i - 1];
            for j in 1..=n {
                if c1 == &seq2[j - 1] {
                    curr[j] = prev[j - 1] + 1;
                } else {
                    curr[j] = prev[j].max(curr[j - 1]);
                }
            }
            std::mem::swap(&mut prev, &mut curr);
        }

        prev[n]
    }

    /// Finds the best matching reference window.
    pub fn find_best_match(
        &self,
        input_seq: &[String],
        reference_seq: &[String],
        reference_text: Option<&[String]>,
        max_window_scale: f64,
        extra_window: usize,
    ) -> MatchResult {
        if input_seq.is_empty() {
            return MatchResult::failure("Input sequence is empty");
        }
        if reference_seq.is_empty() {
            return MatchResult::failure("Reference sequence is empty");
        }

        let input_len = input_seq.len();
        let ref_len = reference_seq.len();
        let long_input_note = if input_len > ref_len {
            "Input longer than reference; attempted approximate alignment"
        } else {
            ""
        };

        let direct_start = find_exact_match(input_seq, reference_seq);
        if let Some(start) = direct_start {
            return build_exact_match_result(start, input_len, reference_seq, reference_text);
        }

        let window_size = determine_window_size(input_len, ref_len, max_window_scale, extra_window);
        let scan_result = self.scan_windows(input_seq, reference_seq, window_size, input_len);
        if scan_result.best_start < 0 {
            let reason = if long_input_note.is_empty() {
                "No matching window found".to_string()
            } else {
                format!("{long_input_note}; no matching window found")
            };
            return MatchResult::failure(reason);
        }

        let mut result = self.build_match_from_alignment(
            input_seq,
            reference_seq,
            reference_text,
            scan_result.best_start as usize,
            window_size,
        );

        if !long_input_note.is_empty() {
            result.reason = if result.reason.is_empty() {
                long_input_note.to_string()
            } else {
                format!("{long_input_note}; {}", result.reason)
            };
        }

        result
    }

    /// Computes edit distance only, with the legacy shorter-sequence swap.
    pub fn compute_edit_distance(&self, seq1: &[String], seq2: &[String]) -> i64 {
        let (seq1, seq2) = if seq1.len() < seq2.len() {
            (seq2, seq1)
        } else {
            (seq1, seq2)
        };
        let len1 = seq1.len();
        let len2 = seq2.len();

        let mut prev = (0..=len2)
            .map(|j| j as i64 * self.insertion_cost)
            .collect::<Vec<_>>();
        let mut curr = vec![0_i64; len2 + 1];

        for i in 1..=len1 {
            curr[0] = i as i64 * self.deletion_cost;
            let s1 = &seq1[i - 1];
            for j in 1..=len2 {
                let s2 = &seq2[j - 1];
                if s1 == s2 {
                    curr[j] = prev[j - 1];
                } else {
                    let sub = prev[j - 1] + self.substitution_cost;
                    let dele = prev[j] + self.deletion_cost;
                    let ins = curr[j - 1] + self.insertion_cost;
                    curr[j] = sub.min(dele).min(ins);
                }
            }
            std::mem::swap(&mut prev, &mut curr);
        }

        prev[len2]
    }

    /// Scans candidate windows using overlap, LCS approximation, and exact edit
    /// distance over the retained candidate set.
    pub fn scan_windows(
        &self,
        input_seq: &[String],
        reference_seq: &[String],
        window_size: usize,
        input_len: usize,
    ) -> ScanResult {
        if input_len == 0 || window_size > reference_seq.len() {
            return ScanResult::none();
        }

        let input_freq = counter(input_seq);
        let mut candidates = Vec::new();

        for start in 0..=(reference_seq.len() - window_size) {
            let window = &reference_seq[start..start + window_size];
            if !window.iter().any(|token| input_freq.contains_key(token)) {
                continue;
            }

            let window_freq = counter(window);
            let overlap = input_freq
                .iter()
                .map(|(token, count)| (*count).min(*window_freq.get(token).unwrap_or(&0)))
                .sum::<usize>();
            let coverage = overlap as f64 / input_len as f64;
            if coverage < OVERLAP_THRESHOLD {
                continue;
            }

            let lcs_len = Self::compute_lcs_length(input_seq, window);
            let approx_dist = input_seq.len() + window.len() - 2 * lcs_len;
            candidates.push((approx_dist, start));
        }

        if candidates.is_empty() {
            return ScanResult::none();
        }

        candidates.sort_by_key(|(approx_dist, _)| *approx_dist);

        let total = candidates.len();
        let num_to_keep = (10_usize.max((0.3 * total as f64) as usize)).min(total);
        let mut best_start = -1_isize;
        let mut min_edit_dist: Option<i64> = None;

        for (_, start) in candidates.into_iter().take(num_to_keep) {
            let window = &reference_seq[start..start + window_size];
            let edit_dist = self.compute_edit_distance(input_seq, window);
            if min_edit_dist.is_none_or(|current| edit_dist < current) {
                min_edit_dist = Some(edit_dist);
                best_start = start as isize;
                if edit_dist == 0 {
                    break;
                }
            }
        }

        ScanResult {
            best_start,
            min_edit_dist,
        }
    }

    /// Builds a public match result from a selected reference window.
    pub fn build_match_from_alignment(
        &self,
        input_seq: &[String],
        reference_seq: &[String],
        reference_text: Option<&[String]>,
        best_start: usize,
        window_size: usize,
    ) -> MatchResult {
        let window_end = best_start.saturating_add(window_size);
        let window_seq = slice_tokens(reference_seq, best_start, window_end);
        let alignment = self.compute_alignment(input_seq, &window_seq);

        let mut matched_phonetic_list = Vec::new();
        let mut matched_text_list = Vec::new();
        let mut win_idx = 0_usize;
        let text_window_len = reference_text
            .map(|text| slice_tokens(text, best_start, window_end).len())
            .unwrap_or(0);

        for (win_char, inp_char) in alignment.aligned2.iter().zip(&alignment.aligned1) {
            if win_char != GAP {
                if inp_char != GAP {
                    matched_phonetic_list.push(win_char.clone());
                    if let Some(text) = reference_text {
                        let text_index = best_start.saturating_add(win_idx);
                        if win_idx < text_window_len && text_index < text.len() {
                            matched_text_list.push(text[text_index].clone());
                        }
                    }
                }
                win_idx += 1;
            }
        }

        if matched_phonetic_list.is_empty() {
            return MatchResult::failure("Alignment produced empty result");
        }

        let matched_text = matched_text_list.join(" ");
        MatchResult {
            matched_text,
            start: best_start as isize,
            end: window_end as isize,
            matched_phonetic_list: Some(matched_phonetic_list),
            matched_text_list: Some(matched_text_list),
            reason: String::new(),
        }
    }

    /// Finds the best match and returns the legacy wrapper tuple shape.
    pub fn find_best_match_and_return_lyrics(
        &self,
        input_pronunciation: &[String],
        reference_text: &[String],
        reference_pronunciation: &[String],
    ) -> LyricMatchResult {
        let result = self.find_best_match(
            input_pronunciation,
            reference_pronunciation,
            Some(reference_text),
            1.3,
            8,
        );
        let matched_phonetic = result
            .matched_phonetic_list
            .as_deref()
            .map(|tokens| tokens.join(" "))
            .unwrap_or_default();
        LyricMatchResult {
            matched_text: result.matched_text,
            matched_phonetic,
            start: result.start,
            end: result.end,
            reason: result.reason,
        }
    }
}

/// Result from `compute_alignment`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentResult {
    pub distance: i64,
    pub aligned1: Vec<String>,
    pub aligned2: Vec<String>,
}

/// Result from `find_best_match`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    pub matched_text: String,
    pub start: isize,
    pub end: isize,
    pub matched_phonetic_list: Option<Vec<String>>,
    pub matched_text_list: Option<Vec<String>>,
    pub reason: String,
}

impl MatchResult {
    fn failure(reason: impl Into<String>) -> Self {
        Self {
            matched_text: String::new(),
            start: -1,
            end: -1,
            matched_phonetic_list: None,
            matched_text_list: None,
            reason: reason.into(),
        }
    }
}

/// Result from the lyric-wrapper helper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricMatchResult {
    pub matched_text: String,
    pub matched_phonetic: String,
    pub start: isize,
    pub end: isize,
    pub reason: String,
}

/// Result from scan-window helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanResult {
    pub best_start: isize,
    pub min_edit_dist: Option<i64>,
}

impl ScanResult {
    const fn none() -> Self {
        Self {
            best_start: -1,
            min_edit_dist: None,
        }
    }
}

/// Result from `SmartHighlighter.highlight_differences`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightResult {
    pub asr_highlighted: String,
    pub phonetic_highlighted: String,
    pub text_highlighted: String,
    pub edit_distance: i64,
}

/// Counts positional differences plus trailing length difference.
pub fn calculate_difference_count(seq1: &[String], seq2: &[String]) -> usize {
    let min_len = seq1.len().min(seq2.len());
    let mut diff = seq1
        .iter()
        .take(min_len)
        .zip(seq2.iter().take(min_len))
        .filter(|(left, right)| left != right)
        .count();
    diff += seq1.len().abs_diff(seq2.len());
    diff
}

/// Highlights ASR/matched-token differences with legacy parenthesized output.
pub fn highlight_differences(
    aligner: &SequenceAligner,
    asr_result: &str,
    match_phonetic: &str,
    match_text: &str,
) -> HighlightResult {
    let asr_tokens = split_tokens(asr_result);
    let match_phonetic_tokens = split_tokens(match_phonetic);
    let match_text_tokens = split_tokens(match_text);

    if match_phonetic_tokens.is_empty() {
        return HighlightResult {
            asr_highlighted: asr_tokens
                .iter()
                .map(|token| format!("({token})"))
                .collect::<Vec<_>>()
                .join(" "),
            phonetic_highlighted: String::new(),
            text_highlighted: String::new(),
            edit_distance: asr_tokens.len() as i64,
        };
    }

    let alignment = aligner.compute_alignment(&asr_tokens, &match_phonetic_tokens);
    let mut asr_highlighted = Vec::new();
    let mut phonetic_highlighted = Vec::new();
    let mut text_highlighted = Vec::new();
    let mut text_index = 0_usize;
    let text_tokens_len = match_text_tokens.len();

    for (asr_token, match_token) in alignment.aligned1.iter().zip(&alignment.aligned2) {
        let mut text_token = "";
        if match_token != GAP && text_index < text_tokens_len {
            text_token = &match_text_tokens[text_index];
            text_index += 1;
        }

        if asr_token == GAP {
            phonetic_highlighted.push(format!("({match_token})"));
            asr_highlighted.push(String::new());
            if !text_token.is_empty() {
                text_highlighted.push(format!("({text_token})"));
            }
        } else if match_token == GAP {
            asr_highlighted.push(format!("({asr_token})"));
            phonetic_highlighted.push(String::new());
            text_highlighted.push(String::new());
        } else if asr_token != match_token {
            asr_highlighted.push(format!("({asr_token})"));
            phonetic_highlighted.push(format!("({match_token})"));
            if !text_token.is_empty() {
                text_highlighted.push(format!("({text_token})"));
            }
        } else {
            asr_highlighted.push(asr_token.clone());
            phonetic_highlighted.push(match_token.clone());
            if !text_token.is_empty() {
                text_highlighted.push(text_token.to_string());
            }
        }
    }

    HighlightResult {
        asr_highlighted: join_non_empty(&asr_highlighted),
        phonetic_highlighted: join_non_empty(&phonetic_highlighted),
        text_highlighted: join_non_empty(&text_highlighted),
        edit_distance: alignment.distance,
    }
}

fn backtrack(
    seq1: &[String],
    seq2: &[String],
    bt: &[Vec<EditOperation>],
) -> (Vec<String>, Vec<String>) {
    let mut i = seq1.len();
    let mut j = seq2.len();
    let max_len = i + j;
    let mut res1 = vec![None; max_len];
    let mut res2 = vec![None; max_len];
    let mut idx = max_len;

    while i > 0 || j > 0 {
        idx -= 1;
        if i > 0 && j > 0 {
            let op = bt[i][j];
            if matches!(op, EditOperation::Match | EditOperation::Substitute) {
                res1[idx] = Some(seq1[i - 1].clone());
                res2[idx] = Some(seq2[j - 1].clone());
                i -= 1;
                j -= 1;
            } else if op == EditOperation::Delete {
                res1[idx] = Some(seq1[i - 1].clone());
                res2[idx] = Some(GAP.to_string());
                i -= 1;
            } else {
                res1[idx] = Some(GAP.to_string());
                res2[idx] = Some(seq2[j - 1].clone());
                j -= 1;
            }
        } else if i > 0 {
            res1[idx] = Some(seq1[i - 1].clone());
            res2[idx] = Some(GAP.to_string());
            i -= 1;
        } else {
            res1[idx] = Some(GAP.to_string());
            res2[idx] = Some(seq2[j - 1].clone());
            j -= 1;
        }
    }

    let aligned1 = res1[idx..].iter().flatten().cloned().collect();
    let aligned2 = res2[idx..].iter().flatten().cloned().collect();
    (aligned1, aligned2)
}

fn find_exact_match(input_seq: &[String], reference_seq: &[String]) -> Option<usize> {
    let input_len = input_seq.len();
    let ref_len = reference_seq.len();
    if input_len > ref_len {
        return None;
    }
    (0..=(ref_len - input_len)).find(|&start| &reference_seq[start..start + input_len] == input_seq)
}

fn build_exact_match_result(
    start: usize,
    length: usize,
    reference_seq: &[String],
    reference_text: Option<&[String]>,
) -> MatchResult {
    let end = start + length;
    let matched_phonetic_list = slice_tokens(reference_seq, start, end);
    let matched_text_list = reference_text
        .map(|text| slice_tokens(text, start, end))
        .unwrap_or_default();
    let matched_text = matched_text_list.join(" ");
    MatchResult {
        matched_text,
        start: start as isize,
        end: end as isize,
        matched_phonetic_list: Some(matched_phonetic_list),
        matched_text_list: Some(matched_text_list),
        reason: String::new(),
    }
}

fn determine_window_size(
    input_len: usize,
    ref_len: usize,
    max_window_scale: f64,
    extra_window: usize,
) -> usize {
    let scaled = (input_len as f64 * max_window_scale) as usize;
    input_len
        .saturating_add(extra_window)
        .min(scaled)
        .min(ref_len)
}

fn counter(tokens: &[String]) -> HashMap<&String, usize> {
    let mut counts = HashMap::new();
    for token in tokens {
        *counts.entry(token).or_insert(0) += 1;
    }
    counts
}

fn slice_tokens(tokens: &[String], start: usize, end: usize) -> Vec<String> {
    if start >= tokens.len() {
        return Vec::new();
    }
    tokens[start..end.min(tokens.len())].to_vec()
}

fn split_tokens(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_string).collect()
}

fn join_non_empty(values: &[String]) -> String {
    values
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const FIXTURES: &str = include_str!("../../../../fixtures/lyric_sequence_alignment_core.jsonl");

    fn parse_string_vec(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn parse_optional_string_vec(value: Option<&Value>) -> Option<Vec<String>> {
        value.and_then(|item| {
            if item.is_null() {
                None
            } else {
                Some(parse_string_vec(item))
            }
        })
    }

    fn make_aligner(case: &Value) -> SequenceAligner {
        let costs = case.get("costs");
        SequenceAligner {
            deletion_cost: costs
                .and_then(|value| value.get("deletion"))
                .and_then(Value::as_i64)
                .unwrap_or(1),
            insertion_cost: costs
                .and_then(|value| value.get("insertion"))
                .and_then(Value::as_i64)
                .unwrap_or(1),
            substitution_cost: costs
                .and_then(|value| value.get("substitution"))
                .and_then(Value::as_i64)
                .unwrap_or(1),
        }
    }

    fn encode_match_result(result: MatchResult) -> Value {
        json!({
            "matched_text": result.matched_text,
            "start": result.start,
            "end": result.end,
            "matched_phonetic_list": result.matched_phonetic_list,
            "matched_text_list": result.matched_text_list,
            "reason": result.reason,
        })
    }

    fn assert_json_close(actual: &Value, expected: &Value, context: &str) {
        match (actual, expected) {
            (Value::Number(left), Value::Number(right)) => {
                if left.is_f64() || right.is_f64() {
                    let left = left.as_f64().unwrap();
                    let right = right.as_f64().unwrap();
                    assert!(
                        (left - right).abs() <= 1e-6,
                        "{context}: {left:?} != {right:?}"
                    );
                } else {
                    assert_eq!(left, right, "{context}");
                }
            }
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
    fn lyric_sequence_alignment_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let case: Value = serde_json::from_str(line).unwrap();
            let aligner = make_aligner(&case);
            let actual = match case["kind"].as_str().unwrap() {
                "alignment" => {
                    let result = aligner.compute_alignment(
                        &parse_string_vec(&case["seq1"]),
                        &parse_string_vec(&case["seq2"]),
                    );
                    json!({
                        "distance": result.distance,
                        "aligned1": result.aligned1,
                        "aligned2": result.aligned2,
                    })
                }
                "find_best_match" => {
                    let reference_text = parse_optional_string_vec(case.get("reference_text"));
                    encode_match_result(
                        aligner.find_best_match(
                            &parse_string_vec(&case["input_seq"]),
                            &parse_string_vec(&case["reference_seq"]),
                            reference_text.as_deref(),
                            case.get("max_window_scale")
                                .and_then(Value::as_f64)
                                .unwrap_or(1.3),
                            case.get("extra_window")
                                .and_then(Value::as_u64)
                                .map_or(8, |value| value as usize),
                        ),
                    )
                }
                "scan_windows" => {
                    let result = aligner.scan_windows(
                        &parse_string_vec(&case["input_seq"]),
                        &parse_string_vec(&case["reference_seq"]),
                        case["window_size"].as_u64().unwrap() as usize,
                        case["input_len"].as_u64().unwrap() as usize,
                    );
                    json!({
                        "best_start": result.best_start,
                        "min_edit_dist": result.min_edit_dist,
                    })
                }
                "build_match" => {
                    let reference_text = parse_optional_string_vec(case.get("reference_text"));
                    encode_match_result(aligner.build_match_from_alignment(
                        &parse_string_vec(&case["input_seq"]),
                        &parse_string_vec(&case["reference_seq"]),
                        reference_text.as_deref(),
                        case["best_start"].as_u64().unwrap() as usize,
                        case["window_size"].as_u64().unwrap() as usize,
                    ))
                }
                "return_lyrics" => {
                    let result = aligner.find_best_match_and_return_lyrics(
                        &parse_string_vec(&case["input_pronunciation"]),
                        &parse_string_vec(&case["reference_text"]),
                        &parse_string_vec(&case["reference_pronunciation"]),
                    );
                    json!({
                        "matched_text": result.matched_text,
                        "matched_phonetic": result.matched_phonetic,
                        "start": result.start,
                        "end": result.end,
                        "reason": result.reason,
                    })
                }
                "lcs" => {
                    json!({
                        "length": SequenceAligner::compute_lcs_length(
                            &parse_string_vec(&case["seq1"]),
                            &parse_string_vec(&case["seq2"]),
                        ),
                    })
                }
                "edit_distance" => {
                    json!({
                        "distance": aligner.compute_edit_distance(
                            &parse_string_vec(&case["seq1"]),
                            &parse_string_vec(&case["seq2"]),
                        ),
                    })
                }
                "difference_count" => {
                    json!({
                        "count": calculate_difference_count(
                            &parse_string_vec(&case["seq1"]),
                            &parse_string_vec(&case["seq2"]),
                        ),
                    })
                }
                "highlight" => {
                    let result = highlight_differences(
                        &aligner,
                        case["asr_result"].as_str().unwrap(),
                        case["match_phonetic"].as_str().unwrap(),
                        case["match_text"].as_str().unwrap(),
                    );
                    json!({
                        "asr_highlighted": result.asr_highlighted,
                        "phonetic_highlighted": result.phonetic_highlighted,
                        "text_highlighted": result.text_highlighted,
                        "edit_distance": result.edit_distance,
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

    #[test]
    fn sequence_aligner_default_costs_match_python_signature() {
        let aligner = SequenceAligner::default();
        assert_eq!(aligner.deletion_cost, 1);
        assert_eq!(aligner.insertion_cost, 1);
        assert_eq!(aligner.substitution_cost, 1);
    }
}
