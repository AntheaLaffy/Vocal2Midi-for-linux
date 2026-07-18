//! HubertFA phoneme, mora, and dictionary G2P compatibility helpers.
//!
//! This module mirrors the deterministic `BaseG2P`, `PhonemeG2P`, and
//! `JapanesePhonemeMoraG2P` behavior plus the immutable `DictionaryG2P`
//! snapshot in `inference/HubertFA/tools/g2p.py`. Python remains the runtime
//! owner for dictionary selection, dataset discovery, model execution,
//! config/export helpers, warning presentation, and routing.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::str::Utf8Error;

const SILENCE: &str = "SP";
const MORA_ONSETS: &[&str] = &[
    "by", "ch", "dy", "fy", "gw", "gy", "hy", "kw", "ky", "my", "ny", "py", "ry", "sh", "ts", "ty",
    "b", "d", "f", "g", "h", "j", "k", "m", "n", "p", "r", "s", "t", "v", "w", "y", "z",
];

/// Ordered output shared by the HubertFA G2P implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaG2pOutput {
    /// The ordered phonemes.
    pub phonemes: Vec<String>,
    /// The ordered words.
    pub words: Vec<String>,
    /// The ordered phoneme to word.
    pub phoneme_to_word: Vec<isize>,
}

/// Failure from the shared legacy `BaseG2P.__call__` output contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HfaG2pError {
    /// Represents the Python-compatible empty phoneme sequence case.
    EmptyPhonemeSequence,
    /// Represents the Python-compatible invalid silence layout case.
    InvalidSilenceLayout,
}

impl HfaG2pError {
    /// Legacy Python exception type used by fixture and future bridge projections.
    pub const fn exception_type(self) -> &'static str {
        match self {
            Self::EmptyPhonemeSequence => "IndexError",
            Self::InvalidSilenceLayout => "AssertionError",
        }
    }

    /// Exact message emitted by the corresponding Python exception.
    pub const fn message(self) -> &'static str {
        match self {
            Self::EmptyPhonemeSequence => "list index out of range",
            Self::InvalidSilenceLayout => "",
        }
    }
}

impl fmt::Display for HfaG2pError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for HfaG2pError {}

/// Operation that produced a structured dictionary G2P diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HfaDictionaryG2pOperation {
    /// Represents the Python-compatible open case.
    Open,
    /// Represents the Python-compatible read case.
    Read,
    /// Represents the Python-compatible decode case.
    Decode,
    /// Represents the Python-compatible parse case.
    Parse,
    /// Represents the Python-compatible convert case.
    Convert,
}

impl HfaDictionaryG2pOperation {
    /// Stable operation name for tracing and future bridge payloads.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Read => "read",
            Self::Decode => "decode",
            Self::Parse => "parse",
            Self::Convert => "convert",
        }
    }
}

/// Python UTF-8 decode reason retained by a dictionary load error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HfaUtf8DecodeReason {
    /// Represents the Python-compatible invalid start byte case.
    InvalidStartByte,
    /// Represents the Python-compatible invalid continuation byte case.
    InvalidContinuationByte,
    /// Represents the Python-compatible unexpected end of data case.
    UnexpectedEndOfData,
}

impl HfaUtf8DecodeReason {
    /// Exact reason text used by Python's `UnicodeDecodeError` projection.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidStartByte => "invalid start byte",
            Self::InvalidContinuationByte => "invalid continuation byte",
            Self::UnexpectedEndOfData => "unexpected end of data",
        }
    }
}

/// Structured constructor failure for an immutable dictionary snapshot.
#[derive(Debug)]
pub enum HfaDictionaryG2pLoadError {
    /// Represents the Python-compatible io case.
    Io {
        /// The operation.
        operation: HfaDictionaryG2pOperation,
        /// The filesystem path.
        path: PathBuf,
        /// The source.
        source: io::Error,
    },
    /// Represents the Python-compatible decode case.
    Decode {
        /// The operation.
        operation: HfaDictionaryG2pOperation,
        /// The filesystem path.
        path: PathBuf,
        /// The start.
        start: usize,
        /// The end.
        end: usize,
        /// The offending byte.
        offending_byte: u8,
        /// The reason.
        reason: HfaUtf8DecodeReason,
        /// The source.
        source: Utf8Error,
    },
    /// Represents the Python-compatible malformed row case.
    MalformedRow {
        /// The operation.
        operation: HfaDictionaryG2pOperation,
        /// The filesystem path.
        path: PathBuf,
        /// The row index.
        row_index: usize,
        /// The field count.
        field_count: usize,
    },
}

impl HfaDictionaryG2pLoadError {
    /// Operation at which loading failed.
    pub const fn operation(&self) -> HfaDictionaryG2pOperation {
        match self {
            Self::Io { operation, .. }
            | Self::Decode { operation, .. }
            | Self::MalformedRow { operation, .. } => *operation,
        }
    }

