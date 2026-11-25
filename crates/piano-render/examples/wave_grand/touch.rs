use piano_core::Event;
use std::{fs, path::Path};

#[derive(Clone, Copy)]
struct KeyCurve {
    low_shift: f64,
    high_shift: f64,
    linear: f64,
    quadratic: f64,
    floor: f64,
    floor_power: f64,
}

impl KeyCurve {
    fn evaluate(&self, input: f64) -> (f64, f64) {
        // Each displacement and its first derivative vanish at both ends.
        let (phase, shift) = if input <= 32.0 {
            ((input - 1.0) / 31.0, self.low_shift)
        } else {
            ((input - 32.0) / 95.0, self.high_shift)
        };
        let physical = input + 16.0 * shift * (phase * (1.0 - phase)).powi(2);
        let x = ((input + 4.0) / 36.0).ln();
        let mut trim = self.linear * x + self.quadratic * x * x;
        if input < 3.0 {
            let t = (input - 1.0) * 0.5;
            let smooth = t * t * (3.0 - 2.0 * t);
            let amplitude = self.floor + (1.0 - self.floor) * smooth.powf(self.floor_power);
            trim += 20.0 * amplitude.log10();
        }
        (physical, trim)
    }
}

pub struct TouchCurve {
    keys: Vec<KeyCurve>,
}

impl TouchCurve {
    pub fn load(path: &Path) -> Option<Self> {
        let data = fs::read_to_string(path).ok()?;
        let mut lines = data.lines();
        if lines.next()?
            != "midi_key,low_shift,high_shift,log_gain_linear,log_gain_quadratic,floor_amplitude,floor_power"
        {
            return None;
        }
        let mut keys = Vec::with_capacity(88);
        for key in 21..=108 {
            let mut fields = lines.next()?.split(',');
            if fields.next()?.parse::<u8>().ok()? != key {
                return None;
            }
            let mut number = || fields.next()?.parse::<f64>().ok().filter(|x| x.is_finite());
            let curve = KeyCurve {
                low_shift: number()?,
                high_shift: number()?,
                linear: number()?,
                quadratic: number()?,
                floor: number()?,
                floor_power: number()?,
            };
            if fields.next().is_some()
                || curve.low_shift.abs() > 6.81
                || curve.high_shift.abs() > 15.01
                || curve.floor <= 0.0
                || curve.floor > 1.0
                || !(1.0..=10.0).contains(&curve.floor_power)
            {
                return None;
            }
            keys.push(curve);
        }
        if lines.next().is_some() {
            return None;
        }
        Some(Self { keys })
    }

    pub fn strike(&self, note: u8, velocity: u8, firmer: bool) -> Event {
        let (physical_velocity, trim_db) =
            self.keys[usize::from(note - 21)].evaluate(f64::from(velocity));
        Event::TouchStrike {
            note,
            physical_velocity,
            trim_db,
            firmer,
        }
    }
}
