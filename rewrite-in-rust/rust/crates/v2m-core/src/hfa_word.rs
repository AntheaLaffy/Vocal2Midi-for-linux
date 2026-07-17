//! HubertFA phoneme, word, and WordList compatibility types.
//!
//! This module mirrors the local `Phoneme`, `Word`, and `WordList` behavior in
//! `inference/HubertFA/tools/align_word.py`. Python remains the runtime owner
//! for decoder and aggregation algorithms, audio/export/model IO, caller
//! warning presentation, and production routing.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

/// Constructor error with the exact legacy Python message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaWordError {
    message: String,
}

impl HfaWordError {
    /// Returns the legacy error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for HfaWordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HfaWordError {}

/// Error from moving a boundary on a Word without phonemes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HfaWordMutationError;

impl HfaWordMutationError {
    /// Legacy Python exception type used by fixture and bridge projections.
    pub const fn exception_type(&self) -> &'static str {
        "IndexError"
    }
}

impl fmt::Display for HfaWordMutationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("list index out of range")
    }
}

impl Error for HfaWordMutationError {}

/// Warning emitted by a local Word mutation when no log list is supplied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaWordWarning {
    pub message: String,
}

impl HfaWordWarning {
    /// Python's `warnings.warn(message)` category for these call sites.
    pub const fn category(&self) -> &'static str {
        "UserWarning"
    }
}

/// One HubertFA phoneme interval.
#[derive(Debug, Clone, PartialEq)]
pub struct Phoneme {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

impl Phoneme {
    /// Constructs a phoneme with Python-compatible start clamping.
    ///
    /// # Errors
    ///
    /// Returns `HfaWordError` unless the clamped start is strictly less than
    /// the end.
    pub fn new(start: f64, end: f64, text: impl Into<String>) -> Result<Self, HfaWordError> {
        let start = clamp_python_start(start);
        let text = text.into();
        if start < end {
            Ok(Self { start, end, text })
        } else {
            Err(HfaWordError {
                message: format!(
                    "Phoneme Invalid: text={} start={}, end={}",
                    text,
                    python_float_string(start),
                    python_float_string(end)
                ),
            })
        }
    }
}

/// One HubertFA word interval and its locally owned phonemes.
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub phonemes: Vec<Phoneme>,
}

impl Word {
    /// Constructs a word, optionally with one full-span initial phoneme.
    ///
    /// # Errors
    ///
    /// Returns `HfaWordError` unless the clamped start is strictly less than
    /// the end.
    pub fn new(
        start: f64,
        end: f64,
        text: impl Into<String>,
        init_phoneme: bool,
    ) -> Result<Self, HfaWordError> {
        let start = clamp_python_start(start);
        let text = text.into();
        if start < end {
            let phonemes = if init_phoneme {
                vec![Phoneme::new(start, end, text.clone()).expect("validated word interval")]
            } else {
                Vec::new()
            };
            Ok(Self {
                start,
                end,
                text,
                phonemes,
            })
        } else {
            Err(HfaWordError {
                message: format!(
                    "Word Invalid: text={} start={}, end={}",
                    text,
                    python_float_string(start),
                    python_float_string(end)
                ),
            })
        }
    }

    /// Returns the legacy `Word.dur` value.
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }

    /// Adds a phoneme when it is fully contained by this word.
    pub fn add_phoneme(
        &mut self,
        phoneme: Phoneme,
        log_list: Option<&mut Vec<String>>,
    ) -> Option<HfaWordWarning> {
        if phoneme.start == phoneme.end {
            return deliver_warning(format!("{} phoneme长度为0，非法", phoneme.text), log_list);
        }
        if phoneme.start >= self.start && phoneme.end <= self.end {
            self.phonemes.push(phoneme);
            None
        } else {
            deliver_warning(
                format!("{}: phoneme边界超出word，添加失败", phoneme.text),
                log_list,
            )
        }
    }

    /// Appends a contiguous phoneme and grows the word end.
    pub fn append_phoneme(
        &mut self,
        phoneme: Phoneme,
        log_list: Option<&mut Vec<String>>,
    ) -> Option<HfaWordWarning> {
        if phoneme.start == phoneme.end {
            return deliver_warning(format!("{} phoneme长度为0，非法", phoneme.text), log_list);
        }

        if let Some(previous) = self.phonemes.last() {
            if phoneme.start == previous.end {
                self.end = phoneme.end;
                self.phonemes.push(phoneme);
                None
            } else {
                deliver_warning(format!("{}: phoneme添加失败", phoneme.text), log_list)
            }
        } else if phoneme.start == self.start {
            self.end = phoneme.end;
            self.phonemes.push(phoneme);
            None
        } else {
            deliver_warning(
                format!("{}: phoneme左边界超出word，添加失败", phoneme.text),
                log_list,
            )
        }
    }

    /// Moves the word and first phoneme start when the new boundary is valid.
    pub fn move_start(
        &mut self,
        new_start: f64,
        log_list: Option<&mut Vec<String>>,
    ) -> Result<Option<HfaWordWarning>, HfaWordMutationError> {
        if new_start < 0.0 || new_start.is_nan() {
            return Ok(deliver_warning(
                format!("{}: start >= first_phone_end，无法调整word边界", self.text),
                log_list,
            ));
        }
        let first = self.phonemes.first_mut().ok_or(HfaWordMutationError)?;
        if new_start < first.end {
            self.start = new_start;
            first.start = new_start;
            Ok(None)
        } else {
            Ok(deliver_warning(
                format!("{}: start >= first_phone_end，无法调整word边界", self.text),
                log_list,
            ))
        }
    }

    /// Moves the word and last phoneme end when the new boundary is valid.
    pub fn move_end(
        &mut self,
        new_end: f64,
        log_list: Option<&mut Vec<String>>,
    ) -> Result<Option<HfaWordWarning>, HfaWordMutationError> {
        let last = self.phonemes.last_mut().ok_or(HfaWordMutationError)?;
        if new_end > last.start && last.start >= 0.0 {
            self.end = new_end;
            last.end = new_end;
            Ok(None)
        } else {
            Ok(deliver_warning(
                format!(
                    "{}: new_end <= first_phone_start，无法调整word边界",
                    self.text
                ),
                log_list,
            ))
        }
    }
}

/// Encapsulated single-threaded Word identity used by the legacy list
/// compatibility layer.
///
/// Python stores object references in `WordList`. `Rc<RefCell<_>>` preserves
/// that aliasing without unsafe code. The interior is private so callers can
/// clone identity but cannot retain a `Ref` or `RefMut` guard across collection
/// calls.
#[derive(Debug, Clone)]
pub struct WordHandle {
    inner: Rc<RefCell<Word>>,
}

impl WordHandle {
    /// Creates a shared handle around a canonical Word.
    pub fn new(word: Word) -> Self {
        Self {
            inner: Rc::new(RefCell::new(word)),
        }
    }

    /// Returns an owned snapshot without exposing an interior borrow guard.
    pub fn snapshot(&self) -> Result<Word, HfaWordListError> {
        self.read(Clone::clone)
    }

    /// Returns whether two handles retain the same Word identity.
    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    /// Mutates a raw public start value as Python callers can after
    /// construction.
    pub fn set_start(&self, start: f64) -> Result<(), HfaWordListError> {
        self.write(|word| word.start = start)
    }

    /// Replaces Word text without exposing mutable interior access.
    pub fn set_text(&self, text: impl Into<String>) -> Result<(), HfaWordListError> {
        let text = text.into();
        self.write(|word| word.text = text)
    }

    /// Replaces Word text and every owned phoneme text, preserving the source
    /// alias mutation exercised by the compatibility contract.
    pub fn set_text_and_all_phonemes(
        &self,
        text: impl Into<String>,
        phoneme_text: impl Into<String>,
    ) -> Result<(), HfaWordListError> {
        let text = text.into();
        let phoneme_text = phoneme_text.into();
        self.write(|word| {
            word.text = text;
            for phoneme in &mut word.phonemes {
                phoneme.text.clone_from(&phoneme_text);
            }
        })
    }

    fn read<R>(&self, operation: impl FnOnce(&Word) -> R) -> Result<R, HfaWordListError> {
        let word = self
            .inner
            .try_borrow()
            .map_err(|_| HfaWordListError::borrow_conflict())?;
        Ok(operation(&word))
    }

    fn write<R>(&self, operation: impl FnOnce(&mut Word) -> R) -> Result<R, HfaWordListError> {
        let mut word = self
            .inner
            .try_borrow_mut()
            .map_err(|_| HfaWordListError::borrow_conflict())?;
        Ok(operation(&mut word))
    }
}

/// Legacy default `WordList.add_AP` minimum residual duration.
pub const DEFAULT_AP_MIN_DURATION: f64 = 0.1;

/// Legacy default `WordList.fill_small_gaps` repair threshold.
pub const DEFAULT_FINALIZE_GAP_LENGTH: f64 = 0.1;

/// Legacy default `WordList.add_SP` inserted phone text.
pub const DEFAULT_SP_PHONE: &str = "SP";

/// One raw entry retained by the heterogeneous legacy WordList.
#[derive(Debug, Clone)]
pub enum WordListEntry {
    Word(WordHandle),
    Invalid(String),
}

impl WordListEntry {
    /// Creates an entry retaining the supplied Word identity.
    pub fn word(word: WordHandle) -> Self {
        Self::Word(word)
    }

    /// Creates the string-valued invalid entry exercised by the compatibility
    /// seam.
    pub fn invalid(value: impl Into<String>) -> Self {
        Self::Invalid(value.into())
    }
}