    /// Dictionary path associated with the constructor failure.
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. } | Self::Decode { path, .. } | Self::MalformedRow { path, .. } => {
                path
            }
        }
    }

    /// Legacy Python exception type used by fixture and future bridge projections.
    pub fn exception_type(&self) -> &'static str {
        match self {
            Self::Io { source, .. } => match source.kind() {
                io::ErrorKind::NotFound => "FileNotFoundError",
                io::ErrorKind::PermissionDenied => "PermissionError",
                io::ErrorKind::AlreadyExists => "FileExistsError",
                io::ErrorKind::NotADirectory => "NotADirectoryError",
                io::ErrorKind::IsADirectory => "IsADirectoryError",
                _ => "OSError",
            },
            Self::Decode { .. } => "UnicodeDecodeError",
            Self::MalformedRow { .. } => "IndexError",
        }
    }

    /// Exact legacy message for the fixture-bound compatibility projection.
    pub fn compatibility_message(&self) -> String {
        match self {
            Self::Io { path, source, .. } => python_io_message(path, source),
            Self::Decode {
                start,
                end,
                offending_byte,
                reason,
                ..
            } => {
                let subject_and_position = if end - start == 1 {
                    format!("byte 0x{offending_byte:02x} in position {start}")
                } else {
                    format!("bytes in position {start}-{}", end - 1)
                };
                format!(
                    "'utf-8' codec can't decode {subject_and_position}: {}",
                    reason.as_str()
                )
            }
            Self::MalformedRow { .. } => "list index out of range".to_string(),
        }
    }
}

impl fmt::Display for HfaDictionaryG2pLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.compatibility_message())
    }
}

impl Error for HfaDictionaryG2pLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Decode { source, .. } => Some(source),
            Self::MalformedRow { .. } => None,
        }
    }
}

/// Ordered warning produced while converting dictionary-backed text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HfaDictionaryG2pWarning {
    /// Represents the Python-compatible missing word case.
    MissingWord {
        /// The input word index.
        input_word_index: usize,
        /// The word.
        word: String,
    },
    /// Represents the Python-compatible edge silence case.
    EdgeSilence {
        /// The input word index.
        input_word_index: usize,
        /// The dictionary phone index.
        dictionary_phone_index: usize,
        /// The word.
        word: String,
    },
}

impl HfaDictionaryG2pWarning {
    /// The conversion operation owns both dictionary warning variants.
    pub const fn operation(&self) -> HfaDictionaryG2pOperation {
        HfaDictionaryG2pOperation::Convert
    }

    /// Raw literal-space token position in the conversion input.
    pub const fn input_word_index(&self) -> usize {
        match self {
            Self::MissingWord {
                input_word_index, ..
            }
            | Self::EdgeSilence {
                input_word_index, ..
            } => *input_word_index,
        }
    }

    /// Python's `warnings.warn(message)` category for both variants.
    pub const fn category(&self) -> &'static str {
        "UserWarning"
    }

    /// Exact warning message emitted by the legacy converter.
    pub fn message(&self) -> String {
        match self {
            Self::MissingWord { word, .. } => {
                format!("Word '{word}' is not in the dictionary. Ignored.")
            }
            Self::EdgeSilence { word, .. } => format!(
                "The first or last phoneme of word {word} is SP, which is not allowed. \
                 Please check your dictionary."
            ),
        }
    }
}

/// One dictionary conversion, retaining warnings even when the base contract fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaDictionaryG2pConversion {
    /// The output.
    pub output: Result<HfaG2pOutput, HfaG2pError>,
    /// The ordered warnings.
    pub warnings: Vec<HfaDictionaryG2pWarning>,
}

/// Immutable dictionary snapshot with the legacy nullable language prefix.
#[derive(Debug)]
pub struct HfaDictionaryG2p {
    dictionary: HashMap<String, Vec<String>>,
    language: Option<String>,
}

