//! Quantization policy helpers.
//!
//! This module mirrors selected policy behavior from
//! `inference/quant/quantization.py` without changing the Python runtime owner.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

const DPQ_INTERNAL_GRID: i64 = 30;
const DPQ_SEGMENT_SHIFT_CANDIDATES: [i64; 3] = [0, 60, 120];
const DPQ_WEIGHTS: [f64; 6] = [0.08971, 0.025064, 0.235416, 0.038973, 0.580321, 0.030516];
const DPQ_SEGMENT_CENTER_WEIGHT: f64 = 0.02;
const DPQ_SEGMENT_SWITCH_PENALTY: f64 = 1.0;
const DPQ_SEGMENT_TIE_DELAY_BONUS: f64 = 0.4;
const DPQ_SEGMENT_ZERO_GAP_BONUS: f64 = 0.3;
const DPQ_SEGMENT_LONG_DELAY_BONUS: f64 = 0.2;
const DPQ_SEGMENT_LONG_THRESHOLD: i64 = 240;
const DPQ_SEGMENT_MAX_NOTES: usize = 16;
const BEAT_TICKS: i64 = 480;
const BAYES_CANDIDATE_RADIUS: i64 = 3;
const BAYES_MOTIF_MIN_COUNT: usize = 6;
const BAYES_MOTIF_PHASE_WEIGHT: f64 = 0.0;
const BAYES_MOTIF_DURATION_WEIGHT: f64 = 0.05;
const BAYES_MOTIF_GAP_WEIGHT: f64 = 0.0;
const BAYES_SEGMENT_CENTER_WEIGHT: f64 = 0.03;
const BAYES_SEGMENT_MAX_NOTES: usize = 48;
const BAYES_LATE_PULLBACK_MIN_PHASE_FACTOR: f64 = 0.5;
const BAYES_LATE_PULLBACK_MAX_SPREAD_FACTOR: f64 = 0.5;
const BAYES_LATE_PULLBACK_MIN_NOTES: usize = 4;
const BAYES_LATE_PULLBACK_WEIGHT: f64 = 0.08;
const BAYES_MAX_START_SHIFT_FACTOR: f64 = 1.0;
const BAYES_MAX_START_SHIFT_FLOOR: i64 = 60;
const BAYES_MAX_START_SHIFT_CAP: i64 = 180;
const BAYES_MAX_END_SHIFT_FACTOR: f64 = 1.0;
const BAYES_MAX_END_SHIFT_FLOOR: i64 = 90;
const BAYES_MAX_END_SHIFT_CAP: i64 = 240;
const BAYES_HALF_GRID_MAX_DUR_FACTOR: f64 = 0.375;
const BAYES_DURATION_PRIOR_WEIGHT: f64 = 0.015;
const DURATION_CANDIDATE_MULTIPLIERS: [i64; 10] = [1, 2, 3, 4, 6, 8, 12, 16, 24, 32];
const GAP_CANDIDATE_MULTIPLIERS: [i64; 7] = [0, 1, 2, 3, 4, 6, 8];

#[derive(Debug, Clone, Copy, PartialEq)]
struct AsymWeights {
    start_early: f64,
    start_late: f64,
    end_early: f64,
    end_late: f64,
    gap: f64,
    dur: f64,
    grid120: f64,
    grid480: f64,
}

const DPQ_DEFAULT_ASYM: AsymWeights = AsymWeights {
    start_early: 2.0,
    start_late: 0.5,
    end_early: 1.5,
    end_late: 0.75,
    gap: 0.75,
    dur: 1.0,
    grid120: 1.0,
    grid480: 1.0,
};

const BAYES_ASYM: AsymWeights = AsymWeights {
    start_early: 1.4,
    start_late: 0.65,
    end_early: 1.3,
    end_late: 0.9,
    gap: 0.75,
    dur: 1.0,
    grid120: 1.0,
    grid480: 1.0,
};

/// One note row accepted by the simple grid quantizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleGridNote<'a> {
    /// The onset.
    pub onset: f64,
    /// The offset.
    pub offset: f64,
    /// The pitch.
    pub pitch: f64,
    /// The lyric.
    pub lyric: &'a str,
}

/// Raw note timing fields used by later quantization algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawNotePair<'a> {
    /// The raw start.
    pub raw_start: i64,
    /// The raw end.
    pub raw_end: i64,
    /// The raw dur.
    pub raw_dur: i64,
    /// The lyrics.
    pub lyrics: &'a str,
}

/// Raw note timing fields with the gap from the previous raw note end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GapAnnotatedNotePair<'a> {
    /// The raw start.
    pub raw_start: i64,
    /// The raw end.
    pub raw_end: i64,
    /// The raw dur.
    pub raw_dur: i64,
    /// The lyrics.
    pub lyrics: &'a str,
    /// The raw gap.
    pub raw_gap: i64,
}

