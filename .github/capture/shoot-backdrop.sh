#!/usr/bin/env bash
# shoot-backdrop.sh BINARY OUTDIR
# The backdrop of full screen lyrics: the words alone, with the player bar,
# an instrumental, and with the art withheld (CAPTURE_NO_ART, a capture-only
# switch from patch_no_art.py), at a narrow and a normal size in dark and
# light. Each still twice, to show it is the same from run to run.
set -u
bin=$1 out=$2
mkdir -p "$out"
env_for() {
  local home; home=$(mktemp -d)
  echo "XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state XDG_CACHE_HOME=$home/cache"
}
failed=0
shot() {
  local name=$1 surfaces=$2 size=$3 theme=$4 delay=$5 noart=${6:-}
  [ "$theme" = light ] && surfaces="$surfaces,light"
  for take in 1 2; do
    env $(env_for) ${noart:+CAPTURE_NO_ART=1} timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
      "$bin" --demo-page home --demo-size "$size" --demo-shot "$out/$name-$size-$theme-$take.png" \
      --demo-shot-delay "$delay" --demo-show "$surfaces" || { echo "FAILED $name $size $theme $take"; failed=1; }
  done
}
for size in 800x600 1280x800; do
  for theme in dark light; do
    shot lyrics lyrics-fullscreen-view "$size" "$theme" 6000
    # Before the player bar's three quiet seconds are up.
    shot lyrics-bar lyrics-fullscreen-view "$size" "$theme" 1500
    shot instrumental lyrics-fullscreen-instrumental "$size" "$theme" 6000
    shot noart lyrics-fullscreen-view "$size" "$theme" 6000 noart
  done
done
ls -la "$out"
exit $failed
