use std::{
	error::Error,
	fmt::Debug,
	ops::{Bound, Index, IndexMut, RangeBounds},
	slice::{Iter, IterMut},
	time::Duration,
};

use rppal::{
	gpio::{Gpio, InputPin, Level, Trigger},
	spi::{Bus, Mode::Mode0, SlaveSelect, Spi},
};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::colour::RGB;

const BLANK: [[u8; 3]; common::LEDS_PER_STRIP] = [[0; 3]; common::LEDS_PER_STRIP];

const SPI_CLOCK: u32 = 30_000_000;

pub struct Controller {
	spi:       Spi,
	ready_pin: InputPin,
	state:     common::LEDs,
	buffers:   [[u8; common::TRANSFER_BUFFER_SIZE]; 2],
}

impl Controller {
	pub fn new(ready_pin: u8, spi_bus: Bus) -> Result<Self, Box<dyn Error>> {
		let spi = Spi::new(spi_bus, SlaveSelect::Ss0, SPI_CLOCK, Mode0).unwrap();

		let mut ready_pin = Gpio::new()?.get(ready_pin)?.into_input();
		ready_pin.set_interrupt(Trigger::RisingEdge)?;

		Ok(Controller {
			spi,
			ready_pin,
			state: [[[0; 3]; common::LEDS_PER_STRIP]; common::STRIPS],
			buffers: [[0; common::TRANSFER_BUFFER_SIZE]; 2],
		})
	}

	fn copy_into_state(&mut self, rgb: &[&[[u8; 3]]; common::STRIPS]) {
		let strips = common::STRIPS;

		for strip in 0..strips {
			let data_in = &rgb[strip];
			let len = common::LEDS_PER_STRIP.max(data_in.len());
			let data_slice = &data_in[..len];
			let state = &mut self.state[strip][..len];
			state.copy_from_slice(data_slice);
			if len < common::LEDS_PER_STRIP {
				let state = &mut self.state[strip][len..];
				state.copy_from_slice(&BLANK[len..]);
			}
		}
	}

	fn encode_state(&self) -> common::LedTransferBuffer {
		unsafe { std::mem::transmute(self.state) }
	}

	fn wait_for_interrupt(&mut self, timeout_ms: u64) -> Option<Level> {
		self.ready_pin
			.poll_interrupt(false, Some(Duration::from_millis(timeout_ms)))
			.expect("should be able to poll interrupt")
	}

	#[instrument(skip(self))]
	fn send_command(&mut self, command: u8) -> Result<(), String> {
		let mut read = [0x0];

		let res = self.spi.transfer(&mut read, &[command]);

		// println!("received {:?}", read);

		res.map_err(|e| format!("sending to spi failed: {:?}", e))
			.and_then(|_| {
				if read[0] == 1 {
					return Ok(());
				}
				Err(format!(
					"sending spi command failed, got {} instead of ack(1)",
					read[0]
				))
			})
	}

	/// Writes the inner state to the strips
	#[instrument(skip(self))]
	fn write_state_internal(&mut self) -> Result<(), String> {
		let buffer = self.encode_state();

		let res = self.wait_for_interrupt(50);

		// timeout, clear out potential unfinished transfer
		if res.is_none() && self.ready_pin.is_low() {
			warn!("waiting for interrupt timed out, clearing spi transfer");

			// writing the whole buffer is fine, because writing 0 just tells it to redraw when
			// it accepts a new command and it clears overrun automatically
			let mut read = self.buffers[0];
			let write = self.buffers[1];

			self.spi
				.transfer(&mut read, &write)
				.map_err(|e| format!("sending to spi failed: {:?}", e))?;

			self.wait_for_interrupt(5).ok_or(
				"!!!! spi still not ready after clear, stm might not be connected / on !!!!"
					.to_string(),
			)?;
		}

		if self.ready_pin.is_high() {
			trace!("sending spi buffer");
			// FIXME: stm never sends ack
			let _ = self.send_command(common::messages::UPDATE_LEDS);

			std::thread::sleep(Duration::from_micros(200));

			let mut read = self.buffers[0];
			self.spi
				.transfer(&mut read, &buffer)
				.map_err(|e| format!("sending to spi failed: {:?}", e))?;

			std::thread::sleep(Duration::from_micros(200));

			self.wait_for_interrupt(5)
				.ok_or("waiting for interrupt failed trying to apply leds".to_string())?;

			// FIXME: stm never sends ack
			let _ = self.send_command(common::messages::APPLY_LEDS);

			trace!("data sent");
		} else {
			trace!("could not send spi buffer, not ready");
		}

		Ok(())
	}

	/// Writes the inner state to the strips
	#[instrument(skip(self))]
	pub fn write_state(&mut self) {
		match self.write_state_internal() {
			Err(e) => println!("error sending state: {}", e),
			_ => {}
		};
	}

	pub fn write_rgb(&mut self, rgb: &[&[[u8; 3]]; common::STRIPS]) {
		self.copy_into_state(rgb);
		self.write_state();
	}

	pub fn write<'a, T, I>(&mut self, data: T)
	where
		T: IntoIterator<Item = I>,
		I: Into<RGB>,
	{
		let vec = data
			.into_iter()
			.map(|c| c.into())
			.map(|c| [c.r, c.g, c.b])
			.collect::<Vec<_>>();

		self.write_rgb(&[vec.as_slice(), vec.as_slice(), vec.as_slice()]);
	}

