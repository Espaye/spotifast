#!/usr/bin/env bash
# shoot-live.sh BINARY OUTDIR STEPS_FILE
# Stills of the live demo under Xvfb :77 after pointer steps, for hover,
# tooltip, scroll and drag states that --demo-shot cannot hold. STEPS_FILE
# lines: NAME PAGE WIDTHxHEIGHT SHOW XDOTOOL_STEPS... with SHOW "-" for none.
# Each line runs in dark and light in fresh XDG directories; the still shows
# the pointer. The window sits at 0,0 on a display with no window manager.
set -u
bin=$1 out=$2 steps=$3
mkdir -p "$out"
failed=0
Xvfb :77 -screen 0 1920x1200x24 &
xvfb=$!
sleep 2
while read -r name page size show rest; do
  if [ -z "$name" ] || [ "${name:0:1}" = "#" ]; then continue; fi
  for theme in dark light; do
    home=$(mktemp -d)
    surfaces=$show
    [ "$surfaces" = - ] && surfaces=
    [ "$theme" = light ] && surfaces="${surfaces:+$surfaces,}light"
    args=(--demo --demo-page "$page" --demo-size "$size")
    [ -n "$surfaces" ] && args+=(--demo-show "$surfaces")
    XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state \
      XDG_CACHE_HOME=$home/cache DISPLAY=:77 timeout 60 "$bin" "${args[@]}" </dev/null &
    app=$!
    sleep 8
    # shellcheck disable=SC2086
    DISPLAY=:77 xdotool mousemove 5 5 sleep 0.3 $rest || { echo "FAILED steps $name"; failed=1; }
    ffmpeg -loglevel error -y -f x11grab -draw_mouse 1 -video_size "$size" -i :77 -frames:v 1 \
      "$out/$name-$size-$theme.png" || { echo "FAILED $name $size $theme"; failed=1; }
    DISPLAY=:77 xdotool mouseup 1 2>/dev/null
    kill $app 2>/dev/null
    wait $app 2>/dev/null
  done
done < "$steps"
kill $xvfb
ls -la "$out"
exit $failed
