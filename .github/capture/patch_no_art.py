"""Capture-only switch, applied the same way to the baseline and the
candidate: with CAPTURE_NO_ART set, full screen lyrics get no art for their
backdrop, as before a cover has loaded."""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
head = (
    "fn preferred_backdrop_art(small: Option<String>, large: Option<String>) -> Option<String> {\n"
)
assert text.count(head) == 1, head
text = text.replace(
    head,
    head
    + '    if std::env::var_os("CAPTURE_NO_ART").is_some() {\n'
    + "        return None;\n"
    + "    }\n",
)
open(path, "w", encoding="utf-8", newline="\n").write(text)
print("no art switch patched")
