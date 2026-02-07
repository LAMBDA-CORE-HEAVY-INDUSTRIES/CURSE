

## Building / Flashing / Running
- Install [probe-rs](https://probe.rs/)
- `cargo run`


## Debugging

- Install [probe-rs](https://probe.rs/)
- Run `./dap_server.sh`
- You can use any editor/IDE/debugger that supports Debugger Adapter Protocol. Vimspector config is already setup (see `.vimspector.json`).
- Use `opt-level = 0` in `Cargo.toml` to preserve debug information.

If things go wrong and you can't flash the firmware, hold and release reset button (b2) or connect NRST to GND and run
  ```
  λ probe-rs run \
  --chip STM32F411RETx \
  --protocol SWD \
  --connect-under-reset \
  target/thumbv7em-none-eabihf/debug/curse
```

## Logging

- Use `rprintln!` from `rtt-target` for logging, e.g. `rprintln!("things are happening to {}", that);`
- Logs are sent over RTT and visible in the probe-rs terminal

## Profiling (cycle counts)

- Enable the `perf` feature: `cargo run --features perf` (or `cargo embed --features perf`)
- Initialize once early in startup: `curse::perf::init_cycle_counter();`
- Measure any code block: `let cycles = curse::perf::measure_cycles(|| { /* ... */ });`
- Print with `rprintln!` if you want the number over RTT.
  Example:
  ```rust
  // Build with: cargo run --features perf
  #[cfg(feature = "perf")]
  curse::perf::init_cycle_counter();
  #[cfg(feature = "perf")]
  {
      let cycles = curse::perf::measure_cycles(|| {
          rebuild_rt_cache(&sequencer_state);
      });
      rtt_target::rprintln!("{}", cycles);
  }
  ```

## Keyboard Input (Development)

To simulate hardware buttons via keyboard during development:

```bash
cargo embed --features keyboard-input
```

In cargo embed TUI, press tab to use input field, enter to send the input.

Key mappings:
- `1-0, q-y`: Steps 0-15
- `a-k`: Tracks 0-7
- `Space`: Play/Pause, `x`: Stop
