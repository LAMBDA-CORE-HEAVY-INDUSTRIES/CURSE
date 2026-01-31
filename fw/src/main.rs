#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use core::sync::atomic::Ordering;

use crate::hal::{pac, prelude::*};
use cortex_m_rt::entry;
use curse::render::{render, render_steps};
use curse::sequencer::{CURRENT_STEP, PREVIOUS_STEP, STEP_FLAG, SequencerState, set_bpm};
use defmt_rtt as _;
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_halt as _;
use stm32f4xx_hal::timer::Event;
use stm32f4xx_hal::{self as hal, spi::Spi};

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

        let mut sequencer_state = SequencerState::new();
        set_bpm(&mut timer, 134);

        // For testing
        sequencer_state.steps[2][2].pitch = 60;
        sequencer_state.steps[2][2].active = true;
        sequencer_state.steps[4][6].pitch = 61;
        sequencer_state.steps[4][6].active = true;
        sequencer_state.steps[4][8].pitch = 62;
        sequencer_state.steps[4][8].active = true;
        sequencer_state.steps[7][1].pitch = 60;
        sequencer_state.steps[7][1].active = true;
        sequencer_state.steps[7][2].pitch = 60;
        sequencer_state.steps[7][2].active = true;

        render(&mut display, &sequencer_state);

        loop {
            if STEP_FLAG.swap(false, Ordering::Acquire) {
                let step = CURRENT_STEP.load(Ordering::Relaxed);
                let previous_step = PREVIOUS_STEP.load(Ordering::Relaxed);
                defmt::trace!("previous step {:?}", previous_step);
                defmt::trace!("step {:?}", step);
                render_steps(&mut display, &sequencer_state, step, true);
                render_steps(&mut display, &sequencer_state, previous_step, false);
            }
        }
    }
    loop {}
}
