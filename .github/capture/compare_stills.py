"""Compare the stills of two capture folders, pixel by pixel.

Usage: python -I compare_stills.py BEFORE_DIR AFTER_DIR [OUT_DIR]

Stills are named NAME-WIDTHxHEIGHT-THEME-TAKE.png. For each name: whether
the takes of a side are the same picture, and how far the before and the
after are apart per colour channel (0 to 255). With OUT_DIR, also writes the
difference of each pair made forty times stronger, as a PNG.
"""
import hashlib
import os
import re
import subprocess
import sys

import numpy

before_dir, after_dir = sys.argv[1], sys.argv[2]
out_dir = sys.argv[3] if len(sys.argv) > 3 else None
NAME = re.compile(r"^([a-z-]+)-(\d+)x(\d+)-(dark|light)-(\d)\.png$")


def pixels(path, width, height):
    raw = subprocess.run(
        ["ffmpeg", "-nostdin", "-loglevel", "error", "-i", path,
         "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
        capture_output=True, check=True).stdout
    assert len(raw) == width * height * 3, (path, len(raw))
    return numpy.frombuffer(raw, dtype=numpy.uint8).reshape(height, width, 3).astype(numpy.int16)


def stills(folder):
    found = {}
    for file in sorted(os.listdir(folder)):
        match = NAME.match(file)
        assert match, file
        name, width, height, theme, take = match.groups()
        key = (name, int(width), int(height), theme)
        found.setdefault(key, []).append(os.path.join(folder, file))
    return found


before, after = stills(before_dir), stills(after_dir)
assert before.keys() == after.keys(), (sorted(before), sorted(after))
print("%-28s %-9s %-9s %4s %7s %9s %9s" % (
    "still", "before", "after", "max", "mean", ">1 of 255", ">2 of 255"))
worst = 0
for key in sorted(before, key=lambda k: (k[1], k[3], k[0])):
    name, width, height, theme = key
    sides = []
    for paths in (before[key], after[key]):
        takes = [pixels(path, width, height) for path in paths]
        same = all((take == takes[0]).all() for take in takes[1:])
        digest = hashlib.sha256(open(paths[0], "rb").read()).hexdigest()[:8]
        sides.append((takes[0], same, digest, len(takes)))
    (a, a_same, a_hash, a_n), (b, b_same, b_hash, b_n) = sides
    apart = numpy.abs(a - b)
    most = int(apart.max())
    worst = max(worst, most)
    per_pixel = apart.max(axis=2)
    label = "%s-%dx%d-%s" % (name, width, height, theme)
    print("%-28s %-9s %-9s %4d %7.3f %8.2f%% %8.2f%%" % (
        label,
        "%s%s" % (a_hash[:6], "" if a_same else "!"),
        "%s%s" % (b_hash[:6], "" if b_same else "!"),
        most, float(apart.mean()),
        100.0 * float((per_pixel > 1).mean()), 100.0 * float((per_pixel > 2).mean())))
    if most > 2:
        rows = numpy.where((per_pixel > 2).any(axis=1))[0]
        cols = numpy.where((per_pixel > 2).any(axis=0))[0]
        print("    over 2: rows %d to %d, columns %d to %d" % (rows[0], rows[-1], cols[0], cols[-1]))
    if out_dir:
        strong = numpy.clip(apart * 40, 0, 255).astype(numpy.uint8)
        subprocess.run(
            ["ffmpeg", "-nostdin", "-loglevel", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgb24",
             "-video_size", "%dx%d" % (width, height), "-i", "-", os.path.join(out_dir, label + "-diff.png")],
            input=strong.tobytes(), check=True)
print("a ! marks takes of one side that differ; largest difference overall:", worst)
