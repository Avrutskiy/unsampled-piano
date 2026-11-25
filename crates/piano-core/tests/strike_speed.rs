//! Numerical-speed events and mapped MIDI events use the same physical strike.
use piano_core::{Engine, EngineConfig, Event, TimedEvent};
use std::sync::Arc;

#[test]
fn mapped_midi_and_direct_speed_match_through_the_dry_instrument() {
    let mut cfg = EngineConfig::default();
    cfg.model.room.enabled = false;
    cfg.model.limiter = 0.0;
    cfg.model.body.enabled = true;
    cfg.model.body.gain = 0.05;
    cfg.model.noise.action = 0.012;
    cfg.model.noise.case = 0.05;
    cfg.model.repetition_time = 0.12;
    let instrument = Arc::new(cfg.quality.instrument(&cfg.model, cfg.sample_rate));
    let render = |physical| {
        let mut engine = Engine::with_instrument(cfg, Arc::clone(&instrument));
        let on = |velocity| {
            if physical {
                Event::Strike { note: 60, speed_mps: cfg.model.velocity.hammer_speed(velocity) }
            } else {
                Event::NoteOn { note: 60, velocity }
            }
        };
        let mut result = Vec::new();
        for block in 0..96 {
            let frame = 11;
            let event = match block {
                1 => Some(on(39)),
                2 => Some(on(39)), // Same keystroke on two paths.
                9 => Some(Event::NoteOff { note: 60, velocity: 37 }),
                16 => Some(on(82)), // Strike a moving string before the action recovers.
                56 => Some(Event::NoteOff { note: 60, velocity: 60 }),
                64 => Some(on(112)),
                _ => None,
            };
            let mut l = [0.0f32; 128];
            let mut r = [0.0f32; 128];
            engine.process_stereo(event.map(|event| TimedEvent { frame, event }).as_slice(), &mut l, &mut r);
            result.extend(l.into_iter().zip(r));
        }
        assert_eq!(engine.doubled_notes(), 1);
        result
    };
    let midi = render(false);
    assert!(midi.iter().any(|&(l, r)| l.abs() + r.abs() > 0.001));
    assert_eq!(midi, render(true));
}
