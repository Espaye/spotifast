"""Print the first and last pixel row holding accent-green letters, per time, for a 1280x800 recording.

Usage: python -I green_rows.py VIDEO.mp4 3,5,9.5
Shows where the sung lyric line rests and when it changes.
"""
import subprocess, sys
path = sys.argv[1]
times = [float(t) for t in sys.argv[2].split(",")]
W, H = 1280, 800
out = []
for t in times:
    raw = subprocess.run(
        ["ffmpeg", "-nostdin", "-loglevel", "error", "-ss", str(t), "-i", path, "-frames:v", "1",
         "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
        capture_output=True, check=True).stdout
    assert len(raw) == W * H * 3, len(raw)
    rows = []
    for y in range(H - 110):
        row = raw[y * W * 3:(y + 1) * W * 3]
        n = 0
        for x in range(560, 1260):
            r, g, b = row[x * 3], row[x * 3 + 1], row[x * 3 + 2]
            if g > 150 and g > r + 90 and g > b + 60:
                n += 1
        if n > 3:
            rows.append(y)
    out.append("%s:%s-%s" % (t, rows[0] if rows else "-", rows[-1] if rows else "-"))
print("  ".join(out))
