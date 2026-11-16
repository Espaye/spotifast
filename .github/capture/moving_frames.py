"""Count the frames of a recording that differ from the frame before.

Usage: python -I moving_frames.py VIDEO WIDTH HEIGHT
"""
import subprocess
import sys

import numpy

path, width, height = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
raw = subprocess.run(
    ["ffmpeg", "-nostdin", "-loglevel", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "gray", "-"],
    capture_output=True, check=True).stdout
frames = numpy.frombuffer(raw, dtype=numpy.uint8).reshape(-1, height, width).astype(numpy.int16)
steps = numpy.abs(frames[1:] - frames[:-1])
ever = steps.max(axis=0) > 12
rows, cols = numpy.where(ever.any(axis=1))[0], numpy.where(ever.any(axis=0))[0]
print(path.replace("\\", "/").split("/")[-1], "frames", len(frames),
      "moving area rows %d-%d columns %d-%d" % (rows[0], rows[-1], cols[0], cols[-1]))
area = steps[:, rows[0]:rows[-1] + 1, cols[0]:cols[-1] + 1]
sums = area.reshape(len(area), -1).sum(axis=1)
print("change per frame:", " ".join(str(int(s)) for s in sums))
for limit in (0, 20, 100):
    changed = int((sums > limit).sum())
    print("over %d: %d of %d, about %.1f a second" % (limit, changed, len(sums), changed / (len(sums) / 30.0)))
