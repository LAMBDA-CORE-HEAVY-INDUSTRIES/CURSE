#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::sync::atomic::Ordering;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;
use curse::render::{render, render_step, render_steps};
use curse::sequencer::{CURRENT_STEP, EDIT_FLAG, SEQ, STEP_FLAG, set_bpm};
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_halt as _;
use stm32f4xx_hal::timer::Event;
use stm32f4xx_hal::{self as hal, spi::Spi};

#[cfg(feature = "keyboard-input")]
use curse::input::{handle_button_press, key_to_button};

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

        let mut timer = dp.TIM3.counter_hz(&clocks);
        timer.listen(Event::Update);
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
        set_bpm(&mut timer, 40);

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

            if STEP_FLAG.swap(false, Ordering::Acquire) {
                let active_step = CURRENT_STEP.load(Ordering::Relaxed);
                let max_steps = sequencer_state.max_steps;
                let inactive_step = if active_step == 0 { max_steps - 1 } else { active_step - 1 };
                render_steps(&mut display, &sequencer_state, active_step, true);
                render_steps(&mut display, &sequencer_state, inactive_step, false);
            }
            if EDIT_FLAG.swap(false, Ordering::Acquire) {
                render_step(&mut display, &sequencer_state, sequencer_state.selected_step.unwrap());
            }
        }
    }
    loop {}
}
