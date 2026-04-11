//! Chart computation formulas and constants.
//!
//! ## Core Constants
//!
//! - `BASE_VELOCITY_FACTOR = 1/240`: Base velocity factor (Y/sec per BPM)
//! - `DEFAULT_BPM = 120`: Default BPM value
//! - `DEFAULT_SPEED = 1`: Default speed factor
//!
//! ## Y Coordinate System
//!
//! - In the default 4/4 time signature, the length of "one measure" is 1.
//! - BMS: `#XXX02:V` message per measure, where `V` represents the multiple of the default length.
//! - BMSON: one measure length is `4 * resolution` pulses; all position y is normalized to measure units through `pulses / (4 * resolution)`.
//!
//! ## Velocity Calculation
//!
//! ```text
//! velocity = (current_bpm / 240) * current_speed * playback_ratio
//! ```
//!
//! ## Visible Range
//!
//! **`VisibleRangePerBpm::new`:**
//! ```text
//! visible_range_per_bpm = reaction_time_seconds * 240 / base_bpm
//! ```
//!
//! **`VisibleRangePerBpm::window_y`:**
//! ```text
//! visible_window_y = (current_speed * playback_ratio / 240) * reaction_time * base_bpm
//! ```
//!
//! This ensures events stay in visible window for exactly `reaction_time * base_bpm / current_bpm` duration.
//!
//! ## Display Ratio
//!
//! ```text
//! display_ratio = (event_y - current_y) / visible_window_y * current_scroll
//! ```
//!
//! The value of this type is only affected by: current Y, Y visible range, and current Speed, Scroll values.
//!
//! ## Reaction Time
//!
//! ```text
//! reaction_time = visible_range_per_bpm / playhead_speed
//! where playhead_speed = 1/240
//! ```
//!
//! ## Time Progression
//!
//! ```text
//! delta_y = velocity * elapsed_time_secs
//! time_to_event = distance_y / velocity
//! ```
//!
//! ## Time Calculation
//!
//! ```text
//! delta_secs = delta_y * 240 / current_bpm
//! stop_duration_secs = stop_duration * 240 / bpm_at_stop
//! ```

/// Base velocity factor (Y/sec per BPM).
pub const BASE_VELOCITY_FACTOR: f64 = 1.0 / 240.0;

#[doc(hidden)]
pub const BASE_VELOCITY_FACTOR_RECIPROCAL: f64 = 240.0;

use std::collections::BTreeMap;
use std::time::Duration;

use gametime::TimeSpan;
use strict_num_extended::{FinF64, NonNegativeF64, PositiveF64};

use crate::chart::event::YCoordinate;
use crate::chart::{MAX_FIN_F64, MAX_NON_NEGATIVE_F64};

/// Computes cumulative time (in seconds) at each Y coordinate point.
///
/// This function calculates the exact time when the playhead reaches each Y coordinate,
/// accounting for BPM changes and stops.
///
/// # Formula
///
/// ```text
/// delta_secs = delta_y * 240 / current_bpm
/// stop_duration_secs = stop_duration * 240 / bpm_at_stop
/// ```
///
/// # Parameters
///
/// * `points` - Sorted set of Y coordinates to compute times for (must include `YCoordinate::ZERO`)
/// * `init_bpm` - Initial BPM value
/// * `bpm_changes` - Iterator of (Y coordinate, BPM) pairs, sorted by Y
/// * `stops` - Iterator of (Y coordinate, stop duration in beats) pairs, sorted by Y
///
/// # Returns
///
/// `BTreeMap` mapping each Y coordinate to its cumulative time in seconds
pub fn calculate_cumulative_times<'a, P, B, S>(
    points: P,
    init_bpm: PositiveF64,
    bpm_changes: B,
    stops: S,
) -> BTreeMap<YCoordinate, f64>
where
    P: IntoIterator<Item = &'a YCoordinate> + Clone,
    B: IntoIterator<Item = &'a (YCoordinate, PositiveF64)>,
    S: IntoIterator<Item = &'a (YCoordinate, NonNegativeF64)>,
{
    let stops: Vec<(YCoordinate, NonNegativeF64)> = stops.into_iter().copied().collect();

    let mut cum_map: BTreeMap<YCoordinate, f64> = BTreeMap::new();
    cum_map.insert(YCoordinate::ZERO, 0.0);

    let mut bpm_map: BTreeMap<YCoordinate, PositiveF64> = BTreeMap::new();
    bpm_map.insert(YCoordinate::ZERO, init_bpm);
    bpm_map.extend(bpm_changes.into_iter().copied());

    let mut stop_idx = 0usize;
    let mut total_secs: f64 = 0.0;
    let mut prev = YCoordinate::ZERO;

    for &curr in points {
        if curr <= prev {
            continue;
        }

        let cur_bpm = bpm_map
            .range(..curr)
            .next_back()
            .map_or(init_bpm, |(_, bpm)| *bpm);

        let delta_y = curr - prev;
        let delta_secs = delta_y.as_f64() * BASE_VELOCITY_FACTOR_RECIPROCAL / cur_bpm.as_f64();
        total_secs = (total_secs + delta_secs).min(f64::MAX);

        while let Some((sy, dur)) = stops.get(stop_idx) {
            if sy > &curr {
                break;
            }
            if sy > &prev {
                let bpm_at_stop = bpm_map
                    .range(..=sy)
                    .next_back()
                    .map_or(init_bpm, |(_, b)| *b);
                let dur_secs =
                    dur.as_f64() * BASE_VELOCITY_FACTOR_RECIPROCAL / bpm_at_stop.as_f64();
                total_secs = (total_secs + dur_secs).min(f64::MAX);
            }
            stop_idx += 1;
        }

        cum_map.insert(curr, total_secs);
        prev = curr;
    }

    cum_map
}