/// Python exception kind projected by WordList operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HfaWordListErrorKind {
    ValueError,
    AttributeError,
    IndexError,
    BorrowError,
}

/// Structured WordList error with exact legacy text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HfaWordListError {
    kind: HfaWordListErrorKind,
    message: String,
}

impl HfaWordListError {
    fn value_error(message: impl Into<String>) -> Self {
        Self {
            kind: HfaWordListErrorKind::ValueError,
            message: message.into(),
        }
    }

    fn invalid_attribute(attribute: &str) -> Self {
        Self {
            kind: HfaWordListErrorKind::AttributeError,
            message: format!("'str' object has no attribute '{attribute}'"),
        }
    }

    fn index_error() -> Self {
        Self {
            kind: HfaWordListErrorKind::IndexError,
            message: "list index out of range".to_string(),
        }
    }

    fn borrow_conflict() -> Self {
        Self {
            kind: HfaWordListErrorKind::BorrowError,
            message: "word handle borrow conflict".to_string(),
        }
    }

    fn is_borrow_error(&self) -> bool {
        self.kind == HfaWordListErrorKind::BorrowError
    }

    /// Legacy Python exception type used by fixture and bridge projections.
    pub const fn exception_type(&self) -> &'static str {
        match self.kind {
            HfaWordListErrorKind::ValueError => "ValueError",
            HfaWordListErrorKind::AttributeError => "AttributeError",
            HfaWordListErrorKind::IndexError => "IndexError",
            HfaWordListErrorKind::BorrowError => "BorrowError",
        }
    }
}

impl fmt::Display for HfaWordListError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for HfaWordListError {}

impl From<HfaWordMutationError> for HfaWordListError {
    fn from(_: HfaWordMutationError) -> Self {
        Self::index_error()
    }
}

/// Canonical heterogeneous HubertFA WordList and its persistent diagnostic log.
#[derive(Debug, Clone, Default)]
pub struct WordList {
    entries: Vec<WordListEntry>,
    log: Vec<String>,
}

