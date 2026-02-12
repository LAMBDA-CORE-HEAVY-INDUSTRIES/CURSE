#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::sync::atomic::Ordering;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;
use seq_08::render::{
    render, render_bpm, render_cells, render_column, render_pattern_indicator,
    render_playhead_marker, render_track_label, CellHighlight,
};
use seq_08::sequencer::{
    init_step_timer, rebuild_rt_cache, set_bpm, take_dirty, CURRENT_STEP, DIRTY_BPM,
    DIRTY_NOTE_DATA, DIRTY_PATTERN, DIRTY_RT_CACHE, DIRTY_STEP_SELECTION, DIRTY_TRACK_SELECTION,
    SEQ, STEP_FLAG, PLAYING,
};
use seq_08::utils::{iter_bits_u8, iter_bits_u16};
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_halt as _;
use stm32f4xx_hal::{self as hal, spi::Spi};

#[cfg(feature = "keyboard-input")]
use seq_08::input::{handle_button_press, key_to_button};

#[cfg(feature = "perf")]
use seq_08::perf::{init_cycle_counter, measure_cycles};
#[cfg(feature = "perf")]
use seq_08::sequencer::take_overrun_stats;

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(100.MHz()).freeze();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();

        let _gate_out_1 = gpioa.pa10.into_push_pull_output(); 
        let sck = gpioa.pa5.into_alternate::<5>(); // SPI1_SCK
        let mosi = gpioa.pa7.into_alternate::<5>(); // SPI1_MOSI / SDO
        let miso = gpioa.pa6.into_alternate::<5>(); // SPI1_MISO / SDI
        let cs = gpioa.pa4.into_push_pull_output(); // SCS
        let res = gpiob
            .pb0
            .into_push_pull_output_in_state(hal::gpio::PinState::High);

        #[cfg(feature = "perf")]
        init_cycle_counter();

        // enable debug in sleep
        unsafe { &*pac::DBGMCU::ptr() }.cr().modify(|_, w| w.dbg_sleep().set_bit());

        init_step_timer(dp.TIM3, &clocks);
        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM3);
        }

        let spi_bus = Spi::new(
            dp.SPI1,
            (sck, miso, mosi),
            embedded_hal::spi::MODE_0,
            10.MHz(),
            &clocks,
        );
        let spi_delay = cp.SYST.delay(&clocks);
        let spi_device = ExclusiveDevice::new(spi_bus, cs, spi_delay).unwrap();

        let pb10_pwm = gpiob.pb10.into_alternate::<1>();
        let (_, (_, _, pwm_ch3, _)) = dp.TIM2.pwm_hz(1.kHz(), &clocks);
        let mut pwm_ch3 = pwm_ch3.with(pb10_pwm);
        pwm_ch3.enable();
        let max_duty = pwm_ch3.get_max_duty();
        pwm_ch3.set_duty(max_duty / 2);

        let mut delay = dp.TIM5.delay_us(&clocks);
        let spi_interface = lt7683::SpiInterface { spi: spi_device };
        let display_config = lt7683::DisplayConfig::new();
        let mut display = lt7683::LT7683::new(spi_interface, res, display_config);
        display.init(&mut delay).unwrap();
        display.clear_screen(0x00).unwrap();

        let sequencer_state = unsafe { &mut *(&raw mut SEQ) };
        set_bpm(140);

        // For testing
        let pattern = &mut sequencer_state.patterns[0];
        pattern.tracks[2].steps[2].pitch = 60;
        pattern.tracks[2].steps[2].active = true;
        pattern.tracks[4].steps[6].pitch = 61;
        pattern.tracks[4].steps[6].active = true;
        pattern.tracks[4].steps[8].pitch = 62;
        pattern.tracks[4].steps[8].active = true;
        pattern.tracks[7].steps[1].pitch = 60;
        pattern.tracks[7].steps[1].active = true;
        pattern.tracks[7].steps[2].pitch = 60;
        pattern.tracks[7].steps[2].active = true;

        rebuild_rt_cache(&sequencer_state);
        render(&mut display, &sequencer_state);

        #[cfg(feature = "keyboard-input")]
        let channels = rtt_target::rtt_init! {
            up: {
                0: {
                    size: 1024,
                    mode: rtt_target::ChannelMode::NoBlockSkip,
                    name: "Terminal"
                }
            }
            down: {
                0: {
                    size: 64,
                    name: "Terminal"
                }
            }
        };

        #[cfg(not(feature = "keyboard-input"))]
        rtt_target::rtt_init_print!();

        #[cfg(feature = "keyboard-input")]
        let mut input_channel = channels.down.0;

        #[cfg(feature = "keyboard-input")]
        rtt_target::set_print_channel(channels.up.0);

        loop {
            #[cfg(feature = "keyboard-input")]
            {
                let mut buf = [0u8; 1];
                if input_channel.read(&mut buf) > 0 {
                    if let Some(button) = key_to_button(buf[0]) {
                        handle_button_press(button, sequencer_state);
                    }
                }
            }
            let step_moved = STEP_FLAG.swap(false, Ordering::Acquire);
            let dirty = take_dirty();
            let mut dirty_steps: u16 = 0;
            let mut dirty_labels = false;

            #[cfg(feature = "perf")]
            {
                let (missed_segments, max_overrun_us) = take_overrun_stats();
                if missed_segments != 0 || max_overrun_us != 0 {
                    rtt_target::rprintln!(
                        "clock overrun: missed_segments={} max_overrun_us={}",
                        missed_segments,
                        max_overrun_us
                    );
                }
            }

            if dirty & DIRTY_RT_CACHE != 0 {
                rebuild_rt_cache(&sequencer_state);
            }
            let playing_step = CURRENT_STEP.load(Ordering::Relaxed);
            let max_steps = sequencer_state.max_steps;
            let prev_step = if playing_step == 0 { max_steps - 1 } else { playing_step - 1 };

            if step_moved {
                render_playhead_marker(&mut display, prev_step, false);
                render_playhead_marker(&mut display, playing_step, true);
            }

            #[cfg(not(feature = "keyboard-input"))]
            if PLAYING.load(Ordering::Relaxed) && !step_moved && dirty == 0 {
                // TODO: could use some other interrupt source to wake from,
                // otherwise loop will spin when not playing
                cortex_m::asm::wfi();
                continue
            }
            if dirty & DIRTY_STEP_SELECTION != 0 {
                if let Some(prev) = sequencer_state.prev_selected_step {
                    dirty_steps |= 1 << prev;
                }
                if let Some(curr) = sequencer_state.selected_step {
                    dirty_steps |= 1 << curr;
                }
            }
            if dirty & (DIRTY_TRACK_SELECTION | DIRTY_NOTE_DATA) != 0 {
                if let Some(curr) = sequencer_state.selected_step {
                    dirty_steps |= 1 << curr;
                }
            }
            if dirty & DIRTY_TRACK_SELECTION != 0 {
                dirty_labels = true;
            }
            if dirty & DIRTY_PATTERN != 0 {
                render_pattern_indicator(&mut display, &sequencer_state);
            }
            if dirty & DIRTY_BPM != 0 {
                render_bpm(&mut display);
            }
            // Render all dirty steps
            if dirty_steps != 0 {
                let selected_step = sequencer_state.selected_step;
                let selected_tracks = sequencer_state.selected_tracks;
                let all_tracks = sequencer_state.get_all_tracks();
                let unselected_tracks = all_tracks & !selected_tracks;

                for step in iter_bits_u16(dirty_steps) {
                    let is_playing = step == playing_step;
                    let is_selected = selected_step == Some(step);
                    let base = if is_playing { CellHighlight::Playing } else { CellHighlight::None };
                    if is_selected {
                        render_cells(&mut display, &sequencer_state, step, selected_tracks, CellHighlight::Selected);
                        render_cells(&mut display, &sequencer_state, step, unselected_tracks, base);
                    } else {
                        render_column(&mut display, &sequencer_state, step, base);
                    }
                }
            }
            if dirty_labels {
                for track in iter_bits_u8(sequencer_state.get_all_tracks()) {
                    render_track_label(&mut display, track, sequencer_state.is_track_selected(track));
                }
            }
        }
    }
    loop {}
}
