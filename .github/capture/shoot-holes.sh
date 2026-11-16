#!/usr/bin/env bash
# shoot-holes.sh BINARY OUTDIR
# Full screen lyrics over a cover whose right half is see-through
# (CAPTURE_HOLES, a capture-only switch from patch_holes.py), at a narrow and
# a normal size in dark and light. Each still twice, to show it is the same
# from run to run.
set -u
bin=$1 out=$2
mkdir -p "$out"
env_for() {
  local home; home=$(mktemp -d)
  echo "XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state XDG_CACHE_HOME=$home/cache"
}
failed=0
for size in 800x600 1280x800; do
  for theme in dark light; do
    surfaces=lyrics-fullscreen-view
    [ "$theme" = light ] && surfaces="$surfaces,light"
    for take in 1 2; do
      env $(env_for) CAPTURE_HOLES=1 timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
        "$bin" --demo-page home --demo-size "$size" --demo-shot "$out/holes-$size-$theme-$take.png" \
        --demo-shot-delay 6000 --demo-show "$surfaces" || { echo "FAILED holes $size $theme $take"; failed=1; }
    done
  done
done
ls -la "$out"
exit $failed