	pub fn state_mut(&mut self) -> &mut common::LEDs {
		&mut self.state
	}

	pub fn state_mut_flat(&mut self) -> &mut [[u8; 3]; common::LEDS_PER_STRIP * common::STRIPS] {
		unsafe { std::mem::transmute(&mut self.state) }
	}

	pub fn views_mut(&mut self) -> Views {
		Views::new(&mut self.state)
	}

	// pub fn one_strip_mut(&mut self) -> &mut [[u8; 3]; common::LEDS_PER_STRIP] {}

	// pub fn named_views_mut<'a>(&mut self) -> &'a mut Views {
	// 	self.state[0].split_at_mut()
	//
	// 	let mut views = Views {}
	// 	&mut views
	// }
}

///
/// strip 1: 0-148  149-308  309-391  392-474
/// strip 2: 0-148  149-308  309-350  351-436
/// strip 3: 0-106  107-153  154-208  209-308  309-350  351-442  443-474
///
pub struct Views<'a> {
	pub sections: [Section<'a>; 15],
}

impl<'a> Views<'a> {
	pub fn new(leds: &'a mut [[[u8; 3]; common::LEDS_PER_STRIP]; common::STRIPS]) -> Self {
		let [first, second, third] = leds;

		let (section1, rest) = first.split_at_mut(149);
		let (section2, rest) = rest.split_at_mut(309 - 149);
		let (section3, rest) = rest.split_at_mut(392 - 309);
		let (section4, _) = rest.split_at_mut(475 - 392);

		let (section5, rest) = second.split_at_mut(149);
		let (section6, rest) = rest.split_at_mut(309 - 149);
		let (section7, rest) = rest.split_at_mut(351 - 309);
		let (section8, _) = rest.split_at_mut(437 - 351);

		let (section9, rest) = third.split_at_mut(107);
		let (section10, rest) = rest.split_at_mut(154 - 107);
		let (section11, rest) = rest.split_at_mut(208 - 154);
		let (section12, rest) = rest.split_at_mut(308 - 208);
		let (section13, rest) = rest.split_at_mut(350 - 308);
		let (section14, rest) = rest.split_at_mut(442 - 350);
		let (section15, _) = rest.split_at_mut(475 - 442);

		let sections = [
			Section::new(section1, true),
			Section::new(section2, true),
			Section::new(section3, true),
			Section::new(section4, true),
			Section::new(section5, true),
			Section::new(section6, true),
			Section::new(section7, true),
			Section::new(section8, true),
			Section::new(section9, true),
			Section::new(section10, false),
			Section::new(section11, false),
			Section::new(section12, false),
			Section::new(section13, false),
			Section::new(section14, false),
			Section::new(section15, false),
		];

		Views { sections }
	}

	pub fn len(&self) -> usize {
		self.sections.len()
	}

	pub fn iter_mut(&mut self) -> IterMut<'_, Section<'a>> {
		self.sections.iter_mut()
	}

	pub fn iter(&mut self) -> Iter<'_, Section<'a>> {
		self.sections.iter()
	}
}

impl<'a> Index<usize> for Views<'a> {
	type Output = Section<'a>;

	fn index(&self, index: usize) -> &Self::Output {
		&self.sections[index]
	}
}
impl<'a> IndexMut<usize> for Views<'a> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.sections[index]
	}
}

pub struct Section<'a> {
	slice:    &'a mut [[u8; 3]],
	inverted: bool,
}

impl<'a> Section<'a> {
	pub fn new(slice: &'a mut [[u8; 3]], inverted: bool) -> Self {
		Section { slice, inverted }
	}

	pub fn len(&self) -> usize {
		self.slice.len()
	}

	pub fn iter_mut(&mut self) -> Box<dyn Iterator<Item = &'_ mut [u8; 3]> + '_> {
		let iter = self.slice.iter_mut();
		if self.inverted {
			Box::new(iter.rev())
		} else {
			Box::new(iter)
		}
	}

	pub fn range<T: RangeBounds<usize> + Debug>(&mut self, range: T) -> Section<'_> {
		let start_bound = bound_to_num(range.start_bound(), true, self.slice.len() - 1);
		let end_bound = bound_to_num(range.end_bound(), false, self.slice.len() - 1);

		let max_idx = self.len();

		let range = if self.inverted {
			let start = max_idx - end_bound;
			let end = max_idx - start_bound;
			start..end
		} else {
			start_bound..end_bound
		};

		let slice = self.slice.index_mut(range);
		Section::new(slice, self.inverted)
	}
}

fn bound_to_num(bound: Bound<&usize>, start: bool, max: usize) -> usize {
	match bound {
		Bound::Included(n) => {
			if start {
				*n
			} else {
				n + 1
			}
		}
		Bound::Excluded(n) => {
			if start {
				n + 1
			} else {
				*n
			}
		}
		Bound::Unbounded => {
			if start {
				0
			} else {
				max
			}
		}
	}
}

impl<'a> Index<usize> for Section<'a> {
	type Output = [u8; 3];

	fn index(&self, mut index: usize) -> &Self::Output {
		assert!(index < self.slice.len());
		if self.inverted {
			index = self.slice.len() - 1 - index;
		}
		self.slice.index(index)
	}
}

impl<'a> IndexMut<usize> for Section<'a> {
	fn index_mut(&mut self, mut index: usize) -> &mut Self::Output {
		if self.inverted {
			index = self.slice.len() - 1 - index;
		}
		self.slice.index_mut(index)
	}
}
