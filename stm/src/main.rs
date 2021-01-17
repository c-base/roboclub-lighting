#![feature(core_intrinsics)]
// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate embedded_hal as ehal;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;

use crate::{
	hal::{prelude::*, spi::Spi},
	rt::ExceptionFrame,
};
use core::{convert::TryInto, mem};
use cortex_m::singleton;
use cortex_m_semihosting::{hio, hprintln};
use ehal::spi::MODE_0;
use hal::{
	dma::{dma1, ReadDma},
	gpio::{gpioa, gpiob, Alternate, Floating, Input, Output, PushPull, AF5},
	rcc,
	spi::{MisoPin, MosiPin, SckPin},
	stm32::{GPIOB, SPI1},
};

#[entry]
fn main() -> ! {
	// let mut cp = cortex_m::Peripherals::take().unwrap();
	let dp = hal::stm32::Peripherals::take().unwrap();

	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();
	let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

	// TRY the other clock configuration
	// let clocks = rcc.cfgr.freeze(&mut flash.acr);
	let _clocks = rcc
		.cfgr
		.sysclk(80.mhz())
		.pclk1(80.mhz())
		.pclk2(80.mhz())
		.freeze(&mut flash.acr, &mut pwr);

	let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
	let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);

	let dma = dp.DMA1.split(&mut rcc.ahb1);

	// hprintln!("init");
	let ready_pin = LEDController::<(), (), ()>::configure_pins(
		gpiob.pb4,
		gpiob.pb13,
		gpiob.pb14,
		gpiob.pb15,
		&mut gpiob.moder,
		&mut gpiob.otyper,
	);
	let spi = LEDController::<gpioa::PA5<_>, gpioa::PA6<_>, gpioa::PA7<_>>::configure_spi(
		gpioa.pa15,
		gpioa.pa5,
		gpioa.pa6,
		gpioa.pa7,
		&mut gpioa.moder,
		&mut gpioa.afrl,
		&mut gpioa.afrh,
		&mut rcc.apb2,
		dp.SPI1,
	);

	let buf = singleton!(: [u8; common::TRANSFER_BUFFER_SIZE] = [0; common::TRANSFER_BUFFER_SIZE])
		.unwrap();

	let mut ctrl = LEDController::new(spi, dma.2, ready_pin, buf);
	// hprintln!("run");

	ctrl.run();
}

const fn create_pin_bits(i: u32) -> (u32, u32) {
	(1 << (i + 16), 1 << i)
}

const P13: (u32, u32) = create_pin_bits(13);
const P14: (u32, u32) = create_pin_bits(14);
const P15: (u32, u32) = create_pin_bits(15);

struct LEDController<SCK, MISO, MOSI> {
	state:       common::LEDs,
	spi:         Spi<SPI1, (SCK, MISO, MOSI)>,
	dma_channel: dma1::C2,
	ready_pin:   gpiob::PB4<Output<PushPull>>,
	buffer:      &'static mut [u8; common::TRANSFER_BUFFER_SIZE],
}

impl<SCK, MISO, MOSI> LEDController<SCK, MISO, MOSI> {
	pub fn new(
		spi: Spi<SPI1, (SCK, MISO, MOSI)>,
		dma_channel: dma1::C2,
		ready_pin: gpiob::PB4<Output<PushPull>>,
		buffer: &'static mut [u8; common::TRANSFER_BUFFER_SIZE],
	) -> Self {
		LEDController {
			state: [
				[[255, 0, 0]; common::LEDS_PER_STRIP],
				[[0, 255, 0]; common::LEDS_PER_STRIP],
				[[0, 0, 255]; common::LEDS_PER_STRIP],
			],
			spi,
			dma_channel,
			ready_pin,
			buffer,
		}
	}

	pub fn configure_spi(
		pa15: gpioa::PA15<Input<Floating>>,
		pa5: gpioa::PA5<Input<Floating>>,
		pa6: gpioa::PA6<Input<Floating>>,
		pa7: gpioa::PA7<Input<Floating>>,
		moder: &mut gpioa::MODER,
		afrl: &mut gpioa::AFRL,
		afrh: &mut gpioa::AFRH,
		apb2: &mut rcc::APB2,
		spi: SPI1,
	) -> Spi<
		SPI1,
		(
			gpioa::PA5<Alternate<AF5, Input<Floating>>>,
			gpioa::PA6<Alternate<AF5, Input<Floating>>>,
			gpioa::PA7<Alternate<AF5, Input<Floating>>>,
		),
	>
	where
		SCK: SckPin<SPI1>,
		MISO: MisoPin<SPI1>,
		MOSI: MosiPin<SPI1>,
	{
		let _nss = pa15.into_af5(moder, afrh);
		let sck = pa5.into_af5(moder, afrl);
		let miso = pa6.into_af5(moder, afrl);
		let mosi = pa7.into_af5(moder, afrl);

		let spi = Spi::spi1_slave(spi, (sck, miso, mosi), MODE_0, apb2);

		spi
	}