impl HfaDictionaryG2p {
    /// Loads, decodes, universal-newline-normalizes, and parses one dictionary.
    ///
    /// The file is read only during construction. Later conversions use this
    /// immutable snapshot even if the source file changes.
    ///
    /// # Errors
    ///
    /// Returns a structured open/read error, UTF-8 decode error, or malformed
    /// row projection with the original dictionary path and operation.
    pub fn from_path(
        path: impl AsRef<Path>,
        language: Option<&str>,
    ) -> Result<Self, HfaDictionaryG2pLoadError> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path).map_err(|source| HfaDictionaryG2pLoadError::Io {
            operation: HfaDictionaryG2pOperation::Open,
            path: path.clone(),
            source,
        })?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|source| HfaDictionaryG2pLoadError::Io {
                operation: HfaDictionaryG2pOperation::Read,
                path: path.clone(),
                source,
            })?;
        let text = std::str::from_utf8(&bytes).map_err(|source| {
            let start = source.valid_up_to();
            let end = source
                .error_len()
                .map_or(bytes.len(), |length| start + length);
            let offending_byte = bytes[start];
            HfaDictionaryG2pLoadError::Decode {
                operation: HfaDictionaryG2pOperation::Decode,
                path: path.clone(),
                start,
                end,
                offending_byte,
                reason: utf8_decode_reason(&bytes, source),
                source,
            }
        })?;
        let text = python_universal_newlines(text);
        let dictionary = parse_dictionary(&text, &path)?;
        Ok(Self {
            dictionary,
            language: language.map(str::to_string),
        })
    }

    /// Borrow the parsed snapshot without exposing a mutation route.
    pub fn dictionary(&self) -> &HashMap<String, Vec<String>> {
        &self.dictionary
    }

    /// Convert literal-space-separated words and retain ordered warnings.
    pub fn convert(&self, input_text: &str) -> HfaDictionaryG2pConversion {
        let mut words = Vec::new();
        let mut phonemes = vec![SILENCE.to_string()];
        let mut phoneme_to_word = vec![-1];
        let mut warnings = Vec::new();

        for (input_word_index, word) in python_strip(input_text).split(' ').enumerate() {
            let Some(phones) = self.dictionary.get(word) else {
                warnings.push(HfaDictionaryG2pWarning::MissingWord {
                    input_word_index,
                    word: word.to_string(),
                });
                continue;
            };
            let word_index = words.len() as isize;
            words.push(word.to_string());
            for (dictionary_phone_index, phone) in phones.iter().enumerate() {
                if (dictionary_phone_index == 0 || dictionary_phone_index + 1 == phones.len())
                    && phone == SILENCE
                {
                    warnings.push(HfaDictionaryG2pWarning::EdgeSilence {
                        input_word_index,
                        dictionary_phone_index,
                        word: word.to_string(),
                    });
                    continue;
                }
                phonemes.push(phone.clone());
                phoneme_to_word.push(word_index);
            }
            if phonemes.last().is_none_or(|phone| phone != SILENCE) {
                phonemes.push(SILENCE.to_string());
                phoneme_to_word.push(-1);
            }
        }

        let output = apply_base_g2p_contract(
            HfaG2pOutput {
                phonemes,
                words,
                phoneme_to_word,
            },
            self.language.as_deref(),
        );
        HfaDictionaryG2pConversion { output, warnings }
    }
}

/// Applies the shared leading/trailing silence assertions and language prefix.
///
/// The function consumes an already-produced output so callers can project a
/// custom `_g2p` implementation through the same contract without copying it.
///
/// # Errors
///
/// Returns the Python-compatible `IndexError` projection for an empty phoneme
/// list, or the assertion projection for invalid boundary/consecutive `SP`.
pub fn apply_base_g2p_contract(
    mut output: HfaG2pOutput,
    language: Option<&str>,
) -> Result<HfaG2pOutput, HfaG2pError> {
    let Some(first) = output.phonemes.first() else {
        return Err(HfaG2pError::EmptyPhonemeSequence);
    };
    if first != SILENCE || output.phonemes.last().is_none_or(|last| last != SILENCE) {
        return Err(HfaG2pError::InvalidSilenceLayout);
    }
    if output
        .phonemes
        .windows(2)
        .any(|pair| pair[0] == SILENCE && pair[1] == SILENCE)
    {
        return Err(HfaG2pError::InvalidSilenceLayout);
    }

    if let Some(language) = language {
        for phoneme in &mut output.phonemes {
            if phoneme != SILENCE {
                *phoneme = format!("{language}/{phoneme}");
            }
        }
    }
    Ok(output)
}

/// Converts literal-space-separated raw phonemes using legacy HubertFA rules.
///
/// # Errors
///
/// Returns an `HfaG2pError` if the generated sequence violates the shared base
/// contract. Inputs accepted by the legacy `PhonemeG2P` always produce a valid
/// sequence.
pub fn phoneme_g2p(input_text: &str, language: Option<&str>) -> Result<HfaG2pOutput, HfaG2pError> {
    let words = python_strip(input_text)
        .split(' ')
        .filter(|phoneme| *phoneme != SILENCE)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut phonemes = vec![SILENCE.to_string()];
    let mut phoneme_to_word = vec![-1];
    for (word_index, word) in words.iter().enumerate() {
        phonemes.push(word.clone());
        phoneme_to_word.push(word_index as isize);
        phonemes.push(SILENCE.to_string());
        phoneme_to_word.push(-1);
    }

    apply_base_g2p_contract(
        HfaG2pOutput {
            phonemes,
            words,
            phoneme_to_word,
        },
        language,
    )
}

