use crate::image::Image;

pub fn nearest_neighbour<P: Copy>(image: &Image<P>, angle: f64, radius: f64) -> Option<P> {
	let (x, y) = polar_to_cartesian(image.width as f64, angle, radius);
	image.get_pixel((x.round() as isize, y.round() as isize)).copied()
}

fn polar_to_cartesian(width: f64, angle: f64, radius: f64) -> (f64, f64) {
	let radius = radius * width / 2.0;
	(radius * angle.cos(), radius * angle.sin())
}

#[cfg(test)]
mod tests {
	use super::polar_to_cartesian;

	use std::f64;

	#[test]
	fn polar_to_cartesian_first_quadrant() {
		polar_to_cartesian_quadrant_test(0.0, 1.0, 1.0);
	}

	#[test]
	fn polar_to_cartesian_second_quadrant() {
		polar_to_cartesian_quadrant_test(1.0, -1.0, 1.0);
	}

	#[test]
	fn polar_to_cartesian_third_quadrant() {
		polar_to_cartesian_quadrant_test(2.0, -1.0, -1.0);
	}

	#[test]
	fn polar_to_cartesian_fourth_quadrant() {
		polar_to_cartesian_quadrant_test(3.0, 1.0, -1.0);
	}

	fn polar_to_cartesian_quadrant_test(quadrant: f64, expected_x: f64, expected_y: f64) {
		let angle = f64::consts::FRAC_PI_4 + f64::consts::FRAC_PI_2 * quadrant;
		let radius = f64::consts::SQRT_2;

		let (x, y) = polar_to_cartesian(2.0, angle, radius);

		assert!(
			(expected_x - x).abs() <= f64::EPSILON,
			"expected {} and actual {} are not equal",
			expected_x,
			x
		);
		assert!(
			(expected_y - y).abs() <= f64::EPSILON,
			"expected {} and actual {} are not equal",
			expected_y,
			y
		);
	}
}
