

## Building / Flashing / Running
- Install [probe-rs](https://probe.rs/)
- `cargo run`


## Debugging

- Install [probe-rs](https://probe.rs/)
- Run `./dap_server.sh`
- You can use any editor/IDE/debugger that supports Debugger Adapter Protocol. Vimspector config is already setup (see `.vimspector.json`).
- Use `opt-level = 0` in `Cargo.toml` to preserve debug information.

## Logging

- You can set log statements with defmt, e.g. `defmt::trace!("things are happening to {:?}", that);`.
- More information of logging levels etc; https://defmt.ferrous-systems.com/
