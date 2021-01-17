pub(crate) fn fast_floor(f: f32) -> i32 {
	if f >= 0.0 {
		f as i32
	} else {
		f as i32 - 1
	}
}

pub(crate) fn lerp(a: f32, b: f32, t: f32) -> f32 {
	a + t * (b - a)
}