#[derive(Debug, Clone, PartialEq)]
struct SegmentCenterOption {
    center: i64,
    seq: Vec<(i64, i64)>,
    cost: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BayesPrior {
    count: usize,
    strength: f64,
    preferred_dur: Option<i64>,
    preferred_gap: Option<i64>,
    preferred_phase: Option<i64>,
}

impl BayesPrior {
    fn fallback() -> Self {
        Self {
            count: 1,
            strength: 0.0,
            preferred_dur: None,
            preferred_gap: None,
            preferred_phase: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BayesCostContext {
    step: i64,
    prev_end: Option<i64>,
    prev_raw_end: Option<i64>,
    segment_center: f64,
    segment_center_weight: f64,
}

/// Returns whether quantization should run for a mode and step.
pub fn should_apply_quantization(mode: Option<&str>, quantization_step: i64) -> bool {
    let mode = match mode {
        Some(value) if !value.is_empty() => value.to_lowercase(),
        _ => "simple".to_string(),
    };
    if mode == "dp" {
        return true;
    }
    quantization_step > 0
}

/// Quantizes notes using Python `_quantize_notes_simple` semantics.
///
/// This pre-promotion helper assumes finite note timings and positive finite
/// tempo. A future bridge must validate or map invalid numeric inputs before
/// routing production calls to Rust.
pub fn quantize_notes_simple(notes: &mut [SimpleGridNote<'_>], tempo: f64, quantization_step: i64) {
    if quantization_step <= 0 || notes.is_empty() {
        return;
    }

    notes.sort_by(|left, right| {
        left.onset
            .partial_cmp(&right.onset)
            .unwrap_or(Ordering::Equal)
    });

    let orig_onsets = notes.iter().map(|note| note.onset).collect::<Vec<_>>();
    let orig_offsets = notes.iter().map(|note| note.offset).collect::<Vec<_>>();
    let step = quantization_step;

    let mut q_onsets = orig_onsets
        .iter()
        .map(|&onset| {
            let ticks = ticks_from_sec(onset, tempo);
            round_half_even(ticks as f64 / step as f64) as i64 * step
        })
        .collect::<Vec<_>>();

    for index in 1..q_onsets.len() {
        if q_onsets[index] <= q_onsets[index - 1] {
            q_onsets[index] = q_onsets[index - 1] + step;
        }
    }

    let mut q_offsets = Vec::with_capacity(notes.len());
    for index in 0..notes.len() {
        let ticks = ticks_from_sec(orig_offsets[index], tempo);
        let mut q_ticks = round_half_even(ticks as f64 / step as f64) as i64 * step;

        if index < notes.len() - 1 && (orig_offsets[index] - orig_onsets[index + 1]).abs() < 1e-3 {
            q_ticks = q_onsets[index + 1];
        }

        if q_ticks <= q_onsets[index] {
            q_ticks = q_onsets[index] + step;
        }

        if index < notes.len() - 1 && q_ticks > q_onsets[index + 1] {
            q_ticks = q_onsets[index + 1];
        }

        q_offsets.push(q_ticks);
    }

    let denominator = tempo * 8.0;
    for (note, (&onset_tick, &offset_tick)) in
        notes.iter_mut().zip(q_onsets.iter().zip(q_offsets.iter()))
    {
        note.onset = onset_tick as f64 / denominator;
        note.offset = offset_tick as f64 / denominator;
    }
}

/// Quantizes notes using Python `_quantize_notes_smart` duration-DP semantics.
///
/// This pre-promotion helper assumes finite note timings and positive finite
/// tempo when quantization runs. A future bridge must validate or map invalid
/// numeric inputs before routing production calls to Rust.
pub fn quantize_notes_smart(notes: &mut [SimpleGridNote<'_>], tempo: f64, quantization_step: i64) {
    if quantization_step <= 0 || notes.is_empty() {
        return;
    }

    notes.sort_by(|left, right| {
        left.onset
            .partial_cmp(&right.onset)
            .unwrap_or(Ordering::Equal)
    });

    let orig_onsets = notes
        .iter()
        .map(|note| ticks_from_sec(note.onset, tempo))
        .collect::<Vec<_>>();
    let orig_offsets = notes
        .iter()
        .map(|note| ticks_from_sec(note.offset, tempo))
        .collect::<Vec<_>>();
    let raw_durs = orig_onsets
        .iter()
        .zip(orig_offsets.iter())
        .map(|(&onset, &offset)| (offset - onset).max(1))
        .collect::<Vec<_>>();
    let max_raw = raw_durs.iter().copied().max().unwrap_or(quantization_step);

    let candidates = build_duration_candidates(quantization_step, max_raw);
    let note_count = notes.len();
    let candidate_count = candidates.len();
    let mut dp = vec![f64::INFINITY; note_count * candidate_count];
    let mut prev = vec![-1_isize; note_count * candidate_count];
    let dur_pref_penalty = candidates
        .iter()
        .map(|candidate| {
            if matches!(candidate / quantization_step, 1 | 2 | 4 | 8 | 16) {
                0.0
            } else {
                0.08 * quantization_step as f64
            }
        })
        .collect::<Vec<_>>();

    for (index, &candidate) in candidates.iter().enumerate() {
        dp[index] = (raw_durs[0] - candidate).abs() as f64 + dur_pref_penalty[index];
    }

    for note_index in 1..note_count {
        for (candidate_index, &candidate) in candidates.iter().enumerate() {
            let local_cost =
                (raw_durs[note_index] - candidate).abs() as f64 + dur_pref_penalty[candidate_index];
            let mut best_prev = 0_usize;
            let mut best_cost = f64::INFINITY;

            for (prev_index, &prev_candidate) in candidates.iter().enumerate() {
                let jump_cost = (prev_candidate - candidate).abs() as f64 * 0.08;
                let total =
                    dp[(note_index - 1) * candidate_count + prev_index] + jump_cost + local_cost;
                if total < best_cost {
                    best_cost = total;
                    best_prev = prev_index;
                }
            }

            let index = note_index * candidate_count + candidate_index;
            dp[index] = best_cost;
            prev[index] = best_prev as isize;
        }
    }

    let last_row_start = (note_count - 1) * candidate_count;
    let mut best_last = 0_usize;
    let mut best_last_cost = f64::INFINITY;
    for candidate_index in 0..candidate_count {
        let cost = dp[last_row_start + candidate_index];
        if cost < best_last_cost {
            best_last_cost = cost;
            best_last = candidate_index;
        }
    }

    let mut q_durs = vec![0_i64; note_count];
    q_durs[note_count - 1] = candidates[best_last];
    let mut candidate_index = best_last as isize;
    for note_index in (1..note_count).rev() {
        let prev_index = prev[note_index * candidate_count + candidate_index as usize];
        candidate_index = if prev_index < 0 { 0 } else { prev_index };
        q_durs[note_index - 1] = candidates[candidate_index as usize];
    }

    let mut q_onsets = vec![0_i64; note_count];
    let mut q_offsets = vec![0_i64; note_count];
    q_onsets[0] = round_half_even(orig_onsets[0] as f64 / quantization_step as f64) as i64
        * quantization_step;

    for note_index in 0..note_count {
        q_offsets[note_index] = q_onsets[note_index] + quantization_step.max(q_durs[note_index]);
        if note_index < note_count - 1 {
            let raw_rest = (orig_onsets[note_index + 1] - orig_offsets[note_index]).max(0);
            let q_rest = if (raw_rest as f64) < quantization_step as f64 * 0.5 {
                0
            } else {
                round_half_even(raw_rest as f64 / quantization_step as f64) as i64
                    * quantization_step
            };
            q_onsets[note_index + 1] = q_offsets[note_index] + q_rest;
        }
    }

    let denominator = tempo * 8.0;
    for (note, (&onset_tick, &offset_tick)) in
        notes.iter_mut().zip(q_onsets.iter().zip(q_offsets.iter()))
    {
        note.onset = onset_tick as f64 / denominator;
        note.offset = offset_tick as f64 / denominator;
    }
}

/// Quantizes notes using Python `_quantize_notes_phrase_hybrid` semantics.
///
/// This is the pre-promotion implementation for the `dp` mode path. It remains
/// outside the Python runtime until a later promotion unit chooses a bridge.
///
/// # Panics
///
/// Panics only if the internal dynamic-programming backtracking state is empty
/// after a candidate path has been selected.
pub fn quantize_notes_phrase_dp(
    notes: &mut [SimpleGridNote<'_>],
    tempo: f64,
    quantization_step: i64,
) {
    if notes.is_empty() {
        return;
    }

    notes.sort_by(|left, right| {
        left.onset
            .partial_cmp(&right.onset)
            .unwrap_or(Ordering::Equal)
    });

    let grid_step = resolve_dp_grid_step(quantization_step);
    let center_candidates = resolve_segment_shift_candidates(grid_step);
    let orig_onsets = notes
        .iter()
        .map(|note| ticks_from_sec(note.onset, tempo))
        .collect::<Vec<_>>();
    let orig_offsets = notes
        .iter()
        .map(|note| ticks_from_sec(note.offset, tempo))
        .collect::<Vec<_>>();
    let pairs = notes
        .iter()
        .zip(orig_onsets.iter().zip(orig_offsets.iter()))
        .map(|(note, (&onset, &offset))| {
            build_note_pair(onset, (onset + 1).max(offset), Some(note.lyric))
        })
        .collect::<Vec<_>>();
    let segments = segment_split_indices(&pairs, grid_step);

    let segment_candidates = segments
        .iter()
        .map(|&(start, end)| {
            center_candidates
                .iter()
                .map(|&center| {
                    let (seq, cost) = decode_segment_with_center(
                        &pairs[start..end],
                        center,
                        grid_step,
                        &DPQ_DEFAULT_ASYM,
                    );
                    SegmentCenterOption { center, seq, cost }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut seg_dp: Vec<Vec<f64>> = Vec::with_capacity(segment_candidates.len());
    let mut seg_back: Vec<Vec<Option<usize>>> = Vec::with_capacity(segment_candidates.len());
    for (segment_index, options) in segment_candidates.iter().enumerate() {
        let mut cur_cost = Vec::with_capacity(options.len());
        let mut cur_back = Vec::with_capacity(options.len());

        for option in options {
            if segment_index == 0 {
                cur_cost.push(option.cost);
                cur_back.push(None);
                continue;
            }

            let mut best_cost = f64::INFINITY;
            let mut best_prev = None;
            for (prev_index, &prev_cost) in seg_dp[segment_index - 1].iter().enumerate() {
                let prev_center = segment_candidates[segment_index - 1][prev_index].center;
                let mut trans = if prev_center == option.center {
                    0.0
                } else {
                    DPQ_SEGMENT_SWITCH_PENALTY
                };

                if segment_index >= 2 && seg_back[segment_index - 1][prev_index].is_some() {
                    trans += 0.08 * (option.center - prev_center).abs() as f64;
                }

                let total = prev_cost + option.cost + trans;
                if total < best_cost {
                    best_cost = total;
                    best_prev = Some(prev_index);
                }
            }

            cur_cost.push(best_cost);
            cur_back.push(best_prev);
        }

        seg_dp.push(cur_cost);
        seg_back.push(cur_back);
    }

    let last_segment = segment_candidates.len() - 1;
    let mut last_index = 0_usize;
    let mut last_cost = f64::INFINITY;
    for (index, &cost) in seg_dp[last_segment].iter().enumerate() {
        if cost < last_cost {
            last_cost = cost;
            last_index = index;
        }
    }

    let mut chosen = vec![last_index];
    for segment_index in (1..segment_candidates.len()).rev() {
        let prev_index = seg_back[segment_index][*chosen.last().unwrap()].unwrap_or(0);
        chosen.push(prev_index);
    }
    chosen.reverse();

    let mut final_ticks = Vec::with_capacity(notes.len());
    for (segment_index, &option_index) in chosen.iter().enumerate() {
        final_ticks.extend(
            segment_candidates[segment_index][option_index]
                .seq
                .iter()
                .copied(),
        );
    }

    let fixed = repair_overlapping_ticks(final_ticks, grid_step);
    let denominator = tempo * 8.0;
    for (note, (start_tick, end_tick)) in notes.iter_mut().zip(fixed) {
        note.onset = start_tick as f64 / denominator;
        note.offset = end_tick as f64 / denominator;
    }
}

/// Quantizes notes using Python `_quantize_notes_bayesian` semantics.
///
/// This is a pre-promotion library implementation. Production Python callers
/// continue to use the legacy implementation until a later promotion unit
/// chooses and verifies a bridge.
pub fn quantize_notes_bayesian(
    notes: &mut [SimpleGridNote<'_>],
    tempo: f64,
    quantization_step: i64,
) {
    if quantization_step <= 0 || notes.is_empty() {
        return;
    }

    notes.sort_by(|left, right| {
        left.onset
            .partial_cmp(&right.onset)
            .unwrap_or(Ordering::Equal)
    });

    let grid_step = quantization_step;
    let orig_onsets = notes
        .iter()
        .map(|note| ticks_from_sec(note.onset, tempo))
        .collect::<Vec<_>>();
    let orig_offsets = notes
        .iter()
        .map(|note| ticks_from_sec(note.offset, tempo))
        .collect::<Vec<_>>();
    let raw_pairs = notes
        .iter()
        .zip(orig_onsets.iter().zip(orig_offsets.iter()))
        .map(|(note, (&onset, &offset))| {
            build_note_pair(onset, (onset + 1).max(offset), Some(note.lyric))
        })
        .collect::<Vec<_>>();
    let pairs = annotate_pairs_with_gap(&raw_pairs);

    let max_raw = pairs
        .iter()
        .map(|pair| pair.raw_dur)
        .max()
        .unwrap_or(grid_step);
    let max_gap = pairs.iter().map(|pair| pair.raw_gap).max().unwrap_or(0);
    let duration_candidates = build_duration_candidates(grid_step, max_raw);
    let gap_candidates = build_gap_candidates(grid_step, max_gap);
    let priors =
        build_piece_specific_priors(&pairs, grid_step, &duration_candidates, &gap_candidates);
    let segments = segment_split_indices_bayesian(&pairs, grid_step);

    let mut final_ticks = Vec::with_capacity(notes.len());
    for (start, end) in segments {
        let (seq, _) = decode_segment_bayesian(&pairs[start..end], &priors[start..end], grid_step);
        final_ticks.extend(seq);
    }

    let fixed = repair_overlapping_ticks(final_ticks, grid_step);
    let denominator = tempo * 8.0;
    for (note, (start_tick, end_tick)) in notes.iter_mut().zip(fixed) {
        note.onset = start_tick as f64 / denominator;
        note.offset = end_tick as f64 / denominator;
    }
}

/// Resolves the internal DP grid step used when the public step is disabled.
pub fn resolve_dp_grid_step(quantization_step: i64) -> i64 {
    if quantization_step > 0 {
        quantization_step
    } else {
        DPQ_INTERNAL_GRID
    }
}

/// Returns segment shift candidates for phrase DP quantization.
pub fn resolve_segment_shift_candidates(grid_step: i64) -> Vec<i64> {
    if grid_step <= DPQ_INTERNAL_GRID {
        return DPQ_SEGMENT_SHIFT_CANDIDATES.to_vec();
    }

    let mut values = BTreeSet::new();
    values.insert(0);
    values.insert(grid_step / 4);
    values.insert(grid_step / 2);
    values.into_iter().filter(|value| *value >= 0).collect()
}

/// Selects the first candidate with the smallest absolute distance to `value`.
pub fn nearest_candidate(value: f64, candidates: &[i64]) -> Option<i64> {
    let mut best: Option<(i64, f64)> = None;
    for &candidate in candidates {
        let distance = (candidate as f64 - value).abs();
        if best.is_none_or(|(_, best_distance)| distance < best_distance) {
            best = Some((candidate, distance));
        }
    }
    best.map(|(candidate, _)| candidate)
}

/// Computes circular distance for a positive modulo.
///
/// # Panics
///
/// Panics when `modulo` is not positive.
pub fn mod_distance(a: i64, b: i64, modulo: i64) -> i64 {
    assert!(modulo > 0, "modulo must be positive");

    let modulo = modulo as i128;
    let diff = ((a as i128 - b as i128).abs() % modulo) as i64;
    diff.min(modulo as i64 - diff)
}

/// Computes the distance from `x` to the nearest positive grid step.
///
/// # Panics
///
/// Panics when `step` is not positive.
pub fn dist_grid(x: i64, step: i64) -> i64 {
    assert!(step > 0, "step must be positive");

    let remainder = x.rem_euclid(step);
    remainder.min(if remainder == 0 { 0 } else { step - remainder })
}

/// Builds sorted unique candidate tick values around `raw_tick`.
///
/// # Panics
///
/// Panics when `step` is not positive.
pub fn candidate_values(raw_tick: i64, radius: i64, step: i64) -> Vec<i64> {
    assert!(step > 0, "step must be positive");
    if radius < 0 {
        return Vec::new();
    }

    let center = round_half_even(raw_tick as f64 / step as f64) as i64;
    let mut values = BTreeSet::new();
    for offset in -radius..=radius {
        values.insert((center + offset) * step);
    }
    values.into_iter().collect()
}

/// Builds the raw note pair dictionary equivalent used by Python quantization.
pub fn build_note_pair<'a>(
    tick_onset: i64,
    tick_offset: i64,
    lyrics: Option<&'a str>,
) -> RawNotePair<'a> {
    RawNotePair {
        raw_start: tick_onset,
        raw_end: tick_offset,
        raw_dur: (tick_offset - tick_onset).max(1),
        lyrics: lyrics.unwrap_or(""),
    }
}

/// Annotates raw note pairs with the non-negative gap from the previous end.
pub fn annotate_pairs_with_gap<'a>(pairs: &[RawNotePair<'a>]) -> Vec<GapAnnotatedNotePair<'a>> {
    let mut annotated = Vec::with_capacity(pairs.len());
    let mut previous_end: Option<i64> = None;

    for pair in pairs {
        let raw_gap = previous_end.map_or(0, |end| (pair.raw_start - end).max(0));
        annotated.push(GapAnnotatedNotePair {
            raw_start: pair.raw_start,
            raw_end: pair.raw_end,
            raw_dur: pair.raw_dur,
            lyrics: pair.lyrics,
            raw_gap,
        });
        previous_end = Some(pair.raw_end);
    }

    annotated
}

/// Builds sorted unique start/end candidate pairs with `end > start`.
pub fn build_candidate_pairs(pair: &RawNotePair<'_>, radius: i64, step: i64) -> Vec<(i64, i64)> {
    let starts = candidate_values(pair.raw_start, radius, step);
    let ends = candidate_values(pair.raw_end, radius, step);
    let mut candidates = BTreeSet::new();

    for start in starts {
        for &end in &ends {
            if end > start {
                candidates.insert((start, end));
            }
        }
    }

    candidates.into_iter().collect()
}

fn local_cost_asym(
    cand_start: i64,
    cand_end: i64,
    pair: &RawNotePair<'_>,
    prev_end: Option<i64>,
    prev_raw_end: Option<i64>,
    asym: &AsymWeights,
) -> f64 {
    let early_start = (pair.raw_start - cand_start).max(0);
    let late_start = (cand_start - pair.raw_start).max(0);
    let early_end = (pair.raw_end - cand_end).max(0);
    let late_end = (cand_end - pair.raw_end).max(0);
    let raw_gap = prev_raw_end.map_or(0, |raw_end| pair.raw_start - raw_end);
    let pred_gap = prev_end.map_or(0, |end| cand_start - end);

    let f1 = DPQ_WEIGHTS[0]
        * (asym.start_early * early_start as f64 + asym.start_late * late_start as f64);
    let f2 = DPQ_WEIGHTS[1] * (asym.end_early * early_end as f64 + asym.end_late * late_end as f64);
    let f3 = DPQ_WEIGHTS[2] * asym.gap * (pred_gap - raw_gap).abs() as f64;
    let f4 = DPQ_WEIGHTS[3] * asym.dur * ((cand_end - cand_start) - pair.raw_dur).abs() as f64;
    let f5 = DPQ_WEIGHTS[4] * asym.grid120 * dist_grid(cand_start, 120) as f64;
    let f6 = DPQ_WEIGHTS[5] * asym.grid480 * dist_grid(cand_start, 480) as f64;

    f1 + f2 + f3 + f4 + f5 + f6
}

fn segment_split_indices(pairs: &[RawNotePair<'_>], step: i64) -> Vec<(usize, usize)> {
    if pairs.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut start = 0_usize;
    let split_gap = step.max(DPQ_INTERNAL_GRID);

    for index in 1..pairs.len() {
        let previous = &pairs[index - 1];
        let current = &pairs[index];
        let raw_gap = current.raw_start - previous.raw_end;
        let previous_tie = previous.lyrics == "-";
        let current_tie = current.lyrics == "-";

        let has_large_gap = raw_gap >= split_gap;
        let has_positive_non_tie_gap = raw_gap > 0 && !previous_tie && !current_tie;
        let reached_segment_max = (index - start) >= DPQ_SEGMENT_MAX_NOTES && raw_gap >= 0;
        let should_split = has_large_gap || has_positive_non_tie_gap || reached_segment_max;

        if should_split {
            segments.push((start, index));
            start = index;
        }
    }

    segments.push((start, pairs.len()));
    segments
}

fn center_adjustment(
    pair: &RawNotePair<'_>,
    cand_start: i64,
    prev_raw_end: Option<i64>,
    center: i64,
) -> f64 {
    let shift_now = cand_start - pair.raw_start;
    let penalty = DPQ_SEGMENT_CENTER_WEIGHT * (shift_now - center).abs() as f64;
    if center <= 0 {
        return penalty;
    }

    let mut bonus = 0.0;
    if pair.lyrics == "-" {
        bonus += DPQ_SEGMENT_TIE_DELAY_BONUS;
    }
    let raw_gap = prev_raw_end.map_or(0, |raw_end| pair.raw_start - raw_end);
    if raw_gap == 0 {
        bonus += DPQ_SEGMENT_ZERO_GAP_BONUS;
    }
    if pair.raw_dur >= DPQ_SEGMENT_LONG_THRESHOLD {
        bonus += DPQ_SEGMENT_LONG_DELAY_BONUS;
    }
    if center >= 120 {
        bonus *= 1.15;
    }

    penalty - bonus
}

fn decode_segment_with_center(
    pairs: &[RawNotePair<'_>],
    center: i64,
    grid_step: i64,
    asym: &AsymWeights,
) -> (Vec<(i64, i64)>, f64) {
    if pairs.is_empty() {
        return (Vec::new(), 0.0);
    }

    let cand_lists = pairs
        .iter()
        .map(|pair| build_candidate_pairs(pair, 3, grid_step))
        .collect::<Vec<_>>();
    let mut dp: Vec<Vec<f64>> = Vec::with_capacity(pairs.len());
    let mut back: Vec<Vec<Option<usize>>> = Vec::with_capacity(pairs.len());

    for (pair_index, pair) in pairs.iter().enumerate() {
        let mut cur_cost = Vec::with_capacity(cand_lists[pair_index].len());
        let mut cur_back = Vec::with_capacity(cand_lists[pair_index].len());
        let prev_raw_end = if pair_index == 0 {
            None
        } else {
            Some(pairs[pair_index - 1].raw_end)
        };

        for &(cand_start, cand_end) in &cand_lists[pair_index] {
            let center_cost = center_adjustment(pair, cand_start, prev_raw_end, center);
            if pair_index == 0 {
                cur_cost.push(
                    local_cost_asym(cand_start, cand_end, pair, None, None, asym) + center_cost,
                );
                cur_back.push(None);
                continue;
            }

            let mut best_cost = f64::INFINITY;
            let mut best_prev = None;
            for (prev_index, &(prev_start, prev_end)) in
                cand_lists[pair_index - 1].iter().enumerate()
            {
                let _ = prev_start;
                let total = dp[pair_index - 1][prev_index]
                    + local_cost_asym(
                        cand_start,
                        cand_end,
                        pair,
                        Some(prev_end),
                        prev_raw_end,
                        asym,
                    )
                    + center_cost;
                if total < best_cost {
                    best_cost = total;
                    best_prev = Some(prev_index);
                }
            }

            cur_cost.push(best_cost);
            cur_back.push(best_prev);
        }

        dp.push(cur_cost);
        back.push(cur_back);
    }

    let last_pair = pairs.len() - 1;
    let mut last_index = 0_usize;
    let mut total_cost = f64::INFINITY;
    for (index, &cost) in dp[last_pair].iter().enumerate() {
        if cost < total_cost {
            total_cost = cost;
            last_index = index;
        }
    }

    let mut indices = vec![last_index];
    for pair_index in (1..pairs.len()).rev() {
        let prev_index = back[pair_index][*indices.last().unwrap()].unwrap_or(0);
        indices.push(prev_index);
    }
    indices.reverse();

    let seq = indices
        .iter()
        .enumerate()
        .map(|(pair_index, &candidate_index)| cand_lists[pair_index][candidate_index])
        .collect::<Vec<_>>();

    (seq, total_cost)
}

fn repair_overlapping_ticks<I>(final_ticks: I, grid_step: i64) -> Vec<(i64, i64)>
where
    I: IntoIterator<Item = (i64, i64)>,
{
    let mut fixed = Vec::new();
    for (mut start_tick, mut end_tick) in final_ticks {
        if let Some(&(_, previous_end)) = fixed.last() {
            if start_tick < previous_end {
                start_tick = previous_end;
            }
        }
        if end_tick <= start_tick {
            end_tick = start_tick + grid_step;
        }
        fixed.push((start_tick, end_tick));
    }
    fixed
}

fn raw_pair_from_gap<'a>(pair: &GapAnnotatedNotePair<'a>) -> RawNotePair<'a> {
    RawNotePair {
        raw_start: pair.raw_start,
        raw_end: pair.raw_end,
        raw_dur: pair.raw_dur,
        lyrics: pair.lyrics,
    }
}

fn resolve_bayes_shift_limit(step: i64, factor: f64, floor: i64, cap: i64) -> i64 {
    cap.min(floor.max(round_half_even(step as f64 * factor) as i64))
}

fn filter_bayes_candidate_pairs(
    pair: &RawNotePair<'_>,
    candidates: &[(i64, i64)],
    step: i64,
) -> Vec<(i64, i64)> {
    let start_limit = resolve_bayes_shift_limit(
        step,
        BAYES_MAX_START_SHIFT_FACTOR,
        BAYES_MAX_START_SHIFT_FLOOR,
        BAYES_MAX_START_SHIFT_CAP,
    );
    let end_limit = resolve_bayes_shift_limit(
        step,
        BAYES_MAX_END_SHIFT_FACTOR,
        BAYES_MAX_END_SHIFT_FLOOR,
        BAYES_MAX_END_SHIFT_CAP,
    );

    let filtered = candidates
        .iter()
        .copied()
        .filter(|&(start, end)| {
            (start - pair.raw_start).abs() <= start_limit && (end - pair.raw_end).abs() <= end_limit
        })
        .collect::<Vec<_>>();
    if !filtered.is_empty() {
        return filtered;
    }

    let mut best: Option<((i64, i64), i64, i64)> = None;
    for &(start, end) in candidates {
        let shift_distance = (start - pair.raw_start).abs() + (end - pair.raw_end).abs();
        let duration_distance = ((end - start) - pair.raw_dur).abs();
        if best.is_none_or(|(_, best_shift, best_duration)| {
            (shift_distance, duration_distance) < (best_shift, best_duration)
        }) {
            best = Some(((start, end), shift_distance, duration_distance));
        }
    }
    best.map_or_else(Vec::new, |(candidate, _, _)| vec![candidate])
}

fn build_bayes_candidate_pairs(pair: &RawNotePair<'_>, step: i64) -> Vec<(i64, i64)> {
    let fine_step = if step > 30 { 30.max(step / 2) } else { step };
    let max_half_grid_dur =
        1.max(round_half_even(step as f64 * BAYES_HALF_GRID_MAX_DUR_FACTOR) as i64);
    let candidate_step = if fine_step < step && pair.raw_dur <= max_half_grid_dur {
        fine_step
    } else {
        step
    };
    let candidates = build_candidate_pairs(pair, BAYES_CANDIDATE_RADIUS, candidate_step);
    filter_bayes_candidate_pairs(pair, &candidates, step)
}

fn median_i64(values: &[i64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[middle] as f64
    } else {
        (sorted[middle - 1] as f64 + sorted[middle] as f64) / 2.0
    }
}

fn mean_abs_distance(values: &[f64], center: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values
        .iter()
        .map(|value| (value - center).abs())
        .sum::<f64>()
        / values.len() as f64
}

fn estimate_segment_phase_center(pairs: &[GapAnnotatedNotePair<'_>], step: i64) -> (f64, f64) {
    if pairs.is_empty() {
        return (0.0, 0.0);
    }

    let raw_shifts = pairs
        .iter()
        .map(|pair| {
            round_half_even(pair.raw_start as f64 / step as f64) as i64 * step - pair.raw_start
        })
        .collect::<Vec<_>>();
    let center = median_i64(&raw_shifts);
    let raw_shift_values = raw_shifts
        .iter()
        .map(|&value| value as f64)
        .collect::<Vec<_>>();
    let spread = mean_abs_distance(&raw_shift_values, center);
    let spread_scale = (step as f64 * 0.4).max(1.0);
    let strength = (1.0 - (spread / spread_scale).min(1.0)).max(0.0);

    let phases = pairs
        .iter()
        .map(|pair| pair.raw_start.rem_euclid(step))
        .collect::<Vec<_>>();
    let phase_center = median_i64(&phases);
    let phase_values = phases.iter().map(|&value| value as f64).collect::<Vec<_>>();
    let phase_spread = mean_abs_distance(&phase_values, phase_center);

    if pairs.len() >= BAYES_LATE_PULLBACK_MIN_NOTES
        && phase_center >= step as f64 * BAYES_LATE_PULLBACK_MIN_PHASE_FACTOR
        && phase_spread <= step as f64 * BAYES_LATE_PULLBACK_MAX_SPREAD_FACTOR
    {
        let pullback_scale = (step as f64 * 0.35).max(1.0);
        let pullback_strength = (1.0 - (phase_spread / pullback_scale).min(1.0)).max(0.0);
        return (
            -phase_center,
            BAYES_LATE_PULLBACK_WEIGHT * strength.max(pullback_strength),
        );
    }

    (center, BAYES_SEGMENT_CENTER_WEIGHT * strength)
}

fn segment_split_indices_bayesian(
    pairs: &[GapAnnotatedNotePair<'_>],
    step: i64,
) -> Vec<(usize, usize)> {
    if pairs.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut start = 0_usize;
    let split_gap = (step * 3).max(240);

    for index in 1..pairs.len() {
        let raw_gap = pairs[index].raw_start - pairs[index - 1].raw_end;
        if raw_gap >= split_gap || (index - start) >= BAYES_SEGMENT_MAX_NOTES {
            segments.push((start, index));
            start = index;
        }
    }

    segments.push((start, pairs.len()));
    segments
}

fn metrical_position_penalty(tick: i64, step: i64) -> f64 {
    let position = tick.rem_euclid(BEAT_TICKS);
    if position == 0 {
        0.0
    } else if position % 240 == 0 {
        0.04 * step as f64
    } else if position % 120 == 0 {
        0.1 * step as f64
    } else if step <= 60 && position % 60 == 0 {
        0.2 * step as f64
    } else if step <= 30 && position % 30 == 0 {
        0.3 * step as f64
    } else {
        0.42 * step as f64
    }
}

fn note_value_penalty(duration_tick: i64, step: i64) -> f64 {
    let multiple = 1.max(round_half_even(duration_tick as f64 / step.max(1) as f64) as i64);
    match multiple {
        1 | 2 | 4 | 8 | 16 | 32 => 0.0,
        3 | 6 | 12 | 24 => 0.05 * step as f64,
        _ => 0.16 * step as f64,
    }
}

fn preferred_sv_duration(raw_duration_tick: i64, step: i64) -> Option<i64> {
    if raw_duration_tick
        <= 1.max(round_half_even(step as f64 * BAYES_HALF_GRID_MAX_DUR_FACTOR) as i64)
    {
        return None;
    }

    let ratio = raw_duration_tick as f64 / step.max(1) as f64;
    let thresholds = [
        (1.5, 1_i64),
        (2.625, 2),
        (3.375, 3),
        (4.625, 4),
        (5.625, 5),
        (7.0, 6),
    ];
    for (upper_ratio, multiple) in thresholds {
        if ratio <= upper_ratio {
            return Some(multiple * step);
        }
    }
    Some(step.max(round_half_even(ratio) as i64 * step))
}

fn bayes_prior_key(pair: &GapAnnotatedNotePair<'_>, step: i64) -> (i64, i64, bool) {
    (
        1.max(round_half_even(pair.raw_dur as f64 / step.max(1) as f64) as i64),
        round_half_even(pair.raw_gap as f64 / step.max(1) as f64) as i64,
        pair.lyrics == "-",
    )
}

fn phase_candidates(step: i64) -> Vec<i64> {
    let mut candidates = Vec::new();
    let mut value = 0_i64;
    while value < BEAT_TICKS {
        candidates.push(value);
        value += step;
    }
    candidates
}

fn build_piece_specific_priors(
    pairs: &[GapAnnotatedNotePair<'_>],
    step: i64,
    duration_candidates: &[i64],
    gap_candidates: &[i64],
) -> Vec<BayesPrior> {
    let mut grouped: BTreeMap<(i64, i64, bool), Vec<GapAnnotatedNotePair<'_>>> = BTreeMap::new();
    for pair in pairs {
        grouped
            .entry(bayes_prior_key(pair, step))
            .or_default()
            .push(*pair);
    }

    let mut prior_by_key = BTreeMap::new();
    for (key, items) in grouped {
        let count = items.len();
        if count < BAYES_MOTIF_MIN_COUNT {
            continue;
        }

        let dur_values = items.iter().map(|pair| pair.raw_dur).collect::<Vec<_>>();
        let gap_values = items.iter().map(|pair| pair.raw_gap).collect::<Vec<_>>();
        let phase_values = items
            .iter()
            .map(|pair| pair.raw_start.rem_euclid(BEAT_TICKS))
            .collect::<Vec<_>>();
        let phase_candidates = phase_candidates(step);
        prior_by_key.insert(
            key,
            BayesPrior {
                count,
                strength: (0.18 + 0.12 * (count - BAYES_MOTIF_MIN_COUNT) as f64).min(0.65),
                preferred_dur: nearest_candidate(median_i64(&dur_values), duration_candidates),
                preferred_gap: nearest_candidate(median_i64(&gap_values), gap_candidates),
                preferred_phase: nearest_candidate(median_i64(&phase_values), &phase_candidates),
            },
        );
    }

    pairs
        .iter()
        .map(|pair| {
            prior_by_key
                .get(&bayes_prior_key(pair, step))
                .copied()
                .unwrap_or_else(BayesPrior::fallback)
        })
        .collect()
}

fn bayes_local_cost(
    cand_start: i64,
    cand_end: i64,
    pair: &GapAnnotatedNotePair<'_>,
    prior: &BayesPrior,
    context: BayesCostContext,
) -> f64 {
    let raw_pair = raw_pair_from_gap(pair);
    let duration_tick = cand_end - cand_start;
    let mut cost = local_cost_asym(
        cand_start,
        cand_end,
        &raw_pair,
        context.prev_end,
        context.prev_raw_end,
        &BAYES_ASYM,
    );
    cost += metrical_position_penalty(cand_start, context.step);
    cost += 0.3 * note_value_penalty(duration_tick, context.step);

    if let Some(preferred_sv_dur) = preferred_sv_duration(pair.raw_dur, context.step) {
        cost += BAYES_DURATION_PRIOR_WEIGHT * (duration_tick - preferred_sv_dur).abs() as f64;
    }
    if context.segment_center_weight > 0.0 {
        cost += context.segment_center_weight
            * ((cand_start - pair.raw_start) as f64 - context.segment_center).abs();
    }

    if prior.strength > 0.0 {
        if let (Some(preferred_dur), Some(preferred_gap), Some(preferred_phase)) = (
            prior.preferred_dur,
            prior.preferred_gap,
            prior.preferred_phase,
        ) {
            let cand_gap = context.prev_end.map_or(0, |end| (cand_start - end).max(0));
            cost += prior.strength
                * BAYES_MOTIF_DURATION_WEIGHT
                * (duration_tick - preferred_dur).abs() as f64;
            cost +=
                prior.strength * BAYES_MOTIF_GAP_WEIGHT * (cand_gap - preferred_gap).abs() as f64;
            cost += prior.strength
                * BAYES_MOTIF_PHASE_WEIGHT
                * mod_distance(
                    cand_start.rem_euclid(BEAT_TICKS),
                    preferred_phase,
                    BEAT_TICKS,
                ) as f64;
        }
    }

    cost
}

fn decode_segment_bayesian(
    pairs: &[GapAnnotatedNotePair<'_>],
    priors: &[BayesPrior],
    grid_step: i64,
) -> (Vec<(i64, i64)>, f64) {
    if pairs.is_empty() {
        return (Vec::new(), 0.0);
    }

    let cand_lists = pairs
        .iter()
        .map(|pair| build_bayes_candidate_pairs(&raw_pair_from_gap(pair), grid_step))
        .collect::<Vec<_>>();
    let (segment_center, segment_center_weight) = estimate_segment_phase_center(pairs, grid_step);
    let mut dp: Vec<Vec<f64>> = Vec::with_capacity(pairs.len());
    let mut back: Vec<Vec<Option<usize>>> = Vec::with_capacity(pairs.len());

    for (pair_index, pair) in pairs.iter().enumerate() {
        let mut cur_cost = Vec::with_capacity(cand_lists[pair_index].len());
        let mut cur_back = Vec::with_capacity(cand_lists[pair_index].len());
        let prev_raw_end = if pair_index == 0 {
            None
        } else {
            Some(pairs[pair_index - 1].raw_end)
        };

        for &(cand_start, cand_end) in &cand_lists[pair_index] {
            if pair_index == 0 {
                cur_cost.push(bayes_local_cost(
                    cand_start,
                    cand_end,
                    pair,
                    &priors[pair_index],
                    BayesCostContext {
                        step: grid_step,
                        prev_end: None,
                        prev_raw_end: None,
                        segment_center,
                        segment_center_weight,
                    },
                ));
                cur_back.push(None);
                continue;
            }

            let non_overlapping = cand_lists[pair_index - 1]
                .iter()
                .enumerate()
                .filter(|&(_, &(_, prev_end))| cand_start >= prev_end)
                .map(|(prev_index, _)| prev_index)
                .collect::<Vec<_>>();
            let prev_indices = if non_overlapping.is_empty() {
                (0..cand_lists[pair_index - 1].len()).collect::<Vec<_>>()
            } else {
                non_overlapping
            };

            let mut best_cost = f64::INFINITY;
            let mut best_prev = None;
            for prev_index in prev_indices {
                let (_, prev_end) = cand_lists[pair_index - 1][prev_index];
                let total = dp[pair_index - 1][prev_index]
                    + bayes_local_cost(
                        cand_start,
                        cand_end,
                        pair,
                        &priors[pair_index],
                        BayesCostContext {
                            step: grid_step,
                            prev_end: Some(prev_end),
                            prev_raw_end,
                            segment_center,
                            segment_center_weight,
                        },
                    );
                if total < best_cost {
                    best_cost = total;
                    best_prev = Some(prev_index);
                }
            }

            cur_cost.push(best_cost);
            cur_back.push(best_prev);
        }

        dp.push(cur_cost);
        back.push(cur_back);
    }

    let last_pair = pairs.len() - 1;
    let mut last_index = 0_usize;
    let mut total_cost = f64::INFINITY;
    for (index, &cost) in dp[last_pair].iter().enumerate() {
        if cost < total_cost {
            total_cost = cost;
            last_index = index;
        }
    }

    let mut indices = vec![last_index];
    for pair_index in (1..pairs.len()).rev() {
        let prev_index = back[pair_index][*indices.last().unwrap()].unwrap_or(0);
        indices.push(prev_index);
    }
    indices.reverse();

    let seq = indices
        .iter()
        .enumerate()
        .map(|(pair_index, &candidate_index)| cand_lists[pair_index][candidate_index])
        .collect::<Vec<_>>();

    (seq, total_cost)
}

/// Builds sorted unique duration candidates from Python's multiplier table.
///
/// # Panics
///
/// Panics when `step` is not positive.
pub fn build_duration_candidates(step: i64, max_raw_tick: i64) -> Vec<i64> {
    assert!(step > 0, "step must be positive");

    let cap = step.max((max_raw_tick as f64 / step as f64).ceil() as i64 * step + 2 * step);
    let mut values = DURATION_CANDIDATE_MULTIPLIERS
        .iter()
        .map(|multiplier| multiplier * step)
        .filter(|value| *value <= cap)
        .collect::<Vec<_>>();
    if values.is_empty() {
        values.push(step);
    }
    values.sort_unstable();
    values.dedup();
    values
}

/// Builds sorted unique gap candidates from Python's multiplier table.
///
/// # Panics
///
/// Panics when `step` is not positive.
pub fn build_gap_candidates(step: i64, max_raw_gap: i64) -> Vec<i64> {
    assert!(step > 0, "step must be positive");

    let cap = (step * 2).max((max_raw_gap as f64 / step as f64).ceil() as i64 * step + step);
    let mut values = GAP_CANDIDATE_MULTIPLIERS
        .iter()
        .map(|multiplier| multiplier * step)
        .filter(|value| *value <= cap)
        .collect::<Vec<_>>();
    if !values.contains(&0) {
        values.push(0);
    }
    values.sort_unstable();
    values.dedup();
    values
}

fn ticks_from_sec(value: f64, tempo: f64) -> i64 {
    round_half_even(value * tempo * 8.0) as i64
}

fn round_half_even(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let floor = value.floor();
    let diff = value - floor;
    if diff < 0.5 {
        floor
    } else if diff > 0.5 {
        floor + 1.0
    } else if (floor as i128).rem_euclid(2) == 0 {
        floor
    } else {
        floor + 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTIVATION_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_activation_policy.tsv");
    const SIMPLE_GRID_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_simple_grid_core.tsv");
    const SMART_DURATION_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_smart_duration_dp.tsv");
    const PHRASE_DP_HELPER_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_phrase_dp_helpers.tsv");
    const PHRASE_DP_CORE_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_phrase_dp_core.tsv");
    const BAYESIAN_HELPER_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_bayesian_helpers.tsv");
    const BAYESIAN_CORE_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_bayesian_core.tsv");
    const CANDIDATE_SCALAR_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_candidate_scalar_primitives.tsv");
    const CANDIDATE_PAIR_FIXTURES: &str =
        include_str!("../../../../fixtures/quantization_candidate_pair_primitives.tsv");

    fn parse_mode(value: &'static str) -> Option<&'static str> {
        match value {
            "__none__" => None,
            "__empty__" => Some(""),
            "__padded_dp__" => Some(" dp "),
            _ => Some(value),
        }
    }

    fn parse_bool(value: &str) -> bool {
        match value {
            "true" => true,
            "false" => false,
            _ => panic!("unknown bool {value}"),
        }
    }

    fn parse_notes(value: &str) -> Vec<OwnedSimpleGridNote> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }

        value
            .split('|')
            .map(|raw_note| {
                let mut fields = raw_note.splitn(4, ',');
                OwnedSimpleGridNote {
                    onset: fields.next().unwrap().parse().unwrap(),
                    offset: fields.next().unwrap().parse().unwrap(),
                    pitch: fields.next().unwrap().parse().unwrap(),
                    lyric: parse_lyric(fields.next().unwrap()),
                }
            })
            .collect()
    }

    fn parse_lyric(value: &str) -> String {
        if value == "__empty__" {
            String::new()
        } else {
            value.to_string()
        }
    }

    fn parse_optional_lyric(value: &str) -> Option<&str> {
        match value {
            "__missing__" => None,
            "__empty__" => Some(""),
            _ => Some(value),
        }
    }

    fn parse_i64_list(value: &str) -> Vec<i64> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }
        value.split(',').map(|item| item.parse().unwrap()).collect()
    }

    fn parse_candidate_pair_list(value: &str) -> Vec<(i64, i64)> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }

        value
            .split('|')
            .map(|raw_pair| {
                let mut fields = raw_pair.split(',');
                (
                    fields.next().unwrap().parse().unwrap(),
                    fields.next().unwrap().parse().unwrap(),
                )
            })
            .collect()
    }

    fn parse_optional_i64(value: &str) -> Option<i64> {
        match value {
            "" | "__none__" => None,
            _ => Some(value.parse().unwrap()),
        }
    }

    fn parse_decode_expected(value: &str) -> (Vec<(i64, i64)>, f64) {
        let mut fields = value.splitn(2, ';');
        let seq = parse_candidate_pair_list(fields.next().unwrap());
        let cost = fields.next().unwrap().parse().unwrap();
        (seq, cost)
    }

    fn assert_float_close(case_id: &str, actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1e-9,
            "{case_id}: {actual:?} != {expected:?}"
        );
    }

    fn assert_notes_close(
        case_id: &str,
        actual: &[SimpleGridNote<'_>],
        expected: &[OwnedSimpleGridNote],
    ) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "{case_id}: note count mismatch"
        );

        for (index, (actual_note, expected_note)) in actual.iter().zip(expected.iter()).enumerate()
        {
            assert!(
                (actual_note.onset - expected_note.onset).abs() <= 1e-12,
                "{case_id}: onset mismatch at {index}: {:?} != {:?}",
                actual_note.onset,
                expected_note.onset
            );
            assert!(
                (actual_note.offset - expected_note.offset).abs() <= 1e-12,
                "{case_id}: offset mismatch at {index}: {:?} != {:?}",
                actual_note.offset,
                expected_note.offset
            );
            assert!(
                (actual_note.pitch - expected_note.pitch).abs() <= 1e-12,
                "{case_id}: pitch mismatch at {index}: {:?} != {:?}",
                actual_note.pitch,
                expected_note.pitch
            );
            assert_eq!(
                actual_note.lyric, expected_note.lyric,
                "{case_id}: lyric mismatch at {index}"
            );
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct OwnedSimpleGridNote {
        onset: f64,
        offset: f64,
        pitch: f64,
        lyric: String,
    }

    impl OwnedSimpleGridNote {
        fn as_note(&self) -> SimpleGridNote<'_> {
            SimpleGridNote {
                onset: self.onset,
                offset: self.offset,
                pitch: self.pitch,
                lyric: &self.lyric,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct OwnedRawNotePair {
        raw_start: i64,
        raw_end: i64,
        raw_dur: i64,
        lyrics: String,
    }

    impl OwnedRawNotePair {
        fn as_pair(&self) -> RawNotePair<'_> {
            RawNotePair {
                raw_start: self.raw_start,
                raw_end: self.raw_end,
                raw_dur: self.raw_dur,
                lyrics: &self.lyrics,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct OwnedBayesPrior {
        count: usize,
        strength: f64,
        preferred_dur: Option<i64>,
        preferred_gap: Option<i64>,
        preferred_phase: Option<i64>,
    }

    impl OwnedBayesPrior {
        fn as_prior(&self) -> BayesPrior {
            BayesPrior {
                count: self.count,
                strength: self.strength,
                preferred_dur: self.preferred_dur,
                preferred_gap: self.preferred_gap,
                preferred_phase: self.preferred_phase,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct OwnedGapAnnotatedNotePair {
        raw_start: i64,
        raw_end: i64,
        raw_dur: i64,
        lyrics: String,
        raw_gap: i64,
    }

    impl OwnedGapAnnotatedNotePair {
        fn as_pair(&self) -> GapAnnotatedNotePair<'_> {
            GapAnnotatedNotePair {
                raw_start: self.raw_start,
                raw_end: self.raw_end,
                raw_dur: self.raw_dur,
                lyrics: &self.lyrics,
                raw_gap: self.raw_gap,
            }
        }
    }

    fn parse_raw_note_pair(value: &str) -> OwnedRawNotePair {
        let mut fields = value.splitn(4, ',');
        OwnedRawNotePair {
            raw_start: fields.next().unwrap().parse().unwrap(),
            raw_end: fields.next().unwrap().parse().unwrap(),
            raw_dur: fields.next().unwrap().parse().unwrap(),
            lyrics: parse_lyric(fields.next().unwrap()),
        }
    }

    fn parse_raw_note_pairs(value: &str) -> Vec<OwnedRawNotePair> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }
        value.split('|').map(parse_raw_note_pair).collect()
    }

    fn parse_gap_annotated_note_pair(value: &str) -> OwnedGapAnnotatedNotePair {
        let mut fields = value.splitn(5, ',');
        OwnedGapAnnotatedNotePair {
            raw_start: fields.next().unwrap().parse().unwrap(),
            raw_end: fields.next().unwrap().parse().unwrap(),
            raw_dur: fields.next().unwrap().parse().unwrap(),
            lyrics: parse_lyric(fields.next().unwrap()),
            raw_gap: fields.next().unwrap().parse().unwrap(),
        }
    }

    fn parse_gap_annotated_note_pairs(value: &str) -> Vec<OwnedGapAnnotatedNotePair> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }
        value
            .split('|')
            .map(parse_gap_annotated_note_pair)
            .collect()
    }

    fn parse_bayes_note_pair(value: &str) -> OwnedGapAnnotatedNotePair {
        let fields = value.split(',').collect::<Vec<_>>();
        assert!(matches!(fields.len(), 4 | 5), "invalid Bayes pair {value}");
        OwnedGapAnnotatedNotePair {
            raw_start: fields[0].parse().unwrap(),
            raw_end: fields[1].parse().unwrap(),
            raw_dur: fields[2].parse().unwrap(),
            lyrics: parse_lyric(fields[3]),
            raw_gap: if fields.len() == 5 {
                fields[4].parse().unwrap()
            } else {
                0
            },
        }
    }

    fn parse_bayes_note_pairs(value: &str) -> Vec<OwnedGapAnnotatedNotePair> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }
        value.split('|').map(parse_bayes_note_pair).collect()
    }

    fn parse_bayes_prior(value: &str) -> OwnedBayesPrior {
        let fields = value.split(',').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5, "invalid Bayes prior {value}");
        OwnedBayesPrior {
            count: fields[0].parse().unwrap(),
            strength: fields[1].parse().unwrap(),
            preferred_dur: parse_optional_i64(fields[2]),
            preferred_gap: parse_optional_i64(fields[3]),
            preferred_phase: parse_optional_i64(fields[4]),
        }
    }

    fn parse_bayes_priors(value: &str) -> Vec<OwnedBayesPrior> {
        if value.is_empty() || value == "__empty__" {
            return Vec::new();
        }
        value.split('|').map(parse_bayes_prior).collect()
    }

    fn assert_priors_close(case_id: &str, actual: &[BayesPrior], expected: &[OwnedBayesPrior]) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "{case_id}: prior count mismatch"
        );
        for (index, (actual_prior, expected_prior)) in
            actual.iter().zip(expected.iter()).enumerate()
        {
            assert_eq!(
                actual_prior.count, expected_prior.count,
                "{case_id}: prior count mismatch at {index}"
            );
            assert_float_close(case_id, actual_prior.strength, expected_prior.strength);
            assert_eq!(
                actual_prior.preferred_dur, expected_prior.preferred_dur,
                "{case_id}: preferred_dur mismatch at {index}"
            );
            assert_eq!(
                actual_prior.preferred_gap, expected_prior.preferred_gap,
                "{case_id}: preferred_gap mismatch at {index}"
            );
            assert_eq!(
                actual_prior.preferred_phase, expected_prior.preferred_phase,
                "{case_id}: preferred_phase mismatch at {index}"
            );
        }
    }

    fn parse_bayes_local_options(value: &str) -> (i64, Option<i64>, Option<i64>, f64, f64) {
        let fields = value.split(',').collect::<Vec<_>>();
        assert_eq!(fields.len(), 5, "invalid Bayes local options {value}");
        (
            fields[0].parse().unwrap(),
            parse_optional_i64(fields[1]),
            parse_optional_i64(fields[2]),
            fields[3].parse().unwrap(),
            fields[4].parse().unwrap(),
        )
    }

    fn own_gap_annotated_pair(pair: &GapAnnotatedNotePair<'_>) -> OwnedGapAnnotatedNotePair {
        OwnedGapAnnotatedNotePair {
            raw_start: pair.raw_start,
            raw_end: pair.raw_end,
            raw_dur: pair.raw_dur,
            lyrics: pair.lyrics.to_string(),
            raw_gap: pair.raw_gap,
        }
    }

    #[test]
    fn quantization_activation_policy_follows_parity_fixture_table() {
        for (line_number, line) in ACTIVATION_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let mode = parse_mode(fields.next().unwrap());
            let step: i64 = fields.next().unwrap().parse().unwrap();
            let expected = parse_bool(fields.next().unwrap());
            assert_eq!(
                should_apply_quantization(mode, step),
                expected,
                "fixture line {}",
                line_number + 1
            );
        }
    }

    #[test]
    fn dp_mode_ignores_step_like_python() {
        for step in [-60, -1, 0, 1, 60] {
            assert!(should_apply_quantization(Some("dp"), step));
            assert!(should_apply_quantization(Some("DP"), step));
        }
    }

    #[test]
    fn mode_is_lowercased_but_not_trimmed() {
        assert!(should_apply_quantization(Some("Dp"), 0));
        assert!(!should_apply_quantization(Some(" dp "), 0));
        assert!(should_apply_quantization(Some(" dp "), 16));
    }

    #[test]
    fn simple_grid_quantization_follows_parity_fixture_table() {
        for (line_number, line) in SIMPLE_GRID_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let case_id = fields.next().unwrap();
            let tempo: f64 = fields.next().unwrap().parse().unwrap();
            let quantization_step: i64 = fields.next().unwrap().parse().unwrap();
            let owned_notes = parse_notes(fields.next().unwrap());
            let mut notes = owned_notes
                .iter()
                .map(OwnedSimpleGridNote::as_note)
                .collect::<Vec<_>>();
            let expected = parse_notes(fields.next().unwrap());

            quantize_notes_simple(&mut notes, tempo, quantization_step);
            assert_notes_close(case_id, &notes, &expected);
            let _ = line_number;
        }
    }

    #[test]
    fn simple_grid_step_zero_preserves_order_and_values() {
        let owned_notes = [
            OwnedSimpleGridNote {
                onset: 0.2,
                offset: 0.3,
                pitch: 62.0,
                lyric: "beta".to_string(),
            },
            OwnedSimpleGridNote {
                onset: 0.1,
                offset: 0.15,
                pitch: 60.0,
                lyric: "alpha".to_string(),
            },
        ];
        let mut notes = owned_notes
            .iter()
            .map(OwnedSimpleGridNote::as_note)
            .collect::<Vec<_>>();

        quantize_notes_simple(&mut notes, 120.0, 0);

        assert_eq!(notes[0].lyric, "beta");
        assert_eq!(notes[0].onset, 0.2);
        assert_eq!(notes[1].lyric, "alpha");
        assert_eq!(notes[1].onset, 0.1);
    }

    #[test]
    fn smart_duration_quantization_follows_parity_fixture_table() {
        for (line_number, line) in SMART_DURATION_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let case_id = fields.next().unwrap();
            let tempo: f64 = fields.next().unwrap().parse().unwrap();
            let quantization_step: i64 = fields.next().unwrap().parse().unwrap();
            let owned_notes = parse_notes(fields.next().unwrap());
            let mut notes = owned_notes
                .iter()
                .map(OwnedSimpleGridNote::as_note)
                .collect::<Vec<_>>();
            let expected = parse_notes(fields.next().unwrap());

            quantize_notes_smart(&mut notes, tempo, quantization_step);
            assert_notes_close(case_id, &notes, &expected);
            let _ = line_number;
        }
    }

