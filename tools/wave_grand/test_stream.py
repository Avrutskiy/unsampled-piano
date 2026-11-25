"""Regression for a held wave across stream windows and a partial final block."""
import io
import struct
import unittest
from unittest.mock import patch

import numpy as np
import stream_bridge as bridge


class StreamTest(unittest.TestCase):
    def test_seams_and_last_block(self):
        chunk = 36864
        frames = 2 * chunk + 177
        bank = bridge.WaveBank.__new__(bridge.WaveBank)
        bank.rows = {0: np.zeros((88, bridge.KERNEL_LENGTH), dtype=np.float32)}
        bank.rows[0][1, [12, 640, 8180]] = [0.2, 0.8, -0.5]
        forces = np.zeros((88, frames), dtype=np.float32)
        records = io.BytesIO()
        for start in [0, chunk - 128, chunk, 2 * chunk - 128, 2 * chunk + 128]:
            n = min(frames - start, 128)
            strike = np.sin(np.arange(n) * .03).astype("<f4")
            forces[0, start:start+n] = strike
            records.write(struct.pack("<IBB", start, 0, n) + strike.tobytes())
        expected = bank.accelerations(forces)
        expected = np.pad(expected, ((0, 0), (0, -frames % 128)))
        records.seek(0)
        dest = io.BytesIO()
        with patch.object(bridge, "CHUNK", chunk):
            bridge.stream(bank, records, frames, dest)
        result = np.frombuffer(dest.getbuffer(), dtype="<f4").reshape(-1, 88, 128)
        result = result.transpose(1, 0, 2).reshape(expected.shape)
        np.testing.assert_allclose(result, expected, rtol=1e-4, atol=0.025)
        self.assertLess(np.linalg.norm(result-expected)/np.linalg.norm(expected), 2e-5)


if __name__ == "__main__":
    unittest.main()
