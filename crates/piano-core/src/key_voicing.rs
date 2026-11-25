//! Fixed mechanical adjustments to individual keys. The preset stores only
//! keyed departures from its register laws; each strike uses the same row.

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(default, deny_unknown_fields))]
pub struct KeyVoicing {
    /// Optional per-wire tuning in cents relative to this key's stretched pitch,
    /// in string order. Only entries for speaking strings are used.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub unison_cents: Option<[f64; 3]>,
    /// Replacement steel core diameter of a wound string, in millimetres.
    /// The outer winding diameter, speaking length and pitch are preserved.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub wound_core_mm: Option<f64>,
    /// Multiplier on the outer felt's Q0 in its nonlinear force law.
    pub felt_stiffness_scale: f64,
    /// Offset from the instrument's strike position, in speaking lengths.
    pub strike_offset: f64,
    /// Multiplier on the string's intrinsic transverse amplitude loss rate.
    pub string_loss_scale: f64,
    /// Multiplier on the speaking string's reciprocal soundboard attachment.
    pub bridge_coupling_scale: f64,
    /// Playback level (dB) at the launch speeds corresponding to MIDI 45, 75 and 105.
    /// This acts on the radiated contribution; it does not change hammer contact.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "flat_level"))]
    pub level_db: [f32; 3],
    /// Empirical corrections to hammer excitation of speaking partials 1 through 8 (dB).
    /// Neutral values retain the geometric contact profile.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "flat_level"))]
    pub strike_partials_db: [f32; 8],
    /// Soft-strike attenuation (dB) of partial 7, partial 8, and partials 9 and up.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "flat_level"))]
    pub upper_strike_cut_db: [f32; 3],
}

#[cfg(feature = "serde")]
fn flat_level<const N: usize>(level: &[f32; N]) -> bool {
    *level == [0.0; N]
}