    #[test]
    fn smart_duration_step_zero_preserves_order_and_values() {
        let owned_notes = [
            OwnedSimpleGridNote {
                onset: 0.2,
                offset: 0.3,
                pitch: 62.0,
                lyric: "beta".to_string(),
            },
            OwnedSimpleGridNote {
                onset: 0.1,
                offset: 0.15,
                pitch: 60.0,
                lyric: "alpha".to_string(),
            },
        ];
        let mut notes = owned_notes
            .iter()
            .map(OwnedSimpleGridNote::as_note)
            .collect::<Vec<_>>();

        quantize_notes_smart(&mut notes, 120.0, 0);

        assert_eq!(notes[0].lyric, "beta");
        assert_eq!(notes[0].onset, 0.2);
        assert_eq!(notes[1].lyric, "alpha");
        assert_eq!(notes[1].onset, 0.1);
    }

    #[test]
    fn phrase_dp_helpers_follow_parity_fixture_table() {
        for (line_number, line) in PHRASE_DP_HELPER_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 8, "fixture line {}", line_number + 1);
            let case_id = fields[0];
            let kind = fields[1];
            let input_a = fields[2];
            let input_b = fields[3];
            let input_c = fields[4];
            let input_d = fields[5];
            let input_e = fields[6];
            let expected = fields[7];

