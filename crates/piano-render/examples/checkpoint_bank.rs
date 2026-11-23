//! Render independently struck keys for the concert checkpoint's acoustic comparison.
//! Each stereo take contains 1.024 s held and 0.512 s released at 48 kHz.
use piano_core::{Engine, EngineConfig, Event, TimedEvent};
use std::io::{BufWriter, Write};
use std::{env, fs, path::PathBuf, sync::Arc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let preset = args.next().ok_or("expected an instrument preset")?;
    let output = args.next().ok_or("expected a new output file")?;
    let mut model = piano_host::load_preset(&preset)?;
    for overlay in args {
        model = piano_host::load_listening(&overlay, &model)?;
    }
    const BLOCK: usize = 128;
    let config = EngineConfig {
        model,
        sample_rate: 48000.0,
        max_block: BLOCK,
        threads: 1,
        ..Default::default()
    };
    let instrument = Arc::new(config.quality.instrument(&config.model, config.sample_rate));
    let mut writer = BufWriter::new(fs::File::create_new(output)?);
    for velocity in [45, 56, 100] {
        for note in 21..=108 {
            let mut engine = Engine::with_instrument(config, Arc::clone(&instrument));
            let (mut left, mut right) = ([0.0; BLOCK], [0.0; BLOCK]);
            let on = [TimedEvent {
                frame: 0,
                event: Event::NoteOn { note, velocity },
            }];
            let off = [TimedEvent {
                frame: 0,
                event: Event::NoteOff { note, velocity: 64 },
            }];
            for block in 0..576 {
                let events: &[TimedEvent] = match block {
                    0 => &on,
                    384 => &off,
                    _ => &[],
                };
                engine.process_channels(events, &mut [&mut left, &mut right]);
                for (&l, &r) in left.iter().zip(&right) {
                    writer.write_all(&l.to_le_bytes())?;
                    writer.write_all(&r.to_le_bytes())?;
                }
            }
        }
    }
    writer.flush()?;
    Ok(())
}
