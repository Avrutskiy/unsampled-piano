use std::io::{BufWriter, Write};
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 2 {
        return Err("usage: wave_shapes PRESET BANK_DIRECTORY".into());
    }
    std::fs::create_dir_all(&a[1])?;
    let model = piano_host::load_preset(Path::new(&a[0]))?.with_macros();
    let board = model.board().ok_or("preset requires a soundboard")?;
    let shape: Vec<_> = (21..=108).map(|k| board.bridge_shapes(k)).collect();
    let mut modes = BufWriter::new(std::fs::File::create(Path::new(&a[1]).join("plate.txt"))?);
    for (m, md) in board.modes.iter().enumerate() {
        write!(modes, "{} {}", md.omega, board.sigma(md) - md.load)?;
        for row in &shape {
            write!(modes, " {}", row[m])?;
        }
        writeln!(modes)?;
    }
    let mut strings = BufWriter::new(std::fs::File::create(Path::new(&a[1]).join("strings.txt"))?);
    for k in 21..=108 {
        let n = model.note(k);
        writeln!(
            strings,
            "{} {} {} {} {} {}",
            k, n.tension, n.mu, n.strings, n.length, n.f0
        )?;
    }
    modes.flush()?;
    strings.flush()?;
    Ok(())
}
