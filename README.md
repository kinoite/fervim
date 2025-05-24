# New Update: fervim 0.1.1 -- The Configuration Update
![alt](/Screenshots/d.png)
New features:
* You can now configure fervim and change so much like the command box and mode bar!
* Change the command box text
* Change their colors
* Add a gradient to the mode bar (gradient support will come to the command bar i promise)



# fervim
A basic vim-like text editor written in Rust 

### screenies
![alt text](/Screenshots/Screenshot_20250524_083901.png)
![alt text](/Screenshots/Screenshot_20250524_083951.png)
![alt text](/Screenshots/Screenshot_20250524_084014.png)

## How to install

Clone this repository then `cd` into the cloned directory, then type out

```
cargo run --release
```

you can alternatively install the precompiled binaries (even though this text editor is light as it is)

## Moving the installed fervim into your $PATH

Move ``target/release/fervim`` to ``/usr/local/bin`` or your prefered path

```
sudo/doas mv target/release/fervim /usr/local/bin
```

and boom! done, enjoy! :>

## Configuration

The configuration file should be located at `~/.config/fervim/config.toml`, though you'd have to create it yourself with `touch`.

```
touch ~/.config/ferivm/config.toml
```

Now that configuration is ready, fire up your text editor for editing config files of your choice and get tweaking and enjoy!, or not, it's your choice!
After you've configured fervim, the configuration should apply now.

### What do I do here?

Copy the example configuration below to the configuration file:
```toml
# config.toml

# General colors, these serve as fallbacks if not specified in specific sections
[colors]
text = "white"
background = "black"
message_text = "#FF0000" # Bright Red for messages

# Configuration for the mode bar at the bottom
["mode bar"]
show_mode = true
show_filename = true
show_dirty_indicator = true
primary_color = "#1bd8f1" # Dark Blue background (hex)
secondary_color = "#61AFEF" # This could be used for a border or other accent (hex)
text_color = "#f1351b"    # White text (hex)
height = 5                # Make the mode bar 3 lines tall
# width is typically ignored for full-width bars

# Configuration for the command box that appears when in Command mode
["command box"]
primary_color = "#282C34" # Dark Grey background (hex)
secondary_color = "#b0c2d8" # Blue border (hex)
text_color = "#48546f"    # Light Grey text (hex)
height = 7                # Taller command box (lines)
width = 60                # Fixed width command box (characters)
text = "Vim Command"      # Custom label for the command box
```

### [colors]
This one's optional to configure, but highly recommended to keep it alone just incase since it's for white and black.


### ["mode bar"]
The mode bar is the bar at the bottom of fervim  that tells you which mode you are using, and the file you are currently editing.

### ["command box"]
The command box is the textbox which appears when you press esc and then colon, to exit or write

### primary_color and secondary_color
primary_color is the color that changes the background of the object you are configuring, you can also tweak secondary_color for a nice little gradient!
Changing where the gradient direction is ain't in the configuration yet, in another version I swear I will make that feature and add support for the command box, since gradients dont support the command box yet unfortunately.

### text_color
Changes the text color, planned to have gradient support soon (in a future release)

### height and width
height and width changes the height and width of the object you are configuring, the border margin that seperates the mode viewer and the filename doesn't go to till top of the mode bar.
That will be fixed in the next version, yeah, next one's gonna be bigger, oh boy.

### text
text is the custom text for the command box and command box only, i'll add support for mode bar but for an extra piece of text for decorations and what not


More configuration stuff will come in later versions!!! :>>>>


## Packaging
Right now, fervim only has manual installation, if you'd like to port this to other distrubutions such as Arch for the AUR or Debian as a .deb package, or Gentoo for the GURU, etc, you're free to do that <3
