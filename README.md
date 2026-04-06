# Jetimer

A system tray timer application designed for Linux and written in Rust 🦀

## Requirements

- Zenity (Input dialogs)
- GTK3 & AppIndicator (Tray icon support)
- Sound Player (One of: pw-play, paplay, canberra-gtk-play or ffplay (ffmpeg))
- libnotify (notify-send) (Notification)

### Fedora

```bash
sudo dnf install gtk3 libappindicator-gtk3 zenity libnotify libxdo
```

### Debian based

```bash
sudo apt install libgtk-3-0 libappindicator3-1 zenity libnotify-bin
```

### Arch

```bash
sudo pacman -S gtk3 libappindicator-gtk3 zenity libnotify
```

## Screenshots

### Title entry

![SS1](https://github.com/sunaipa5/jetimer/blob/main/assets/ss1.png?raw=true)

### Time selector

![SS2](https://github.com/sunaipa5/jetimer/blob/main/assets/ss2.png?raw=true)

## Tray

![SS3](https://github.com/sunaipa5/jetimer/blob/main/assets/ss3.png?raw=true)

## Tray right click

![SS4](https://github.com/sunaipa5/jetimer/blob/main/assets/ss4.png?raw=true)
