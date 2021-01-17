use std::{
	collections::{HashMap, VecDeque},
	fmt::Debug,
	future::Future,
	ops::Add,
	time::{Duration, Instant},
};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, Instrument};

use crate::{
	colour::{HSV, RGB},
	controller::Controller,
};

const NUM_LEDS: usize = common::LEDS_PER_STRIP;

fn sleep_ms(ms: u64) {
	std::thread::sleep(Duration::from_millis(ms));
}

pub trait EffectConfig: Serialize + Deserialize<'static> + Debug + Clone + Sized {}

impl<T: Serialize + Deserialize<'static> + Debug + Clone + Sized> EffectConfig for T {}

pub trait Effect<C: EffectConfig> {
	fn config(&self) -> Option<C> {
		None
	}
	fn set_config(&mut self, config: C) {}
	fn run(&mut self, ctrl: &mut Controller);
}

// struct EffectRunner {
// 	effects:       HashMap<String, Box<dyn Effect<dyn EffectConfig>>>,
// 	active_effect: String,
// }
//
// impl EffectRunner {
// 	pub fn new(
// 		effects: HashMap<String, Box<dyn Effect<dyn EffectConfig>>>,
// 		active_effect: String,
// 	) -> Self {
// 		if !effects.contains_key(&active_effect) {
// 			panic!(
// 				"effect runner initialized with an invalid `active_effect`: {:?} (map: {:?})",
// 				active_effect, effects
// 			);
// 		}
//
// 		EffectRunner {
// 			effects,
// 			active_effect,
// 		}
// 	}
//
// 	pub fn get_effect_names(&mut self) -> Vec<String> {
// 		self.effects.keys().cloned().collect()
// 	}
//
// 	pub fn set_active_effect(&mut self, effect: String) {
// 		if self.effects.contains_key(&effect) {
// 			self.active_effect = effect;
// 		}
// 	}
//
// 	pub fn get_current_effect_config(&self) -> Option<Box<dyn EffectConfig>> {
// 		let effect = self
// 			.effects
// 			.get(&self.active_effect)
// 			.expect("should only ever be set to a valid effect");
//
// 		effect.config().map(|c| Box::new(c))
// 	}
//
// 	pub fn set_current_effect_config(&mut self, config: Box<dyn EffectConfig>) {
// 		let effect = self
// 			.effects
// 			.get_mut(&self.active_effect)
// 			.expect("should only ever be set to a valid effect");
//
// 		effect.set_config(config);
// 	}
//
// 	pub async fn run(&mut self, &mut ctrl: Controller) {
// 		let mut timer = Timer::new();
// 	}
// }

impl<FN> Effect<()> for FN
where
	FN: FnMut(&mut Controller),
{
	fn run(&mut self, ctrl: &mut Controller) {
		self(ctrl)
	}
}

#[instrument(skip(ctrl, effect))]
pub fn run<C: EffectConfig, E: Effect<C>>(mut ctrl: Controller, mut effect: E) -> ! {
	let mut timer = Timer::new();

	let mut counter = 0;
	loop {
		effect.run(&mut ctrl);

		let stats = timer.tick();
		if counter == 0 {
			debug!(
				"avg time to update: {:.2}ms (now {:.2}ms, min {:.2}ms, max {:.2}ms)",
				stats.avg, stats.dt, stats.min, stats.max
			);
		}
		counter = (counter + 1) % 60;
	}
}

// fn apply(ctrl: &mut Controller, leds: &mut [RGB; NUM_LEDS]) {
// 	ctrl.write(leds.iter().cloned())
// }

#[allow(unused)]
pub fn _base() -> impl FnMut(&mut [RGB; NUM_LEDS], &mut Controller) {
	move |_leds, _ctrl| {}
}

#[allow(unused)]
pub fn meteors(
	meteor_size: u8,
	meteor_trail_decay: u8,
	speed_delay: u64,
	meteor_count: usize,
) -> impl FnMut(&mut Controller) {
	let mut rand = thread_rng();

	// let mut counter = NUM_LEDS + meteor_size as usize * 2;
	let mut offset = 0u8;
	let leds_per_meteor = NUM_LEDS / meteor_count;
	let mut meteors = vec![0; meteor_count];
	for i in 0..meteor_count {
		meteors[i] = leds_per_meteor * i;
	}

	move |ctrl| {
		let leds = ctrl.state_mut_flat();

		for counter in meteors.iter_mut() {
			if *counter > NUM_LEDS + meteor_size as usize * 2 {
				*counter = 0;
				// for i in 0..NUM_LEDS {
				// 	leds[i] = RGB::default();
				// }
			}
			// fade brightness all LEDs one step
			for j in 0..NUM_LEDS {
				fade_to_black_col(
					&mut leds[j],
					rand.gen_range(0, meteor_trail_decay),
					rand.gen_range(0, meteor_trail_decay),
					rand.gen_range(0, meteor_trail_decay),
				)
			}

			// draw meteor
			for j in 0..meteor_size as usize {
				if (*counter - j < NUM_LEDS) && (*counter + 1 - j != 0) {
					leds[*counter - j] = RGB::from(
						HSV::new(((offset as usize + j + *counter) % 256) as u8, 255, 255).to_rgb(),
					)
					.into();
				}
			}

			*counter += 1;
		}
		offset += 2;
		ctrl.write_state();
		sleep_ms(speed_delay);
	}
}

