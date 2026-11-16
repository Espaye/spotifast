"""Place a before and an after recording side by side, started together.

Usage: python -I compare_recordings.py BEFORE.mp4 AFTER.mp4 OUT_PREFIX WIDTH HEIGHT

Run on the Windows PC on recordings downloaded from the capture run (needs
ffmpeg on the path; the labels use Segoe UI).

Finds the first line change at least four seconds into each recording (a
burst of movement), trims both to start four seconds before it, and writes
OUT_PREFIX.mp4 (twelve seconds, both sides) and OUT_PREFIX-slow.mp4 (the
1.5 s around that change, four times slower).
"""
import subprocess
import sys

before, after, prefix = sys.argv[1], sys.argv[2], sys.argv[3]
width, height = int(sys.argv[4]), int(sys.argv[5])
FPS = 30
SMALL_W, SMALL_H = 160, 100
FONT = "C\\:/Windows/Fonts/segoeui.ttf"


def movement(path):
    """Mean change per pixel between consecutive frames, per frame."""
    raw = subprocess.run(
        ["ffmpeg", "-nostdin", "-loglevel", "error", "-i", path, "-vf",
         "scale=%d:%d:flags=area,format=gray" % (SMALL_W, SMALL_H), "-f", "rawvideo", "-"],
        capture_output=True, check=True).stdout
    size = SMALL_W * SMALL_H
    frames = [raw[i:i + size] for i in range(0, len(raw) - size + 1, size)]
    return [
        sum(abs(a - b) for a, b in zip(frames[i], frames[i - 1])) / size
        for i in range(1, len(frames))
    ]


def changes(path):
    """Start times of the bursts of movement in a recording."""
    moved = movement(path)
    still = sorted(moved)[len(moved) // 2]
    threshold = max(still * 6, 0.25)
    starts, last = [], -100
    for index, value in enumerate(moved):
        if value > threshold:
            if index - last > FPS:
                starts.append((index + 1) / FPS)
            last = index
    return starts, still, threshold


def first_change(path):
    starts, still, threshold = changes(path)
    print(path, "changes at", ["%.2f" % t for t in starts], "still %.3f threshold %.3f" % (still, threshold))
    usable = [t for t in starts if 4.0 <= t <= 8.5]
    assert usable, starts
    return usable[0]


def render(out, lead, length, slow):
    pts = "%d*(PTS-STARTPTS)" % slow
    label = (
        "drawtext=fontfile='%s':fontsize=22:fontcolor=white:y=10:" % FONT
    )
    graph = (
        "[0:v]trim=start=%.4f:duration=%.4f,setpts=%s[b];"
        "[1:v]trim=start=%.4f:duration=%.4f,setpts=%s[a];"
        "[b][a]hstack=inputs=2,pad=iw:ih+44:0:44:color=0x101010,"
        "%sx=20:text='Before (main 5286363)',"
        "%sx=%d:text='After (this pull request)'[out]"
    ) % (
        t_before - lead, length, pts,
        t_after - lead, length, pts,
        label, label, width + 20,
    )
    subprocess.run(
        ["ffmpeg", "-nostdin", "-loglevel", "error", "-y", "-i", before, "-i", after,
         "-filter_complex", graph, "-map", "[out]", "-r", str(FPS),
         "-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "20", "-movflags", "+faststart", out],
        check=True)
    print("wrote", out)


t_before = first_change(before)
t_after = first_change(after)
render(prefix + ".mp4", 4.0, 12.0, 1)
render(prefix + "-slow.mp4", 0.3, 1.5, 4)
