#!/usr/bin/env bash
# shoot-bars.sh BINARY OUTDIR [record|only]
# Home and a playlist with its playing song at a narrow and a normal size in
# dark and light, each in fresh XDG directories. With "record", also four
# seconds of the playlist at 1280x800 in each theme, to show the bars moving.
# With "only", the recordings alone.
set -u
bin=$1 out=$2 record=${3:-}
mkdir -p "$out"
env_for() {
  local home; home=$(mktemp -d)
  echo "XDG_CONFIG_HOME=$home/config XDG_DATA_HOME=$home/data XDG_STATE_HOME=$home/state XDG_CACHE_HOME=$home/cache"
}
shot() {
  local name=$1 page=$2 size=$3 theme=$4
  local show=()
  [ "$theme" = light ] && show=(--demo-show light)
  env $(env_for) timeout 120 xvfb-run -a -s "-screen 0 1920x1200x24" \
    "$bin" --demo-page "$page" --demo-size "$size" --demo-shot "$out/$name-$size-$theme.png" \
    "${show[@]}" || { echo "FAILED $name $size $theme"; failed=1; }
}
failed=0
for size in 800x600 1280x800; do
  [ "$record" = only ] && break
  for theme in dark light; do
    shot home home $size $theme
    shot playlist playlist:pl1 $size $theme
  done
done
# xvfb-run leaves its own display, :99, locked behind it.
if [ "$record" = record ] || [ "$record" = only ]; then
  Xvfb :77 -screen 0 1280x800x24 &
  xvfb=$!
  sleep 2
  for theme in dark light; do
    show=()
    [ "$theme" = light ] && show=(--demo-show light)
    env $(env_for) DISPLAY=:77 timeout 20 "$bin" --demo --demo-page playlist:pl1 \
      --demo-size 1280x800 "${show[@]}" &
    app=$!
    sleep 8
    ffmpeg -loglevel error -y -f x11grab -draw_mouse 0 -video_size 1280x800 -framerate 30 -i :77 -t 4 \
      -c:v libx264 -pix_fmt yuv420p -crf 20 "$out/bars-1280x800-$theme.mp4" \
      || { echo "FAILED recording $theme"; failed=1; }
    kill $app 2>/dev/null; wait $app 2>/dev/null
  done
  kill $xvfb
fi
ls -la "$out"
exit $failed
