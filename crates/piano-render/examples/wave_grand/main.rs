mod touch;
use piano_core::{Engine, EngineConfig, Event, TimedEvent};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;

const RATE: usize = 48_000;
const BLOCK: usize = 128;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 5 {
        return Err("usage: wave_grand capture|listen PRESET CURVE MIDI WORK_DIR".into());
    }
    let capture = match args[0].to_str() {
        Some("capture") => true,
        Some("listen") => false,
        _ => return Err("expected capture or listen".into()),
    };
    let dir = Path::new(&args[4]);
    let mut model = piano_host::load_preset(Path::new(&args[1]))?;
    let curve = touch::TouchCurve::load(Path::new(&args[2])).ok_or("invalid touch curve")?;
    let events: Vec<_> = piano_host::read_midi_file(Path::new(&args[3]))?
        .into_iter()
        .map(|(time, event)| {
            let event = match event {
                Event::NoteOn { note, velocity } if velocity > 0 && (21..=108).contains(&note) => {
                    curve.strike(note, velocity, false)
                }
                other => other,
            };
            ((time * RATE as f64).round() as usize, event)
        })
        .collect();
    let frames = events.last().ok_or("empty MIDI")?.0 + 4 * RATE;
    eprintln!(
        "{} MIDI events, {} frames including four seconds of tail",
        events.len(),
        frames
    );
    if !capture {
        model.sympathetic.self_cancel = false;
    }
    let mut engine = Engine::new(EngineConfig {
        model,
        sample_rate: RATE as f64,
        max_block: BLOCK,
        threads: 4,
        ..Default::default()
    });
    std::fs::create_dir_all(dir)?;
    let mut forces = if capture {
        engine.enable_wave_capture();
        Some(BufWriter::new(File::create(dir.join("forces.bin"))?))
    } else {
        None
    };
    let mut output = if capture {
        None
    } else {
        Some(BufWriter::new(File::create(dir.join("stereo.f32"))?))
    };
    let mut input = BufReader::new(io::stdin().lock());
    let mut next = 0;
    let mut block_events = Vec::new();
    for start in (0..frames).step_by(BLOCK) {
        let n = (frames - start).min(BLOCK);
        block_events.clear();
        while next < events.len() && events[next].0 < start + n {
            let (f, ev) = events[next];
            block_events.push(TimedEvent {
                frame: (f - start) as u32,
                event: ev,
            });
            next += 1;
        }
        if !capture {
            let mut buf = [0u8; BLOCK * 4];
            for key in engine.wave_input() {
                input.read_exact(&mut buf)?;
                for (x, b) in key[..n].iter_mut().zip(buf.chunks_exact(4)) {
                    *x = f32::from_le_bytes(b.try_into()?);
                }
            }
        }
        let (mut l, mut r) = ([0.; BLOCK], [0.; BLOCK]);
        engine.process_channels(&block_events, &mut [&mut l[..n], &mut r[..n]]);
        if let Some(f) = &mut forces {
            for (key, data) in engine.wave_forces().iter().enumerate() {
                if data[..n].iter().any(|&v| v != 0.0) {
                    f.write_all(&u32::try_from(start)?.to_le_bytes())?;
                    f.write_all(&[key as u8, n as u8])?;
                    for &x in &data[..n] {
                        f.write_all(&x.to_le_bytes())?;
                    }
                }
            }
        }
        if let Some(out) = &mut output {
            for i in 0..n {
                for s in [l[i], r[i]] {
                    if !s.is_finite() {
                        return Err("non-finite sample".into());
                    }
                    out.write_all(&(s * 0.818052_f32).to_le_bytes())?;
                }
            }
        }
        if start % (RATE * 60) == 0 {
            eprintln!("processed {} s", start / RATE);
        }
    }
    if let Some(f) = &mut forces {
        f.flush()?;
        std::fs::write(dir.join("frames.txt"), frames.to_string())?;
    }
    if let Some(f) = &mut output {
        f.flush()?;
        if input.read(&mut [0u8; 1])? != 0 {
            return Err("unexpected extra acceleration frames".into());
        }
    }
    Ok(())
}
