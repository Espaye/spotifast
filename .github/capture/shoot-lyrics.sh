#!/usr/bin/env bash
# shoot-lyrics.sh BINARY OUTDIR
# Full screen lyrics drawn in the window, alone, with the Queue and Recent
# column, and for an instrumental, at a narrow, a normal and a large size in
# dark and light, each in fresh XDG directories.
set -u
bin=$1 out=$2
mkdir -p "$out"
shot() {
  local name=$1 surfaces=$2 size=$3 theme=$4
  local home; home=$(mktemp -d)
  [ "$theme" = light ] && surfaces="$surfaces,light"
  XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state \
    XDG_CACHE_HOME=$home/cache timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
    "$bin" --demo-page home --demo-size "$size" --demo-shot "$out/$name-$size-$theme.png" \
    --demo-show "$surfaces" || { echo "FAILED $name $size $theme"; failed=1; }
}
failed=0
for size in 800x600 1280x800 1920x1080; do
  for theme in dark light; do
    shot lyrics lyrics-fullscreen-view "$size" "$theme"
    shot queue lyrics-fullscreen-view,queue "$size" "$theme"
    shot recent lyrics-fullscreen-view,recents "$size" "$theme"
    shot instrumental lyrics-fullscreen-instrumental "$size" "$theme"
  done
done
ls -la "$out"
exit $failed
