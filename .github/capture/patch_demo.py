"""Capture-only demo data, applied the same way to the baseline and each
candidate: a link in Discover Weekly's description and monthly listeners for
the demo artist."""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()


def swap(old, new):
    global text
    assert text.count(old) == 1, old
    text = text.replace(old, new)


swap(
    "picked for you. Updates every Monday.\".into()",
    "picked for you. Updates every Monday. Also try "
    "<a href=spotify:playlist:pl1>Late night focus</a>.\".into()",
)
swap(
    '    app.saved.insert("spotify:artist:art0".into(), true);\n',
    '    app.saved.insert("spotify:artist:art0".into(), true);\n'
    '    app.play_counts.insert("spotify:artist:art0".into(), 4_812_330);\n',
)
open(path, "w", encoding="utf-8", newline="\n").write(text)
