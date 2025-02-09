#![deny(unsafe_code)]
#![allow(clippy::empty_loop)]
#![no_main]
#![no_std]

use panic_halt as _; 
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;
use crate::hal::{pac, prelude::*};
use ra8835a::{RA8835A, ParallelBus};

#[derive(Debug)]
pub enum BusError {
    Pin,
    Direction,
}

pub struct DataBus {
    d0: hal::gpio::Pin<'A', 0, hal::gpio::Output>,
    d1: hal::gpio::Pin<'A', 1, hal::gpio::Output>,
    d2: hal::gpio::Pin<'A', 2, hal::gpio::Output>,
    d3: hal::gpio::Pin<'A', 3, hal::gpio::Output>,
    d4: hal::gpio::Pin<'A', 4, hal::gpio::Output>,
    d5: hal::gpio::Pin<'A', 5, hal::gpio::Output>,
    d6: hal::gpio::Pin<'A', 6, hal::gpio::Output>,
    d7: hal::gpio::Pin<'A', 7, hal::gpio::Output>,
}

impl ParallelBus for DataBus {
    type Error = BusError;

    fn write(&mut self, value: u8) -> Result<(), Self::Error> {
        self.d0.set_state(hal::gpio::PinState::from((value & 0x01) != 0));
        self.d1.set_state(hal::gpio::PinState::from((value & 0x02) != 0));
        self.d2.set_state(hal::gpio::PinState::from((value & 0x04) != 0));
        self.d3.set_state(hal::gpio::PinState::from((value & 0x08) != 0));
        self.d4.set_state(hal::gpio::PinState::from((value & 0x10) != 0));
        self.d5.set_state(hal::gpio::PinState::from((value & 0x20) != 0));
        self.d6.set_state(hal::gpio::PinState::from((value & 0x40) != 0));
        self.d7.set_state(hal::gpio::PinState::from((value & 0x80) != 0));
        Ok(())
    }

    fn read(&mut self) -> Result<u8, Self::Error> {
        todo!()
    }

    fn set_input(&mut self) -> Result<(), Self::Error> {
        todo!()
    }

    fn set_output(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();

        let data_bus = DataBus {
            d0: gpioa.pa0.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d1: gpioa.pa1.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d2: gpioa.pa2.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d3: gpioa.pa3.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d4: gpioa.pa4.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d5: gpioa.pa5.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d6: gpioa.pa6.into_push_pull_output_in_state(hal::gpio::PinState::Low),
            d7: gpioa.pa7.into_push_pull_output_in_state(hal::gpio::PinState::Low),
        };

        let res = gpiob.pb0.into_push_pull_output_in_state(hal::gpio::PinState::High);
        let a0 = gpiob.pb1.into_push_pull_output_in_state(hal::gpio::PinState::High);
        let cs = gpiob.pb2.into_push_pull_output_in_state(hal::gpio::PinState::High);
        let rd = gpiob.pb3.into_push_pull_output_in_state(hal::gpio::PinState::High);
        let wr = gpiob.pb4.into_push_pull_output_in_state(hal::gpio::PinState::High);
        
        let mut delay = cp.SYST.delay(&clocks);
        delay.delay_ms(1000);

        let mut display = RA8835A::new(
            data_bus,
            a0,
            wr,
            rd,
            cs,
            res,
            delay,
        ).unwrap();
        loop {}
    }
    loop {}
}
