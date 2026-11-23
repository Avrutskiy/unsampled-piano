# Concert grand listening checkpoint · 23 November 2026

The [concert reconstruction](../presets/instruments/Concert%20grand/measured_concert_grand_20261123_23.toml) is the reference voicing for subsequent listening work on this fork. In one listener's blind C-major chord comparison it joined Pianoteq in the preferred group in the middle and high registers. The listener preferred Pianoteq in the bass and described the reconstruction as lacking depth. The comparison used one dynamic and one listening setup.

## The instrument

The starting point is the fork's [Concert grand](../presets/instruments/Concert%20grand/concert_grand.toml), a physical model of a 2.74 m instrument informed by the IRCAM grand. The checkpoint freezes a fully resolved preset. Its only numerical changes from that parent at `ac7caee` are the two bass coefficients of the string inharmonicity law. It uses the parent’s felt, damper, board, radiation and microphone settings. They combine measurements and empirical voicing from several instruments. The model's provenance and the source of each dimension are in the [scale notes](steinway_d.md) and [Chabassier and Duruflé's research bibliography](https://juliettechabassier.perso.math.cnrs.fr/public/focus/2023/piano/).

| Part | Computation and physical meaning |
|---|---|
| Hammer | MIDI velocity maps to impact speed in m/s. A flexible shank and a nonlinear felt compression law determine the force during contact. A faster strike changes the shape and duration of the force pulse as well as its amplitude. |
| Strings | The instrument has 243 strings across 88 keys. The A0 speaking length is 2.007 m, C2 is 1.600 m and C8 is 0.049 m. Each key excites one to three modal strings with tension, stiffness and loss. A0 has one wound string and C2 has three. Their calculated static tensions sum to 189.64 kN across the instrument. |
| Bridge and soundboard | A key drives the board at its bridge location. The structural model includes a tapered spruce plate with different stiffness along and across the grain, 17 ribs with feathered ends, both bridges, string loading and rotational support at the rim. It solves the small-vibration eigenproblem on a 1 cm grid, with a 6 kHz structural cutoff. The loaded reference calculation returned 825 structural modes, with the first three at 78.2, 99.6 and 135.8 Hz. |
| Radiation and release | Board motion contributes to pressure at the configured microphones. The render also includes case, action and room sound. The renderer approximates radiation using stored modal weights and propagation delays. Key release brings the felt damper back onto a damped string. Undamped upper strings and the board can continue ringing. The sustain pedal controls damper contact and sympathetic response. |

For MIDI key number $m$, the engine computes the dimensionless stiffness coefficient as $B(m)=\exp(a_b m+b_b)+\exp(a_t m+b_t)$. This sets how far upper partials depart from integer multiples of the fundamental. The bass coefficients change from $(-0.0730,-6.901)$ to $(-0.1163,-6.727)$, the fit documented for the wrapped-string scale in INRIA RT-425. The treble coefficients stay at $(0.0903,-13.526)$. The earlier Studio Grand experiments also changed the scale, board and voicing. Comparisons against Studio Grand therefore describe two whole instruments and cannot isolate the effect of this coefficient change.

## Blind listening result

The three sources were the concert reconstruction, the [feathered Studio Grand comparator](../bench/concert-checkpoint/feathered_studio_comparator.toml) and Pianoteq 9's NY Steinway D Classical preset. Each register had an independently shuffled order. The three source MIDIs contain simultaneous C-major triads and inversions at velocity 75, held 680 ms, with 1.55 s between onsets and the pedal up. The Pianoteq reference used the earlier fixed −5.45 dB calibration and all clips shared one subsequent playback gain. The listener heard the instruments with their respective listening setups.

| Register | Take 1 | Take 2 | Take 3 | Listener's assessment |
|---|---|---|---|---|
| Low | Studio | Concert | Pianoteq | Pianoteq preferred. Concert second with less bass depth. Studio last. |
| Middle | Concert | Pianoteq | Studio | Concert and Pianoteq both good. Studio last. |
| High | Concert | Pianoteq | Studio | Concert and Pianoteq both good. Studio last. |

On the earlier melodic phrase the listener identified all three instruments and heard some upper notes as loud. In the C-major fixture the reconstruction's last two treble attacks were 5.1 and 7.2 dB above Pianoteq under the fixed gain. The register preference and the last chords' balance are distinct observations. Stereo RMS levels in [chord_levels.csv](../bench/concert-checkpoint/chord_levels.csv) use the first 120 ms for the attack and 200–600 ms for the held sound, before the common playback gain.

## Recorded-grand comparison

The two model presets were each rendered on all 88 keys at velocities 45, 56 and 100. Every take starts from rest, holds for 1.024 s and rings for 0.512 s after note-off. Both use the benchmark's playback touch mapping, close microphones and studio room. The eight acoustic target collections supply stored measurements. Coverage varies by key and partial. Pianoteq is used only in listening comparisons.

For each key, velocity and time window, the [existing bench estimator](../tools/bench.py) averages channel powers and measures partials 2–8 in dB relative to the fundamental. Attack covers 0–60 ms and sustain covers 500–1000 ms from detected acoustic onset. The target is the mean in dB among available acoustic grands for that same component. Each row retains the signed model-minus-target difference when both models and at least two references have a measurement. The table gives the median absolute difference across paired rows, weighting each row equally.

| Register | Paired rows | Studio (dB) | Concert (dB) |
|---|---:|---:|---:|
| Bass, A0–B2 | 1,134 | 13.63 | 6.50 |
| Middle, C3–B4 | 1,008 | 5.86 | 5.55 |
| Treble, C5–B6 | 708 | 8.90 | 7.78 |
| Top, C7–C8 | 104 | 14.34 | 7.52 |

These values describe overtone balance. The listener's bass-depth preference also depends on absolute levels and decay. The physical-synthesis checkpoint establishes a listening baseline. Derivatives with respect to performance timing and strike strength require separate validation.

## Reproduce

From the repository root, render the [low](../bench/concert-checkpoint/low.mid), [middle](../bench/concert-checkpoint/mid.mid), [high](../bench/concert-checkpoint/high.mid) or [melodic](../bench/concert-checkpoint/phrase.mid) fixture. The command below uses the same instrument and full quality as the listening take. For the register clips, allow the last chord to decay for at least 3 s.

```sh
concert='presets/instruments/Concert grand/measured_concert_grand_20261123_23.toml'
cargo run --release -p piano-render -- --preset "$concert" --tail 3.2 midi bench/concert-checkpoint/low.mid low.wav
python3 bench/concert-checkpoint/summarize.py
```

To regenerate [harmonics.csv](../bench/concert-checkpoint/harmonics.csv) into a fresh output directory, use Python with NumPy and pandas. The example uses the normal engine, shares its instrument tables, and resets its playing state for each strike.

```sh
work=$(mktemp -d)
fixture=bench/concert-checkpoint
bank() {
  cargo run --release -p piano-render --example checkpoint_bank -- "$1" "$2" \
    archive/presets/touch/disklavier_playback.toml \
    'archive/presets/listening/Microphones/close.toml' \
    'archive/presets/listening/Room/studio.toml'
}
bank "$fixture/feathered_studio_comparator.toml" "$work/studio.raw"
bank "$concert" "$work/concert.raw"
python3 "$fixture/measure.py" "$work/studio.raw" "$work/concert.raw" "$work/harmonics.csv"
diff "$fixture/harmonics.csv" "$work/harmonics.csv"
```

The original blind files used engine commit `fe17c2f`. With the same presets on fork main `ac7caee`, the concert register waveforms differed by −105.1, −105.3 and −100.2 dB RMS relative to those original files, respectively. The residual is 20 log10 of RMS(main minus original) divided by RMS(original), computed over both channels. All 528 isolated renders across the two instruments were sample-identical between builds. Those test events fall on block boundaries. The earlier engine fix moves keybed and pedal forces to their exact event frames. It remains on the original feature branch. This checkpoint carries the tested instrument and comparison protocol.