fn fade(val: u8, fade_value: u8) -> u8 {
	let val = val as f32;
	if val <= 10.0 {
		0
	} else {
		(val - (val * fade_value as f32 / 256.0)) as u8
	}
}

fn fade_to_black_col(led: &mut [u8; 3], fade_value_r: u8, fade_value_g: u8, fade_value_b: u8) {
	led[0] = fade(led[0], fade_value_r);
	led[1] = fade(led[1], fade_value_g);
	led[2] = fade(led[2], fade_value_b);
}

#[allow(unused)]
pub fn police() -> impl FnMut(&mut Controller) {
	let blue = RGB::new(0, 0, 255).into();
	move |ctrl| {
		set_all_delay(ctrl, blue, true, 150);
		set_all_delay(ctrl, blue, false, 47);
		set_all_delay(ctrl, blue, true, 16);
		set_all_delay(ctrl, blue, false, 24);
		set_all_delay(ctrl, blue, true, 16);
		set_all_delay(ctrl, blue, false, 24);
		set_all_delay(ctrl, blue, true, 16);
		set_all_delay(ctrl, blue, false, 186);
	}
}

struct Explosion {
	strip: usize,
	pos:   i32,
	speed: f32,
	width: f32,
	col:   HSV,
}

#[allow(unused)]
pub fn explosions(
	explosion_interval: u64,
	start_speed: f32,
	speed_falloff: f32,
	darken_factor: f32,
) -> impl FnMut(&mut Controller) {
	let mut rand = thread_rng();

	let mut explosions = VecDeque::new();

	let mut last = Instant::now();
	let mut duration = Duration::from_millis(explosion_interval);

	let mut counter = 0.0;

	let hue_min = 150;
	let hue_max = 200;

	move |ctrl| {
		let mut state = ctrl.views_mut();
		counter = (counter + 0.01) % 255.0;

		let now = Instant::now();
		if now - last > duration {
			let strip = rand.gen_range(0, state.len());
			let pos = rand.gen_range(0, state[strip].len() as i32);

			explosions.push_back(Explosion {
				strip,
				pos,
				speed: start_speed,
				width: 0.0,
				col: HSV::new(
					((rand.gen_range(0, hue_max - hue_min) + hue_min) % 255) as u8, //((rand.gen_range(0, 50) + counter as u16) % 255) as u8,
					255,
					255,
				),
			});
			last = now;
		}

		for strip in state.iter_mut() {
			for led in strip.iter_mut() {
				*led = darken_rgb(*led, darken_factor);
			}
		}

		let mut pop_count = 0;
		for explosion in explosions.iter_mut() {
			let mut strip = &mut state[explosion.strip];

			let start_width = explosion.width;
			explosion.width += explosion.speed;
			let end_width = explosion.width;

			explosion.speed *= speed_falloff;
			if explosion.speed <= 0.05 {
				pop_count += 1;
			}

			let lower_end = explosion.pos as f32 - end_width;
			let upper_end = explosion.pos as f32 + end_width;

			let len = strip.len();

			let lower_idx = (lower_end.floor() as usize).max(0);
			let upper_idx = (upper_end.ceil() as usize).min(len);

			if upper_idx > 0 && lower_idx <= len {
				let slice = &mut strip.range(lower_idx..upper_idx);

				let col = explosion.col.into();

				// anti-aliasing™
				let lower_factor = 1.0 - lower_end % 1.0;
				let upper_factor = upper_end % 1.0;

				for i in 0..slice.len() {
					let cur = slice[i];
					slice[i] = if i == 0 {
						blend_rgb(cur, col, lower_factor)
					} else if i == slice.len() - 1 {
						blend_rgb(cur, col, upper_factor)
					} else {
						explosion.col.into()
					};
				}
			}
		}
		for i in 0..pop_count {
			explosions.pop_front();
		}

		ctrl.write_state();
		// sleep_ms(100);
	}
}

