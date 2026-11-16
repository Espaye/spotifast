#!/usr/bin/env bash
# shoot-growth.sh BINARY OUTDIR
# The sung line of full screen lyrics growing. Stills at a narrow and a
# normal size in dark and light, then the live demo recorded at 1280x800 for
# twenty seconds (lines change every 6.5 s): plain in both themes, and in
# dark with an emoji ending each line. Each recording also gets a copy of
# its first ten seconds slowed four times.
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
    env $(env_for) timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
      "$bin" --demo-page home --demo-size "$size" --demo-shot "$out/lyrics-$size-$theme.png" \
      --demo-shot-delay 6000 --demo-show "$surfaces" || { echo "FAILED still $size $theme"; failed=1; }
  done
done
# xvfb-run leaves its own display, :99, locked behind it.
Xvfb :77 -screen 0 1280x800x24 &
xvfb=$!
sleep 2
record() {
  local name=$1 surfaces=$2
  env $(env_for) DISPLAY=:77 timeout 50 "$bin" --demo --demo-page home \
    --demo-size 1280x800 --demo-show "$surfaces" &
  local app=$!
  # Covers load and the player bar slides away.
  sleep 8
  ffmpeg -loglevel error -y -f x11grab -draw_mouse 0 -video_size 1280x800 -framerate 30 -i :77 -t 20 \
    -c:v libx264 -pix_fmt yuv420p -crf 20 "$out/$name.mp4" || { echo "FAILED recording $name"; failed=1; }
  kill $app 2>/dev/null; wait $app 2>/dev/null
  ffmpeg -loglevel error -y -i "$out/$name.mp4" -vf "setpts=4*PTS" -r 30 -t 40 \
    -c:v libx264 -pix_fmt yuv420p -crf 22 "$out/$name-slow.mp4" || { echo "FAILED slowing $name"; failed=1; }
}
record growth-1280x800-dark lyrics-fullscreen-view
record growth-1280x800-light lyrics-fullscreen-view,light
record growth-emoji-1280x800-dark lyrics-fullscreen-view,emoji
kill $xvfb
ls -la "$out"
exit $failed