impl KeyVoicing {
    pub(crate) fn playback_gain(&self, speed: f64, law: &crate::params::VelocityLaw) -> f64 {
        if self.level_db == [0.0; 3] {
            return 1.0;
        }
        let middle = law.hammer_speed(75);
        let (lo, hi, a, b) = if speed <= middle {
            (law.hammer_speed(45), middle, self.level_db[0], self.level_db[1])
        } else {
            (middle, law.hammer_speed(105), self.level_db[1], self.level_db[2])
        };
        let t = if hi > lo && lo > 0.0 {
            ((speed / lo).ln() / (hi / lo).ln()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        10f64.powf((a as f64 + (b - a) as f64 * t) / 20.0)
    }

    pub(crate) fn touch_playback_gain(&self, speed: f64, law: &crate::params::VelocityLaw) -> f64 {
        if self.level_db == [0.0; 3] {
            return 1.0;
        }
        let positions = [45, 75, 105].map(|v| law.hammer_speed(v).ln());
        let [lo, mid, hi] = positions;
        if !(lo < mid && mid < hi) { return 1.0; }
        let [a, b, c] = self.level_db.map(f64::from);
        let left = (b - a) / (mid - lo);
        let right = (c - b) / (hi - mid);
        let x = speed.ln();
        // Round the three original piecewise-linear gain corners in log speed.
        let hinge = |z: f64| {
            let width = 0.018_f64.min(0.25 * (mid - lo).min(hi - mid));
            if z <= -width { 0.0 } else if z >= width { z } else { (z + width).powi(2) / (4.0 * width) }
        };
        let db = a + left * hinge(x - lo) + (right - left) * hinge(x - mid) - right * hinge(x - hi);
        10f64.powf(db / 20.0)
    }

    const NEUTRAL: Self = Self { unison_cents: None, wound_core_mm: None, felt_stiffness_scale: 1.0, strike_offset: 0.0, string_loss_scale: 1.0, bridge_coupling_scale: 1.0, level_db: [0.0; 3], strike_partials_db: [0.0; 8], upper_strike_cut_db: [0.0; 3] };
}

impl Default for KeyVoicing {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::VelocityLaw;

    #[test]
    fn playback_level_follows_speed_and_holds_its_endpoints() {
        let law = VelocityLaw::default();
        let row = KeyVoicing { level_db: [0.0, -3.0, -9.0], ..KeyVoicing::default() };
        let middle = law.hammer_speed(75);
        let upper = law.hammer_speed(105);
        for (speed, expected_db) in [
            (law.hammer_speed(1), 0.0),
            (law.hammer_speed(45), 0.0),
            (middle, -3.0),
            ((middle * upper).sqrt(), -6.0),
            (upper, -9.0),
            (law.hammer_speed(127), -9.0),
        ] {
            let actual_db = 20.0 * row.playback_gain(speed, &law).log10();
            assert!((actual_db - expected_db).abs() < 1e-8);
            assert_eq!(KeyVoicing::default().playback_gain(speed, &law), 1.0);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyVoicingTable([KeyVoicing; 88]);

impl Default for KeyVoicingTable {
    fn default() -> Self {
        Self([KeyVoicing::NEUTRAL; 88])
    }
}

impl KeyVoicingTable {
    pub fn at(&self, note: u8) -> KeyVoicing {
        note.checked_sub(21).and_then(|i| self.0.get(i as usize)).copied().unwrap_or_default()
    }

    pub fn set(&mut self, note: u8, value: KeyVoicing) -> Result<(), &'static str> {
        if !(21..=108).contains(&note) {
            return Err("voiced key must be in the MIDI range 21 through 108");
        }
        if value.unison_cents.is_some_and(|cents| cents.iter().any(|v| !v.is_finite())) {
            return Err("unison_cents values must be finite");
        }
        if value.wound_core_mm.is_some_and(|d| !d.is_finite() || d <= 0.0) {
            return Err("wound_core_mm must be finite and positive");
        }
        if !(value.felt_stiffness_scale.is_finite() && value.felt_stiffness_scale > 0.0) {
            return Err("felt_stiffness_scale must be finite and positive");
        }
        if !(value.strike_offset.abs() < 1.0) {
            return Err("strike_offset must be finite and smaller than one speaking length in magnitude");
        }
        if !(value.string_loss_scale.is_finite() && value.string_loss_scale > 0.0) {
            return Err("string_loss_scale must be finite and positive");
        }
        if !(value.bridge_coupling_scale.is_finite() && value.bridge_coupling_scale >= 0.0) {
            return Err("bridge_coupling_scale must be finite and nonnegative");
        }
        if !value.level_db.iter().all(|v| v.is_finite() && v.abs() <= 60.0) {
            return Err("level_db values must be finite and within ±60 dB");
        }
        if !value.strike_partials_db.iter().all(|v| v.is_finite() && v.abs() <= 18.0) {
            return Err("strike_partials_db values must be finite and within ±18 dB");
        }
        if !value.upper_strike_cut_db.iter().all(|v| (0.0..=60.0).contains(v)) {
            return Err("upper_strike_cut_db must be between 0 and 60 dB");
        }
        self.0[(note - 21) as usize] = value;
        Ok(())
    }

    #[cfg(feature = "serde")]
    pub(crate) fn is_empty(&self) -> bool {
        self.0.iter().all(|v| *v == KeyVoicing::NEUTRAL)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for KeyVoicingTable {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.0.iter().enumerate().filter(|(_, row)| **row != KeyVoicing::NEUTRAL).map(|(i, row)| ((i + 21).to_string(), row)))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for KeyVoicingTable {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let rows = std::collections::BTreeMap::<String, KeyVoicing>::deserialize(deserializer)?;
        let mut table = Self::default();
        for (key, row) in rows {
            let note = key.parse::<u8>().map_err(D::Error::custom)?;
            if key != note.to_string() {
                return Err(D::Error::custom("key_voicing keys must be decimal MIDI numbers without a sign or leading zeros"));
            }
            table.set(note, row).map_err(D::Error::custom)?;
        }
        Ok(table)
    }
}
