"""Recompute paired errors from two stereo float32 files produced by checkpoint_bank."""
import argparse
import csv
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
import numpy as np
import bench


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("studio_notes", type=Path)
    parser.add_argument("concert_notes", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    with open(bench.REFS) as stream:
        targets = bench.load_profile_refs(json.load(stream))
    scores = []
    for path in [args.studio_notes, args.concert_notes]:
        shape = (len(bench.LAYERS), len(bench.PROFILE_KEYS), 73728, 2)
        if path.stat().st_size != np.prod(shape) * 4:
            parser.error(f"{path} has an unexpected length for a complete stereo bank")
        bank = np.memmap(path, dtype="<f4", mode="r", shape=shape)
        profiles = {}
        for i, layer in enumerate(bench.LAYERS):
            for j, midi in enumerate(bench.PROFILE_KEYS):
                signal = bank[i, j].astype(float)
                if not np.isfinite(signal).all() or not np.any(signal):
                    parser.error(f"{path} has an invalid take at key {midi}, layer {layer}")
                for phase, (start, duration) in bench.PROFILE_WINDOWS.items():
                    levels = bench.stereo_partial_levels(signal, midi, start, duration)
                    relative = levels[1:] - levels[0]
                    profiles[f"{midi}|{layer}|{phase}"] = [
                        float(v) if np.isfinite(v) else None for v in relative
                    ]
        scores.append(bench.score_profiles(profiles, targets))
    studio, concert = scores
    with args.output.open("x", newline="") as stream:
        writer = csv.writer(stream, lineterminator="\n")
        writer.writerow(["midi", "velocity", "phase", "partial", "recorded_grands",
                         "studio_minus_reference_mean_db", "concert_minus_reference_mean_db"])
        for key, row in concert.items():
            for new, old in zip(row["components"], studio[key]["components"]):
                if new["delta_db"] is None or old["delta_db"] is None:
                    continue
                writer.writerow([row["midi"], row["velocity"], row["phase"], new["partial"],
                                 len(new["refs"]), round(old["delta_db"], 3), round(new["delta_db"], 3)])


if __name__ == "__main__":
    main()
