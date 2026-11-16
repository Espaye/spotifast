"""Capture-only switch, applied the same way to the baseline and the
candidate: with CAPTURE_HOLES set, the right half of the artwork behind a
blurred backdrop is made see-through, as a PNG cover embedded in a local
file can be."""
import sys

path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = "    Some(blurred_image(decode(bytes).ok()?, sigma))\n"
assert text.count(old) == 1, old
new = (
    "    let mut image = decode(bytes).ok()?;\n"
    '    if std::env::var_os("CAPTURE_HOLES").is_some() {\n'
    "        let mut holed = image.to_rgba8();\n"
    "        let half = holed.width() / 2;\n"
    "        for (x, _, pixel) in holed.enumerate_pixels_mut() {\n"
    "            if x >= half {\n"
    "                pixel.0[3] = 0;\n"
    "            }\n"
    "        }\n"
    "        image = image::DynamicImage::ImageRgba8(holed);\n"
    "    }\n"
    "    Some(blurred_image(image, sigma))\n"
)
open(path, "w", encoding="utf-8", newline="\n").write(text.replace(old, new))
print("see-through switch patched")
