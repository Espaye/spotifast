#!/usr/bin/env bash
# shoot-round30.sh BINARY OUTDIR
# Stills for the queue, Save as playlist, player bar and full screen lyrics
# pull requests: the queue and Recent panels, the New playlist dialog, a
# playlist with the player bar, and full screen lyrics with its bar, at a
# narrow and a normal size in dark and light, each in fresh XDG directories.
set -u
bin=$1 out=$2
mkdir -p "$out"
failed=0
shot() {
  local name=$1 page=$2 size=$3 theme=$4 show=$5 delay=${6:-}
  local home; home=$(mktemp -d)
  [ "$theme" = light ] && show="${show:+$show,}light"
  local args=(--demo-page "$page" --demo-size "$size" --demo-shot "$out/$name-$size-$theme.png")
  [ -n "$show" ] && args+=(--demo-show "$show")
  [ -n "$delay" ] && args+=(--demo-shot-delay "$delay")
  XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state \
    XDG_CACHE_HOME=$home/cache timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
    "$bin" "${args[@]}" || { echo "FAILED $name $size $theme"; failed=1; }
}
for size in 800x600 1280x800; do
  for theme in dark light; do
    shot queue playlist:pl0 "$size" "$theme" queue
    shot recent playlist:pl0 "$size" "$theme" queue,recents
    shot create home "$size" "$theme" create
    shot playlist playlist:pl0 "$size" "$theme" ""
    shot lyrics home "$size" "$theme" lyrics-fullscreen-view 1500
  done
done
ls -la "$out"
exit $failed
