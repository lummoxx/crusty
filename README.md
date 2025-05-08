## Prerequisites
install rust
https://www.rust-lang.org/tools/install

install node
https://nodejs.org/en/download

clone this repo

## Setup dev environment

`cargo install tauri-cli` or `npm install -g @tauri-apps/cli `

`cargo install create-tauri-app --locked`

`npm install --global yarn`

## Start dev environment and GUI
from the crusty-gui/src-tauri directory, run:
```
yarn
yarn add --dev vite
yarn tauri dev
```

to get msi:
`yarn tauri build`

## Setup car
stand in embassy/examples/rp


### With debug probe
installation guide:
https://probe.rs/docs/getting-started/installation

`cargo run --bin crusty --release`


### Without debug probe
#### creates crusty.uf2 file in /crusty/embassy/examples/rp/target/thumbv6m-none-eabi/release

`cargo install elf2uf2-rs`

`cargo build --bin crusty --release`

`elf2uf2-rs ./target/thumbv6m-none-eabi/release/crusty`


## Freenove car tutorial
hardware-instructions.pdf in root
