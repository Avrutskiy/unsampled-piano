use crate::board::{BoardBank, REGIONS};
use crate::engine::SUB_BLOCK;

/// One-way mechanical path for the brief felt contact pulse. The extra board
/// motion drives listening strings and is never mixed straight into the microphones.
pub(crate) struct SympatheticStrike {
    pub(crate) board: BoardBank,
    pub(crate) input: Vec<Vec<f32>>,
    velocity: Vec<Vec<f32>>,
    last: [f32; REGIONS],
    delayed: [[f32; SUB_BLOCK]; REGIONS],
    rear_gain: f32,
    pub(crate) speaking_gain: f32,
    pub(crate) speaking: Vec<Vec<f32>>,
}
impl SympatheticStrike {
    pub(crate) fn new(inst: &crate::instrument::Instrument, rate: f64) -> Option<Self> {
        let gain = inst.model.sympathetic.rear_attack_gain;
        let speaking = inst.model.sympathetic.speaking_attack_gain;
        if !inst.model.sympathetic.enabled
            || !gain.is_finite()
            || !speaking.is_finite()
            || (gain <= 0.0 && speaking <= 0.0)
        {
            return None;
        }
        let modes: Vec<_> = inst
            .board
            .as_ref()?
            .iter()
            .copied()
            .filter(|m| m.freq_hz < 4500.0)
            .collect();
        let buffers = || vec![vec![0.0; SUB_BLOCK]; REGIONS];
        Some(Self {
            board: BoardBank::new(rate, &modes, SUB_BLOCK),
            input: buffers(),
            velocity: buffers(),
            last: [0.0; REGIONS],
            delayed: [[0.0; SUB_BLOCK]; REGIONS],
            rear_gain: gain.max(0.0) as f32,
            speaking_gain: speaking.max(0.0) as f32,
            speaking: buffers(),
        })
    }
    pub(crate) fn add(
        input: &mut [Vec<f32>],
        region: usize,
        pulse: &[f32],
        from: usize,
        to: usize,
    ) {
        for (sum, p) in input[region][from..to].iter_mut().zip(pulse) {
            *sum += *p;
        }
    }
    pub(crate) fn prepare_listeners(&mut self, original: &[Vec<f32>], len: usize, clock: u64) {
        for (r, src) in original.iter().enumerate() {
            for i in 0..len {
                self.speaking[r][i] =
                    src[i] + self.speaking_gain * self.delayed[r][(clock as usize + i) % SUB_BLOCK];
            }
        }
    }
    pub(crate) fn drive(&mut self, acceleration: &mut [Vec<f32>], len: usize, clock: u64, fs: f32) {
        for v in &mut self.velocity {
            v[..len].fill(0.0);
        }
        self.board
            .process(&self.input, len, &mut [], Some(&mut self.velocity), None);
        for (r, a) in acceleration.iter_mut().enumerate() {
            for (i, dst) in a[..len].iter_mut().enumerate() {
                let idx = (clock as usize + i) % SUB_BLOCK;
                *dst += self.rear_gain * self.delayed[r][idx];
                let v = self.velocity[r][i];
                self.delayed[r][idx] = (v - self.last[r]) * fs;
                self.last[r] = v;
            }
            self.input[r][..len].fill(0.0);
        }
    }
}
