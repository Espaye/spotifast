"""Capture-only demo data, applied the same way to the baseline and each
candidate: a link in Discover Weekly's description, and monthly listeners
instead of followers for the demo artists, as the Web API gives none for a
real account."""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()


def swap(old, new):
    global text
    assert text.count(old) == 1, old
    text = text.replace(old, new)


swap(
    "\"Your weekly mixtape of fresh music. Enjoy new music and deep cuts "
    "picked for you. Updates every Monday.\".into()",
    "\"Pairs well with <a href=spotify:playlist:pl1>Late night focus</a>. "
    "Updates every Monday.\".into()",
)
swap(
    '    app.saved.insert("spotify:artist:art0".into(), true);\n',
    '    app.saved.insert("spotify:artist:art0".into(), true);\n'
    '    app.play_counts.insert("spotify:artist:art0".into(), 4_812_330);\n',
)
swap(
    "        followers: Some(Followers {\n"
    "            total: 1_284_930 + index as u64 * 10_431,\n"
    "        }),\n",
    "        followers: None,\n",
)
open(path, "w", encoding="utf-8", newline="\n").write(text)
