use stm32f4xx_hal as hal;
use ra8835a::{RA8835A, ParallelBus, Config};
use hal::gpio::DynamicPin;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[derive(Debug)]
pub enum BusError {
    Pin,
    Direction,
}

pub struct DataBus {
    pub d0: DynamicPin<'A', 0>,
    pub d1: DynamicPin<'A', 1>,
    pub d2: DynamicPin<'A', 8>,
    pub d3: DynamicPin<'A', 9>,
    pub d4: DynamicPin<'A', 4>,
    pub d5: DynamicPin<'A', 5>,
    pub d6: DynamicPin<'A', 6>,
    pub d7: DynamicPin<'A', 7>,
    is_output: bool,
}

impl DataBus {
    pub fn new(
        d0: DynamicPin<'A', 0>,
        d1: DynamicPin<'A', 1>,
        d2: DynamicPin<'A', 8>,
        d3: DynamicPin<'A', 9>,
        d4: DynamicPin<'A', 4>,
        d5: DynamicPin<'A', 5>,
        d6: DynamicPin<'A', 6>,
        d7: DynamicPin<'A', 7>,
    ) -> Self {
        let mut bus = Self {
            d0, d1, d2, d3, d4, d5, d6, d7,
            is_output: false,
        };
        bus.set_output();
        bus
    }
}

impl ParallelBus for DataBus {
    type Error = BusError;

    fn write(&mut self, value: u8) -> () {
        if !self.is_output { self.set_output(); }
        if (value & 0x01) != 0 { self.d0.set_high().unwrap(); } else { self.d0.set_low().unwrap(); }
        if (value & 0x02) != 0 { self.d1.set_high().unwrap(); } else { self.d1.set_low().unwrap(); }
        if (value & 0x04) != 0 { self.d2.set_high().unwrap(); } else { self.d2.set_low().unwrap(); }
        if (value & 0x08) != 0 { self.d3.set_high().unwrap(); } else { self.d3.set_low().unwrap(); }
        if (value & 0x10) != 0 { self.d4.set_high().unwrap(); } else { self.d4.set_low().unwrap(); }
        if (value & 0x20) != 0 { self.d5.set_high().unwrap(); } else { self.d5.set_low().unwrap(); }
        if (value & 0x40) != 0 { self.d6.set_high().unwrap(); } else { self.d6.set_low().unwrap(); }
        if (value & 0x80) != 0 { self.d7.set_high().unwrap(); } else { self.d7.set_low().unwrap(); }
    }

    fn read(&mut self) -> Result<u8, Self::Error> {
        if self.is_output { self.set_input(); }
        let mut value: u8 = 0;
        if self.d0.is_high().map_err(|_| BusError::Pin)? { value |= 0x01; }
        if self.d1.is_high().map_err(|_| BusError::Pin)? { value |= 0x02; }
        if self.d2.is_high().map_err(|_| BusError::Pin)? { value |= 0x04; }
        if self.d3.is_high().map_err(|_| BusError::Pin)? { value |= 0x08; }
        if self.d4.is_high().map_err(|_| BusError::Pin)? { value |= 0x10; }
        if self.d5.is_high().map_err(|_| BusError::Pin)? { value |= 0x20; }
        if self.d6.is_high().map_err(|_| BusError::Pin)? { value |= 0x40; }
        if self.d7.is_high().map_err(|_| BusError::Pin)? { value |= 0x80; }
        Ok(value)
    }

    fn set_input(&mut self) -> () {
        self.d0.make_pull_up_input();
        self.d1.make_pull_up_input();
        self.d2.make_pull_up_input();
        self.d3.make_pull_up_input();
        self.d4.make_pull_up_input();
        self.d5.make_pull_up_input();
        self.d6.make_pull_up_input();
        self.d7.make_pull_up_input();
        self.is_output = false;
    }

    fn set_output(&mut self) -> () {
        self.d0.make_push_pull_output();
        self.d1.make_push_pull_output();
        self.d2.make_push_pull_output();
        self.d3.make_push_pull_output();
        self.d4.make_push_pull_output();
        self.d5.make_push_pull_output();
        self.d6.make_push_pull_output();
        self.d7.make_push_pull_output();
        self.is_output = true;
    }
}

pub fn initialize_display<DB, A0, WR, RD, CS, RES, DELAY>(
    data_bus: DB,
    a0: A0,
    wr: WR,
    rd: RD,
    cs: CS,
    res: RES,
    delay: &mut DELAY,
)
where
    DB: ParallelBus,
    A0: OutputPin,
    WR: OutputPin,
    RD: OutputPin,
    CS: OutputPin,
    RES: OutputPin,
    DELAY: DelayNs,
{
    let config = Config::new(8, 8, 320, 240).unwrap();
    let mut display = RA8835A::new(
        data_bus,
        a0,
        wr,
        rd,
        cs,
        res,
        delay,
        config,
    ).ok().unwrap();
    display.clear_display();
    display.write_text("Hell world");
    // return display
}