/// Visible range per BPM, representing the relationship between BPM and visible Y range.
/// See [`crate::chart`] for the formula.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleRangePerBpm {
    value: FinF64,
    base_bpm: FinF64,
    reaction_time_seconds: FinF64,
}

impl AsRef<FinF64> for VisibleRangePerBpm {
    fn as_ref(&self) -> &FinF64 {
        &self.value
    }
}

impl VisibleRangePerBpm {
    /// Create a new `VisibleRangePerBpm` from base BPM and reaction time.
    ///
    /// # Formula
    ///
    /// ```text
    /// visible_range_per_bpm = reaction_time_seconds * 240 / base_bpm
    /// ```
    #[must_use]
    pub fn new(base_bpm: &PositiveF64, reaction_time: TimeSpan) -> Self {
        if base_bpm.as_f64() <= f64::EPSILON {
            Self {
                value: FinF64::ZERO,
                base_bpm: FinF64::ZERO,
                reaction_time_seconds: FinF64::ZERO,
            }
        } else {
            let reaction_time_seconds =
                FinF64::new(reaction_time.as_secs_f64().max(0.0)).unwrap_or(FinF64::ZERO);
            let step1 =
                FinF64::new(reaction_time_seconds.as_f64() * BASE_VELOCITY_FACTOR_RECIPROCAL)
                    .unwrap_or(MAX_FIN_F64);
            let value = FinF64::new(step1.as_f64() / base_bpm.as_f64()).unwrap_or(MAX_FIN_F64);
            Self {
                value,
                base_bpm: FinF64::new(base_bpm.as_f64()).unwrap_or(FinF64::ONE),
                reaction_time_seconds,
            }
        }
    }

    /// Returns a reference to the contained value.
    #[must_use]
    pub const fn value(&self) -> &FinF64 {
        &self.value
    }

    /// Consumes self and returns the contained value.
    #[must_use]
    pub const fn into_value(self) -> FinF64 {
        self.value
    }

