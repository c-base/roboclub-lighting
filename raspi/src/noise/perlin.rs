use rand::prelude::*;

use super::util::{fast_floor, lerp};

const GRAD_X: [f32; 12] = [
	1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0,
];
const GRAD_Y: [f32; 12] = [
	1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, -1.0, 1.0, -1.0,
];

pub struct Perlin {
	m_perm:   [u8; 512],
	m_perm12: [u8; 512],
}

impl Perlin {
	pub fn new(_seed: i32) -> Self {
		// let seed = seed.to_ne_bytes();
		// let seed = [
		// 	seed[0], seed[1], seed[2], seed[3], seed[0], seed[1], seed[2], seed[3], seed[0],
		// 	seed[1], seed[2], seed[3], seed[0], seed[1], seed[2], seed[3],
		// ];
		let mut rng = thread_rng();

		let mut m_perm = [u8::default(); 512];
		let mut m_perm12 = [u8::default(); 512];

		for i in 0..256 {
			m_perm[i] = i as u8;
		}

		for j in 0..256 {
			let rng = rng.gen::<usize>() % (256 - j);
			let k = rng + j;
			let l = m_perm[j] as usize;
			m_perm[j + 256] = m_perm[k];
			m_perm[j] = m_perm[k];
			m_perm[k] = l as u8;
			m_perm12[j + 256] = m_perm[j] % 12;
			m_perm12[j] = m_perm[j] % 12;
		}

		Perlin { m_perm, m_perm12 }
	}

	fn index_2d_12(&self, offset: u8, x: i32, y: i32) -> u8 {
		return self.m_perm12
			[(x & 0xff) as usize + self.m_perm[(y as usize & 0xff) + offset as usize] as usize];
	}

	fn grad_coord_2d(&self, offset: u8, x: i32, y: i32, xd: f32, yd: f32) -> f32 {
		let lut_pos = self.index_2d_12(offset, x, y) as usize;

		return xd * GRAD_X[lut_pos] + yd * GRAD_Y[lut_pos];
	}

	pub fn perlin(&self, x: f32, y: f32) -> f32 {
		let offset = 0u8;

		let x0 = fast_floor(x);
		let y0 = fast_floor(y);
		let x1 = x0 + 1;
		let y1 = y0 + 1;

		let xs: f32;
		let ys: f32;
		xs = x - x0 as f32;
		ys = y - y0 as f32;

		let xd0 = x - x0 as f32;
		let yd0 = y - y0 as f32;
		let xd1 = xd0 - 1.0;
		let yd1 = yd0 - 1.0;

		let xf0 = lerp(
			self.grad_coord_2d(offset, x0, y0, xd0, yd0),
			self.grad_coord_2d(offset, x1, y0, xd1, yd0),
			xs,
		);
		let xf1 = lerp(
			self.grad_coord_2d(offset, x0, y1, xd0, yd1),
			self.grad_coord_2d(offset, x1, y1, xd1, yd1),
			xs,
		);

		return lerp(xf0, xf1, ys);
	}
}
