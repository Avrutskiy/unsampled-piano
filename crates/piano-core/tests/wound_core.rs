use piano_core::key_voicing::KeyVoicing;
use piano_core::params::{PianoModel, StringGeometry};

#[test]
fn replacing_a_wound_core_preserves_pitch_and_outer_diameter() {
    let mut model = PianoModel::default();
    let before = model.note(21);
    let next = model.note(22);
    let young = model.nonlinear.young;
    let outer = StringGeometry::of(&before, young).d_outer;
    let diameter = outer * 0.55;
    model.key_voicing.set(21, KeyVoicing { wound_core_mm: Some(diameter * 1000.0), ..Default::default() }).unwrap();
    let after = model.note(21);
    let d4 = 64.0 * after.b * after.tension * after.length.powi(2) / (std::f64::consts::PI.powi(3) * young);
    assert!((d4.powf(0.25) - diameter).abs() < 1e-12);
    assert!((StringGeometry::of(&after, young).d_outer - outer).abs() < 1e-12);
    assert_eq!(before.f0, after.f0);
    assert_eq!(before.length, after.length);
    assert_eq!(model.note(22).b, next.b);
    assert_eq!(model.note(22).tension, next.tension);
}
