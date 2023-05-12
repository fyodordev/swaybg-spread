swaybg-spread
=============

A simple CLI tool to set wallpapers spanning over multiple monitors in sway.

It uses the Wayland output configuration to determine monitor size and position, and splits the provided image into separate images for each monitor. These are saved to `~/.cache` by default. It then uses `swaybg` to set the images on the corresponding monitors. To persist the changes the program outputs sway configuration lines which can be written to a sway configuration file.



Originally forked off of [0xk1f0/rwpspread](https://github.com/0xk1f0/rwpspread)

Installation
------------

Available from the [AUR](https://aur.archlinux.org/packages/swaybg-spread):

```bash
yay -S swaybg-spread
```

Usage
-----

Set wallpaper spanning all monitors once:

```bash
swaybg-spread -i ~/some-wallpaper.jpg
```

Set wallpaper and write it to a sway configuration file to persist sway reloads (In this example the file is imported in the main sway configuration file with `include ~/.config/sway/config.d/*`):

```bash
swaybg-spread -i ~/some-wallpaper.jpg > ~/.config/sway/config.d/bg.conf 
```

Use custom location for saving image fragments:

```bash
swaybg-spread -i ~/some-wallpaper.jpg -o ~/.theme/fragments > ~/.config/sway/config.d/bg.conf 
```


Building
--------

```bash
git clone https://github.com/fyodordev/swaybg-spread.git
cd swaybg-spread
cargo build --release
```