#[allow(unused)]
pub fn rainbow() -> impl FnMut(&mut Controller) {
	let mut sin: f32 = 0.0;
	let mut hue: f32 = 0.0;

	move |ctrl| {
		let leds = ctrl.state_mut_flat();

		sin = sin + 0.25;
		hue = hue + 0.5;

		for i in 0..NUM_LEDS {
			let progress: f32 = ((sin + NUM_LEDS as f32 - i as f32 - 1.0) % 64.0) as f32 / 64.0
				* 2.0 * core::f32::consts::PI;
			let val = ((progress.sin() + 1.0) * 0.5 * 254.0) as u8 + 1;
			// let val = perlin.perlin(i as f32 / 10.0, progress / 20.0);
			// if i == 0 {
			//   print(val);
			// }

			let hue = ((i as f32 + hue) % 255.0) as u8;

			leds[i] = HSV::new(hue, 255, val).into();
		}

		ctrl.write_state();
	}
}

fn set_all_delay(ctrl: &mut Controller, colour: [u8; 3], on: bool, delay_ms: u64) {
	set_all(ctrl, if on { colour } else { [0, 0, 0] });
	sleep_ms(delay_ms);
}

fn set_all(ctrl: &mut Controller, colour: [u8; 3]) {
	let data = ctrl.state_mut_flat();
	for i in 0..data.len() {
		data[i] = colour;
	}
	ctrl.write_state();
}

#[allow(unused)]
pub fn flash_rainbow() -> impl FnMut(&mut Controller) {
	let mut counter: f32 = 0.0;
	let delay_on = 10;
	let delay_off = 200;

	move |ctrl| {
		counter = counter + 71.0;

		set_all_delay(
			ctrl,
			HSV::new((counter % 256.0) as u8, 255, 255).into(),
			true,
			delay_on,
		);
		set_all_delay(ctrl, [0, 0, 0], false, delay_off);
	}
}

#[allow(unused)]
pub fn random() -> impl FnMut(&mut Controller) {
	let mut rand = thread_rng();
	use crate::noise::simplex3d;

	let mut counter = 0.0;

	move |ctrl| {
		let state = ctrl.state_mut();

		counter += 0.03;

		for strip in 0..state.len() {
			// for i in 0..state[strip].len() / 8 {
			// 	let col = HSV::new(rand.gen(), 255, 255).into();
			//
			// 	let slice = &mut state[strip][i * 8..i * 8 + 8];
			// 	for i in slice {
			// 		*i = col;
			// 	}
			// }
			// for i in 0..state[strip].len() {
			// 	let col = HSV::new(rand.gen(), 255, 255).into();
			//
			// 	state[strip][i] = col;
			// 	// for i in slice {
			// 	// 	*i = col;
			// 	// }
			// }
			for i in 0..state[strip].len() {
				// let num = rand.gen_range(0, 55);
				// let num = rand.gen_range(0.0, 1.0);
				let num = 0.0;
				let num = (simplex3d(i as f32 / 20.0, num as f32, counter) * 255.0) as u8;
				let col = [num, num, num];

				state[strip][i] = col;
				// for i in slice {
				// 	*i = col;
				// }
			}
		}

		ctrl.write_state();
		// sleep_ms(1000);
	}
}

pub struct Timer {
	last:           Instant,
	moving:         [u128; 10],
	moving_min_max: [u128; 240],
}

pub struct Stats {
	dt:  f32,
	avg: f32,
	min: f32,
	max: f32,
}

impl Timer {
	pub fn new() -> Self {
		Timer {
			last:           Instant::now(),
			moving:         [0; 10],
			moving_min_max: [0; 240],
		}
	}

	pub fn tick(&mut self) -> Stats {
		let current = Instant::now();
		let diff = current - self.last;
		self.last = current;

		self.moving.rotate_right(1);
		self.moving[0] = diff.as_micros();

		self.moving_min_max.rotate_right(1);
		self.moving_min_max[0] = diff.as_micros();

		let mut avg = 0;
		for i in self.moving.iter() {
			avg += i;
		}
		avg /= self.moving.len() as u128;

		Stats {
			dt:  diff.as_micros() as f32 / 1000.0,
			avg: avg as f32 / 1000.0,
			min: self
				.moving_min_max
				.iter()
				.fold(u128::MAX, |min, cur| min.min(*cur)) as f32
				/ 1000.0,
			max: self
				.moving_min_max
				.iter()
				.fold(u128::MIN, |max, cur| max.max(*cur)) as f32
				/ 1000.0,
		}
	}
}

