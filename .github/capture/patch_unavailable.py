"""Capture-only demo data, applied the same way to the baseline and the
candidate: the second song in Discover Weekly is one removed from Spotify,
which the Web API then reports with no name, artists or length and as not
playable. The third song is one of the demo's own two hidden songs, which
also cannot play but keeps its title."""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = '    app.playlist_pages.insert("pl0".into(), discover_page);\n'
assert text.count(old) == 1, old
new = (
    "    if let Some(PlaylistItem {\n"
    "        item: Some(PlayableItem::Track(track)),\n"
    "        ..\n"
    "    }) = discover_page.items.items.get_mut(1)\n"
    "    {\n"
    "        track.name = String::new();\n"
    "        track.artists.clear();\n"
    "        track.duration_ms = 0;\n"
    "        track.is_playable = Some(false);\n"
    "    }\n"
) + old
open(path, "w", encoding="utf-8", newline="\n").write(text.replace(old, new))
print("removed song patched")