	pub fn configure_pins(
		pb4: gpiob::PB4<Input<Floating>>,
		pb13: gpiob::PB13<Input<Floating>>,
		pb14: gpiob::PB14<Input<Floating>>,
		pb15: gpiob::PB15<Input<Floating>>,
		moder: &mut gpiob::MODER,
		otyper: &mut gpiob::OTYPER,
	) -> gpiob::PB4<Output<PushPull>> {
		pb13.into_push_pull_output(moder, otyper);
		pb14.into_push_pull_output(moder, otyper);
		pb15.into_push_pull_output(moder, otyper);

		pb4.into_push_pull_output(moder, otyper)
	}

	fn set_pin_registers(&mut self, bits: u32) {
		unsafe {
			(*GPIOB::ptr()).bsrr.write(|w| w.bits(bits));
		};
	}

	fn write_byte(&mut self, mut data: (u8, u8, u8)) {
		for _ in 0..8 {
			cortex_m::asm::delay(30);
			self.set_pin_registers(P13.1 | P14.1 | P15.1);
			cortex_m::asm::delay(10);
			self.set_pin_registers(
				if data.0 & 0x80 != 0 { 0 } else { P13.0 }
					| if data.1 & 0x80 != 0 { 0 } else { P14.0 }
					| if data.2 & 0x80 != 0 { 0 } else { P15.0 },
			);
			cortex_m::asm::delay(15);
			self.set_pin_registers(P13.0 | P14.0 | P15.0);
			data.0 <<= 1;
			data.1 <<= 1;
			data.2 <<= 1;
		}
	}

	fn write_data(&mut self) {
		for i in 0..self.state[0].len() {
			// G R B
			self.write_byte((
				self.state[0][i][1],
				self.state[1][i][1],
				self.state[2][i][1],
			));
			self.write_byte((
				self.state[0][i][0],
				self.state[1][i][0],
				self.state[2][i][0],
			));
			self.write_byte((
				self.state[0][i][2],
				self.state[1][i][2],
				self.state[2][i][2],
			));
		}

		cortex_m::asm::delay(80 * 300);
	}

	fn dma_spi_buffer_read(&mut self) {
		replace_with::replace_with(
			self,
			|| panic!("replace failed, maybe UB"),
			|LEDController {
			     spi,
			     dma_channel,
			     ready_pin,
			     buffer,
			     state,
			 }| {
				let dma_spi = spi.with_rx_dma(dma_channel);
				let transfer = dma_spi.read(buffer.as_mut());
				let (buffer, dma_spi) = transfer.wait();
				let (spi, dma_channel) = dma_spi.free();
				LEDController {
					spi,
					dma_channel,
					ready_pin,
					buffer: buffer.try_into().unwrap(),
					state,
				}
			},
		);
	}

	/// reads updates from spi, returns whether the stripes should be updated
	fn read_from_spi(&mut self) -> Result<bool, hal::spi::Error> {
		nb::block!(self.spi.send(0b1))?;
		self.ready_pin.set_high().ok();
		let instr = nb::block!(self.spi.read());
		self.ready_pin.set_low().ok();
		let instr = instr?;
		match instr {
			common::messages::NO_UPDATE => {}
			common::messages::UPDATE_LEDS => {
				self.dma_spi_buffer_read();
				// hprintln!("buffer: {:?}", self.buffer).ok();
				self.state = unsafe { mem::transmute(*self.buffer) };
			}
			common::messages::APPLY_LEDS => return Ok(true),
			_ => {
				// hprintln!("received invalid instruction: {}", instr).ok();
			}
		}
		Ok(false)
	}

	fn run(&mut self) -> ! {
		loop {
			let write = match self.read_from_spi() {
				Ok(write) => write,
				Err(e) => {
					match e {
						hal::spi::Error::Overrun => {
							self.spi.clear_overrun();
							// hprintln!("overrun").ok();
						}
						_ => {
							// hprintln!("failed reading from buffer: {:?}", e).ok();
						}
					}
					continue;
				}
			};
			if write {
				// hprintln!("write to strip");
				self.write_data();
			}
		}
	}
}

// fn print<T: Display>(out: T) {
// 	if let Ok(mut hstdout) = hio::hstdout() {
// 		writeln!(hstdout, "{}", out).ok();
// 	}
// }

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
	panic!("{:#?}", ef);
}
