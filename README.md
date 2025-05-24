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
