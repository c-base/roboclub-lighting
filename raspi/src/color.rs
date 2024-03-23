use std::ops::{Deref, DerefMut};

use palette::{encoding, IntoColor};
use serde::{Deserialize, Serialize};

type PRgb = palette::LinSrgb;
type PRgba = palette::LinSrgba;
type PHsv = palette::Hsv<encoding::Linear<encoding::Srgb>>;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rgb(palette::LinSrgb);
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rgba(palette::LinSrgba);
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Hsv(palette::Hsv<encoding::Linear<encoding::Srgb>>);

impl Rgb {
	pub fn new(red: f32, green: f32, blue: f32) -> Self {
		Rgb(PRgb::new(red, green, blue))
	}
}

impl From<PRgb> for Rgb {
	fn from(c: PRgb) -> Self {
		Rgb(c)
	}
}

impl From<Hsv> for Rgb {
	fn from(c: Hsv) -> Self {
		Rgb(c.0.into_color())
	}
}

impl Deref for Rgb {
	type Target = PRgb;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Rgb {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Rgba {
	pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
		Rgba(PRgba::new(red, green, blue, alpha))
	}
}

impl From<PRgba> for Rgba {
	fn from(c: PRgba) -> Self {
		Rgba(c)
	}
}

impl From<PHsv> for Rgba {
	fn from(c: PHsv) -> Self {
		Rgba(c.into_color())
	}
}

impl From<Hsv> for Rgba {
	fn from(c: Hsv) -> Self {
		Rgba(c.0.into_color())
	}
}

impl From<Rgb> for Rgba {
	fn from(c: Rgb) -> Self {
		Rgba(c.0.into_color())
	}
}

impl Deref for Rgba {
	type Target = PRgba;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Rgba {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Hsv {
	pub fn new(hue: f32, saturation: f32, value: f32) -> Self {
		Hsv(PHsv::new(hue, saturation, value))
	}
}

impl From<PHsv> for Hsv {
	fn from(c: PHsv) -> Self {
		Hsv(c)
	}
}

impl Deref for Hsv {
	type Target = PHsv;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Hsv {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

// #[derive(Copy, Clone, Debug, Default)]
// pub struct RGB {
// 	pub r: u8,
// 	pub g: u8,
// 	pub b: u8,
// }
//
// impl RGB {
// 	pub fn new(r: u8, g: u8, b: u8) -> Self {
// 		RGB { r, g, b }
// 	}
// }
//
// impl Into<[u8; 3]> for RGB {
// 	fn into(self) -> [u8; 3] {
// 		[self.r, self.g, self.b]
// 	}
// }
//
// impl From<(u8, u8, u8)> for RGB {
// 	fn from(from: (u8, u8, u8)) -> Self {
// 		RGB::new(from.0, from.1, from.2)
// 	}
// }
//
// impl From<&(u8, u8, u8)> for RGB {
// 	fn from(from: &(u8, u8, u8)) -> Self {
// 		RGB::new(from.0, from.1, from.2)
// 	}
// }
//
// impl From<[u8; 3]> for RGB {
// 	fn from(from: [u8; 3]) -> Self {
// 		RGB::new(from[0], from[1], from[2])
// 	}
// }
//
// impl From<&[u8; 3]> for RGB {
// 	fn from(from: &[u8; 3]) -> Self {
// 		RGB::new(from[0], from[1], from[2])
// 	}
// }
//
// #[derive(Copy, Clone, Debug, Default)]
// pub struct HSV {
// 	pub hue:        u8,
// 	pub saturation: u8,
// 	pub value:      u8,
// }
//
// impl HSV {
// 	pub fn new(hue: u8, saturation: u8, value: u8) -> Self {
// 		HSV {
// 			hue,
// 			saturation,
// 			value,
// 		}
// 	}
//
// 	pub fn to_rgb(self) -> (u8, u8, u8) {
// 		hsv2rgb_rainbow(self)
// 	}
// }
//
// impl Into<[u8; 3]> for HSV {
// 	fn into(self) -> [u8; 3] {
// 		let rgb: RGB = self.into();
// 		rgb.into()
// 	}
// }
//
// impl From<HSV> for RGB {
// 	fn from(hsv: HSV) -> Self {
// 		hsv.to_rgb().into()
// 	}
// }
//
// // from fastled
// fn scale8(i: u8, scale: u8) -> u8 {
// 	(((i as u16) * (1 + scale as u16)) >> 8) as u8
// }
//
// // from fastled
// fn scale8_video(i: u8, scale: u8) -> u8 {
// 	(((i as usize * scale as usize) >> 8) + if i > 0 && scale > 0 { 1 } else { 0 }) as u8
// }
//
// // from fastled
// fn hsv2rgb_rainbow(hsv: HSV) -> (u8, u8, u8) {
// 	const K255: u8 = 255;
// 	const K171: u8 = 171;
// 	const K170: u8 = 170;
// 	const K85: u8 = 85;
//
// 	// Yellow has a higher inherent brightness than
// 	// any other color; 'pure' yellow is perceived to
// 	// be 93% as bright as white.  In order to make
// 	// yellow appear the correct relative brightness,
// 	// it has to be rendered brighter than all other
// 	// colors.
// 	// Level Y1 is a moderate boost, the default.
// 	// Level Y2 is a strong boost.
// 	const Y1: bool = true;
// 	const Y2: bool = false;
//
// 	// G2: Whether to divide all greens by two.
// 	// Depends GREATLY on your particular LEDs
// 	const G2: bool = false;
//
// 	// GSCALE: what to scale green down by.
// 	// Depends GREATLY on your particular LEDs
// 	const GSCALE: u8 = 0;
//
// 	let hue: u8 = hsv.hue;
// 	let sat: u8 = hsv.saturation;
// 	let mut val: u8 = hsv.value;
//
// 	let offset: u8 = hue & 0x1F; // 0..31
//
// 	// offset8 = offset * 8
// 	let mut offset8: u8 = offset;
// 	{
// 		offset8 <<= 3;
// 	}
//
// 	let third: u8 = scale8(offset8, (256u16 / 3) as u8); // max = 85
//
// 	let mut r = 0;
// 	let mut g = 0;
// 	let mut b = 0;
//
// 	if hue & 0x80 == 0 {
// 		// 0XX
// 		if hue & 0x40 == 0 {
// 			// 00X
// 			//section 0-1
// 			if hue & 0x20 == 0 {
// 				// 000
// 				//case 0: // R -> O
// 				r = K255 - third;
// 				g = third;
// 				b = 0;
// 			} else {
// 				// 001
// 				//case 1: // O -> Y
// 				if Y1 {
// 					r = K171;
// 					g = K85 + third;
// 					b = 0;
// 				}
// 				if Y2 {
// 					r = K170 + third;
// 					//uint8_t twothirds = (third << 1);
// 					let twothirds = scale8(offset8, ((256 * 2) / 3) as u8); // max=170
// 					g = K85 + twothirds;
// 					b = 0;
// 				}
// 			}
// 		} else {
// 			//01X
// 			// section 2-3
// 			if hue & 0x20 == 0 {
// 				// 010
// 				//case 2: // Y -> G
// 				if Y1 {
// 					//uint8_t twothirds = (third << 1);
// 					let twothirds = scale8(offset8, ((256 * 2) / 3) as u8); // max=170
// 					r = K171 - twothirds;
// 					g = K170 + third;
// 					b = 0;
// 				}
// 				if Y2 {
// 					r = K255 - offset8;
// 					g = K255;
// 					b = 0;
// 				}
// 			} else {
// 				// 011
// 				// case 3: // G -> A
// 				r = 0;
// 				g = K255 - third;
// 				b = third;
// 			}
// 		}
// 	} else {
// 		// section 4-7
// 		// 1XX
// 		if hue & 0x40 == 0 {
// 			// 10X
// 			if hue & 0x20 == 0 {
// 				// 100
// 				//case 4: // A -> B
// 				r = 0;
// 				//uint8_t twothirds = (third << 1);
// 				let twothirds = scale8(offset8, ((256 * 2) / 3) as u8); // max=170
// 				g = K171 - twothirds; //K170?
// 				b = K85 + twothirds;
// 			} else {
// 				// 101
// 				//case 5: // B -> P
// 				r = third;
// 				g = 0;
//
// 				b = K255 - third;
// 			}
// 		} else {
// 			if hue & 0x20 == 0 {
// 				// 110
// 				//case 6: // P -- K
// 				r = K85 + third;
// 				g = 0;
//
// 				b = K171 - third;
// 			} else {
// 				// 111
// 				//case 7: // K -> R
// 				r = K170 + third;
// 				g = 0;
//
// 				b = K85 - third;
// 			}
// 		}
// 	}
//
// 	// This is one of the good places to scale the green down,
// 	// although the client can scale green down as well.
// 	if G2 {
// 		g = g >> 1;
// 	}
// 	if GSCALE > 0 {
// 		g = scale8_video(g, GSCALE);
// 	}
//
// 	// Scale down colors if we're desaturated at all
// 	// and add the brightness_floor to r, g, and b.
// 	if sat != 255 {
// 		if sat == 0 {
// 			r = 255;
// 			b = 255;
// 			g = 255;
// 		} else {
// 			//nscale8x3_video( r, g, b, sat);
// 			if r > 0 {
// 				r = scale8(r, sat)
// 			}
// 			if g > 0 {
// 				g = scale8(g, sat)
// 			}
// 			if b > 0 {
// 				b = scale8(b, sat)
// 			}
//
// 			let mut desat = 255 - sat;
// 			desat = scale8(desat, desat);
//
// 			let brightness_floor = desat;
// 			r += brightness_floor;
// 			g += brightness_floor;
// 			b += brightness_floor;
// 		}
// 	}
//
// 	// Now scale everything down if we're at value < 255.
// 	if val != 255 {
// 		val = scale8_video(val, val);
// 		if val == 0 {
// 			r = 0;
// 			g = 0;
// 			b = 0;
// 		} else {
// 			if r > 0 {
// 				r = scale8(r, val)
// 			}
// 			if g > 0 {
// 				g = scale8(g, val)
// 			}
// 			if b > 0 {
// 				b = scale8(b, val)
// 			}
// 		}
// 	}
//
// 	(r, g, b)
// }
