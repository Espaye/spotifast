#!/usr/bin/env bash
# shoot-lyrics.sh BINARY OUTDIR [record]
# Full screen lyrics drawn in the window, alone, with the Queue and Recent
# column, and for an instrumental, at a narrow, a normal and a large size in
# dark and light, each in fresh XDG directories. With "record", also sixteen
# seconds of full screen lyrics playing at 1280x800 in each theme: the
# pointer moves once, the player bar comes and slides away, lines go by.
set -u
bin=$1 out=$2 record=${3:-}
mkdir -p "$out"
env_for() {
  local home; home=$(mktemp -d)
  echo "XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state XDG_CACHE_HOME=$home/cache"
}
shot() {
  local name=$1 surfaces=$2 size=$3 theme=$4 delay=${5:-6000}
  [ "$theme" = light ] && surfaces="$surfaces,light"
  env $(env_for) timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
    "$bin" --demo-page home --demo-size "$size" --demo-shot "$out/$name-$size-$theme.png" \
    --demo-shot-delay "$delay" --demo-show "$surfaces" || { echo "FAILED $name $size $theme"; failed=1; }
}
failed=0
for size in 800x600 1280x800 1920x1080; do
  for theme in dark light; do
    shot lyrics lyrics-fullscreen-view "$size" "$theme"
    shot queue lyrics-fullscreen-view,queue "$size" "$theme"
    shot recent lyrics-fullscreen-view,recents "$size" "$theme"
    shot instrumental lyrics-fullscreen-instrumental "$size" "$theme"
    # Before the player bar's three quiet seconds are up.
    shot lyrics-bar lyrics-fullscreen-view "$size" "$theme" 1500
    shot queue-bar lyrics-fullscreen-view,queue "$size" "$theme" 1500
  done
done
if [ "$record" = record ]; then
  Xvfb :99 -screen 0 1280x800x24 &
  xvfb=$!
  sleep 2
  for theme in dark light; do
    surfaces=lyrics-fullscreen-view
    [ "$theme" = light ] && surfaces="$surfaces,light"
    env $(env_for) DISPLAY=:99 timeout 40 "$bin" --demo --demo-page home \
      --demo-size 1280x800 --demo-show "$surfaces" &
    app=$!
    # Covers load and the player bar slides away.
    sleep 8
    ffmpeg -loglevel error -y -f x11grab -draw_mouse 0 -video_size 1280x800 -framerate 30 -i :99 -t 16 \
      -c:v libx264 -pix_fmt yuv420p -crf 20 "$out/full-screen-1280x800-$theme.mp4" &
    rec=$!
    sleep 1.5
    DISPLAY=:99 xdotool mousemove 600 400 sleep 0.2 mousemove 640 420
    wait $rec || { echo "FAILED recording $theme"; failed=1; }
    kill $app 2>/dev/null; wait $app 2>/dev/null
  done
  kill $xvfb
fi
ls -la "$out"
exit $failed
