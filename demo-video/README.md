```
# DO THIS IN A VM

python3 -m venv env
source env/bin/activate
python3 -m pip install qecore


sudo dnf install gnome-ponytail-daemon python3-gnome-ponytail-daemon
# https://github.com/pygobject/pygobject-stubs/blob/master/src/gi-stubs/repository/Atspi.pyi not in release yet
pip install dogtail git+https://github.com/pygobject/pygobject-stubs --no-cache-dir --config-settings=config=Atspi
gsettings set org.gnome.desktop.interface toolkit-accessibility true

Enable Introspect support in Looking Glass. Press `Alt+F2`, type `lg` and
press enter to open Looking Glass. Finally enable unsafe mode:
>>> global.context.unsafe_mode = true

Fix Settings -> Accessibility -> Typing -> Repeat Keys -> Delay to the default value

https://gitlab.gnome.org/GNOME/mutter/-/issues/2099
# https://forums.linuxmint.com/viewtopic.php?t=367400
nano ~/.config/gtk-3.0/gtk.css
decoration, decoration:backdrop, .csd.popup decoration, .fullscreen decoration, .maximized decoration, .tiled decoration, .tiled decoration:backdrop {box-shadow: none; margin: 0;}
window decoration, window paned, window paned headerbar {
/* square top corners */
    border-radius: 0;
}

window paned headerbar
{
/* header top shadow */
    box-shadow: none;
}   

window decoration
{
/* Remove shadows */
    box-shadow: none;
/* window border */
    border: none;
}

setsid firefox -P tmp -width 1920 -height 1080
sniff # show tree
python3 -i main.py
```

https://modehnal.github.io/ The main queries you will be using

https://fedoramagazine.org/automation-through-accessibility/

# Kdenlive

Enable Proxy Clips in project settings
