# hexdino

A hex editor with vim like keybindings written in Rust.

## Dependencies
- ncurses

## Building

### Ubuntu

```Shell
sudo apt install libncursesw5-dev cargo
cargo build
```

### Nixos
Create default.nix
```Shell
with import <nixpkgs> {}; {
  hexdinoEnv = stdenv.mkDerivation {
    name = "hexdino";
    buildInputs = [ stdenv ncurses pkgconfig ];
  };  
}
```
Then build with cargo
```Shell
nix-shell . --command "cargo build"
```

## Logo
![logo](https://raw.githubusercontent.com/Luz/hexdino/master/logo.png)

## Related projects
- [command_parser](https://github.com/Luz/command_parser)
- [pest-parser](https://github.com/pest-parser/pest)