    /// Calculate visible window length in y units based on current BPM, speed, and playback ratio.
    ///
    /// # Formula
    ///
    /// This ensures events stay in visible window for exactly `reaction_time * base_bpm / current_bpm` duration.
    ///
    /// ```text
    /// visible_window_y = (current_speed * playback_ratio / 240) * reaction_time * base_bpm
    /// ```
    #[must_use]
    pub fn window_y(
        &self,
        current_bpm: PositiveF64,
        current_speed: PositiveF64,
        playback_ratio: FinF64,
    ) -> YCoordinate {
        if current_bpm.as_f64() <= f64::EPSILON {
            return YCoordinate::ZERO;
        }

        let speed_factor =
            FinF64::new(current_speed.as_f64() * playback_ratio.as_f64()).unwrap_or(MAX_FIN_F64);

        let bpm_div_240 =
            FinF64::new(current_bpm.as_f64() * BASE_VELOCITY_FACTOR).unwrap_or(MAX_FIN_F64);
        let velocity =
            FinF64::new(bpm_div_240.as_f64() * speed_factor.as_f64()).unwrap_or(MAX_FIN_F64);

        let step1 = FinF64::new(velocity.as_f64() * self.reaction_time_seconds.as_f64())
            .unwrap_or(MAX_FIN_F64);
        let step2 = FinF64::new(step1.as_f64() * self.base_bpm.as_f64()).unwrap_or(MAX_FIN_F64);
        let adjusted = FinF64::new(step2.as_f64() / current_bpm.as_f64()).unwrap_or(MAX_FIN_F64);

        YCoordinate::new(NonNegativeF64::new(adjusted.as_f64()).unwrap_or(MAX_NON_NEGATIVE_F64))
    }

    /// Calculate reaction time from visible range per BPM.
    ///
    /// # Formula
    ///
    /// ```text
    /// reaction_time = visible_range_per_bpm / playhead_speed
    /// where playhead_speed = 1/240
    /// ```
    #[must_use]
    pub fn to_reaction_time(&self) -> TimeSpan {
        if self.reaction_time_seconds.as_f64() == 0.0 {
            TimeSpan::ZERO
        } else {
            TimeSpan::from_duration(Duration::from_secs_f64(self.reaction_time_seconds.as_f64()))
        }
    }
}

impl From<FinF64> for VisibleRangePerBpm {
    fn from(value: FinF64) -> Self {
        let base_bpm = FinF64::ONE;
        let reaction_time_seconds = (value * BASE_VELOCITY_FACTOR).unwrap_or(FinF64::ZERO);
        Self {
            value,
            base_bpm,
            reaction_time_seconds,
        }
    }
}

impl From<VisibleRangePerBpm> for FinF64 {
    fn from(value: VisibleRangePerBpm) -> Self {
        value.value
    }
}

/// Convert STOP duration from 192nd-note units to beats (measure units).
///
/// In 4/4 time signature:
/// - 192nd-note represents 1/192 of a whole note
/// - One measure (4/4) = 4 beats = 192/48 beats
/// - Therefore: 1 unit of 192nd-note = 1/48 beat
///
/// # Formula
///
/// ```text
/// beats = 192nd_note_value / 48
/// ```
#[must_use]
pub fn convert_stop_duration_to_beats(duration_192nd: NonNegativeF64) -> NonNegativeF64 {
    NonNegativeF64::new(duration_192nd.as_f64() / 48.0).unwrap_or(NonNegativeF64::ZERO)
}

/// Display ratio representing the relative position of a note on screen.
///
/// 0 is the judgment line, 1 is the position where the note generally starts to appear.
///
/// # Formula
///
/// ```text
/// display_ratio = (event_y - current_y) / visible_window_y * current_scroll
/// ```
///
/// The value of this type is only affected by: current Y, Y visible range, and current Speed, Scroll values.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct DisplayRatio(pub FinF64);

impl AsRef<FinF64> for DisplayRatio {
    fn as_ref(&self) -> &FinF64 {
        &self.0
    }
}

impl DisplayRatio {
    /// Create a new `DisplayRatio`.
    #[must_use]
    pub const fn new(value: FinF64) -> Self {
        Self(value)
    }

    /// Returns a reference to the contained value.
    #[must_use]
    pub const fn value(&self) -> &FinF64 {
        &self.0
    }

    /// Consumes self and returns the contained value.
    #[must_use]
    pub const fn into_value(self) -> FinF64 {
        self.0
    }

    /// Create a `DisplayRatio` representing the judgment line (value 0).
    #[must_use]
    pub const fn at_judgment_line() -> Self {
        Self(FinF64::ZERO)
    }

    /// Create a `DisplayRatio` representing the position where note starts to appear (value 1).
    #[must_use]
    pub const fn at_appearance() -> Self {
        Self(FinF64::ONE)
    }
}

impl From<FinF64> for DisplayRatio {
    fn from(value: FinF64) -> Self {
        Self(value)
    }
}

impl From<DisplayRatio> for FinF64 {
    fn from(value: DisplayRatio) -> Self {
        value.0
    }
}

