

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
  target/thumbv7em-none-eabihf/debug/seq_08
```

## Logging

- Use `rprintln!` from `rtt-target` for logging, e.g. `rprintln!("things are happening to {}", that);`
- Logs are sent over RTT and visible in the probe-rs terminal

## Profiling (cycle counts)

- Enable the `perf` feature: `cargo run --features perf` (or `cargo embed --features perf`)
- Initialize once early in startup: `seq_08::perf::init_cycle_counter();`
- Measure any code block: `let cycles = seq_08::perf::measure_cycles(|| { /* ... */ });`
- Print with `rprintln!` if you want the number over RTT.
  Example:
  ```rust
  // Build with: cargo run --features perf
  #[cfg(feature = "perf")]
  seq_08::perf::init_cycle_counter();
  #[cfg(feature = "perf")]
  {
      let cycles = seq_08::perf::measure_cycles(|| {
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

## Raw RTT input

`cargo embed` is line-buffered for RTT input. Which is a bit annoying as you have to press enter
after every key press. For per-key input, use the helper tool:

```bash
cd fw/tools/rtt_raw
cargo run --release -- --chip STM32F411RE --down 0
```