/// Converts Japanese phoneme tokens into mora words with phoneme-level output.
///
/// # Errors
///
/// Returns an `HfaG2pError` if the generated sequence violates the shared base
/// contract. Inputs accepted by the legacy mora converter always produce a
/// valid sequence.
pub fn japanese_phoneme_mora_g2p(
    input_text: &str,
    language: Option<&str>,
) -> Result<HfaG2pOutput, HfaG2pError> {
    let groups = parse_mora_groups(input_text);
    let mut words = Vec::with_capacity(groups.len());
    let mut phonemes = vec![SILENCE.to_string()];
    let mut phoneme_to_word = vec![-1];

    for (word_index, (mora_text, mora_phones)) in groups.into_iter().enumerate() {
        words.push(mora_text);
        for phoneme in mora_phones {
            phonemes.push(phoneme);
            phoneme_to_word.push(word_index as isize);
        }
        if phonemes.last().is_none_or(|phoneme| phoneme != SILENCE) {
            phonemes.push(SILENCE.to_string());
            phoneme_to_word.push(-1);
        }
    }

    apply_base_g2p_contract(
        HfaG2pOutput {
            phonemes,
            words,
            phoneme_to_word,
        },
        language,
    )
}

fn parse_mora_groups(input_text: &str) -> Vec<(String, Vec<String>)> {
    let tokens = python_strip(input_text)
        .split(' ')
        .map(python_strip)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut groups = Vec::new();
    let mut index = 0_usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if matches!(token.as_str(), "SP" | "AP" | "EP") {
            index += 1;
            continue;
        }
        if token == "N" {
            groups.push(("n".to_string(), vec!["N".to_string()]));
            index += 1;
            continue;
        }
        if token == "cl" {
            groups.push(("cl".to_string(), vec!["cl".to_string()]));
            index += 1;
            continue;
        }

        let mora_token = python_15_lowercase(token);
        if let Some(phones) = split_mora_token(&mora_token) {
            groups.push((mora_token, phones));
            index += 1;
            continue;
        }
        if let Some(vowel) = vowel_phone(token) {
            groups.push((vowel.to_string(), vec![vowel.to_string()]));
            index += 1;
            continue;
        }
        if index + 1 < tokens.len() && is_consonant(token) {
            if let Some(vowel) = vowel_phone(&tokens[index + 1]) {
                groups.push((
                    joined_mora(token, vowel),
                    vec![token.clone(), vowel.to_string()],
                ));
                index += 2;
                continue;
            }
        }
        groups.push((mora_token, vec![token.clone()]));
        index += 1;
    }
    groups
}

fn split_mora_token(token: &str) -> Option<Vec<String>> {
    if let Some(vowel) = vowel_phone(token) {
        return Some(vec![vowel.to_string()]);
    }
    if token == "n" {
        return Some(vec!["N".to_string()]);
    }
    if token == "hu" {
        return Some(vec!["h".to_string(), "u".to_string()]);
    }
    if token == "fy" {
        return Some(vec!["f".to_string(), "y".to_string()]);
    }
    if let Some(rest) = token.strip_prefix("fy") {
        if !rest.is_empty() && rest.chars().all(is_vowel_char) {
            let mut phones = vec!["f".to_string(), "y".to_string()];
            phones.extend(rest.chars().map(|character| character.to_string()));
            return Some(phones);
        }
    }

    for onset in MORA_ONSETS {
        if token == *onset {
            return Some(vec![(*onset).to_string()]);
        }
        if let Some(rest) = token.strip_prefix(onset) {
            if !rest.is_empty() && rest.chars().all(is_vowel_char) {
                let mut phones = vec![(*onset).to_string()];
                phones.extend(rest.chars().map(|character| character.to_string()));
                return Some(phones);
            }
        }
    }

    if token.chars().count() >= 2 {
        let vowel = token.chars().next_back().unwrap();
        if is_vowel_char(vowel) {
            let onset = token.strip_suffix(vowel).unwrap();
            if let Some(base_onset) = onset.strip_suffix('y') {
                if is_consonant(base_onset) {
                    return Some(vec![
                        base_onset.to_string(),
                        "y".to_string(),
                        vowel.to_string(),
                    ]);
                }
            }
            if onset.chars().count() == 1 && is_consonant(onset) {
                return Some(vec![onset.to_string(), vowel.to_string()]);
            }
        }
    }
    None
}

fn vowel_phone(token: &str) -> Option<&'static str> {
    match token {
        "a" => Some("a"),
        "i" | "I" => Some("i"),
        "u" | "U" => Some("u"),
        "e" => Some("e"),
        "o" => Some("o"),
        _ => None,
    }
}

fn is_vowel_char(character: char) -> bool {
    matches!(character, 'a' | 'i' | 'u' | 'e' | 'o')
}

