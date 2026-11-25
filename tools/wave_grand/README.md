# Render the wave-coupled concert grand

**Playback scope.** The Liszt recording uses the offline renderer. It applies the baked bridge transfer and the per-key touch calibration. The standalone player and plug-in can load the same grand preset and play it live with their existing sympathetic-string response and MIDI velocity mapping. Hearing the exact offline sound from a live keyboard requires further integration.

The offline renderer uses the physical model twice. It first captures the vertical force delivered by each struck string group. It then renders the performance with the solved motion of every other bridge contact driving the receiving strings. Both passes follow the complete MIDI timeline, including pedal changes and a four-second tail. Playback uses the touch calibration and a fixed stereo output gain.

The shell commands below run from the repository root and require Cargo, Python 3, NumPy, SciPy, and FFmpeg. Use a separate bank directory for each instrument and a separate work directory for each render. A full-size force capture can use several GB of temporary space. The block stream carries acceleration between processes.

```sh
set -e
set -o pipefail
preset='presets/instruments/Concert grand/wave_grand_balance_20261125_138.toml'
curve=tools/wave_grand/velocity_coefficients_20261125_115.csv
bank=target/wave-bank
work=target/wave-performance
midi=/path/to/performance.mid
mkdir -p "$bank" "$work"

cargo build --release -p piano-render --example wave_shapes --example wave_grand
target/release/examples/wave_shapes "$preset" "$bank"
python3 tools/wave_grand/bake.py "$bank"
python3 tools/wave_grand/verify.py "$bank"

target/release/examples/wave_grand capture "$preset" "$curve" "$midi" "$work"
python3 tools/wave_grand/stream_bridge.py "$bank" "$work" |
  target/release/examples/wave_grand listen "$preset" "$curve" "$midi" "$work"
ffmpeg -f f32le -ar 48000 -ac 2 -i "$work/stereo.f32" -c:a flac -sample_fmt s32 "$work/performance.flac"
```

The bank stores the upper triangle of the reciprocal transfer matrix and a per-source impedance correction. The 12 kHz impulse responses cover the listening band through 5 kHz. Streaming uses overlapping windows long enough for the entire impulse response, resampling filters, and backward velocity difference. The native engine runs continuously through window boundaries.

For this preset the solve uses 1,621 mass-normalized board modes. The verification script checks the A5–D6 transfer in both directions at their first three partials against an independent solve. The [voicing notes](../../docs/concert_grand_voicing_20261125_139.md) describe the source geometry, bass construction, touch, and listening decisions.
