#![feature(core_intrinsics)]
// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use core::{
	convert::TryInto,
	mem,
	sync::atomic::{AtomicUsize, Ordering},
};

use cortex_m::{
	interrupt::{self, CriticalSection},
	singleton,
};
use cortex_m_rt as rt;
use defmt_rtt as _; // global logger
use ehal::spi::MODE_0;
use embedded_hal as ehal;
use hal::{
	dma::{StreamsTuple, Transfer},
	gpio::{gpioa, Floating, Input, Output, PushPull},
	pac,
	prelude::*,
	spi::Spi,
};
use pac::{DMA2, GPIOA, SPI1};
use panic_probe as _;
use rt::{exception, ExceptionFrame};
use stm32f4xx_hal as hal;
use stm32f4xx_hal::{
	dma::{config::DmaConfig, StreamX},
	spi::TransferModeNormal,
};

#[cortex_m_rt::entry]
fn main() -> ! {
	let dp = pac::Peripherals::take().unwrap();

	let rcc = dp.RCC.constrain();

	// TRY the other clock configuration
	let _clocks = rcc
		.cfgr
		.use_hse(25.mhz())
		.require_pll48clk()
		.sysclk(100.mhz())
		.hclk(100.mhz())
		.pclk1(50.mhz())
		.pclk2(100.mhz())
		.freeze();

	let gpioa = dp.GPIOA.split();

	let dma = StreamsTuple::new(dp.DMA2);

	defmt::info!("init");
	let (spi, ready_pin) = LEDController::<()>::configure_hardware(
		gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa4, gpioa.pa5, gpioa.pa6, gpioa.pa7, dp.SPI1,
	);

	let buf = singleton!(: [u8; common::TRANSFER_BUFFER_SIZE] = [0; common::TRANSFER_BUFFER_SIZE])
		.unwrap();

	let mut ctrl = LEDController::new(spi, dma.0, ready_pin, buf);
	defmt::info!("run");

	ctrl.run();
}

const fn create_pin_bits(i: u32) -> (u32, u32) {
	(1 << (i + 16), 1 << i)
}

const P0: (u32, u32) = create_pin_bits(0);
const P1: (u32, u32) = create_pin_bits(1);
const P2: (u32, u32) = create_pin_bits(2);

struct LEDController<SpiPins> {
	state:      common::LEDs,
	spi:        Spi<SPI1, SpiPins, TransferModeNormal>,
	dma_stream: StreamX<DMA2, 0>,
	ready_pin:  gpioa::PA4<Output<PushPull>>,
	buffer:     &'static mut [u8; common::TRANSFER_BUFFER_SIZE],
}

impl<SpiPins> LEDController<SpiPins> {
	pub fn new(
		spi: Spi<SPI1, SpiPins, TransferModeNormal>,
		dma_stream: StreamX<DMA2, 0>,
		ready_pin: gpioa::PA4<Output<PushPull>>,
		buffer: &'static mut [u8; common::TRANSFER_BUFFER_SIZE],
	) -> Self {
		LEDController {
			state: [
				[[255, 0, 0]; common::LEDS_PER_STRIP],
				[[0, 255, 0]; common::LEDS_PER_STRIP],
				[[0, 0, 255]; common::LEDS_PER_STRIP],
			],
			spi,
			dma_stream,
			ready_pin,
			buffer,
		}
	}

	pub fn configure_hardware(
		pa0: gpioa::PA0<Input<Floating>>,
		pa1: gpioa::PA1<Input<Floating>>,
		pa2: gpioa::PA2<Input<Floating>>,
		pa4: gpioa::PA4<Input<Floating>>,
		pa5: gpioa::PA5<Input<Floating>>,
		pa6: gpioa::PA6<Input<Floating>>,
		pa7: gpioa::PA7<Input<Floating>>,
		spi: SPI1,
	) -> (
		Spi<
			SPI1,
			(
				gpioa::PA5<Input<Floating>>,
				gpioa::PA6<Input<Floating>>,
				gpioa::PA7<Input<Floating>>,
			),
			TransferModeNormal,
		>,
		gpioa::PA4<Output<PushPull>>,
	) {
		pa0.into_push_pull_output();
		pa1.into_push_pull_output();
		pa2.into_push_pull_output();

		let spi = Spi::new_slave(spi, (pa5, pa6, pa7), MODE_0);
		let ready_pin = pa4.into_push_pull_output();

		(spi, ready_pin)
	}

	#[inline(always)]
	fn set_pin_registers(&mut self, bits: u32) {
		unsafe {
			(*GPIOA::ptr()).bsrr.write(|w| w.bits(bits));
		};
	}