impl WordList {
    /// Creates an empty WordList.
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            log: Vec::new(),
        }
    }

    /// Creates a WordList from raw entries without append validation.
    pub fn from_raw(entries: Vec<WordListEntry>) -> Self {
        Self {
            entries,
            log: Vec::new(),
        }
    }

    /// Returns raw entries in current list order.
    pub fn entries(&self) -> &[WordListEntry] {
        &self.entries
    }

    /// Extends the raw list without append validation, matching Python list
    /// inheritance.
    pub fn raw_extend(&mut self, entries: impl IntoIterator<Item = WordListEntry>) {
        self.entries.extend(entries);
    }

    /// Clears all entries while retaining the persistent diagnostic buffer.
    pub fn clear_entries(&mut self) {
        self.entries.clear();
    }

    /// Returns accumulated diagnostics joined by newlines.
    pub fn log(&self) -> String {
        self.log.join("\n")
    }

    /// Clears the persistent diagnostic buffer.
    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    /// Returns overlapping Word handles in current list order, ignoring invalid
    /// entries and treating touching boundaries as non-overlapping.
    pub fn overlapping_words(
        &self,
        new_word: &WordHandle,
    ) -> Result<Vec<WordHandle>, HfaWordListError> {
        Self::overlapping_words_in(&self.entries, new_word)
    }

    fn overlapping_words_in(
        entries: &[WordListEntry],
        new_word: &WordHandle,
    ) -> Result<Vec<WordHandle>, HfaWordListError> {
        let (new_start, new_end) = new_word.read(|word| (word.start, word.end))?;
        let mut overlapping = Vec::new();
        for entry in entries {
            let WordListEntry::Word(word) = entry else {
                continue;
            };
            let overlaps = word.read(|word| !(new_end <= word.start || new_start >= word.end))?;
            if overlaps {
                overlapping.push(word.clone());
            }
        }
        Ok(overlapping)
    }

    /// Appends a validated Word reference or records the exact legacy warning.
    pub fn append(&mut self, word: WordHandle) -> Result<(), HfaWordListError> {
        Self::append_into(&mut self.entries, &mut self.log, word)
    }

    fn append_into(
        entries: &mut Vec<WordListEntry>,
        log: &mut Vec<String>,
        word: WordHandle,
    ) -> Result<(), HfaWordListError> {
        let empty_warning = word.read(|word_value| {
            word_value
                .phonemes
                .is_empty()
                .then(|| format!("{}: phones为空，非法word", python_word_repr(word_value)))
        })?;
        if let Some(message) = empty_warning {
            log.push(format!("WARNING: {message}"));
            return Ok(());
        }

        if entries.is_empty() || Self::overlapping_words_in(entries, &word)?.is_empty() {
            entries.push(WordListEntry::Word(word));
        } else {
            let message = word.read(|word_value| {
                format!("{}: 区间重叠，无法添加word", python_word_repr(word_value))
            })?;
            log.push(format!("WARNING: {message}"));
        }
        Ok(())
    }

    /// Repairs leading, trailing, and interior gaps using the legacy order.
    ///
    /// Semantic finalization errors are caught and appended to the persistent
    /// log, matching Python. An internal handle borrow conflict is returned so
    /// callers never observe a panic or a misleading compatibility diagnostic.
    ///
    /// # Errors
    ///
    /// Returns `BorrowError` when canonical Word identity cannot be accessed.
    pub fn fill_small_gaps(
        &mut self,
        wav_length: f64,
        gap_length: f64,
    ) -> Result<(), HfaWordListError> {
        if let Err(error) = self.fill_small_gaps_inner(wav_length, gap_length) {
            if error.is_borrow_error() {
                return Err(error);
            }
            self.add_log(format!("ERROR in fill_small_gaps: {error}"));
        }
        Ok(())
    }

    /// Repairs gaps using the legacy `0.1` threshold.
    ///
    /// # Errors
    ///
    /// Returns `BorrowError` when canonical Word identity cannot be accessed.
    pub fn fill_small_gaps_default(&mut self, wav_length: f64) -> Result<(), HfaWordListError> {
        self.fill_small_gaps(wav_length, DEFAULT_FINALIZE_GAP_LENGTH)
    }

    fn fill_small_gaps_inner(
        &mut self,
        wav_length: f64,
        gap_length: f64,
    ) -> Result<(), HfaWordListError> {
        let entry_count = self.entries.len();
        let first = Self::word_at(&self.entries, 0, "start")?;
        if first.read(|word| word.start)? < 0.0 {
            first.write(|word| word.start = 0.0)?;
        }

        if first.read(|word| word.start)? > 0.0 {
            let leading_start = first.read(|word| word.start)?;
            if leading_start.abs() < gap_length {
                let duration = first.read(Word::duration)?;
                if gap_length < duration {
                    first.write(|word| word.move_start(0.0, Some(&mut self.log)))??;
                }
            }
        }

        let last = Self::word_at(&self.entries, entry_count.wrapping_sub(1), "end")?;
        if last.read(|word| word.end)? >= wav_length - gap_length {
            last.write(|word| word.move_end(wav_length, Some(&mut self.log)))??;
        }

        for index in 1..entry_count {
            let current = Self::word_at(&self.entries, index, "start")?;
            let current_start = current.read(|word| word.start)?;
            let previous = Self::word_at(&self.entries, index - 1, "end")?;
            let previous_end = previous.read(|word| word.end)?;
            let gap = current_start - previous_end;
            if gap > 0.0 && gap <= gap_length {
                previous.write(|word| word.move_end(current_start, Some(&mut self.log)))??;
            }
        }
        Ok(())
    }

    /// Inserts gap Words, replaces entries after successful construction, and
    /// runs the legacy final check while ignoring its boolean result.
    ///
    /// Semantic finalization errors are caught and appended to the persistent
    /// log. Candidate append warnings are written directly to that same log and
    /// therefore survive an error that prevents entry replacement.
    ///
    /// # Errors
    ///
    /// Returns `BorrowError` when canonical Word identity cannot be accessed.
    pub fn add_sp(
        &mut self,
        wav_length: f64,
        add_phone: impl Into<String>,
    ) -> Result<(), HfaWordListError> {
        let add_phone = add_phone.into();
        if let Err(error) = self.add_sp_inner(wav_length, &add_phone) {
            if error.is_borrow_error() {
                return Err(error);
            }
            self.add_log(format!("ERROR in add_SP: {error}"));
        }
        Ok(())
    }

    /// Inserts `SP` gap Words using the legacy default text.
    ///
    /// # Errors
    ///
    /// Returns `BorrowError` when canonical Word identity cannot be accessed.
    pub fn add_sp_default(&mut self, wav_length: f64) -> Result<(), HfaWordListError> {
        self.add_sp(wav_length, DEFAULT_SP_PHONE)
    }

    fn add_sp_inner(&mut self, wav_length: f64, add_phone: &str) -> Result<(), HfaWordListError> {
        let source_count = self.entries.len();
        let mut candidates = Vec::new();

        let first = Self::word_at(&self.entries, 0, "start")?;
        let first_start = first.read(|word| word.start)?;
        if first_start > 0.0 {
            match Word::new(0.0, first.read(|word| word.start)?, add_phone, true) {
                Ok(word) => {
                    Self::append_into(&mut candidates, &mut self.log, WordHandle::new(word))?
                }
                Err(error) => self.add_log(format!("ERROR: {error}")),
            }
        }

        Self::append_into(&mut candidates, &mut self.log, first)?;
        for index in 1..source_count {
            let word = Self::word_at(&self.entries, index, "start")?;
            let word_start = word.read(|word| word.start)?;
            let candidate_end =
                Self::word_at(&candidates, candidates.len().wrapping_sub(1), "end")?
                    .read(|word| word.end)?;
            if word_start > candidate_end {
                let gap_start =
                    Self::word_at(&candidates, candidates.len().wrapping_sub(1), "end")?
                        .read(|word| word.end)?;
                let gap_end = word.read(|word| word.start)?;
                match Word::new(gap_start, gap_end, add_phone, true) {
                    Ok(word) => {
                        Self::append_into(&mut candidates, &mut self.log, WordHandle::new(word))?
                    }
                    Err(error) => self.add_log(format!("ERROR: {error}")),
                }
            }
            Self::append_into(&mut candidates, &mut self.log, word)?;
        }

        let original_last = Self::word_at(&self.entries, source_count.wrapping_sub(1), "end")?;
        if original_last.read(|word| word.end)? < wav_length {
            let trailing_start = original_last.read(|word| word.end)?;
            match Word::new(trailing_start, wav_length, add_phone, true) {
                Ok(word) => {
                    Self::append_into(&mut candidates, &mut self.log, WordHandle::new(word))?
                }
                Err(error) => self.add_log(format!("ERROR: {error}")),
            }
        }

        self.entries = candidates;
        let _ = self.check()?;
        Ok(())
    }

    /// Checks all Word and phoneme invariants in exact legacy failure order.
    ///
    /// The first invalid state appends one warning and returns `false`. The
    /// empty collection and fully valid state return `true`.
    ///
    /// # Errors
    ///
    /// Returns `BorrowError` when canonical Word identity cannot be accessed.
    pub fn check(&mut self) -> Result<bool, HfaWordListError> {
        let entry_count = self.entries.len();
        for index in 0..entry_count {
            let handle = match self.entries.get(index) {
                Some(WordListEntry::Word(handle)) => handle.clone(),
                Some(WordListEntry::Invalid(_)) => {
                    self.add_log(format!(
                        "WARNING: Element at index {index} is not a Word instance"
                    ));
                    return Ok(false);
                }
                None => unreachable!("entry count is fixed during check"),
            };
            let warning = handle.read(|word| {
                if word.start.partial_cmp(&word.end) != Some(Ordering::Less) {
                    return Some(format!(
                        "Word '{}' has invalid time order: start={}, end={}",
                        word.text,
                        python_float_string(word.start),
                        python_float_string(word.end)
                    ));
                }
                if word.phonemes.is_empty() {
                    return Some(format!("Word '{}' has no phonemes", word.text));
                }
                if word.phonemes[0].start != word.start {
                    return Some(format!(
                        "Word '{}' first phoneme start({}) != word start({})",
                        word.text,
                        python_float_string(word.phonemes[0].start),
                        python_float_string(word.start)
                    ));
                }
                if word.phonemes[word.phonemes.len() - 1].end != word.end {
                    return Some(format!(
                        "Word '{}' last phoneme end({}) != word end({})",
                        word.text,
                        python_float_string(word.phonemes[word.phonemes.len() - 1].end),
                        python_float_string(word.end)
                    ));
                }
                for (phone_index, phoneme) in word.phonemes.iter().enumerate() {
                    if phoneme.start.partial_cmp(&phoneme.end) != Some(Ordering::Less) {
                        return Some(format!(
                            "Word '{}' phoneme '{}' has invalid time order: start={}, end={}",
                            word.text,
                            phoneme.text,
                            python_float_string(phoneme.start),
                            python_float_string(phoneme.end)
                        ));
                    }
                    if let Some(next) = word.phonemes.get(phone_index + 1)
                        && phoneme.end != next.start
                    {
                        return Some(format!(
                            "Word '{}' phoneme '{}' end({}) != next phoneme '{}' start({})",
                            word.text,
                            phoneme.text,
                            python_float_string(phoneme.end),
                            next.text,
                            python_float_string(next.start)
                        ));
                    }
                }
                None
            })?;

            if let Some(warning) = warning {
                self.add_log(format!("WARNING: {warning}"));
                return Ok(false);
            }
        }

        for index in 0..entry_count.saturating_sub(1) {
            let current = Self::word_at(&self.entries, index, "end")?;
            let current_end = current.read(|word| word.end)?;
            let next = Self::word_at(&self.entries, index + 1, "start")?;
            let next_start = next.read(|word| word.start)?;
            if current_end != next_start {
                let current_text = current.read(|word| word.text.clone())?;
                let next_text = next.read(|word| word.text.clone())?;
                self.add_log(format!(
                    "WARNING: Word '{}' end({}) != next word '{}' start({})",
                    current_text,
                    python_float_string(current_end),
                    next_text,
                    python_float_string(next_start)
                ));
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn word_at(
        entries: &[WordListEntry],
        index: usize,
        attribute: &str,
    ) -> Result<WordHandle, HfaWordListError> {
        match entries
            .get(index)
            .ok_or_else(HfaWordListError::index_error)?
        {
            WordListEntry::Word(word) => Ok(word.clone()),
            WordListEntry::Invalid(_) => Err(HfaWordListError::invalid_attribute(attribute)),
        }
    }

    /// Subtracts one interval with legacy validation and evaluation order.
    ///
    /// # Errors
    ///
    /// Returns the exact legacy `ValueError` projection when either interval is
    /// not strictly increasing.
    pub fn remove_overlapping_intervals(
        raw_interval: (f64, f64),
        remove_interval: (f64, f64),
    ) -> Result<Vec<(f64, f64)>, HfaWordListError> {
        let (raw_start, raw_end) = raw_interval;
        let (remove_start, remove_end) = remove_interval;
        if raw_start.partial_cmp(&raw_end) != Some(Ordering::Less) {
            return Err(HfaWordListError::value_error(
                "raw_interval.start must be smaller than raw_interval.end",
            ));
        }
        if remove_start.partial_cmp(&remove_end) != Some(Ordering::Less) {
            return Err(HfaWordListError::value_error(
                "remove_interval.start must be smaller than remove_interval.end",
            ));
        }

        let overlap_start = python_max(raw_start, remove_start);
        let overlap_end = python_min(raw_end, remove_end);
        if overlap_start >= overlap_end {
            return Ok(vec![raw_interval]);
        }

        let mut result = Vec::with_capacity(2);
        if raw_start < overlap_start {
            result.push((raw_start, overlap_start));
        }
        if overlap_end < raw_end {
            result.push((overlap_end, raw_end));
        }
        Ok(result)
    }

    /// Adds an AP Word using legacy aliasing, subtraction, filtering, sorting,
    /// partial-mutation, and caught-error behavior.
    pub fn add_ap(
        &mut self,
        new_word: WordHandle,
        min_duration: f64,
    ) -> Result<(), HfaWordListError> {
        let (is_empty, text) =
            new_word.read(|word| (word.phonemes.is_empty(), word.text.clone()))?;
        if is_empty {
            self.add_log(format!("WARNING: {text} phonemes为空，非法word"));
            return Ok(());
        }

        if self.entries.is_empty() {
            self.append(new_word)?;
            return Ok(());
        }

        if self.overlapping_words(&new_word)?.is_empty() {
            self.append(new_word)?;
            if let Err(error) = self.sort_by_start() {
                if error.is_borrow_error() {
                    return Err(error);
                }
                self.add_log(format!("ERROR in add_AP: {error}"));
            }
            return Ok(());
        }

        let (new_start, new_end, new_text) =
            new_word.read(|word| (word.start, word.end, word.text.clone()))?;
        let mut ap_intervals = vec![(new_start, new_end)];
        for entry in &self.entries {
            if ap_intervals.is_empty() {
                continue;
            }
            let WordListEntry::Word(word) = entry else {
                self.add_log(format!(
                    "ERROR in add_AP: {}",
                    HfaWordListError::invalid_attribute("start")
                ));
                return Ok(());
            };
            let (word_start, word_end) = word.read(|word| (word.start, word.end))?;
            let mut next_intervals = Vec::new();
            for interval in ap_intervals {
                match Self::remove_overlapping_intervals(interval, (word_start, word_end)) {
                    Ok(residuals) => next_intervals.extend(residuals),
                    Err(error) => {
                        self.add_log(format!("ERROR in add_AP: {error}"));
                        return Ok(());
                    }
                }
            }
            ap_intervals = next_intervals;
        }

        ap_intervals.retain(|(start, end)| end - start >= min_duration);
        for (start, end) in ap_intervals {
            match Word::new(start, end, new_text.clone(), true) {
                Ok(word) => self.append(WordHandle::new(word))?,
                Err(error) => self.add_log(format!("ERROR: {error}")),
            }
        }
        if let Err(error) = self.sort_by_start() {
            if error.is_borrow_error() {
                return Err(error);
            }
            self.add_log(format!("ERROR in add_AP: {error}"));
        }
        Ok(())
    }

    /// Adds an AP Word using the legacy default minimum duration.
    pub fn add_ap_default(&mut self, new_word: WordHandle) -> Result<(), HfaWordListError> {
        self.add_ap(new_word, DEFAULT_AP_MIN_DURATION)
    }

    /// Returns flattened phoneme text in list and phoneme order.
    ///
    /// # Errors
    ///
    /// Returns the exact invalid-entry `AttributeError` projection.
    pub fn phoneme_texts(&self) -> Result<Vec<String>, HfaWordListError> {
        let mut phonemes = Vec::new();
        for entry in &self.entries {
            let WordListEntry::Word(word) = entry else {
                return Err(HfaWordListError::invalid_attribute("phonemes"));
            };
            phonemes.extend(word.read(|word| {
                word.phonemes
                    .iter()
                    .map(|phoneme| phoneme.text.clone())
                    .collect::<Vec<_>>()
            })?);
        }
        Ok(phonemes)
    }

    /// Returns Word start/end pairs in list order.
    ///
    /// # Errors
    ///
    /// Returns the exact invalid-entry `AttributeError` projection.
    pub fn intervals(&self) -> Result<Vec<(f64, f64)>, HfaWordListError> {
        self.entries
            .iter()
            .map(|entry| match entry {
                WordListEntry::Word(word) => word.read(|word| (word.start, word.end)),
                WordListEntry::Invalid(_) => Err(HfaWordListError::invalid_attribute("start")),
            })
            .collect()
    }

    /// Removes all slash-delimited language prefixes from stored phoneme text.
    ///
    /// # Errors
    ///
    /// Returns the exact invalid-entry `AttributeError` projection after any
    /// preceding valid entries have already been mutated.
    pub fn clear_language_prefix(&mut self) -> Result<(), HfaWordListError> {
        for entry in &self.entries {
            let WordListEntry::Word(word) = entry else {
                return Err(HfaWordListError::invalid_attribute("phonemes"));
            };
            word.write(|word| {
                for phoneme in &mut word.phonemes {
                    phoneme.text = phoneme.text.rsplit('/').next().unwrap().to_string();
                }
            })?;
        }
        Ok(())
    }

    fn add_log(&mut self, message: String) {
        self.log.push(message);
    }

    fn sort_by_start(&mut self) -> Result<(), HfaWordListError> {
        if self
            .entries
            .iter()
            .any(|entry| matches!(entry, WordListEntry::Invalid(_)))
        {
            return Err(HfaWordListError::invalid_attribute("start"));
        }
        let mut decorated = self
            .entries
            .iter()
            .map(|entry| match entry {
                WordListEntry::Word(word) => word.read(|word| PythonSortItem {
                    key: word.start,
                    entry: entry.clone(),
                }),
                WordListEntry::Invalid(_) => unreachable!("invalid entries rejected above"),
            })
            .collect::<Result<Vec<_>, _>>()?;
        python_list_sort(&mut decorated);
        self.entries = decorated.into_iter().map(|item| item.entry).collect();
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PythonSortItem {
    key: f64,
    entry: WordListEntry,
}

#[derive(Debug, Clone, Copy)]
struct PythonSortRun {
    start: usize,
    len: usize,
    power: usize,
}

#[derive(Debug)]
struct PythonMergeState {
    min_gallop: usize,
}

const PYTHON_MIN_GALLOP: usize = 7;

/// Safe Rust adaptation of CPython v3.12.13's list sort comparison schedule:
/// <https://github.com/python/cpython/blob/v3.12.13/Objects/listobject.c>.
/// Design reference:
/// <https://github.com/python/cpython/blob/v3.12.13/Objects/listsort.txt>.
///
/// This keeps natural-run discovery, stable binary insertion, powersort,
/// pre-merge gallops, and `merge_lo`/`merge_hi` galloping. The schedule matters
/// because Python float `<` is not a total order in the presence of NaN. Since
/// binary insertion is capped by a 64-element minrun, worst-case sorting is
/// O(n log n).
fn python_list_sort(items: &mut [PythonSortItem]) {
    let list_len = items.len();
    if list_len < 2 {
        return;
    }

    let min_run = python_min_run(list_len);
    let mut state = PythonMergeState {
        min_gallop: PYTHON_MIN_GALLOP,
    };
    let mut runs = Vec::<PythonSortRun>::new();
    let mut start = 0_usize;
    while start < list_len {
        let remaining = list_len - start;
        let (mut run_len, descending) = python_count_run(&items[start..]);
        if descending {
            items[start..start + run_len].reverse();
        }
        if run_len < min_run {
            let forced = remaining.min(min_run);
            python_binary_insertion_sort(&mut items[start..start + forced], run_len);
            run_len = forced;
        }

        python_found_new_run(items, &mut state, &mut runs, run_len, list_len);
        runs.push(PythonSortRun {
            start,
            len: run_len,
            power: 0,
        });
        start += run_len;
    }

    while runs.len() > 1 {
        let mut index = runs.len() - 2;
        if index > 0 && runs[index - 1].len < runs[index + 1].len {
            index -= 1;
        }
        python_merge_at(items, &mut state, &mut runs, index);
    }
}

fn python_count_run(items: &[PythonSortItem]) -> (usize, bool) {
    if items.len() == 1 {
        return (1, false);
    }
    let descending = items[1].key < items[0].key;
    let mut len = 2_usize;
    if descending {
        while len < items.len() && items[len].key < items[len - 1].key {
            len += 1;
        }
    } else {
        while len < items.len()
            && items[len].key.partial_cmp(&items[len - 1].key) != Some(Ordering::Less)
        {
            len += 1;
        }
    }
    (len, descending)
}

fn python_binary_insertion_sort(items: &mut [PythonSortItem], sorted: usize) {
    let mut start = sorted.max(1);
    while start < items.len() {
        let pivot = items[start].clone();
        let mut left = 0_usize;
        let mut right = start;
        while left < right {
            let middle = left + ((right - left) >> 1);
            if pivot.key < items[middle].key {
                right = middle;
            } else {
                left = middle + 1;
            }
        }
        items[left..=start].rotate_right(1);
        items[left] = pivot;
        start += 1;
    }
}

fn python_min_run(mut len: usize) -> usize {
    let mut shifted_bit = 0_usize;
    while len >= 64 {
        shifted_bit |= len & 1;
        len >>= 1;
    }
    len + shifted_bit
}

fn python_power(start: usize, first_len: usize, second_len: usize, total: usize) -> usize {
    let mut result = 0_usize;
    let mut left_midpoint = 2 * start + first_len;
    let mut right_midpoint = left_midpoint + first_len + second_len;
    loop {
        result += 1;
        if left_midpoint >= total {
            left_midpoint -= total;
            right_midpoint -= total;
        } else if right_midpoint >= total {
            return result;
        }
        left_midpoint <<= 1;
        right_midpoint <<= 1;
    }
}

fn python_found_new_run(
    items: &mut [PythonSortItem],
    state: &mut PythonMergeState,
    runs: &mut Vec<PythonSortRun>,
    next_len: usize,
    total: usize,
) {
    if runs.is_empty() {
        return;
    }
    let last = runs[runs.len() - 1];
    let power = python_power(last.start, last.len, next_len, total);
    while runs.len() > 1 && runs[runs.len() - 2].power > power {
        let index = runs.len() - 2;
        python_merge_at(items, state, runs, index);
    }
    runs.last_mut().unwrap().power = power;
}

fn python_merge_at(
    items: &mut [PythonSortItem],
    state: &mut PythonMergeState,
    runs: &mut Vec<PythonSortRun>,
    index: usize,
) {
    let left = runs[index];
    let right = runs[index + 1];
    debug_assert_eq!(left.start + left.len, right.start);
    runs[index].len = left.len + right.len;
    if index + 2 < runs.len() {
        runs[index + 1] = runs[index + 2];
    }
    runs.pop();

    let left_slice = &items[left.start..left.start + left.len];
    let left_skip = python_gallop_right(right_key(items, right.start), left_slice, 0);
    let merge_start = left.start + left_skip;
    let left_len = left.len - left_skip;
    if left_len == 0 {
        return;
    }

    let right_slice = &items[right.start..right.start + right.len];
    let right_len = python_gallop_left(
        items[merge_start + left_len - 1].key,
        right_slice,
        right.len - 1,
    );
    if right_len == 0 {
        return;
    }

    let merge = &mut items[merge_start..merge_start + left_len + right_len];
    if left_len <= right_len {
        python_merge_lo(merge, left_len, state);
    } else {
        python_merge_hi(merge, left_len, state);
    }
}

fn right_key(items: &[PythonSortItem], index: usize) -> f64 {
    items[index].key
}

fn python_gallop_left(key: f64, items: &[PythonSortItem], hint: usize) -> usize {
    debug_assert!(!items.is_empty() && hint < items.len());
    let mut last_offset = 0_usize;
    let mut offset = 1_usize;
    let (mut last, mut right): (isize, usize);

    if items[hint].key < key {
        let max_offset = items.len() - hint;
        while offset < max_offset && items[hint + offset].key < key {
            last_offset = offset;
            offset = (offset << 1) + 1;
        }
        offset = offset.min(max_offset);
        last = (last_offset + hint) as isize;
        right = offset + hint;
    } else {
        let max_offset = hint + 1;
        while offset < max_offset {
            if items[hint - offset].key < key {
                break;
            }
            last_offset = offset;
            offset = (offset << 1) + 1;
        }
        offset = offset.min(max_offset);
        let previous_last = last_offset;
        last = hint as isize - offset as isize;
        right = hint - previous_last;
    }

    last += 1;
    while last < right as isize {
        let middle = last as usize + ((right - last as usize) >> 1);
        if items[middle].key < key {
            last = middle as isize + 1;
        } else {
            right = middle;
        }
    }
    right
}

fn python_gallop_right(key: f64, items: &[PythonSortItem], hint: usize) -> usize {
    debug_assert!(!items.is_empty() && hint < items.len());
    let mut last_offset = 0_usize;
    let mut offset = 1_usize;
    let (mut last, mut right): (isize, usize);

    if key < items[hint].key {
        let max_offset = hint + 1;
        while offset < max_offset && key < items[hint - offset].key {
            last_offset = offset;
            offset = (offset << 1) + 1;
        }
        offset = offset.min(max_offset);
        let previous_last = last_offset;
        last = hint as isize - offset as isize;
        right = hint - previous_last;
    } else {
        let max_offset = items.len() - hint;
        while offset < max_offset {
            if key < items[hint + offset].key {
                break;
            }
            last_offset = offset;
            offset = (offset << 1) + 1;
        }
        offset = offset.min(max_offset);
        last = (last_offset + hint) as isize;
        right = offset + hint;
    }

    last += 1;
    while last < right as isize {
        let middle = last as usize + ((right - last as usize) >> 1);
        if key < items[middle].key {
            right = middle;
        } else {
            last = middle as isize + 1;
        }
    }
    right
}

fn copy_forward(destination: &mut [PythonSortItem], start: usize, source: &[PythonSortItem]) {
    destination[start..start + source.len()].clone_from_slice(source);
}

fn python_merge_lo(items: &mut [PythonSortItem], middle: usize, state: &mut PythonMergeState) {
    let left = items[..middle].to_vec();
    let right = items[middle..].to_vec();
    let (mut left_index, mut right_index, mut destination) = (0_usize, 0_usize, 0_usize);
    let (mut left_len, mut right_len) = (left.len(), right.len());

    items[destination] = right[right_index].clone();
    destination += 1;
    right_index += 1;
    right_len -= 1;
    if right_len == 0 {
        copy_forward(items, destination, &left[left_index..left_index + left_len]);
        return;
    }
    if left_len == 1 {
        copy_forward(
            items,
            destination,
            &right[right_index..right_index + right_len],
        );
        items[destination + right_len] = left[left_index].clone();
        return;
    }

    let mut min_gallop = state.min_gallop;
    loop {
        let (mut left_count, mut right_count) = (0_usize, 0_usize);
        loop {
            debug_assert!(left_len > 1 && right_len > 0);
            if right[right_index].key < left[left_index].key {
                items[destination] = right[right_index].clone();
                destination += 1;
                right_index += 1;
                right_len -= 1;
                right_count += 1;
                left_count = 0;
                if right_len == 0 {
                    copy_forward(items, destination, &left[left_index..left_index + left_len]);
                    return;
                }
                if right_count >= min_gallop {
                    break;
                }
            } else {
                items[destination] = left[left_index].clone();
                destination += 1;
                left_index += 1;
                left_len -= 1;
                left_count += 1;
                right_count = 0;
                if left_len == 1 {
                    copy_forward(
                        items,
                        destination,
                        &right[right_index..right_index + right_len],
                    );
                    items[destination + right_len] = left[left_index].clone();
                    return;
                }
                if left_count >= min_gallop {
                    break;
                }
            }
        }

        min_gallop += 1;
        loop {
            min_gallop = min_gallop.saturating_sub(usize::from(min_gallop > 1));
            state.min_gallop = min_gallop;

            left_count = python_gallop_right(
                right[right_index].key,
                &left[left_index..left_index + left_len],
                0,
            );
            if left_count > 0 {
                copy_forward(
                    items,
                    destination,
                    &left[left_index..left_index + left_count],
                );
                destination += left_count;
                left_index += left_count;
                left_len -= left_count;
                if left_len <= 1 {
                    break;
                }
            }
            items[destination] = right[right_index].clone();
            destination += 1;
            right_index += 1;
            right_len -= 1;
            if right_len == 0 {
                copy_forward(items, destination, &left[left_index..left_index + left_len]);
                return;
            }

            right_count = python_gallop_left(
                left[left_index].key,
                &right[right_index..right_index + right_len],
                0,
            );
            if right_count > 0 {
                copy_forward(
                    items,
                    destination,
                    &right[right_index..right_index + right_count],
                );
                destination += right_count;
                right_index += right_count;
                right_len -= right_count;
                if right_len == 0 {
                    copy_forward(items, destination, &left[left_index..left_index + left_len]);
                    return;
                }
            }
            items[destination] = left[left_index].clone();
            destination += 1;
            left_index += 1;
            left_len -= 1;
            if left_len == 1 {
                break;
            }
            if left_count < PYTHON_MIN_GALLOP && right_count < PYTHON_MIN_GALLOP {
                break;
            }
        }

        if left_len == 1 {
            copy_forward(
                items,
                destination,
                &right[right_index..right_index + right_len],
            );
            items[destination + right_len] = left[left_index].clone();
            return;
        }
        if left_len == 0 {
            copy_forward(
                items,
                destination,
                &right[right_index..right_index + right_len],
            );
            return;
        }
        min_gallop += 1;
        state.min_gallop = min_gallop;
    }
}

fn python_merge_hi(items: &mut [PythonSortItem], middle: usize, state: &mut PythonMergeState) {
    let left = items[..middle].to_vec();
    let right = items[middle..].to_vec();
    let (mut left_len, mut right_len) = (left.len(), right.len());
    let mut destination = items.len();

    destination -= 1;
    left_len -= 1;
    items[destination] = left[left_len].clone();
    if left_len == 0 {
        copy_forward(items, 0, &right[..right_len]);
        return;
    }
    if right_len == 1 {
        let first = destination - left_len;
        copy_forward(items, first, &left[..left_len]);
        items[first - 1] = right[0].clone();
        return;
    }

    let mut min_gallop = state.min_gallop;
    loop {
        let (mut left_count, mut right_count) = (0_usize, 0_usize);
        loop {
            debug_assert!(left_len > 0 && right_len > 1);
            destination -= 1;
            if right[right_len - 1].key < left[left_len - 1].key {
                left_len -= 1;
                items[destination] = left[left_len].clone();
                left_count += 1;
                right_count = 0;
                if left_len == 0 {
                    copy_forward(items, destination - right_len, &right[..right_len]);
                    return;
                }
                if left_count >= min_gallop {
                    break;
                }
            } else {
                right_len -= 1;
                items[destination] = right[right_len].clone();
                right_count += 1;
                left_count = 0;
                if right_len == 1 {
                    let first = destination - left_len;
                    copy_forward(items, first, &left[..left_len]);
                    items[first - 1] = right[0].clone();
                    return;
                }
                if right_count >= min_gallop {
                    break;
                }
            }
        }

        min_gallop += 1;
        loop {
            min_gallop = min_gallop.saturating_sub(usize::from(min_gallop > 1));
            state.min_gallop = min_gallop;

            let split =
                python_gallop_right(right[right_len - 1].key, &left[..left_len], left_len - 1);
            left_count = left_len - split;
            if left_count > 0 {
                destination -= left_count;
                copy_forward(items, destination, &left[split..left_len]);
                left_len -= left_count;
                if left_len == 0 {
                    copy_forward(items, destination - right_len, &right[..right_len]);
                    return;
                }
            }
            destination -= 1;
            right_len -= 1;
            items[destination] = right[right_len].clone();
            if right_len == 1 {
                break;
            }

            let split =
                python_gallop_left(left[left_len - 1].key, &right[..right_len], right_len - 1);
            right_count = right_len - split;
            if right_count > 0 {
                destination -= right_count;
                copy_forward(items, destination, &right[split..right_len]);
                right_len -= right_count;
                if right_len <= 1 {
                    break;
                }
            }
            destination -= 1;
            left_len -= 1;
            items[destination] = left[left_len].clone();
            if left_len == 0 {
                copy_forward(items, destination - right_len, &right[..right_len]);
                return;
            }
            if left_count < PYTHON_MIN_GALLOP && right_count < PYTHON_MIN_GALLOP {
                break;
            }
        }

        if right_len == 1 {
            let first = destination - left_len;
            copy_forward(items, first, &left[..left_len]);
            items[first - 1] = right[0].clone();
            return;
        }
        if right_len == 0 {
            copy_forward(items, destination - left_len, &left[..left_len]);
            return;
        }
        min_gallop += 1;
        state.min_gallop = min_gallop;
    }
}

fn python_max(first: f64, second: f64) -> f64 {
    if second > first { second } else { first }
}

fn python_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}

fn python_word_repr(word: &Word) -> String {
    let phonemes = word
        .phonemes
        .iter()
        .map(python_phoneme_repr)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Word(start={}, end={}, text={}, phonemes=[{}])",
        python_float_string(word.start),
        python_float_string(word.end),
        python_string_repr(&word.text),
        phonemes
    )
}

fn python_phoneme_repr(phoneme: &Phoneme) -> String {
    format!(
        "Phoneme(start={}, end={}, text={})",
        python_float_string(phoneme.start),
        python_float_string(phoneme.end),
        python_string_repr(&phoneme.text)
    )
}

fn python_string_repr(value: &str) -> String {
    crate::python_15_nonprintable::string_repr(value)
}

fn clamp_python_start(start: f64) -> f64 {
    if start.is_nan() || start <= 0.0 {
        0.0
    } else {
        start
    }
}

fn deliver_warning(message: String, log_list: Option<&mut Vec<String>>) -> Option<HfaWordWarning> {
    if let Some(log_list) = log_list {
        log_list.push(format!("WARNING: {message}"));
        None
    } else {
        Some(HfaWordWarning { message })
    }
}

fn python_float_string(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value == f64::INFINITY {
        return "inf".to_string();
    }
    if value == f64::NEG_INFINITY {
        return "-inf".to_string();
    }

    let rendered = format!("{value:?}");
    let Some(exponent_index) = rendered.find('e') else {
        return rendered;
    };
    let (mantissa, exponent) = rendered.split_at(exponent_index);
    let exponent = exponent[1..]
        .parse::<i32>()
        .expect("Rust float debug exponent is an integer");
    format!("{mantissa}e{exponent:+03}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use md5::{Digest, Md5};
    use serde_json::{Value, json};
    use std::collections::HashMap;

    const FIXTURES: &str = include_str!("../../../../fixtures/hfa_word_model_core.jsonl");
    const COLLECTION_FIXTURES: &str =
        include_str!("../../../../fixtures/hfa_wordlist_collection_ap_core.jsonl");
    const FINALIZE_FIXTURES: &str =
        include_str!("../../../../fixtures/hfa_wordlist_finalize_core.jsonl");

    fn parse_float(value: &Value) -> f64 {
        if let Some(value) = value.as_f64() {
            return value;
        }
        match value["$float"].as_str().unwrap() {
            "nan" => f64::NAN,
            "+inf" => f64::INFINITY,
            "-inf" => f64::NEG_INFINITY,
            "-0.0" => -0.0,
            other => panic!("unknown special float {other}"),
        }
    }

    fn encode_float(value: f64) -> Value {
        if value.is_nan() {
            return json!({"$float": "nan"});
        }
        if value == f64::INFINITY {
            return json!({"$float": "+inf"});
        }
        if value == f64::NEG_INFINITY {
            return json!({"$float": "-inf"});
        }
        if value == 0.0 && value.is_sign_negative() {
            return json!({"$float": "-0.0"});
        }
        json!(value)
    }

    fn encode_phoneme(phoneme: &Phoneme) -> Value {
        json!({
            "start": encode_float(phoneme.start),
            "end": encode_float(phoneme.end),
            "text": phoneme.text,
        })
    }

    fn encode_word(word: &Word) -> Value {
        json!({
            "start": encode_float(word.start),
            "end": encode_float(word.end),
            "text": word.text,
            "dur": encode_float(word.duration()),
            "phonemes": word.phonemes.iter().map(encode_phoneme).collect::<Vec<_>>(),
        })
    }

    fn parse_phoneme(value: &Value) -> Phoneme {
        let mut phoneme = Phoneme::new(
            parse_float(&value["start"]),
            parse_float(&value["end"]),
            value["text"].as_str().unwrap(),
        )
        .unwrap();
        if value["force_zero"].as_bool().unwrap_or(false) {
            phoneme.end = phoneme.start;
        }
        phoneme
    }

    fn parse_word(value: &Value) -> Word {
        let mut word = Word::new(
            parse_float(&value["start"]),
            parse_float(&value["end"]),
            value["text"].as_str().unwrap(),
            value["init_phoneme"].as_bool().unwrap_or(false),
        )
        .unwrap();
        if let Some(phonemes) = value["phonemes"].as_array() {
            word.phonemes = phonemes.iter().map(parse_phoneme).collect();
        }
        if !value["start_override"].is_null() {
            word.start = parse_float(&value["start_override"]);
        }
        word
    }

    fn encode_phoneme_constructor(item: &Value) -> Value {
        match Phoneme::new(
            parse_float(&item["start"]),
            parse_float(&item["end"]),
            item["text"].as_str().unwrap(),
        ) {
            Ok(phoneme) => json!({"ok": encode_phoneme(&phoneme)}),
            Err(error) => json!({
                "error": {"type": "ValueError", "message": error.to_string()}
            }),
        }
    }

    fn encode_word_constructor(item: &Value) -> Value {
        match Word::new(
            parse_float(&item["start"]),
            parse_float(&item["end"]),
            item["text"].as_str().unwrap(),
            item["init_phoneme"].as_bool().unwrap(),
        ) {
            Ok(word) => json!({"ok": encode_word(&word)}),
            Err(error) => json!({
                "error": {"type": "ValueError", "message": error.to_string()}
            }),
        }
    }

    fn encode_logged_mutations(case: &Value, append: bool) -> Value {
        let mut word = parse_word(&case["word"]);
        let mut logs = Vec::new();
        for item in case["phonemes"].as_array().unwrap() {
            let phoneme = parse_phoneme(item);
            if append {
                word.append_phoneme(phoneme, Some(&mut logs));
            } else {
                word.add_phoneme(phoneme, Some(&mut logs));
            }
        }
        json!({"word": encode_word(&word), "logs": logs})
    }

    fn encode_boundary_moves(case: &Value) -> Value {
        let mut word = parse_word(&case["word"]);
        let mut logs = Vec::new();
        for movement in case["moves"].as_array().unwrap() {
            let value = parse_float(&movement["value"]);
            match movement["kind"].as_str().unwrap() {
                "start" => {
                    word.move_start(value, Some(&mut logs)).unwrap();
                }
                "end" => {
                    word.move_end(value, Some(&mut logs)).unwrap();
                }
                other => panic!("unknown boundary move {other}"),
            }
        }
        json!({"word": encode_word(&word), "logs": logs})
    }

    fn encode_boundary_errors(case: &Value) -> Value {
        let results = case["moves"]
            .as_array()
            .unwrap()
            .iter()
            .map(|movement| {
                let mut word = parse_word(&case["word"]);
                let result = match movement["kind"].as_str().unwrap() {
                    "start" => word.move_start(parse_float(&movement["value"]), None),
                    "end" => word.move_end(parse_float(&movement["value"]), None),
                    other => panic!("unknown boundary move {other}"),
                };
                match result {
                    Ok(_) => json!({"ok": encode_word(&word)}),
                    Err(error) => json!({
                        "error": {
                            "type": error.exception_type(),
                            "message": error.to_string(),
                        }
                    }),
                }
            })
            .collect::<Vec<_>>();
        Value::Array(results)
    }

    fn encode_warning_sink(case: &Value) -> Value {
        let mut word = parse_word(&case["word"]);
        let mut warnings = Vec::new();
        for operation in case["operations"].as_array().unwrap() {
            let warning = match operation["kind"].as_str().unwrap() {
                "add" => word.add_phoneme(parse_phoneme(&operation["phoneme"]), None),
                "append" => word.append_phoneme(parse_phoneme(&operation["phoneme"]), None),
                "move_start" => word
                    .move_start(parse_float(&operation["value"]), None)
                    .unwrap(),
                "move_end" => word
                    .move_end(parse_float(&operation["value"]), None)
                    .unwrap(),
                other => panic!("unknown warning operation {other}"),
            };
            if let Some(warning) = warning {
                warnings.push(json!({
                    "category": warning.category(),
                    "message": warning.message,
                }));
            }
        }
        json!({"warnings": warnings, "word": encode_word(&word)})
    }

    fn parse_word_handle(value: &Value) -> WordHandle {
        WordHandle::new(parse_word(value))
    }

    fn parse_collection_entry(value: &Value) -> WordListEntry {
        if value["kind"] == "invalid" {
            WordListEntry::invalid(value["value"].as_str().unwrap())
        } else {
            WordListEntry::word(parse_word_handle(value))
        }
    }

    fn parse_word_list(seed: &Value) -> WordList {
        WordList::from_raw(
            seed.as_array()
                .unwrap()
                .iter()
                .map(parse_collection_entry)
                .collect(),
        )
    }

    fn encode_collection_word(word: &Word) -> Value {
        json!({
            "start": encode_float(word.start),
            "end": encode_float(word.end),
            "text": word.text,
            "phonemes": word.phonemes.iter().map(encode_phoneme).collect::<Vec<_>>(),
        })
    }

    fn encode_collection_entry(entry: &WordListEntry) -> Value {
        match entry {
            WordListEntry::Word(word) => {
                json!({
                    "kind": "word",
                    "word": encode_collection_word(&word.snapshot().unwrap()),
                })
            }
            WordListEntry::Invalid(value) => json!({"kind": "invalid", "value": value}),
        }
    }

    fn encode_collection_entries(words: &WordList) -> Value {
        Value::Array(
            words
                .entries()
                .iter()
                .map(encode_collection_entry)
                .collect(),
        )
    }

    fn encode_intervals(intervals: &[(f64, f64)]) -> Value {
        Value::Array(
            intervals
                .iter()
                .map(|(start, end)| json!([encode_float(*start), encode_float(*end)]))
                .collect(),
        )
    }

    fn encode_word_list_error(error: HfaWordListError) -> Value {
        json!({
            "error": {
                "type": error.exception_type(),
                "message": error.to_string(),
            }
        })
    }

    fn encode_append_sequence(case: &Value) -> Value {
        let mut words = WordList::new();
        for operation in case["operations"].as_array().unwrap() {
            words.append(parse_word_handle(operation)).unwrap();
        }
        json!({
            "entries": encode_collection_entries(&words),
            "phonemes": words.phoneme_texts().unwrap(),
            "intervals": encode_intervals(&words.intervals().unwrap()),
            "log": words.log(),
        })
    }

    fn encode_raw_seed_extend(case: &Value) -> Value {
        let mut words = parse_word_list(&case["seed"]);
        words.raw_extend(
            case["extend"]
                .as_array()
                .unwrap()
                .iter()
                .map(parse_collection_entry),
        );
        json!({"entries": encode_collection_entries(&words), "log": words.log()})
    }

    fn encode_overlap_scan(case: &Value) -> Value {
        let words = parse_word_list(&case["seed"]);
        let overlaps = case["queries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|query| {
                let query = parse_word_handle(query);
                Value::Array(
                    words
                        .overlapping_words(&query)
                        .unwrap()
                        .iter()
                        .map(|word| Value::String(word.snapshot().unwrap().text))
                        .collect(),
                )
            })
            .collect::<Vec<_>>();
        json!({"seed_entries": encode_collection_entries(&words), "overlaps": overlaps})
    }

    fn encode_log_lifecycle(case: &Value) -> Value {
        let mut words = WordList::from_raw(
            case["seed"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| WordListEntry::word(parse_word_handle(item)))
                .collect(),
        );
        for item in case["before_clear"].as_array().unwrap() {
            words.append(parse_word_handle(item)).unwrap();
        }
        let before = words.log();
        words.clear_log();
        let cleared = words.log();
        for item in case["after_clear"].as_array().unwrap() {
            words.append(parse_word_handle(item)).unwrap();
        }
        json!({"before": before, "cleared": cleared, "after": words.log()})
    }

    fn encode_remove_intervals(case: &Value) -> Value {
        Value::Array(
            case["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    let raw = (parse_float(&item["raw"][0]), parse_float(&item["raw"][1]));
                    let remove = (
                        parse_float(&item["remove"][0]),
                        parse_float(&item["remove"][1]),
                    );
                    match WordList::remove_overlapping_intervals(raw, remove) {
                        Ok(intervals) => json!({"ok": encode_intervals(&intervals)}),
                        Err(error) => encode_word_list_error(error),
                    }
                })
                .collect(),
        )
    }

    fn encode_add_ap(case: &Value) -> Value {
        let mut words = parse_word_list(&case["seed"]);
        for call in case["calls"].as_array().unwrap() {
            let word = parse_word_handle(&call["word"]);
            if call["min_dur"].is_null() {
                words.add_ap_default(word).unwrap();
            } else {
                words.add_ap(word, parse_float(&call["min_dur"])).unwrap();
            }
        }
        json!({"entries": encode_collection_entries(&words), "log": words.log()})
    }

    fn encode_add_ap_alias(case: &Value) -> Value {
        let mut words = parse_word_list(&case["seed"]);
        let new_word = parse_word_handle(&case["word"]);
        words
            .add_ap(new_word.clone(), parse_float(&case["min_dur"]))
            .unwrap();
        let stored_is_original = words.entries().iter().any(|entry| match entry {
            WordListEntry::Word(word) => word.same_identity(&new_word),
            WordListEntry::Invalid(_) => false,
        });
        new_word
            .set_text_and_all_phonemes(
                case["mutate_text"].as_str().unwrap(),
                case["mutate_phoneme_text"].as_str().unwrap(),
            )
            .unwrap();
        json!({
            "stored_is_original": stored_is_original,
            "entries_after_source_mutation": encode_collection_entries(&words),
            "log": words.log(),
        })
    }

    fn encode_sort_add_ap(case: &Value) -> Value {
        let mut words = parse_word_list(&case["seed"]);
        for call in case["calls"].as_array().unwrap() {
            words
                .add_ap(parse_word_handle(call), DEFAULT_AP_MIN_DURATION)
                .unwrap();
        }
        let order = words
            .entries()
            .iter()
            .map(|entry| match entry {
                WordListEntry::Word(word) => {
                    let word = word.snapshot().unwrap();
                    json!({"text": word.text, "start": encode_float(word.start)})
                }
                WordListEntry::Invalid(_) => unreachable!("sort fixture contains only words"),
            })
            .collect::<Vec<_>>();
        json!({"order": order, "log": words.log()})
    }

    fn encode_sort_key_corpus(case: &Value) -> Value {
        let mut items = case["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| PythonSortItem {
                key: parse_float(&item["key"]),
                entry: WordListEntry::Invalid(item["label"].as_str().unwrap().to_string()),
            })
            .collect::<Vec<_>>();
        python_list_sort(&mut items);
        let order = items
            .into_iter()
            .map(|item| match item.entry {
                WordListEntry::Invalid(label) => label,
                WordListEntry::Word(_) => unreachable!("key corpus stores labels"),
            })
            .collect::<Vec<_>>();
        json!({"order": order})
    }

    fn encode_unicode_printability_digest() -> Value {
        let mut digest = Md5::new();
        let mut scalar_count = 0_u64;
        let mut nonprintable_count = 0_u64;
        for codepoint in 0..=0x10ffff {
            let Some(character) = char::from_u32(codepoint) else {
                continue;
            };
            let nonprintable = crate::python_15_nonprintable::contains(character);
            digest.update([u8::from(nonprintable)]);
            scalar_count += 1;
            nonprintable_count += u64::from(nonprintable);
        }
        json!({
            "unidata_version": "15.0.0",
            "scalar_count": scalar_count,
            "nonprintable_count": nonprintable_count,
            "md5": format!("{:x}", digest.finalize()),
        })
    }

    fn encode_projection_errors(case: &Value) -> Value {
        let mut results = serde_json::Map::new();
        for operation in case["operations"].as_array().unwrap() {
            let operation = operation.as_str().unwrap();
            let mut words = parse_word_list(&case["seed"]);
            let result = match operation {
                "phonemes" => match words.phoneme_texts() {
                    Ok(value) => json!({"ok": value}),
                    Err(error) => encode_word_list_error(error),
                },
                "intervals" => match words.intervals() {
                    Ok(value) => json!({"ok": encode_intervals(&value)}),
                    Err(error) => encode_word_list_error(error),
                },
                "clear_language_prefix" => match words.clear_language_prefix() {
                    Ok(()) => json!({"ok": null}),
                    Err(error) => encode_word_list_error(error),
                },
                other => panic!("unknown projection operation {other}"),
            };
            results.insert(operation.to_string(), result);
        }
        Value::Object(results)
    }

    fn encode_prefix_partial_error(case: &Value) -> Value {
        let mut words = parse_word_list(&case["seed"]);
        let result = match words.clear_language_prefix() {
            Ok(()) => json!({"ok": null}),
            Err(error) => encode_word_list_error(error),
        };
        json!({"result": result, "entries": encode_collection_entries(&words)})
    }

    fn encode_projections_and_prefix(case: &Value) -> Value {
        let mut words = parse_word_list(&case["seed"]);
        let before_phonemes = words.phoneme_texts().unwrap();
        let intervals = words.intervals().unwrap();
        words.clear_language_prefix().unwrap();
        json!({
            "before_phonemes": before_phonemes,
            "intervals": encode_intervals(&intervals),
            "after_phonemes": words.phoneme_texts().unwrap(),
            "entries": encode_collection_entries(&words),
        })
    }

    #[derive(Default)]
    struct FinalizeIdentityState {
        source_ids: Vec<(WordHandle, String)>,
        generated_ids: Vec<(WordHandle, String)>,
    }

    fn parse_finalize_word(value: &Value) -> Word {
        let mut word = Word::new(
            parse_float(&value["start"]),
            parse_float(&value["end"]),
            value["text"].as_str().unwrap(),
            value["init_phoneme"].as_bool().unwrap_or(false),
        )
        .unwrap();
        if !value["start_override"].is_null() {
            word.start = parse_float(&value["start_override"]);
        }
        if !value["end_override"].is_null() {
            word.end = parse_float(&value["end_override"]);
        }
        if value["empty"].as_bool().unwrap_or(false) {
            word.phonemes.clear();
        }
        if let Some(specifications) = value["phoneme_specs"].as_array() {
            word.phonemes = specifications
                .iter()
                .map(|specification| Phoneme {
                    start: parse_float(&specification["start"]),
                    end: parse_float(&specification["end"]),
                    text: specification["text"].as_str().unwrap().to_string(),
                })
                .collect();
        }
        if let Some(overrides) = value["phoneme_overrides"].as_array() {
            for item in overrides {
                let phoneme = &mut word.phonemes[item["index"].as_u64().unwrap() as usize];
                if !item["start"].is_null() {
                    phoneme.start = parse_float(&item["start"]);
                }
                if !item["end"].is_null() {
                    phoneme.end = parse_float(&item["end"]);
                }
                if let Some(text) = item["text"].as_str() {
                    phoneme.text = text.to_string();
                }
            }
        }
        word
    }

    fn parse_finalize_entry(
        value: &Value,
        aliases: &mut HashMap<String, WordHandle>,
        identities: &mut FinalizeIdentityState,
    ) -> WordListEntry {
        if let Some(reuse_id) = value["reuse_id"].as_str() {
            return WordListEntry::word(aliases[reuse_id].clone());
        }
        if value["kind"] == "invalid" {
            return WordListEntry::invalid(value["value"].as_str().unwrap());
        }

        let handle = WordHandle::new(parse_finalize_word(value));
        if let Some(fixture_id) = value["fixture_id"].as_str() {
            aliases.insert(fixture_id.to_string(), handle.clone());
            identities
                .source_ids
                .push((handle.clone(), fixture_id.to_string()));
        }
        WordListEntry::word(handle)
    }

    fn parse_finalize_word_list(case: &Value) -> (WordList, FinalizeIdentityState) {
        let mut aliases = HashMap::new();
        let mut identities = FinalizeIdentityState::default();
        let entries = case["seed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| parse_finalize_entry(item, &mut aliases, &mut identities))
            .collect();
        let mut words = WordList::from_raw(entries);
        if let Some(pre_log) = case["pre_log"].as_array() {
            words.log = pre_log
                .iter()
                .map(|item| item.as_str().unwrap().to_string())
                .collect();
        }
        (words, identities)
    }

    fn finalize_identity(handle: &WordHandle, state: &mut FinalizeIdentityState) -> String {
        if let Some((_, identity)) = state
            .source_ids
            .iter()
            .find(|(source, _)| source.same_identity(handle))
        {
            return identity.clone();
        }
        if let Some((_, identity)) = state
            .generated_ids
            .iter()
            .find(|(generated, _)| generated.same_identity(handle))
        {
            return identity.clone();
        }
        let identity = format!("new{}", state.generated_ids.len());
        state.generated_ids.push((handle.clone(), identity.clone()));
        identity
    }

    fn encode_finalize_entries(words: &WordList, state: &mut FinalizeIdentityState) -> Value {
        Value::Array(
            words
                .entries()
                .iter()
                .map(|entry| match entry {
                    WordListEntry::Invalid(value) => json!({"kind": "invalid", "value": value}),
                    WordListEntry::Word(handle) => {
                        let identity = finalize_identity(handle, state);
                        let word = handle.snapshot().unwrap();
                        json!({
                            "kind": "word",
                            "value": {
                                "identity": identity,
                                "start": encode_float(word.start),
                                "end": encode_float(word.end),
                                "text": word.text,
                                "dur": encode_float(word.duration()),
                                "phonemes": word.phonemes.iter().map(encode_phoneme).collect::<Vec<_>>(),
                            }
                        })
                    }
                })
                .collect(),
        )
    }

    fn run_finalize_case(case: &Value) -> Value {
        let (mut words, mut identities) = parse_finalize_word_list(case);
        let returns: Vec<Value> = match case["kind"].as_str().unwrap() {
            "fill_small_gaps" => case["calls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|call| {
                    let wav_length = parse_float(&call["wav_length"]);
                    if call["gap_length"].is_null() {
                        words.fill_small_gaps_default(wav_length).unwrap();
                    } else {
                        words
                            .fill_small_gaps(wav_length, parse_float(&call["gap_length"]))
                            .unwrap();
                    }
                    Value::Null
                })
                .collect(),
            "add_SP" => case["calls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|call| {
                    let wav_length = parse_float(&call["wav_length"]);
                    if call["add_phone"].is_null() {
                        words.add_sp_default(wav_length).unwrap();
                    } else {
                        words
                            .add_sp(wav_length, call["add_phone"].as_str().unwrap())
                            .unwrap();
                    }
                    Value::Null
                })
                .collect(),
            "clear_extend_check" => {
                let mut aliases = identities
                    .source_ids
                    .iter()
                    .map(|(handle, identity)| (identity.clone(), handle.clone()))
                    .collect::<HashMap<_, _>>();
                case["actions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|action| match action["kind"].as_str().unwrap() {
                        "add_SP" => {
                            let wav_length = parse_float(&action["wav_length"]);
                            if action["add_phone"].is_null() {
                                words.add_sp_default(wav_length).unwrap();
                            } else {
                                words
                                    .add_sp(wav_length, action["add_phone"].as_str().unwrap())
                                    .unwrap();
                            }
                            Value::Null
                        }
                        "clear" => {
                            words.clear_entries();
                            Value::Null
                        }
                        "extend" => {
                            let entries = action["entries"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|item| {
                                    parse_finalize_entry(item, &mut aliases, &mut identities)
                                })
                                .collect::<Vec<_>>();
                            words.raw_extend(entries);
                            Value::Null
                        }
                        "check" => Value::Bool(words.check().unwrap()),
                        other => panic!("unknown finalizer action {other}"),
                    })
                    .collect()
            }
            "check" => (0..case["repeat"].as_u64().unwrap_or(1))
                .map(|_| Value::Bool(words.check().unwrap()))
                .collect(),
            other => panic!("unknown finalizer fixture kind {other}"),
        };

        json!({
            "returns": returns,
            "entries": encode_finalize_entries(&words, &mut identities),
            "log": words.log(),
        })
    }

    fn assert_json_close(actual: &Value, expected: &Value, context: &str) {
        match (actual, expected) {
            (Value::Number(left), Value::Number(right)) => {
                let left = left.as_f64().unwrap();
                let right = right.as_f64().unwrap();
                assert!(
                    (left - right).abs() <= 1e-12,
                    "{context}: {left} != {right}"
                );
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
    fn hfa_word_model_follows_parity_fixture_table() {
        for (line_index, line) in FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "phoneme_constructor" => Value::Array(
                    case["items"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(encode_phoneme_constructor)
                        .collect(),
                ),
                "word_constructor" => Value::Array(
                    case["items"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(encode_word_constructor)
                        .collect(),
                ),
                "add_phoneme" => encode_logged_mutations(&case, false),
                "append_phoneme" => encode_logged_mutations(&case, true),
                "move_boundaries" => encode_boundary_moves(&case),
                "move_boundary_errors" => encode_boundary_errors(&case),
                "warning_sink" => encode_warning_sink(&case),
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
    fn hfa_wordlist_collection_follows_parity_fixture_table() {
        for (line_index, line) in COLLECTION_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = match case["kind"].as_str().unwrap() {
                "unicode_printability_digest" => encode_unicode_printability_digest(),
                "sort_key_corpus" => encode_sort_key_corpus(&case),
                "append_sequence" => encode_append_sequence(&case),
                "raw_seed_extend" => encode_raw_seed_extend(&case),
                "overlap_scan" => encode_overlap_scan(&case),
                "log_lifecycle" => encode_log_lifecycle(&case),
                "remove_intervals" => encode_remove_intervals(&case),
                "add_ap" => encode_add_ap(&case),
                "add_ap_alias" => encode_add_ap_alias(&case),
                "sort_add_ap" => encode_sort_add_ap(&case),
                "projection_errors" => encode_projection_errors(&case),
                "prefix_partial_error" => encode_prefix_partial_error(&case),
                "projections_and_prefix" => encode_projections_and_prefix(&case),
                other => panic!("unknown collection fixture kind {other}"),
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
    fn hfa_wordlist_finalize_follows_parity_fixture_table() {
        let mut case_count = 0;
        for (line_index, line) in FINALIZE_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            case_count += 1;
            let case: Value = serde_json::from_str(line).unwrap();
            let actual = run_finalize_case(&case);
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
        assert_eq!(case_count, 53);
    }

    #[test]
    fn hfa_wordlist_finalize_preserves_structured_integrity_errors() {
        let mapped = HfaWordListError::from(HfaWordMutationError);
        assert_eq!(mapped.exception_type(), "IndexError");
        assert_eq!(mapped.to_string(), "list index out of range");

        for operation in ["fill", "add_sp", "check"] {
            let source = WordHandle::new(Word::new(0.0, 1.0, "source", true).unwrap());
            let mut words = WordList::from_raw(vec![WordListEntry::word(source.clone())]);
            let borrow = source.inner.borrow_mut();
            let error = match operation {
                "fill" => words.fill_small_gaps_default(1.0).unwrap_err(),
                "add_sp" => words.add_sp_default(1.0).unwrap_err(),
                "check" => words.check().unwrap_err(),
                _ => unreachable!(),
            };
            assert_eq!(error.exception_type(), "BorrowError");
            assert_eq!(words.log(), "");
            drop(borrow);
            assert_eq!(words.entries().len(), 1);
            assert!(matches!(words.entries()[0], WordListEntry::Word(_)));
        }
    }

    #[test]
    fn hfa_wordlist_finalize_scales_and_short_circuits_invalid_first() {
        const ENTRY_COUNT: usize = 10_000;

        let valid_entries = (0..ENTRY_COUNT)
            .map(|index| {
                let start = index as f64;
                WordListEntry::word(WordHandle::new(
                    Word::new(start, start + 1.0, format!("word-{index}"), true).unwrap(),
                ))
            })
            .collect();
        let mut valid_words = WordList::from_raw(valid_entries);
        valid_words
            .fill_small_gaps_default(ENTRY_COUNT as f64)
            .unwrap();
        assert!(valid_words.check().unwrap());
        assert_eq!(valid_words.log(), "");

        let mut invalid_entries = Vec::with_capacity(ENTRY_COUNT + 1);
        invalid_entries.push(WordListEntry::invalid("invalid-first"));
        invalid_entries.extend(
            (0..ENTRY_COUNT).map(|index| WordListEntry::invalid(format!("tail-{index:05}"))),
        );
        let mut invalid_words = WordList::from_raw(invalid_entries);

        invalid_words.fill_small_gaps_default(1.0).unwrap();
        assert_eq!(
            invalid_words.log(),
            "ERROR in fill_small_gaps: 'str' object has no attribute 'start'"
        );
        invalid_words.clear_log();

        invalid_words.add_sp_default(1.0).unwrap();
        assert_eq!(
            invalid_words.log(),
            "ERROR in add_SP: 'str' object has no attribute 'start'"
        );
        assert_eq!(invalid_words.entries().len(), ENTRY_COUNT + 1);
        invalid_words.clear_log();

        assert!(!invalid_words.check().unwrap());
        assert_eq!(
            invalid_words.log(),
            "WARNING: Element at index 0 is not a Word instance"
        );
    }

    #[test]
    fn python_float_error_formatting_normalizes_exponents() {
        assert_eq!(python_float_string(1e20), "1e+20");
        assert_eq!(python_float_string(1e-7), "1e-07");
        assert_eq!(python_float_string(-0.0), "-0.0");
    }

    #[test]
    fn word_handles_preserve_aliases_without_exposing_borrow_guards() {
        let source = WordHandle::new(Word::new(0.0, 1.0, "source", true).unwrap());
        let mut words = WordList::new();
        words.append(source.clone()).unwrap();

        source
            .set_text_and_all_phonemes("source-mutated", "source-phone")
            .unwrap();
        let stored = match &words.entries()[0] {
            WordListEntry::Word(word) => word.clone(),
            WordListEntry::Invalid(_) => unreachable!("test stores a Word"),
        };
        assert!(stored.same_identity(&source));
        assert_eq!(stored.snapshot().unwrap().text, "source-mutated");

        stored.set_text("entry-alias-mutated").unwrap();
        assert_eq!(source.snapshot().unwrap().text, "entry-alias-mutated");
        assert_eq!(words.intervals().unwrap(), vec![(0.0, 1.0)]);
    }
}