fn is_consonant(token: &str) -> bool {
    matches!(
        token,
        "b" | "by"
            | "ch"
            | "d"
            | "dy"
            | "f"
            | "fy"
            | "g"
            | "gw"
            | "gy"
            | "h"
            | "hy"
            | "j"
            | "k"
            | "kw"
            | "ky"
            | "m"
            | "my"
            | "n"
            | "ny"
            | "p"
            | "py"
            | "r"
            | "ry"
            | "s"
            | "sh"
            | "t"
            | "ts"
            | "ty"
            | "v"
            | "w"
            | "y"
            | "z"
    )
}

fn joined_mora(consonant: &str, vowel: &str) -> String {
    match (consonant, vowel) {
        ("sh", "a") => "sha".to_string(),
        ("sh", "i") => "shi".to_string(),
        ("sh", "u") => "shu".to_string(),
        ("sh", "e") => "she".to_string(),
        ("sh", "o") => "sho".to_string(),
        ("ch", "a") => "cha".to_string(),
        ("ch", "i") => "chi".to_string(),
        ("ch", "u") => "chu".to_string(),
        ("ch", "e") => "che".to_string(),
        ("ch", "o") => "cho".to_string(),
        ("j", "a") => "ja".to_string(),
        ("j", "i") => "ji".to_string(),
        ("j", "u") => "ju".to_string(),
        ("j", "e") => "je".to_string(),
        ("j", "o") => "jo".to_string(),
        ("ts", "u") => "tsu".to_string(),
        ("k", "i") => "ki".to_string(),
        ("g", "i") => "gi".to_string(),
        ("s", "i") => "shi".to_string(),
        ("z", "i") => "ji".to_string(),
        ("n", "i") => "ni".to_string(),
        ("h", "i") => "hi".to_string(),
        ("b", "i") => "bi".to_string(),
        ("p", "i") => "pi".to_string(),
        ("m", "i") => "mi".to_string(),
        ("r", "i") => "ri".to_string(),
        ("f", "u") => "fu".to_string(),
        ("ky", "i") => "ki".to_string(),
        ("gy", "i") => "gi".to_string(),
        ("ny", "i") => "ni".to_string(),
        ("hy", "i") => "hi".to_string(),
        ("my", "i") => "mi".to_string(),
        ("ry", "i") => "ri".to_string(),
        ("by", "i") => "bi".to_string(),
        ("py", "i") => "pi".to_string(),
        ("ty", "i") => "chi".to_string(),
        ("ty", "u") => "chu".to_string(),
        ("ty", "o") => "cho".to_string(),
        ("dy", "i") => "ji".to_string(),
        ("dy", "u") => "ju".to_string(),
        ("dy", "o") => "jo".to_string(),
        _ => format!("{consonant}{vowel}"),
    }
}

fn parse_dictionary(
    text: &str,
    path: &Path,
) -> Result<HashMap<String, Vec<String>>, HfaDictionaryG2pLoadError> {
    let stripped = python_strip(text);
    let mut dictionary = HashMap::new();
    for (row_index, row) in stripped.split('\n').enumerate() {
        let fields = row.split('\t').collect::<Vec<_>>();
        if fields.len() < 2 {
            return Err(HfaDictionaryG2pLoadError::MalformedRow {
                operation: HfaDictionaryG2pOperation::Parse,
                path: path.to_path_buf(),
                row_index,
                field_count: fields.len(),
            });
        }
        dictionary.insert(
            python_strip(fields[0]).to_string(),
            python_strip(fields[1])
                .split(' ')
                .map(str::to_string)
                .collect(),
        );
    }
    Ok(dictionary)
}

fn python_universal_newlines(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\r' {
            if characters.peek() == Some(&'\n') {
                characters.next();
            }
            output.push('\n');
        } else {
            output.push(character);
        }
    }
    output
}

fn utf8_decode_reason(bytes: &[u8], source: Utf8Error) -> HfaUtf8DecodeReason {
    if source.error_len().is_none() {
        return HfaUtf8DecodeReason::UnexpectedEndOfData;
    }
    match bytes[source.valid_up_to()] {
        0xc2..=0xf4 => HfaUtf8DecodeReason::InvalidContinuationByte,
        _ => HfaUtf8DecodeReason::InvalidStartByte,
    }
}

fn python_io_message(path: &Path, source: &io::Error) -> String {
    let reason = source.to_string();
    let Some(error_number) = source.raw_os_error() else {
        return reason;
    };
    let suffix = format!(" (os error {error_number})");
    let reason = reason.strip_suffix(&suffix).unwrap_or(&reason);
    format!(
        "[Errno {error_number}] {reason}: {}",
        path.to_str().map_or_else(
            || format!("{:?}", path.as_os_str()),
            crate::python_15_nonprintable::string_repr,
        )
    )
}

fn python_strip(value: &str) -> &str {
    value.trim_matches(is_python_whitespace)
}

