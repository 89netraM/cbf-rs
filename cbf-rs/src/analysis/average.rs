use std::ops::{AddAssign, Div};

use num::{BigInt, One, Zero};

pub struct Average<P: BigNum> {
	sum: <P as BigNum>::BigType,
	count: <P as BigNum>::BigType,
}

impl<P: BigNum> Default for Average<P> {
	fn default() -> Self {
		Self { sum: <P as BigNum>::BigType::zero(), count: <P as BigNum>::BigType::zero() }
	}
}

impl<P: BigNum> Average<P> {
	pub fn add(&mut self, value: P) {
		self.sum += value;
		self.count += <P as BigNum>::BigType::one();
	}

	pub fn average(&self) -> P {
		<P as BigNum>::div(&self.sum, &self.count)
	}
}

pub trait BigNum: Sized {
	type BigType: AddAssign<Self> + AddAssign<Self::BigType> + TryInto<Self> + Div<Output = Self::BigType> + Zero + One;

	fn div(a: &Self::BigType, b: &Self::BigType) -> Self;
}

macro_rules! integer_big_num {
	($($type:ty),*) => {
		$(impl BigNum for $type {
			type BigType = BigInt;

			fn div(a: &BigInt, b: &BigInt) -> $type {
				a.checked_div(b).and_then(|r| r.try_into().ok()).unwrap()
			}
		})*
	};
}

integer_big_num!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);

macro_rules! float_big_num {
	($($type:ty),*) => {
		$(impl BigNum for $type {
			type BigType = $type;

			fn div(a: &$type, b: &$type) -> $type {
				a / b
			}
		})*
	};
}

float_big_num!(f32, f64);

#[cfg(test)]
mod tests {
	use super::Average;

	#[test]
	fn simple_u8() {
		let mut average: Average<u8> = Average::default();
		average.add(1);
		average.add(2);
		average.add(3);
		average.add(4);
		average.add(5);
		assert_eq!(average.average(), 3);
	}

	#[test]
	fn simple_usize() {
		let mut average: Average<usize> = Average::default();
		average.add(1);
		average.add(2);
		average.add(3);
		average.add(4);
		average.add(5);
		assert_eq!(average.average(), 3);
	}

	#[test]
	fn simple_f32() {
		let mut average: Average<f32> = Average::default();
		average.add(1.0);
		average.add(2.0);
		average.add(3.0);
		average.add(4.0);
		average.add(5.0);
		assert_eq!(average.average(), 3.0);
	}

	#[test]
	fn simple_f64() {
		let mut average: Average<f64> = Average::default();
		average.add(1.0);
		average.add(2.0);
		average.add(3.0);
		average.add(4.0);
		average.add(5.0);
		assert_eq!(average.average(), 3.0);
	}

	#[test]
	fn negative_isize() {
		let mut average: Average<isize> = Average::default();
		average.add(1);
		average.add(2);
		average.add(3);
		average.add(4);
		average.add(-5);
		assert_eq!(average.average(), 1);
	}

	#[test]
	fn idempotency() {
		let mut average: Average<usize> = Average::default();
		average.add(1);
		average.add(2);
		average.add(3);
		average.add(4);
		average.add(5);
		assert_eq!(average.average(), 3);
		assert_eq!(average.average(), 3);
	}

	#[test]
	fn big_u8() {
		let mut average: Average<u8> = Average::default();
		average.add(121);
		average.add(122);
		average.add(123); // Sum goes over u8::MAX
		average.add(124);
		average.add(125);
		assert_eq!(average.average(), 123);
	}

	#[test]
	fn small_i32() {
		let mut average: Average<i32> = Average::default();
		average.add(-1073741821);
		average.add(-1073741822);
		average.add(-1073741823); // Sum goes under i32::MIN
		average.add(-1073741824);
		average.add(-1073741825);
		assert_eq!(average.average(), -1073741823);
	}
}
