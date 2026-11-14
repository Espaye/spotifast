"""Capture-only demo data, applied the same way to the baseline and each
candidate: a link in Discover Weekly's description, monthly listeners
instead of followers for the demo artists, as the Web API gives none for a
real account, and playlist headers as Spotify reports them."""
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
# Discover Weekly as the Web API reports a saved one: its per-listener id
# and not public on the profile.
swap(
    "        playlist: Loadable::Loaded(playlists[0].clone()),\n",
    "        playlist: Loadable::Loaded(Playlist {\n"
    "            id: \"37i9dQZEVXcJZyENOWUFo7\".into(),\n"
    "            public: Some(false),\n"
    "            ..playlists[0].clone()\n"
    "        }),\n",
)
# Late night focus as one's own private playlist, not made together.
swap('    playlist_page.contributors.insert("kasia".into());\n', "")
swap('    playlist_page.contributors.insert("sam".into());\n', "")
open(path, "w", encoding="utf-8", newline="\n").write(text)
