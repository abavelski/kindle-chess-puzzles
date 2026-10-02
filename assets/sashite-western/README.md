# Sashité Western Chess pieces

These 12 SVG files are the First Player (white) and Second Player (black)
Western chess pieces from <https://sashite.dev/assets/chess/>.

The upstream source describes this set as public domain; this repository uses
the files under CC0 1.0. The SVG files are preserved unchanged and remain the
source of truth.

`scripts/generate_sashite.py` converts the SVG path data into deterministic
1000-unit grayscale vector layers in
`crates/chess-render/src/pieces_generated.rs`. The renderer rasterizes those
generated layers into the project-owned Gray8 frame; it never parses SVG on the
Kindle.

Regenerate or verify the checked-in generated file with:

```sh
python3 scripts/generate_sashite.py
python3 scripts/generate_sashite.py --check
```

The source files and conversion approach match the Sashité Western assets used
by the reference `abavelski/eink-chess-app` project.
