#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

#[entry]
fn main() -> ! {
	// device core peripherals
	let dcp = CorePeripherals::take().unwrap();
	// device peripherals
	let dp = Peripherals::take().unwrap();

	let mut flash = dp.FLASH.constrain();

	// setup clocks
	let mut rcc = dp.RCC.constrain();
	let clocks = rcc.cfgr.use_hse(8.MHz()).freeze(&mut flash.acr);

	let mut delay = Delay::new(dcp.SYST, clocks);

	let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

	// PB10
	let mut led1 = gpiob
		.pb10
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
	let mut led2 = gpiob
		.pb11
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

	loop {
		led1.set_low();
		delay.delay_ms(1000u16);
		led1.set_high();

		led2.set_low();
		delay.delay_ms(1000u16);
		led2.set_high();
	}
}
