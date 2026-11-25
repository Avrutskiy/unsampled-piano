"""Convolve recorded bridge forces and send 128-frame acceleration blocks to stdout."""
import argparse
from pathlib import Path
import struct
import sys

import numpy as np
from scipy import fft, signal

KEYS = 88
BLOCK = 128
FACTOR = 4
KERNEL_LENGTH = 8192
ALIGNMENT = 48  # samples at 12 kHz, matching the bake's 4 ms delay
CHUNK = 4 * 48_000
# Adjacent windows have the entire causal FIR history, resampling margins
# and one preceding acceleration sample. All cuts are on the audio block grid.
HISTORY = FACTOR * KERNEL_LENGTH + 512
LOOKAHEAD = 512
WINDOW = ("kaiser", 9.0)


def force_records(file):
    header = struct.Struct("<IBB")
    while raw := file.read(header.size):
        pos, key, count = header.unpack(raw)
        data = np.frombuffer(file.read(4 * count), dtype="<f4")
        if key >= KEYS or not 1 <= count <= BLOCK or len(data) != count:
            raise ValueError("invalid bridge-force record")
        yield pos, key, data


class WaveBank:
    def __init__(self, directory):
        taps = np.load(directory / "bridge_fir.npy", mmap_mode="r")
        correction = np.load(directory / "source_correction.npy")
        low, high = np.triu_indices(KEYS)
        if taps.shape != (len(low), KERNEL_LENGTH) or correction.shape != (4097, KEYS):
            raise ValueError("incompatible wave bank")
        pair = np.zeros((KEYS, KEYS), dtype=int)
        pair[low, high] = pair[high, low] = np.arange(len(low))
        self.rows = []
        for key in range(KEYS):
            h = fft.irfft(fft.rfft(taps[pair[key]], axis=1) * correction[:, key], axis=1)
            h[key] = 0  # the struck voice already renders its own returning wave
            self.rows.append(h)

    def accelerations(self, forces):
        n = forces.shape[1]
        velocity = np.zeros((KEYS, (n + 3) // 4 + ALIGNMENT), dtype=np.float32)
        for key in np.flatnonzero(forces.any(axis=1)):
            x = signal.resample_poly(forces[key], 1, FACTOR, window=WINDOW)
            nonzero = np.flatnonzero(x)
            if not len(nonzero):
                continue
            start, end = nonzero[0], nonzero[-1] + 1
            for j in range(0, KEYS, 8):
                idx = slice(j, j + 8)
                conv = signal.oaconvolve(self.rows[key][idx], x[None, start:end], axes=1)
                stop = min(velocity.shape[1], start + conv.shape[1])
                velocity[idx, start:stop] += conv[:, :stop-start]
        velocity = signal.resample_poly(velocity[:, ALIGNMENT:], FACTOR, 1, axis=1, window=WINDOW)
        return np.diff(velocity[:, :n], axis=1, prepend=np.zeros((KEYS, 1), np.float32)) * 48000


def stream(bank, source, frames, dest):
    records = iter(force_records(source))
    pending = next(records, None)
    held = []
    for start in range(0, frames, CHUNK):
        stop = min(frames, start + CHUNK)
        left = max(0, start - HISTORY)
        right = min(frames, stop + LOOKAHEAD)
        held = [row for row in held if row[0] + len(row[2]) > left]
        while pending is not None and pending[0] < right:
            if pending[0] + len(pending[2]) > frames:
                raise ValueError("force record extends past end of score")
            held.append(pending)
            pending = next(records, None)
        forces = np.zeros((KEYS, right - left), dtype=np.float32)
        for pos, key, samples in held:
            a, b = max(pos, left), min(pos + len(samples), right)
            forces[key, a-left:b-left] += samples[a-pos:b-pos]
        acc = bank.accelerations(forces)[:, start-left:stop-left]
        if not np.isfinite(acc).all():
            raise ValueError("nonfinite acceleration")
        pad = (-acc.shape[1]) % BLOCK
        if pad:
            acc = np.pad(acc, ((0, 0), (0, pad)))
        dest.write(acc.reshape(KEYS, -1, BLOCK).transpose(1, 0, 2).astype("<f4", copy=False).tobytes())
    if pending is not None:
        raise ValueError("unexpected data after end of score")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bank", type=Path)
    parser.add_argument("work_dir", type=Path)
    args = parser.parse_args()
    frames = int((args.work_dir / "frames.txt").read_text())
    with (args.work_dir / "forces.bin").open("rb") as source:
        stream(WaveBank(args.bank), source, frames, sys.stdout.buffer)


if __name__ == "__main__":
    main()
