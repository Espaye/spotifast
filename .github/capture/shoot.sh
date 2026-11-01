#!/usr/bin/env bash
# shoot.sh BINARY OUTDIR [CLICKS_FILE]
# Every page at a narrow and a normal size in dark and light, each in fresh
# XDG directories. CLICKS_FILE lines: NAME PAGE WIDTHxHEIGHT X,Y
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
    "${show[@]}" "$@" || echo "FAILED $name $size $theme"
}
for size in 800x600 1280x800; do
  for theme in dark light; do
    shot home home $size $theme
    shot playlist playlist:pl0 $size $theme
    shot artist artist:art0 $size $theme
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