fn python_15_lowercase(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chunk_start = 0_usize;
    for (index, character) in value.char_indices() {
        if has_post_python_15_lowercase_mapping(character) {
            output.push_str(&value[chunk_start..index].to_lowercase());
            output.push(character);
            chunk_start = index + character.len_utf8();
        }
    }
    output.push_str(&value[chunk_start..].to_lowercase());
    output
}

fn has_post_python_15_lowercase_mapping(character: char) -> bool {
    matches!(
        character,
        '\u{1c89}'
            | '\u{a7cb}'
            | '\u{a7cc}'
            | '\u{a7ce}'
            | '\u{a7d2}'
            | '\u{a7d4}'
            | '\u{a7da}'
            | '\u{a7dc}'
            | '\u{10d50}'..='\u{10d65}'
            | '\u{16ea0}'..='\u{16eb8}'
    )
}

fn is_python_whitespace(character: char) -> bool {
    matches!(character, '\u{001c}'..='\u{001f}') || character.is_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;
    use md5::{Digest, Md5};
    use serde_json::{Value, json};
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_phoneme_mora_g2p_core.jsonl");
    const DICTIONARY_FIXTURES: &str =
        include_str!("../../../../fixtures/hfa_dictionary_g2p_core.jsonl");
    static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            loop {
                let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "v2m-hfa-dictionary-g2p-{}-{sequence}",
                    std::process::id()
                ));
                match fs::create_dir(&path) {
                    Ok(()) => return Self(path),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("failed to create test directory: {error}"),
                }
            }
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn parse_strings(value: &Value) -> Vec<String> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_str().unwrap().to_string())
            .collect()
    }

    fn parse_indexes(value: &Value) -> Vec<isize> {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item.as_i64().unwrap() as isize)
            .collect()
    }

    fn parse_output(value: &Value) -> HfaG2pOutput {
        HfaG2pOutput {
            phonemes: parse_strings(&value["phonemes"]),
            words: parse_strings(&value["words"]),
            phoneme_to_word: parse_indexes(&value["phoneme_to_word"]),
        }
    }

    fn encode_result(result: Result<HfaG2pOutput, HfaG2pError>) -> Value {
        match result {
            Ok(output) => json!({
                "value": {
                    "phonemes": output.phonemes,
                    "words": output.words,
                    "phoneme_to_word": output.phoneme_to_word,
                }
            }),
            Err(error) => json!({
                "error": {
                    "type": error.exception_type(),
                    "message": error.to_string(),
                }
            }),
        }
    }

    fn decode_hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let pair = std::str::from_utf8(pair).unwrap();
                u8::from_str_radix(pair, 16).unwrap()
            })
            .collect()
    }

    fn encode_dictionary(converter: &HfaDictionaryG2p, expected_dictionary: &Value) -> Value {
        let expected = expected_dictionary.as_object().unwrap();
        assert_eq!(converter.dictionary().len(), expected.len());
        let mut dictionary = serde_json::Map::new();
        for word in expected.keys() {
            dictionary.insert(
                word.clone(),
                json!(converter.dictionary().get(word).unwrap()),
            );
        }
        Value::Object(dictionary)
    }

    fn encode_dictionary_conversion(conversion: HfaDictionaryG2pConversion) -> Value {
        let warnings = conversion
            .warnings
            .iter()
            .map(|warning| {
                json!({
                    "category": warning.category(),
                    "message": warning.message(),
                })
            })
            .collect::<Vec<_>>();
        match conversion.output {
            Ok(output) => json!({
                "value": {
                    "phonemes": output.phonemes,
                    "words": output.words,
                    "phoneme_to_word": output.phoneme_to_word,
                },
                "warnings": warnings,
            }),
            Err(error) => json!({
                "error": {
                    "type": error.exception_type(),
                    "message": error.message(),
                },
                "warnings": warnings,
            }),
        }
    }

    fn encode_load_error(error: HfaDictionaryG2pLoadError, temp_root: &Path) -> Value {
        json!({
            "error": {
                "type": error.exception_type(),
                "message": error.compatibility_message().replace(
                    temp_root.to_string_lossy().as_ref(),
                    "<TMP>",
                ),
            }
        })
    }

    fn lowercase_scalar_digest() -> Value {
        let mut digest = Md5::new();
        let mut scalar_count = 0_u64;
        for codepoint in 0_u32..=0x10ffff {
            let Some(character) = char::from_u32(codepoint) else {
                continue;
            };
            digest.update(codepoint.to_be_bytes());
            digest.update(python_15_lowercase(&character.to_string()).as_bytes());
            digest.update(b"\0");
            scalar_count += 1;
        }
        json!({
            "value": {
                "unicode_version": "15.0.0",
                "scalar_count": scalar_count,
                "md5": format!("{:x}", digest.finalize()),
            }
        })
    }

    #[test]
    fn hfa_phoneme_mora_g2p_core_follows_python_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let kind = case["kind"].as_str().unwrap();
            let language = case.get("language").and_then(Value::as_str);
            let actual = match kind {
                "lowercase_scalar_digest" => lowercase_scalar_digest(),
                "phoneme" => encode_result(phoneme_g2p(case["text"].as_str().unwrap(), language)),
                "mora" => encode_result(japanese_phoneme_mora_g2p(
                    case["text"].as_str().unwrap(),
                    language,
                )),
                "base_contract" => encode_result(apply_base_g2p_contract(
                    parse_output(&case["output"]),
                    language,
                )),
                kind => panic!("line {} has unsupported kind {kind}", line_index + 1),
            };
            assert_eq!(
                actual,
                case["expect"],
                "{}",
                case["case_id"].as_str().unwrap()
            );
        }
    }

    #[test]
    fn hfa_phoneme_mora_g2p_core_scales_linearly_for_large_token_lists() {
        let input = std::iter::repeat_n("ka", 10_000)
            .collect::<Vec<_>>()
            .join(" ");
        let output = japanese_phoneme_mora_g2p(&input, Some("ja")).unwrap();
        assert_eq!(output.words.len(), 10_000);
        assert_eq!(output.phonemes.len(), 30_001);
        assert_eq!(output.phoneme_to_word.len(), output.phonemes.len());
        assert_eq!(output.phonemes.first().unwrap(), "SP");
        assert_eq!(output.phonemes.last().unwrap(), "SP");
    }

    #[test]
    fn hfa_phoneme_mora_g2p_core_repeated_calls_are_stable() {
        let first = japanese_phoneme_mora_g2p("ka SP shi", Some("ja")).unwrap();
        let second = japanese_phoneme_mora_g2p("ka SP shi", Some("ja")).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn hfa_dictionary_g2p_core_follows_python_fixture_table() {
        for (line_index, line) in DICTIONARY_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let directory = TestDirectory::new();
            let dictionary_spec = &case["dictionary"];
            let dictionary_path = directory.path().join(
                dictionary_spec
                    .get("path_name")
                    .and_then(Value::as_str)
                    .unwrap_or("dictionary.txt"),
            );
            match dictionary_spec
                .get("path_kind")
                .and_then(Value::as_str)
                .unwrap_or("file")
            {
                "file" => {
                    let bytes = dictionary_spec
                        .get("bytes_hex")
                        .and_then(Value::as_str)
                        .map(decode_hex)
                        .unwrap_or_else(|| {
                            dictionary_spec["content"]
                                .as_str()
                                .unwrap()
                                .as_bytes()
                                .to_vec()
                        });
                    fs::write(&dictionary_path, bytes).unwrap();
                }
                "directory" => fs::create_dir(&dictionary_path).unwrap(),
                "missing" => {}
                path_kind => panic!("unsupported path_kind {path_kind}"),
            }

            let language = case.get("language").and_then(Value::as_str);
            let actual = match HfaDictionaryG2p::from_path(&dictionary_path, language) {
                Err(error) => encode_load_error(error, directory.path()),
                Ok(converter) => {
                    let mut result = serde_json::Map::new();
                    result.insert(
                        "dictionary".to_string(),
                        encode_dictionary(&converter, &case["expect"]["dictionary"]),
                    );
                    if let Some(content) = dictionary_spec
                        .get("after_construct_content")
                        .and_then(Value::as_str)
                    {
                        fs::write(&dictionary_path, content.as_bytes()).unwrap();
                    }
                    result.insert(
                        "calls".to_string(),
                        Value::Array(
                            case["texts"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|text| {
                                    encode_dictionary_conversion(
                                        converter.convert(text.as_str().unwrap()),
                                    )
                                })
                                .collect(),
                        ),
                    );
                    Value::Object(result)
                }
            };
            assert_eq!(
                actual,
                case["expect"],
                "line {}: {}",
                line_index + 1,
                case["case_id"].as_str().unwrap()
            );
        }
    }

    #[test]
    fn hfa_dictionary_g2p_core_retains_structured_load_context() {
        let directory = TestDirectory::new();
        let missing_path = directory.path().join("missing.txt");
        let missing = HfaDictionaryG2p::from_path(&missing_path, None).unwrap_err();
        assert_eq!(missing.operation(), HfaDictionaryG2pOperation::Open);
        assert_eq!(missing.path(), missing_path);
        match missing {
            HfaDictionaryG2pLoadError::Io { source, .. } => {
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            error => panic!("expected IO error, got {error:?}"),
        }

        let invalid_path = directory.path().join("invalid.txt");
        fs::write(&invalid_path, b"word\t\xff").unwrap();
        let invalid = HfaDictionaryG2p::from_path(&invalid_path, None).unwrap_err();
        match invalid {
            HfaDictionaryG2pLoadError::Decode {
                operation,
                path,
                start,
                end,
                offending_byte,
                reason,
                source,
            } => {
                assert_eq!(operation, HfaDictionaryG2pOperation::Decode);
                assert_eq!(path, invalid_path);
                assert_eq!(start, 5);
                assert_eq!(end, 6);
                assert_eq!(offending_byte, 0xff);
                assert_eq!(reason, HfaUtf8DecodeReason::InvalidStartByte);
                assert_eq!(source.valid_up_to(), 5);
            }
            error => panic!("expected decode error, got {error:?}"),
        }

        fs::write(&invalid_path, b"word\t\xe2\x82A").unwrap();
        let invalid = HfaDictionaryG2p::from_path(&invalid_path, None).unwrap_err();
        match invalid {
            HfaDictionaryG2pLoadError::Decode {
                start,
                end,
                offending_byte,
                reason,
                source,
                ..
            } => {
                assert_eq!((start, end), (5, 7));
                assert_eq!(offending_byte, 0xe2);
                assert_eq!(reason, HfaUtf8DecodeReason::InvalidContinuationByte);
                assert_eq!(source.error_len(), Some(2));
            }
            error => panic!("expected decode error, got {error:?}"),
        }

        fs::write(&invalid_path, b"word\t\xf0\x90\x80").unwrap();
        let invalid = HfaDictionaryG2p::from_path(&invalid_path, None).unwrap_err();
        match invalid {
            HfaDictionaryG2pLoadError::Decode {
                start,
                end,
                offending_byte,
                reason,
                source,
                ..
            } => {
                assert_eq!((start, end), (5, 8));
                assert_eq!(offending_byte, 0xf0);
                assert_eq!(reason, HfaUtf8DecodeReason::UnexpectedEndOfData);
                assert_eq!(source.error_len(), None);
            }
            error => panic!("expected decode error, got {error:?}"),
        }

        let malformed_path = directory.path().join("malformed.txt");
        fs::write(&malformed_path, b"ok\toh k\nbad\nlast\tl").unwrap();
        let malformed = HfaDictionaryG2p::from_path(&malformed_path, None).unwrap_err();
        match malformed {
            HfaDictionaryG2pLoadError::MalformedRow {
                operation,
                path,
                row_index,
                field_count,
            } => {
                assert_eq!(operation, HfaDictionaryG2pOperation::Parse);
                assert_eq!(path, malformed_path);
                assert_eq!(row_index, 1);
                assert_eq!(field_count, 1);
            }
            error => panic!("expected row error, got {error:?}"),
        }
    }

    #[test]
    fn hfa_dictionary_g2p_core_repeats_and_recovers_after_assertion_errors() {
        let directory = TestDirectory::new();
        let dictionary_path = directory.path().join("dictionary.txt");
        fs::write(
            &dictionary_path,
            b"edge\tSP eh SP\nbad\tb SP SP d\ngood\tg uh d",
        )
        .unwrap();
        let converter = HfaDictionaryG2p::from_path(&dictionary_path, None).unwrap();

        let failed = converter.convert("edge bad missing");
        assert_eq!(failed.output, Err(HfaG2pError::InvalidSilenceLayout));
        assert_eq!(failed.warnings.len(), 3);
        assert_eq!(failed.warnings[0].input_word_index(), 0);
        assert_eq!(failed.warnings[1].input_word_index(), 0);
        assert_eq!(failed.warnings[2].input_word_index(), 2);

        let first = converter.convert("good");
        let second = converter.convert("good");
        assert_eq!(first, second);
        assert!(first.warnings.is_empty());
        assert_eq!(first.output.unwrap().words, ["good"]);
    }

    #[test]
    fn hfa_dictionary_g2p_core_scales_for_large_dictionary_and_input() {
        let directory = TestDirectory::new();
        let dictionary_path = directory.path().join("dictionary.txt");
        let dictionary = (0..10_000)
            .map(|index| format!("word{index}\tphone{index}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&dictionary_path, dictionary).unwrap();
        let converter = HfaDictionaryG2p::from_path(&dictionary_path, Some("x")).unwrap();
        let input = (0..10_000)
            .map(|index| format!("word{index}"))
            .collect::<Vec<_>>()
            .join(" ");

        let conversion = converter.convert(&input);
        assert!(conversion.warnings.is_empty());
        let output = conversion.output.unwrap();
        assert_eq!(output.words.len(), 10_000);
        assert_eq!(output.phonemes.len(), 20_001);
        assert_eq!(output.phonemes.first().unwrap(), "SP");
        assert_eq!(output.phonemes[1], "x/phone0");
        assert_eq!(output.phonemes[19_999], "x/phone9999");
        assert_eq!(output.phonemes.last().unwrap(), "SP");
        assert_eq!(output.phoneme_to_word.len(), output.phonemes.len());
    }
}
