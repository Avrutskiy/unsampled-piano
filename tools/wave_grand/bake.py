"""Bake the 88-contact vertical mobility of a grand from wave_shapes exports."""
import argparse
from pathlib import Path

import numpy as np

sample_rate = 12_000
length = 8192
lo, hi = np.triu_indices(88)


def bake(directory):
    modes = np.loadtxt(directory / "plate.txt")
    strings = np.loadtxt(directory / "strings.txt")
    omega, damping = modes[:, :2].T
    phi = modes[:, 2:]
    impedance = strings[:, 3] * np.sqrt(strings[:, 1] * strings[:, 2])
    if not (np.all(damping > 0) and np.all(impedance > 0)):
        raise ValueError("the bake requires positive modal losses and string impedances")
    z = np.sqrt(impedance)
    pairs = np.ascontiguousarray(phi[:, lo] * phi[:, hi])
    freq = np.fft.rfftfreq(length, 1 / sample_rate)
    h = np.empty((len(freq), len(lo)), dtype="complex64")
    identity = np.eye(88)
    worst = reciprocity = 0.0
    least = np.inf
    for start in range(0, len(freq), 32):
        w = 2 * np.pi * freq[start:start+32, None]
        poles = 1j*w / (omega**2 - w**2 + 2j*damping*w)
        values = poles.real @ pairs + 1j*(poles.imag @ pairs)
        bare = np.empty((len(w), 88, 88), dtype=complex)
        bare[:, lo, hi] = bare[:, hi, lo] = values
        # Power-normalized admittance makes the loaded solve reciprocal.
        scaled = bare * z[:, None] * z[None, :]
        loaded = np.linalg.solve(identity + scaled, scaled)
        residual = (identity + scaled) @ loaded - scaled
        worst = max(worst, np.max(abs(residual)))
        reciprocity = max(reciprocity, np.max(abs(loaded - loaded.transpose(0, 2, 1))))
        if start % 256 == 0:
            least = min(least, np.linalg.eigvalsh(loaded[-1].real).min())
        h[start:start+len(w)] = (loaded / z[:, None] / z[None, :])[:, lo, hi]
    if worst > 1e-10 or reciprocity > 1e-10 or least < -1e-10:
        raise ValueError(f"invalid solve: residual {worst}, reciprocity {reciprocity}, passivity {least}")
    # Delivered force is known. Remove the emitter's own termination using a
    # rank-one update, then use the same symmetric bank for every emitter.
    correction = 1 / (1 - h[:, lo == hi] * impedance)
    roll = np.clip((freq - 5000) / 900, 0, 1)
    window = (1 + np.cos(np.pi * roll)) / 2
    h *= (window * np.exp(-2j * np.pi * freq * .004))[:, None]
    taps = np.empty((len(lo), length), dtype="float32")
    for start in range(0, len(lo), 48):
        taps[start:start+48] = np.fft.irfft(h[:, start:start+48], n=length, axis=0).T
    tail = taps[:, -length//8:]
    tail_fraction = np.sum(tail.astype(float)**2) / np.sum(taps.astype(float)**2)
    if not tail_fraction < 5e-5:
        raise ValueError(f"IR window too short, tail energy fraction {tail_fraction}")
    tail *= np.linspace(1., 0., length//8)**2
    np.save(directory / "source_correction.npy", correction.astype("complex64"))
    np.save(directory / "bridge_fir.npy", taps)
    print(f"{len(omega)} board modes, residual {worst:.3g}, tail energy fraction {tail_fraction:.3g}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bank_directory", type=Path)
    bake(parser.parse_args().bank_directory)
