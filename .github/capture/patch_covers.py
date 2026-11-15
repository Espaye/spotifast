"""Capture-only demo covers, applied the same way to the baseline and each
candidate: bright photos from the same placeholder service instead of its
random seeds, with the playing song's album (seed 200) on a colourful one.
Also shows the demo's recent plays in the Recent tab: the demo fills them
but never rebuilds the list the tab draws."""
import os
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()

head = "fn image(seed: u32) -> Vec<Image> {\n    vec![\n"
assert text.count(head) == 1, head
text = text.replace(
    head,
    "fn image(seed: u32) -> Vec<Image> {\n"
    "    const BRIGHT: [u32; 25] = [\n"
    "        56, 158, 360, 499, 274, 402, 184, 110, 152, 82, 503, 167, 429, 53, 54, 63, 102,\n"
    "        106, 165, 247, 287, 352, 164, 21, 75,\n"
    "    ];\n"
    "    let seed = BRIGHT[seed as usize % BRIGHT.len()];\n"
    "    vec![\n",
)
old = "picsum.photos/seed/spotifast{seed}/"
assert text.count(old) == 3, old
text = text.replace(old, "picsum.photos/id/{seed}/")

recents = "    app.recents.error = None;\n"
assert text.count(recents) == 1, recents
text = text.replace(recents, recents + "    app.rebuild_recents();\n")
open(path, "w", encoding="utf-8", newline="\n").write(text)

app_path = os.path.join(os.path.dirname(path), "app.rs")
app = open(app_path, encoding="utf-8").read()
private = "    fn rebuild_recents(&mut self) {\n"
assert app.count(private) == 1, private
app = app.replace(private, "    pub(crate) fn rebuild_recents(&mut self) {\n")
open(app_path, "w", encoding="utf-8", newline="\n").write(app)
print("covers and recents patched")
