# assets/upstream — local only, never committed

This folder holds the original PICO-8 cart we are porting. It is **gitignored**
(see the repo `.gitignore`): the cart is published by Bee_Randon as "No License",
so it must not be redistributed through this repository.

Expected contents (none of these are tracked by git):

| File | What it is |
|---|---|
| `bubblegum_spin.p8` | The cart exported from PICO-8 (`save bubblegum_spin.p8`). Full text format, version 43: `__lua__`, `__gfx__` (128 rows), `__map__`, `__sfx__` (64 rows), `__music__` (64 rows). This is the authoritative source for sprites, the 4x6 font, map and audio data. |
| `bubblegum_spin-4.p8.png` | Cart image as downloaded from the BBS. Its steganographic payload does not decode (the image was re-encoded), so it is only useful as a visual reference. |
| `bubblegum_spin_code.txt` | Plain-text dump of the cart's Lua, used for code reading and for diffing after a re-export. |

## Re-creating this folder

1. Get the cart from https://www.lexaloffle.com/bbs/?pid=143764 (or ask the
   project owner for their copy).
2. Open it in PICO-8 and run `save bubblegum_spin.p8` to get the full text export.
3. Put the files here. `git status` must stay clean afterwards — if it shows
   these files, the `.gitignore` rule broke and must be fixed before committing.

## Derived assets

Anything we generate from these files (sprite sheet, font, sfx/music binaries)
goes in `assets/` **outside** this folder, produced by `tools/p8_import.py`, and
is reviewed for licensing before it is committed.
