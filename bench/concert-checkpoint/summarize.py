"""Summarize paired acoustic harmonic measurements using only the standard library."""
import csv
from pathlib import Path
from statistics import median

REGISTERS = [(21, 47, "Bass"), (48, 71, "Middle"), (72, 95, "Treble"), (96, 108, "Top")]


def main():
    with Path(__file__).with_name("harmonics.csv").open(newline="") as stream:
        rows = list(csv.DictReader(stream))
    print("Register   Paired rows   Studio (dB)   Concert (dB)")
    for lo, hi, name in REGISTERS:
        selected = [r for r in rows if lo <= int(r["midi"]) <= hi]
        values = [
            median(abs(float(row[f"{model}_minus_reference_mean_db"])) for row in selected)
            for model in ("studio", "concert")
        ]
        print(f"{name:<9} {len(selected):>11} {values[0]:>13.2f} {values[1]:>14.2f}")


if __name__ == "__main__":
    main()