impl From<f64> for DisplayRatio {
    fn from(value: f64) -> Self {
        Self(FinF64::new(value).unwrap_or(FinF64::ZERO))
    }
}

/// Velocity calculator for Y units per second.
///
/// Uses caching to avoid redundant computations when parameters haven't changed.
#[derive(Debug, Clone)]
pub struct VelocityCalculator {
    cached_velocity: Option<FinF64>,
    velocity_dirty: bool,
}

impl VelocityCalculator {
    /// Create a new `VelocityCalculator`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached_velocity: None,
            velocity_dirty: true,
        }
    }

    /// Calculate velocity with caching.
    ///
    /// # Formula
    ///
    /// ```text
    /// velocity = (current_bpm / 240) * current_speed * playback_ratio
    /// ```
    pub fn calculate(
        &mut self,
        speed: PositiveF64,
        current_bpm: PositiveF64,
        playback_ratio: FinF64,
    ) -> FinF64 {
        if self.velocity_dirty || self.cached_velocity.is_none() {
            let computed = Self::compute(speed, current_bpm, playback_ratio);
            self.cached_velocity = Some(computed);
            self.velocity_dirty = false;
            computed
        } else {
            // SAFETY: velocity_dirty=false implies cached_velocity=Some
            unsafe { self.cached_velocity.unwrap_unchecked() }
        }
    }

    /// Compute velocity without caching (internal use).
    ///
    /// # Formula
    ///
    /// ```text
    /// velocity = (current_bpm / 240) * current_speed * playback_ratio
    /// ```
    #[must_use]
    pub fn compute(speed: PositiveF64, current_bpm: PositiveF64, playback_ratio: FinF64) -> FinF64 {
        if current_bpm.as_f64() <= 0.0 {
            FinF64::new(f64::EPSILON).unwrap_or(FinF64::ZERO)
        } else {
            let base =
                FinF64::new(current_bpm.as_f64() * BASE_VELOCITY_FACTOR).unwrap_or(FinF64::ZERO);
            let v1 = (base * speed).unwrap_or(MAX_FIN_F64);
            let v = (v1 * playback_ratio).unwrap_or(MAX_FIN_F64);
            FinF64::new(v.as_f64().max(f64::EPSILON)).unwrap_or(MAX_FIN_F64)
        }
    }

    /// Mark velocity cache as dirty.
    pub const fn mark_dirty(&mut self) {
        self.velocity_dirty = true;
    }
}

impl Default for VelocityCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// BMSON resolution type.
///
/// In BMSON, `info.resolution` is the number of pulses corresponding to a quarter note (1/4).
#[derive(Debug, Clone, Copy)]
pub struct BmsonResolution(pub u64);

impl BmsonResolution {
    /// Create a new resolution value.
    #[must_use]
    pub const fn new(resolution: u64) -> Self {
        Self(resolution)
    }

    /// Get the number of pulses per measure (4 * resolution).
    #[must_use]
    pub fn pulses_per_measure(&self) -> FinF64 {
        FinF64::new((4 * self.0) as f64).unwrap_or(MAX_FIN_F64)
    }
}

/// Convert BMSON pulses to Y coordinate.
///
/// In BMSON, `info.resolution` is the number of pulses corresponding to a quarter note (1/4),
/// so one measure length is `4 * resolution` pulses; all position y is normalized
/// to measure units through `pulses / (4 * resolution)`.
///
/// # Formula
///
/// ```text
/// y = pulses / (4 * resolution)
/// ```
///
/// # Panics
///
/// Panics if `denom` (4 * resolution) is zero or if conversion fails.
#[must_use]
pub fn pulses_to_y(pulses: u64, resolution: BmsonResolution) -> YCoordinate {
    let denom = resolution.pulses_per_measure();
    let denom_inv = if denom.as_f64() == 0.0 {
        FinF64::ZERO
    } else {
        FinF64::new(1.0 / denom.as_f64()).expect("denom_inv should be finite")
    };
    let pulses_f = FinF64::new(pulses as f64).expect("pulses should be finite");
    let y: NonNegativeF64 = (pulses_f * denom_inv)
        .unwrap_or(MAX_FIN_F64)
        .try_into()
        .unwrap_or(MAX_NON_NEGATIVE_F64);
    YCoordinate::new(y)
}