#[allow(unused)]
pub fn snake() -> impl FnMut(&mut Controller) {
	let mut sin: f32 = 0.0;
	let mut hue: f32 = 0.0;

	let hue_min = 150;
	let hue_max = 200;

	move |ctrl| {
		let state = ctrl.state_mut();

		sin = sin + 0.25;
		hue = hue + 0.5;

		for i in 0..NUM_LEDS {
			let progress: f32 = ((sin + NUM_LEDS as f32 - i as f32 - 1.0) % 64.0) as f32 / 32.0
				* 2.0 * core::f32::consts::PI;

			let val_top = ((progress.sin() + 1.0) * 0.5 * 254.0) as u8 + 1;
			let val_bottom = 255 - val_top;
			// let val_top = 255;
			// let val_bottom = 255;

			let hue =
				((hue_min + ((i as f32 + hue) % (hue_max - hue_min) as f32) as u16) % 255) as u8;

			state[0][state[0].len() - i - 1] = HSV::new(hue, 255, val_top).into();
			state[1][i] = HSV::new(hue, 255, val_bottom).into();
			state[2][state[2].len() - i - 1] = HSV::new(hue, 255, val_bottom).into();
		}
		ctrl.write_state();
	}
}

#[allow(unused)]
pub fn moving_lights(
	frequency: u64,
	impulse_len: usize,
	pulse_delay_ms: u64,
) -> impl FnMut(&mut Controller) {
	let frequency_ms = 1000 / frequency;

	let mut anim = MovingLightStripsAnimation::new(NUM_LEDS, impulse_len);

	let mut next_light_time = Instant::now();

	move |ctrl| {
		let now = Instant::now();
		if now >= next_light_time {
			anim.add_next_light_impulse();
			next_light_time = now.add(Duration::from_millis(pulse_delay_ms))
		}
		anim.shift_all_pixels();

		ctrl.write(anim.as_slice().iter());

		sleep_ms(frequency_ms);
	}
}

pub struct MovingLightStripsAnimation {
	rgb_data:    Vec<[u8; 3]>,
	impulse_len: usize,
}

impl MovingLightStripsAnimation {
	pub fn new(led_count: usize, impulse_len: usize) -> Self {
		MovingLightStripsAnimation {
			rgb_data: vec![[0; 3]; led_count + impulse_len],
			impulse_len,
		}
	}

	pub fn as_slice(&self) -> &[[u8; 3]] {
		&self.rgb_data[self.impulse_len..]
	}
}

impl MovingLightStripsAnimation {
	/// Shifts all pixel to the next position. Beginning is filled
	/// with black (0, 0, 0).
	fn shift_all_pixels(&mut self) {
		let upper_border = self.rgb_data.len();
		for i in 0..upper_border {
			// loop backwards
			let i = upper_border - 1 - i;

			if i == 0 {
				self.rgb_data[i] = [0; 3];
			} else {
				self.rgb_data.swap(i, i - 1);
			}
		}
	}

	fn add_next_light_impulse(&mut self) {
		// let (r, g, b) = get_random_pixel_val();

		let i = rand::random::<u8>();
		let rgb = HSV::new(i, 255, 255).into();

		for i in 0..self.impulse_len {
			let factor = 1.0 - ((i as f32 / (self.impulse_len as f32 / 2.0)) - 1.0).abs();
			self.rgb_data[i] = darken_rgb(rgb, factor);
		}

		// self.rgb_data[00] = darken_rgb(r, g, b, 0.1);
		// self.rgb_data[01] = darken_rgb(r, g, b, 0.2);
		// self.rgb_data[02] = darken_rgb(r, g, b, 0.4);
		// self.rgb_data[03] = darken_rgb(r, g, b, 0.6);
		// self.rgb_data[04] = darken_rgb(r, g, b, 0.7);
		// self.rgb_data[05] = darken_rgb(r, g, b, 0.8);
		// self.rgb_data[06] = darken_rgb(r, g, b, 0.9);
		// self.rgb_data[07] = [r, g, b];
		// self.rgb_data[08] = darken_rgb(r, g, b, 0.9);
		// self.rgb_data[09] = darken_rgb(r, g, b, 0.8);
		// self.rgb_data[10] = darken_rgb(r, g, b, 0.7);
		// self.rgb_data[11] = darken_rgb(r, g, b, 0.6);
		// self.rgb_data[12] = darken_rgb(r, g, b, 0.4);
		// self.rgb_data[13] = darken_rgb(r, g, b, 0.2);
		// self.rgb_data[14] = darken_rgb(r, g, b, 0.1);
	}
}

pub fn darken_rgb(rgb: [u8; 3], factor: f32) -> [u8; 3] {
	[
		((rgb[0] as f32) * factor) as u8,
		((rgb[1] as f32) * factor) as u8,
		((rgb[2] as f32) * factor) as u8,
	]
}

pub fn blend_rgb(from: [u8; 3], to: [u8; 3], factor: f32) -> [u8; 3] {
	let mut iter = (0..3).map(|i| lerp(from[i] as f32, to[i] as f32, factor) as u8);
	[
		iter.next().unwrap(),
		iter.next().unwrap(),
		iter.next().unwrap(),
	]
}

pub fn lerp(from: f32, to: f32, factor: f32) -> f32 {
	from + factor * (to - from)
}
