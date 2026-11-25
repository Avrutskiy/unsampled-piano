"""Compare the baked source-to-receiver transfer with an independent direct solve."""
import argparse
from pathlib import Path

import numpy as np
from scipy import fft
from bake import sample_rate, length, lo, hi


def verify(directory):
    plate = np.loadtxt(directory / "plate.txt")
    shapes = plate[:, 2:]
    strings = np.loadtxt(directory / "strings.txt")
    impedance = strings[:, 3] * np.sqrt(strings[:, 1] * strings[:, 2])
    taps = np.load(directory / "bridge_fir.npy", mmap_mode="r")
    correction = np.load(directory / "source_correction.npy")
    for note, receiver in [(81, 86), (86, 81)]:
        k, j = note - 21, receiver - 21
        pair = np.flatnonzero((lo == min(k,j)) & (hi == max(k,j)))[0]
        transfer = fft.rfft(taps[pair]) * correction[:, k]
        for partial in (1, 2, 3):
            index = round(strings[k, 5] * partial * length / sample_rate)
            omega = 2 * np.pi * index * sample_rate / length
            poles = 1j*omega / (plate[:, 0]**2 - omega**2 + 2j*omega*plate[:, 1])
            bare = (shapes.T * poles) @ shapes
            load = impedance.copy()
            load[k] = 0
            direct = np.linalg.solve(np.eye(88) + bare * load, bare[:, k])
            aligned = transfer[index] * np.exp(1j*.004*omega)
            error = abs(aligned - direct[j]) / abs(direct[j])
            print(f"{note} to {receiver}, partial {partial}, relative error {error:.5g}")
            if not error < .002:
                raise ValueError("transfer differs from direct solve")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bank_directory", type=Path)
    verify(parser.parse_args().bank_directory)
