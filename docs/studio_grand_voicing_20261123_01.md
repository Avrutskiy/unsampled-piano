# Studio grand, first treble voicing

The [Studio grand preset](../presets/instruments/Studio%20grand/studio_grand_20261123_01.toml) starts with the stock two-metre grand at commit `ac7caee`. Two changes affect the upper register. The top hammer-felt stiffness grows faster along the keyboard (`hardness_slope_top` from 0.15 to 0.20). The unison width increases gradually towards 7.5 cents at C8 (stock 5 cents). The first change affects the hammer pulse. The second changes how energy in the slightly detuned strings beats and decays. The shank setting in the preset explicitly selects the stock instrument's 700 Hz middle-register value.

## Comparison with recorded grands

The instrument was evaluated against the repository's existing [acoustic reference profiles](../bench/harmonic_references.json). These are upstream measurements. We generated fresh model renders and used only sources marked `target` in [the reference inventory](../bench/references.json). Commercial modeled instruments supplied no targets for this voicing.

Six design keys (MIDI 87, 91, 95, 99, 103 and 107) were rendered at velocities 45, 56 and 100. We compared five hammer slopes (0.05, 0.10, 0.15, 0.20 and 0.25), then compared unison widths of 2.5, 5 and 7.5 cents at slope 0.20, with a narrower-unison control at slope 0.15. The chosen pair had the lowest spread-normalized error for both attack and sustain among these candidates. We froze it before evaluating seven additional upper keys and four lower controls.

For each note we used the benchmark's close microphones, studio room and Disklavier playback touch mapping at 48 kHz. We held the key for 1.2 s, rendered another 60 ms after release, and measured partials 2–8 relative to the fundamental with `tools/bench.py`'s stereo-power estimator. Windows start at its detected acoustic onset and cover 0–60 ms and 500–1000 ms. The real-piano target is the mean in dB over available acoustic instruments for that key, velocity and partial, with at least two references required. The table uses only partials measurable in both renders. One design-set attack partial (MIDI 107, velocity 45, partial 3) dropped below the estimator's detection criteria. All held-out comparisons retained identical coverage.

| Keys and window | Paired partial measurements | Mean absolute error, stock → voiced (dB) | RMS error in reference spreads, stock → voiced |
| --- | ---: | ---: | ---: |
| Six design keys, attack | 61 | 8.56 → 8.85 | 2.48 → 2.35 |
| Six design keys, sustain | 40 | 14.08 → 13.38 | 3.90 → 3.70 |
| Seven held-out treble keys, attack | 66 | 7.98 → 8.20 | 1.62 → 1.56 |
| Seven held-out treble keys, sustain | 46 | 14.35 → 12.61 | 3.64 → 2.95 |
| Four lower controls, attack | 78 | 9.11 → 9.07 | 1.75 → 1.74 |
| Four lower controls, sustain | 78 | 6.68 → 6.79 | 1.46 → 1.49 |

The held-out treble keys were MIDI 85, 89, 93, 97, 101, 105 and 108. The lower controls were 60, 69, 76 and 82. The spread-normalized column divides each error by the acoustic standard deviation with the benchmark's 4 dB floor. [Measurements by key, velocity and partial](../bench/studio_grand_partials_20261123_01.csv) include both levels, the target mean and the denominator.

The clearest improvement was in sustained treble color. Across attack and sustain together, F7's mean absolute error fell from 9.43 to 6.70 dB and C8's from 23.25 to 18.08 dB. A7 worsened from 15.04 to 17.15 dB. All seven keys still gained attack level at each successively stronger strike. Their median velocity-45 to velocity-100 increase in stereo RMS over the first 100 ms moved from 18.60 to 17.87 dB. This is a selectable voicing. Listener preference, the beat trajectories of individual unisons and continuous strike derivatives remain untested.

For a stock A0 control at velocity 56 (held 600 ms, followed by 400 ms of release), the full stereo output differed by −73.06 dB RMS relative to the stock render. The overall RMS level moved by less than 0.00001 dB.

## Timing of mechanical impacts

The keybed and sustain-pedal forces now enter the bridge at their event frame. Previously all new forces in a 64-sample sub-block began at its first sample. Two single-event MIDI files placed a key strike and a pedal press separately at sample 63 from silence. At 48 kHz both stock renders had nonzero output at sample 0. Both revised renders were silent through sample 62 and first produced output at sample 63. The test in `crates/piano-core/tests/pedals.rs` checks the bridge force across that boundary.

## Audition

The [eight-second audition phrase](../bench/studio_grand_phrase_20261123_01.mid) plays four rolled chords beneath a treble line with repeated notes and sustain pedal. Render it from the repository root.

```sh
cargo run --release -p piano-render -- midi bench/studio_grand_phrase_20261123_01.mid studio.wav \
  --preset 'presets/instruments/Studio grand/studio_grand_20261123_01.toml'
```

Omit the preset to hear the stock two-metre grand with the same renderer. Keep the listening setup and output gain the same in both runs.
