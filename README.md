# fervim
A basic vim-like text editor written in Rust


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
