# A minimal 8-bit style maze game with an editor

## History

The game grew from experiments with the sdl2 crate circa july 2019.
Game developed during 2020 lockdowns.  Editor developed in 2024 using
Egui.

## Installation and usage

Copy `mzg-edit` and `mzg-play` to `/usr/local/bin`
Copy `gfx/` to `/usr/local/mazegame`
Copy `first.wld` to `/usr/local/mazegame`

To play the provided world: Launch `mzg-play /usr/local/mazegame/first.wld`

To edit a new world:

- Laungh `mzg-edit`
- Add rooms by clicking add
- Save
- Click play

## Editor usage

- Right click: Places GREEN selection
- Shift+Right click: Places RED selection

You need to select a starting position.  GREEN click then press START

Rooms may be too large for any given screen (scrolling is yet to be implemented.)
Crop a room by GREEN and RED-selecting two corners then press CROP.

## Author

Berké DURAK <bd@exhrd.fr>