            match kind {
                "local_cost_asym" => {
                    let pair = parse_raw_note_pair(input_c);
                    let actual = local_cost_asym(
                        input_a.parse().unwrap(),
                        input_b.parse().unwrap(),
                        &pair.as_pair(),
                        parse_optional_i64(input_d),
                        parse_optional_i64(input_e),
                        &DPQ_DEFAULT_ASYM,
                    );
                    assert_float_close(case_id, actual, expected.parse().unwrap());
                }
                "segment_split_indices" => {
                    let owned_pairs = parse_raw_note_pairs(input_b);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedRawNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let actual = segment_split_indices(&pairs, input_a.parse().unwrap())
                        .iter()
                        .map(|&(start, end)| (start as i64, end as i64))
                        .collect::<Vec<_>>();
                    assert_eq!(actual, parse_candidate_pair_list(expected), "{case_id}");
                }
                "center_adjustment" => {
                    let pair = parse_raw_note_pair(input_b);
                    let actual = center_adjustment(
                        &pair.as_pair(),
                        input_a.parse().unwrap(),
                        parse_optional_i64(input_c),
                        input_d.parse().unwrap(),
                    );
                    assert_float_close(case_id, actual, expected.parse().unwrap());
                }
                "decode_segment_with_center" => {
                    let owned_pairs = parse_raw_note_pairs(input_c);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedRawNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let (actual_seq, actual_cost) = decode_segment_with_center(
                        &pairs,
                        input_b.parse().unwrap(),
                        input_a.parse().unwrap(),
                        &DPQ_DEFAULT_ASYM,
                    );
                    let (expected_seq, expected_cost) = parse_decode_expected(expected);
                    assert_eq!(actual_seq, expected_seq, "{case_id}");
                    assert_float_close(case_id, actual_cost, expected_cost);
                }
                _ => panic!(
                    "unknown phrase DP helper kind {kind} on line {}",
                    line_number + 1
                ),
            }
        }
    }

    #[test]
    fn phrase_dp_quantization_follows_parity_fixture_table() {
        for (line_number, line) in PHRASE_DP_CORE_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let case_id = fields.next().unwrap();
            let tempo: f64 = fields.next().unwrap().parse().unwrap();
            let quantization_step: i64 = fields.next().unwrap().parse().unwrap();
            let owned_notes = parse_notes(fields.next().unwrap());
            let mut notes = owned_notes
                .iter()
                .map(OwnedSimpleGridNote::as_note)
                .collect::<Vec<_>>();
            let expected = parse_notes(fields.next().unwrap());

            quantize_notes_phrase_dp(&mut notes, tempo, quantization_step);
            assert_notes_close(case_id, &notes, &expected);
            let _ = line_number;
        }
    }

    #[test]
    fn bayesian_helpers_follow_parity_fixture_table() {
        for (line_number, line) in BAYESIAN_HELPER_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 8, "fixture line {}", line_number + 1);
            let case_id = fields[0];
            let kind = fields[1];
            let input_a = fields[2];
            let input_b = fields[3];
            let input_c = fields[4];
            let input_d = fields[5];
            let input_e = fields[6];
            let expected = fields[7];

            match kind {
                "resolve_bayes_shift_limit" => assert_eq!(
                    resolve_bayes_shift_limit(
                        input_a.parse().unwrap(),
                        input_b.parse().unwrap(),
                        input_c.parse().unwrap(),
                        input_d.parse().unwrap(),
                    )
                    .to_string(),
                    expected,
                    "{case_id}"
                ),
                "filter_bayes_candidate_pairs" => {
                    let pair = parse_bayes_note_pair(input_b);
                    let actual = filter_bayes_candidate_pairs(
                        &raw_pair_from_gap(&pair.as_pair()),
                        &parse_candidate_pair_list(input_c),
                        input_a.parse().unwrap(),
                    );
                    assert_eq!(actual, parse_candidate_pair_list(expected), "{case_id}");
                }
                "build_bayes_candidate_pairs" => {
                    let pair = parse_bayes_note_pair(input_b);
                    let actual = build_bayes_candidate_pairs(
                        &raw_pair_from_gap(&pair.as_pair()),
                        input_a.parse().unwrap(),
                    );
                    assert_eq!(actual, parse_candidate_pair_list(expected), "{case_id}");
                }
                "estimate_segment_phase_center" => {
                    let owned_pairs = parse_bayes_note_pairs(input_b);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedGapAnnotatedNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let (actual_center, actual_weight) =
                        estimate_segment_phase_center(&pairs, input_a.parse().unwrap());
                    let mut expected_fields = expected.split(',');
                    assert_float_close(
                        case_id,
                        actual_center,
                        expected_fields.next().unwrap().parse().unwrap(),
                    );
                    assert_float_close(
                        case_id,
                        actual_weight,
                        expected_fields.next().unwrap().parse().unwrap(),
                    );
                }
                "segment_split_indices_bayesian" => {
                    let owned_pairs = parse_bayes_note_pairs(input_b);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedGapAnnotatedNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let actual = segment_split_indices_bayesian(&pairs, input_a.parse().unwrap())
                        .iter()
                        .map(|&(start, end)| (start as i64, end as i64))
                        .collect::<Vec<_>>();
                    assert_eq!(actual, parse_candidate_pair_list(expected), "{case_id}");
                }
                "metrical_position_penalty" => {
                    let actual = metrical_position_penalty(
                        input_a.parse().unwrap(),
                        input_b.parse().unwrap(),
                    );
                    assert_float_close(case_id, actual, expected.parse().unwrap());
                }
                "note_value_penalty" => {
                    let actual =
                        note_value_penalty(input_a.parse().unwrap(), input_b.parse().unwrap());
                    assert_float_close(case_id, actual, expected.parse().unwrap());
                }
                "preferred_sv_duration" => assert_eq!(
                    preferred_sv_duration(input_a.parse().unwrap(), input_b.parse().unwrap()),
                    parse_optional_i64(expected),
                    "{case_id}"
                ),
                "build_piece_specific_priors" => {
                    let owned_pairs = parse_bayes_note_pairs(input_b);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedGapAnnotatedNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let actual = build_piece_specific_priors(
                        &pairs,
                        input_a.parse().unwrap(),
                        &parse_i64_list(input_c),
                        &parse_i64_list(input_d),
                    );
                    let expected = parse_bayes_priors(expected);
                    assert_priors_close(case_id, &actual, &expected);
                }
                "bayes_local_cost" => {
                    let pair = parse_bayes_note_pair(input_c);
                    let prior = parse_bayes_prior(input_d);
                    let (step, prev_end, prev_raw_end, segment_center, segment_weight) =
                        parse_bayes_local_options(input_e);
                    let actual = bayes_local_cost(
                        input_a.parse().unwrap(),
                        input_b.parse().unwrap(),
                        &pair.as_pair(),
                        &prior.as_prior(),
                        BayesCostContext {
                            step,
                            prev_end,
                            prev_raw_end,
                            segment_center,
                            segment_center_weight: segment_weight,
                        },
                    );
                    assert_float_close(case_id, actual, expected.parse().unwrap());
                }
                "decode_segment_bayesian" => {
                    let owned_pairs = parse_bayes_note_pairs(input_b);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedGapAnnotatedNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let owned_priors = parse_bayes_priors(input_c);
                    let priors = owned_priors
                        .iter()
                        .map(OwnedBayesPrior::as_prior)
                        .collect::<Vec<_>>();
                    let (actual_seq, actual_cost) =
                        decode_segment_bayesian(&pairs, &priors, input_a.parse().unwrap());
                    let (expected_seq, expected_cost) = parse_decode_expected(expected);
                    assert_eq!(actual_seq, expected_seq, "{case_id}");
                    assert_float_close(case_id, actual_cost, expected_cost);
                }
                _ => panic!(
                    "unknown Bayesian helper kind {kind} on line {}",
                    line_number + 1
                ),
            }
        }
    }

    #[test]
    fn bayesian_quantization_follows_parity_fixture_table() {
        for (line_number, line) in BAYESIAN_CORE_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut fields = line.split('\t');
            let case_id = fields.next().unwrap();
            let tempo: f64 = fields.next().unwrap().parse().unwrap();
            let quantization_step: i64 = fields.next().unwrap().parse().unwrap();
            let owned_notes = parse_notes(fields.next().unwrap());
            let mut notes = owned_notes
                .iter()
                .map(OwnedSimpleGridNote::as_note)
                .collect::<Vec<_>>();
            let expected = parse_notes(fields.next().unwrap());

            quantize_notes_bayesian(&mut notes, tempo, quantization_step);
            assert_notes_close(case_id, &notes, &expected);
            let _ = line_number;
        }
    }

    #[test]
    fn quant_candidate_scalar_primitives_follow_parity_fixture_table() {
        for (line_number, line) in CANDIDATE_SCALAR_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 7, "fixture line {}", line_number + 1);
            let case_id = fields[0];
            let kind = fields[1];
            let input_a = fields[2];
            let input_b = fields[3];
            let input_c = fields[4];
            let expected = fields[6];

            match kind {
                "resolve_dp_grid_step" => assert_eq!(
                    resolve_dp_grid_step(input_a.parse().unwrap()).to_string(),
                    expected,
                    "{case_id}"
                ),
                "resolve_segment_shift_candidates" => assert_eq!(
                    resolve_segment_shift_candidates(input_a.parse().unwrap()),
                    parse_i64_list(expected),
                    "{case_id}"
                ),
                "nearest_candidate" => assert_eq!(
                    nearest_candidate(input_a.parse().unwrap(), &parse_i64_list(input_b))
                        .unwrap()
                        .to_string(),
                    expected,
                    "{case_id}"
                ),
                "mod_distance" => assert_eq!(
                    mod_distance(
                        input_a.parse().unwrap(),
                        input_b.parse().unwrap(),
                        input_c.parse().unwrap(),
                    )
                    .to_string(),
                    expected,
                    "{case_id}"
                ),
                "dist_grid" => assert_eq!(
                    dist_grid(input_a.parse().unwrap(), input_b.parse().unwrap()).to_string(),
                    expected,
                    "{case_id}"
                ),
                "candidate_values" => assert_eq!(
                    candidate_values(
                        input_a.parse().unwrap(),
                        input_b.parse().unwrap(),
                        input_c.parse().unwrap(),
                    ),
                    parse_i64_list(expected),
                    "{case_id}"
                ),
                "duration_candidates" => assert_eq!(
                    build_duration_candidates(input_a.parse().unwrap(), input_b.parse().unwrap()),
                    parse_i64_list(expected),
                    "{case_id}"
                ),
                "gap_candidates" => assert_eq!(
                    build_gap_candidates(input_a.parse().unwrap(), input_b.parse().unwrap()),
                    parse_i64_list(expected),
                    "{case_id}"
                ),
                _ => panic!(
                    "unknown candidate scalar kind {kind} on line {}",
                    line_number + 1
                ),
            }
        }
    }

    #[test]
    fn quant_candidate_pair_primitives_follow_parity_fixture_table() {
        for (line_number, line) in CANDIDATE_PAIR_FIXTURES.lines().enumerate() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 9, "fixture line {}", line_number + 1);
            let case_id = fields[0];
            let kind = fields[1];
            let raw_start = fields[2];
            let raw_end = fields[3];
            let lyric = fields[4];
            let radius = fields[5];
            let step = fields[6];
            let input_pairs = fields[7];
            let expected = fields[8];

            match kind {
                "build_note_pair" => {
                    let actual = build_note_pair(
                        raw_start.parse().unwrap(),
                        raw_end.parse().unwrap(),
                        parse_optional_lyric(lyric),
                    );
                    let expected = parse_raw_note_pair(expected);
                    assert_eq!(actual.raw_start, expected.raw_start, "{case_id}");
                    assert_eq!(actual.raw_end, expected.raw_end, "{case_id}");
                    assert_eq!(actual.raw_dur, expected.raw_dur, "{case_id}");
                    assert_eq!(actual.lyrics, expected.lyrics, "{case_id}");
                }
                "annotate_pairs_with_gap" => {
                    let owned_pairs = parse_raw_note_pairs(input_pairs);
                    let pairs = owned_pairs
                        .iter()
                        .map(OwnedRawNotePair::as_pair)
                        .collect::<Vec<_>>();
                    let actual = annotate_pairs_with_gap(&pairs)
                        .iter()
                        .map(own_gap_annotated_pair)
                        .collect::<Vec<_>>();
                    let expected = parse_gap_annotated_note_pairs(expected);
                    assert_eq!(actual, expected, "{case_id}");
                }
                "build_candidate_pairs" => {
                    let pair = build_note_pair(
                        raw_start.parse().unwrap(),
                        raw_end.parse().unwrap(),
                        parse_optional_lyric(lyric),
                    );
                    let actual = build_candidate_pairs(
                        &pair,
                        radius.parse().unwrap(),
                        step.parse().unwrap(),
                    );
                    let expected = parse_candidate_pair_list(expected);
                    assert_eq!(actual, expected, "{case_id}");
                }
                _ => panic!(
                    "unknown candidate pair kind {kind} on line {}",
                    line_number + 1
                ),
            }
        }
    }

    #[test]
    fn quant_candidate_nearest_returns_none_for_empty_rust_slice() {
        assert_eq!(nearest_candidate(1.0, &[]), None);
    }

    #[test]
    fn tick_conversion_uses_half_even_rounding() {
        assert_eq!(ticks_from_sec(30.0 / 960.0, 120.0), 30);
        assert_eq!(round_half_even(0.5), 0.0);
        assert_eq!(round_half_even(1.5), 2.0);
        assert_eq!(round_half_even(2.5), 2.0);
    }
}
