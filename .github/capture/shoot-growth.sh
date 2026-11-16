#!/usr/bin/env bash
# shoot-growth.sh BINARY OUTDIR
# The sung line of full screen lyrics growing. Stills at a narrow and a
# normal size in dark and light, each taken three times because where the
# lines come to rest differs a little from run to run. Then the live demo
# recorded for twenty seconds (lines change every 6.5 s): at both sizes in
# both themes, and at the normal size with an emoji ending each line.
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
    for take in 1 2 3; do
      env $(env_for) timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
        "$bin" --demo-page home --demo-size "$size" --demo-shot "$out/lyrics-$size-$theme-$take.png" \
        --demo-shot-delay 6000 --demo-show "$surfaces" || { echo "FAILED still $size $theme $take"; failed=1; }
    done
  done
done
# xvfb-run leaves its own display, :99, locked behind it.
record() {
  local name=$1 size=$2 surfaces=$3
  Xvfb :77 -screen 0 "${size}x24" &
  local xvfb=$!
  sleep 2
  env $(env_for) DISPLAY=:77 timeout 50 "$bin" --demo --demo-page home \
    --demo-size "$size" --demo-show "$surfaces" &
  local app=$!
  # Covers load and the player bar slides away.
  sleep 8
  ffmpeg -loglevel error -y -f x11grab -draw_mouse 0 -video_size "$size" -framerate 30 -i :77 -t 20 \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "$out/$name.mp4" || { echo "FAILED recording $name"; failed=1; }
  kill $app 2>/dev/null; wait $app 2>/dev/null
  kill $xvfb; wait $xvfb 2>/dev/null
}
record growth-1280x800-dark 1280x800 lyrics-fullscreen-view
record growth-1280x800-light 1280x800 lyrics-fullscreen-view,light
record growth-800x600-dark 800x600 lyrics-fullscreen-view
record growth-800x600-light 800x600 lyrics-fullscreen-view,light
record growth-emoji-1280x800-dark 1280x800 lyrics-fullscreen-view,emoji
ls -la "$out"
exit $failed