	#[inline]
	fn write_byte(&mut self, _: &CriticalSection, mut data: (u8, u8, u8)) {
		for _ in 0..8 {
			cortex_m::asm::delay(30);
			self.set_pin_registers(P0.1 | P1.1 | P2.1);
			cortex_m::asm::delay(10);
			self.set_pin_registers(
				if data.0 & 0x80 != 0 { 0 } else { P0.0 }
					| if data.1 & 0x80 != 0 { 0 } else { P1.0 }
					| if data.2 & 0x80 != 0 { 0 } else { P2.0 },
			);
			cortex_m::asm::delay(15);
			self.set_pin_registers(P0.0 | P1.0 | P2.0);
			data.0 <<= 1;
			data.1 <<= 1;
			data.2 <<= 1;
		}
	}

	fn write_data(&mut self, c: &CriticalSection) {
		for i in 0..self.state[0].len() {
			// G R B
			self.write_byte(
				c,
				(
					self.state[0][i][1],
					self.state[1][i][1],
					self.state[2][i][1],
				),
			);
			self.write_byte(
				c,
				(
					self.state[0][i][0],
					self.state[1][i][0],
					self.state[2][i][0],
				),
			);
			self.write_byte(
				c,
				(
					self.state[0][i][2],
					self.state[1][i][2],
					self.state[2][i][2],
				),
			);
		}

		cortex_m::asm::delay(80 * 300);
	}

	fn dma_spi_buffer_read(&mut self) {
		replace_with::replace_with(
			self,
			|| panic!("replace failed, maybe UB"),
			|LEDController {
			     spi,
			     dma_stream,
			     ready_pin,
			     buffer,
			     state,
			 }| {
				let rx = spi.use_dma().rx();
				let config = DmaConfig::default()
					.transfer_complete_interrupt(true)
					.memory_increment(true)
					.double_buffer(false);
				let mut transfer = Transfer::init_peripheral_to_memory(
					dma_stream,
					rx,
					buffer.as_mut(),
					None,
					config,
				);
				transfer.start(|_| {});
				transfer.wait();
				let (dma_stream, rx, buffer, _) = transfer.release();
				let spi = rx.release();
				LEDController {
					spi,
					dma_stream,
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
		self.ready_pin.set_high();
		let instr = nb::block!(self.spi.read());
		self.ready_pin.set_low();
		let instr = instr?;
		match instr {
			common::messages::NO_UPDATE => {}
			common::messages::UPDATE_LEDS => {
				self.dma_spi_buffer_read();
				// defmt::info!("buffer: {:?}", self.buffer).ok();
				self.state = unsafe { mem::transmute_copy(self.buffer) };
			}
			common::messages::APPLY_LEDS => return Ok(true),
			_ => {
				// defmt::info!("received invalid instruction: {}", instr).ok();
			}
		}
		Ok(false)
	}

	fn run(&mut self) -> ! {
		loop {
			let write = match self.read_from_spi() {
				Ok(write) => write,
				Err(e) => {
					match SpiError::from(e) {
						SpiError::Overrun => {
							self.spi.clear_ovr();
							defmt::info!("overrun");
						}
						e => {
							defmt::info!("failed reading from buffer: {:?}", e);
						}
					}
					continue;
				}
			};
			if write {
				// defmt::info!("write to strip");
				// can't have any interrupts while writing
				interrupt::free(|c| {
					self.write_data(c);
				});
			}
		}
	}
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
	cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
	// NOTE(no-CAS) `timestamps` runs with interrupts disabled
	let n = COUNT.load(Ordering::Relaxed);
	COUNT.store(n + 1, Ordering::Relaxed);
	n
});

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
	loop {
		cortex_m::asm::bkpt();
	}
}

/// SPI error
#[non_exhaustive]
#[derive(defmt::Format)]
pub enum SpiError {
	/// Overrun occurred
	Overrun,
	/// Mode fault occurred
	ModeFault,
	/// CRC error
	Crc,
}

impl From<hal::spi::Error> for SpiError {
	fn from(e: hal::spi::Error) -> Self {
		use hal::spi::Error::*;
		match e {
			Overrun => SpiError::Overrun,
			ModeFault => SpiError::ModeFault,
			Crc => SpiError::Crc,
			_ => defmt::unimplemented!("new spi error not handled"),
		}
	}
}

// #[allow(non_snake_case)]
// #[exception]
// unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
// 	panic!("{:#?}", ef);
// }
