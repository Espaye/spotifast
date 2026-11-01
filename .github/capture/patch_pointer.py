"""Capture-only pointer, applied the same way to the baseline and each
candidate: `--demo-drag FROM:TO` right-clicks at FROM and then glides to TO,
so a shot can open a row's menu and hover one of its submenus."""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()


def swap(old, new):
    global text
    assert text.count(old) == 1, old
    text = text.replace(old, new)


swap(
    "        .map(|[from, to]| DemoDrag {\n"
    "            from,\n"
    "            to,\n"
    "            frame: 0,\n"
    "            click: false,\n",
    "        .map(|[from, to]| DemoDrag {\n"
    "            from,\n"
    "            to,\n"
    "            frame: 0,\n"
    "            click: true,\n",
)
# Release where the press was, so it counts as a click.
swap(
    "        let pos = if frame <= Self::REST {\n",
    "        let pos = if frame <= Self::REST + 1 {\n",
)
swap(
    "                button: egui::PointerButton::Primary,\n",
    "                button: if self.from == self.to {\n"
    "                    egui::PointerButton::Primary\n"
    "                } else {\n"
    "                    egui::PointerButton::Secondary\n"
    "                },\n",
)
open(path, "w", encoding="utf-8", newline="\n").write(text)
