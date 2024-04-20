## Contributing

### Prerequisites

#### Nix - way (the best way)

1. Install Nix via Determinate Systems installer
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
    ```
2. nstall (nix-direnv support)[https://github.com/nix-community/nix-direnv?tab=readme-ov-file#with-nix-profile]

3. copy .envrc.example to .envrc

4. modify .envrc to set your own WIFI credentials

5. ```direnv allow .``` to enable the env

#### NonNix way (the medium way)

1. instal Rust via rustup 
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2. install espflash https://github.com/esp-rs/espflash

3. Set envirionment variable to set your wifi credentials (`WIFI_SSID`, `WIFI_PASSWORD`)

### Building/running

1. ```bash
    cargo run --release # will build and deploy the code to the micro controller and open logs
    ```

2. ```bash 
    espflash monitor # will open console to the device and start streaming logs
```