# Nexadu
Nexadu is a mini impletation of Xanadu (the original hypertext model founded in 1960), It's currently fully written in rust and some of the key futures of it is version control, spans, refrences and networking **(networking is not fully finished right now)**
(this project isnt finished and im realesing it to see if anyone will actually use it)
# Current futures
- version control
- spans
- references
- slight mark down support
- tui
- networking

# Installation process

## pre-requiretis
- Rust toolchain (rustc,cargo)
- Obviously git

## Installing
first clone the repo and enter the directory of the cloned repo

```sh
git clone https://github.com/KishaWeb/Nexadu.git
cd Nexadu
cargo build --release
```
if you want to run it directly use
```sh
./target/release/nexadu
```
### install locally
To install the complied binary on the locals user directory you can do:
```sh
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/nexadu "$HOME/.local/bin/nexadu"
```
also make sure ```$HOME/.local/bin``` is in your ```PATH```

for example this works for POSIX-compatible shells
```sh
export PATH="$HOME/.local/bin:$PATH"
```
and than you can just do
```sh
nexadu
```
## Updating
Incase of an update you can do

```sh
git pull
cargo build --release
install -m 755 target/release/nexadu "$HOME/.local/bin/nexadu"
```
# Documentation
- [server setup](docs/server.md)
- [client setup](docs/client.md)

