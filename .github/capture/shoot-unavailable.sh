#!/usr/bin/env bash
# shoot-unavailable.sh BINARY OUTDIR [CLICKS_FILE]
# Discover Weekly (a removed song in row 2, a hidden song in row 3) and Late
# night focus (a local file missing from this computer in row 5) at a narrow
# and a normal size in dark and light. CLICKS_FILE lines: NAME PAGE
# WIDTHxHEIGHT X,Y; the shot clicks that point and leaves the pointer on it.
set -u
bin=$1 out=$2 clicks=${3:-}
mkdir -p "$out"
shot() {
  local name=$1 page=$2 size=$3 theme=$4; shift 4
  local home; home=$(mktemp -d)
  local show=()
  [ "$theme" = light ] && show=(--demo-show light)
  XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state \
    XDG_CACHE_HOME=$home/cache timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
    "$bin" --demo-page "$page" --demo-size "$size" --demo-shot "$out/$name-$size-$theme.png" \
    "${show[@]}" "$@" || { echo "FAILED $name $size $theme"; failed=1; }
}
failed=0
for size in 800x600 1280x800; do
  for theme in dark light; do
    shot discover playlist:pl0 $size $theme
    shot focus playlist:pl1 $size $theme
  done
done
if [ -n "$clicks" ]; then
  while read -r name page size point; do
    [ -z "$name" ] && continue
    for theme in dark light; do
      shot "$name" "$page" "$size" "$theme" --demo-click "$point"
    done
  done < "$clicks"
fi
ls -la "$out"
exit $failed
