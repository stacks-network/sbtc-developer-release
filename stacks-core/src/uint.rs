use std::{
	cmp::Ordering,
	fmt, io,
	mem::transmute,
	ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Shl, Shr, Sub},
};

use serde::{Deserialize, Serialize};

use crate::{
	codec::Codec,
	crypto::{
		sha256::{DoubleSha256Hasher, SHA256_LENGTH},
		Hashing,
	},
	StacksError, StacksResult,
};

/// A structure that represents large integers and provides basic arithmetic.
/// It accepts a const generic `N` which controls the number of u64s used to
/// represent the number.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(try_from = "Vec<u64>")]
#[serde(into = "Vec<u64>")]
#[repr(C)]
pub struct Uint<const N: usize>([u64; N]);

impl<const N: usize> Uint<N> {
	/// The maximum value that can be represented by this type
	pub const MAX: Self = Self([0xffffffffffffffff; N]);
	/// The minimum value that can be represented by this type
	pub const MIN: Self = Self([0; N]);

	/// Build a Uint from a u64 array
	pub fn from_u64_array(data: [u64; N]) -> Self {
		Self(data)
	}

	/// Conversion to u32
	pub fn low_u32(&self) -> u32 {
		self.0[0] as u32
	}

	/// Conversion to u64
	pub fn low_u64(&self) -> u64 {
		self.0[0]
	}

	/// Return the least number of bits needed to represent the number
	pub fn bits(&self) -> usize {
		for i in 1..N {
			if self.0[N - i] > 0 {
				return (0x40 * (N - i + 1))
					- self.0[N - i].leading_zeros() as usize;
			}
		}

		0x40 - self.0[0].leading_zeros() as usize
	}

	/// Multiply by a u32
	pub fn mul_u32(self, other: u32) -> Self {
		let mut carry = [0u64; N];
		let mut ret = [0u64; N];

		for i in 0..N {
			let not_last_word = i < N - 1;
			let upper = other as u64 * (self.0[i] >> 32);
			let lower = other as u64 * (self.0[i] & 0xFFFFFFFF);

			if not_last_word {
				carry[i + 1] += upper >> 32;
			}

			let (sum, overflow) = lower.overflowing_add(upper << 32);
			ret[i] = sum;

			if overflow && not_last_word {
				carry[i + 1] += 1;
			}
		}

		Self(ret) + Self(carry)
	}

	/// To litte-endian byte array
	pub fn to_le_bytes(&self) -> Vec<u8> {
		let mut buffer = vec![0; N * 8];

		self.0
			.iter()
			.flat_map(|part| part.to_le_bytes())
			.enumerate()
			.for_each(|(i, byte)| buffer[i] = byte);

		buffer
	}

	/// To big-endian byte array
	pub fn to_be_bytes(&self) -> Vec<u8> {
		let mut ret = vec![0; N * 8];

		for i in 0..N {
			let word_end = N * 8 - (i * 8);
			let word_start = word_end - 8;

			ret[word_start..word_end].copy_from_slice(&self.0[i].to_be_bytes());
		}

		ret
	}

	/// Build from a little-endian hex string (padding expected)
	pub fn from_le_bytes(bytes: impl AsRef<[u8]>) -> StacksResult<Self> {
		let bytes = bytes.as_ref();

		if bytes.len() % 8 != 0 {
			return Err(StacksError::InvalidUintBytes(bytes.len()));
		}

		if bytes.len() / 8 != N {
			return Err(StacksError::InvalidUintBytes(bytes.len()));
		}

		let mut ret = [0u64; N];
		for i in 0..(bytes.len() / 8) {
			let mut next_bytes = [0u8; 8];
			next_bytes.copy_from_slice(&bytes[8 * i..(8 * (i + 1))]);

			let next = u64::from_le_bytes(next_bytes);

			ret[i] = next;
		}

		Ok(Self(ret))
	}

	/// Build from a big-endian hex string (padding expected)
	pub fn from_be_bytes(bytes: impl AsRef<[u8]>) -> StacksResult<Self> {
		let bytes = bytes.as_ref();

		if bytes.len() % 8 != 0 {
			return Err(StacksError::InvalidUintBytes(bytes.len()));
		}

		if bytes.len() / 8 != N {
			return Err(StacksError::InvalidUintBytes(bytes.len()));
		}

		let mut ret = [0u64; N];
		for i in 0..(bytes.len() / 8) {
			let mut next_bytes = [0u8; 8];
			next_bytes.copy_from_slice(&bytes[8 * i..(8 * (i + 1))]);

			let next = u64::from_be_bytes(next_bytes);

			ret[(bytes.len() / 8) - 1 - i] = next;
		}

		Ok(Self(ret))
	}

	/// Convert to a little-endian hex string
	pub fn to_le_hex(&self) -> String {
		hex::encode(self.to_le_bytes())
	}

	/// Convert to a big-endian hex string
	pub fn to_be_hex(&self) -> String {
		hex::encode(self.to_be_bytes())
	}

	/// Build from a little-endian hex string
	pub fn from_le_hex(data: impl AsRef<str>) -> StacksResult<Self> {
		Self::from_le_bytes(hex::decode(data.as_ref())?)
	}

	/// Build from a big-endian hex string
	pub fn from_be_hex(data: impl AsRef<str>) -> StacksResult<Self> {
		Self::from_be_bytes(hex::decode(data.as_ref())?)
	}

	/// Wrapping add by one operation
	pub fn increment(&mut self) {
		let &mut Uint(ref mut arr) = self;

		for item in arr.iter_mut().take(N) {
			*item = item.wrapping_add(1);

			if *item != 0 {
				break;
			}
		}
	}

	/// Create a new Uint from the provided Uint
	pub fn from_uint<const M: usize>(source: impl AsRef<Uint<M>>) -> Self {
		assert!(M < N, "Cannot convert larger Uint to smaller");

		let source = source.as_ref();
		let mut dest = [0u64; N];

		dest[..M].copy_from_slice(&source.0[..M]);

		Uint(dest)
	}

	/// Create a new Uint from the provided Uint, truncating if necessary
	pub fn from_uint_lossy<const M: usize>(
		source: impl AsRef<Uint<M>>,
	) -> Self {
		let source = source.as_ref();
		let mut dest = [0u64; N];
		let bytes_shared = M.min(N);

		dest[..bytes_shared].copy_from_slice(&source.0[..bytes_shared]);

		Uint(dest)
	}

	/// Convert to a smaller Uint
	pub fn to_uint<const M: usize>(&self) -> Uint<M> {
		assert!(M >= N, "Cannot convert larger Uint to smaller");

		let mut dest = [0u64; M];

		dest[..M].copy_from_slice(&self.0[..M]);

		Uint(dest)
	}

	/// Convert to a smaller Uint, truncating if necessary
	pub fn to_uint_lossy<const M: usize>(&self) -> Uint<M> {
		let mut dest = [0u64; M];
		let bytes_shared = M.min(N);

		dest[..bytes_shared].copy_from_slice(&self.0[..bytes_shared]);

		Uint(dest)
	}

	fn one() -> Self {
		let mut ret = [0; N];
		ret[0] = 1;

		Uint(ret)
	}
}

impl<const N: usize> Add<Uint<N>> for Uint<N> {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		let Self(ref me) = self;
		let Self(ref you) = other;

		let mut ret = [0u64; N];
		let mut carry = [0u64; N];
		let mut b_carry = false;

		for i in 0..N {
			ret[i] = me[i].wrapping_add(you[i]);
			if i < N - 1 && ret[i] < me[i] {
				carry[i + 1] = 1;
				b_carry = true;
			}
		}

		if b_carry {
			Self(ret) + Self(carry)
		} else {
			Self(ret)
		}
	}
}

impl<const N: usize> Sub<Uint<N>> for Uint<N> {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		self + !other + Self::one()
	}
}

impl<const N: usize> Mul<Uint<N>> for Uint<N> {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		let mut me = Self::MIN;

		for i in 0..(2 * N) {
			let to_mul = (other >> (32 * i)).low_u32();
			me = me + (self.mul_u32(to_mul) << (32 * i));
		}
		me
	}
}

impl<const N: usize> Div<Uint<N>> for Uint<N> {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		let mut sub_copy = self;
		let mut shift_copy = other;
		let mut ret = [0u64; N];

		let my_bits = self.bits();
		let your_bits = other.bits();

		// Check for division by 0
		assert!(your_bits != 0);

		// Early return in case we are dividing by a larger number than us
		if my_bits < your_bits {
			return Self(ret);
		}

		// Bitwise long division
		let mut shift = my_bits - your_bits;
		shift_copy = shift_copy << shift;

		loop {
			if sub_copy >= shift_copy {
				ret[shift / 64] |= 1 << (shift % 64);
				sub_copy = sub_copy - shift_copy;
			}
			shift_copy = shift_copy >> 1;

			if shift == 0 {
				break;
			}

			shift -= 1;
		}

		Self(ret)
	}
}

impl<const N: usize> Ord for Uint<N> {
	fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
		// manually implement comparison to get little-endian ordering
		// (we need this for our numeric types; non-numeric ones shouldn't
		// be ordered anyway except to put them in BTrees or whatever, and
		// they don't care how we order as long as we're consisistent).
		for i in 0..N {
			if self.0[N - 1 - i] < other.0[N - 1 - i] {
				return Ordering::Less;
			}

			if self.0[N - 1 - i] > other.0[N - 1 - i] {
				return Ordering::Greater;
			}
		}

		Ordering::Equal
	}
}

impl<const N: usize> PartialOrd for Uint<N> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl<const N: usize> Default for Uint<N> {
	fn default() -> Self {
		Self::MIN
	}
}

impl<const N: usize> BitAnd<Uint<N>> for Uint<N> {
	type Output = Uint<N>;

	fn bitand(self, other: Uint<N>) -> Self {
		let Uint(ref arr1) = self;
		let Uint(ref arr2) = other;

		let mut ret = [0u64; N];
		for i in 0..N {
			ret[i] = arr1[i] & arr2[i];
		}

		Uint(ret)
	}
}

impl<const N: usize> BitXor<Uint<N>> for Uint<N> {
	type Output = Uint<N>;

	fn bitxor(self, other: Uint<N>) -> Self {
		let Uint(ref arr1) = self;
		let Uint(ref arr2) = other;

		let mut ret = [0u64; N];
		for i in 0..N {
			ret[i] = arr1[i] ^ arr2[i];
		}

		Uint(ret)
	}
}

impl<const N: usize> BitOr<Uint<N>> for Uint<N> {
	type Output = Uint<N>;

	fn bitor(self, other: Uint<N>) -> Self {
		let Uint(ref arr1) = self;
		let Uint(ref arr2) = other;

		let mut ret = [0u64; N];
		for i in 0..N {
			ret[i] = arr1[i] | arr2[i];
		}

		Uint(ret)
	}
}

impl<const N: usize> Not for Uint<N> {
	type Output = Uint<N>;

	fn not(self) -> Self {
		let Uint(ref arr) = self;

		let mut ret = [0u64; N];
		for i in 0..N {
			ret[i] = !arr[i];
		}

		Uint(ret)
	}
}

impl<const N: usize> Shl<usize> for Uint<N> {
	type Output = Uint<N>;

	fn shl(self, shift: usize) -> Self {
		let Uint(ref original) = self;
		let word_shift = shift / 64;
		let bit_shift = shift % 64;

		let mut ret = [0u64; N];
		for i in 0..N {
			// Shift
			if bit_shift < 64 && i + word_shift < N {
				ret[i + word_shift] += original[i] << bit_shift;
			}

			// Carry
			if bit_shift > 0 && i + word_shift + 1 < N {
				ret[i + word_shift + 1] += original[i] >> (64 - bit_shift);
			}
		}

		Uint(ret)
	}
}

impl<const N: usize> Shr<usize> for Uint<N> {
	type Output = Uint<N>;

	fn shr(self, shift: usize) -> Self {
		let Uint(ref original) = self;
		let word_shift = shift / 64;
		let bit_shift = shift % 64;

		let mut ret = [0u64; N];
		for i in word_shift..N {
			// Shift
			ret[i - word_shift] += original[i] >> bit_shift;

			// Carry
			if bit_shift > 0 && i < N - 1 {
				ret[i - word_shift] += original[i + 1] << (64 - bit_shift);
			}
		}

		Uint(ret)
	}
}

impl<const N: usize> AsRef<Uint<N>> for Uint<N> {
	fn as_ref(&self) -> &Uint<N> {
		self
	}
}

impl<const N: usize> fmt::Debug for Uint<N> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let Uint(data) = self;

		write!(f, "0x")?;

		for ch in data.iter().rev() {
			write!(f, "{:016x}", ch)?;
		}

		Ok(())
	}
}

impl<const N: usize> fmt::Display for Uint<N> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		<dyn fmt::Debug>::fmt(self, f)
	}
}

impl<const N: usize> From<u8> for Uint<N> {
	fn from(value: u8) -> Self {
		(value as u64).into()
	}
}

impl<const N: usize> From<u16> for Uint<N> {
	fn from(value: u16) -> Self {
		(value as u64).into()
	}
}

impl<const N: usize> From<u32> for Uint<N> {
	fn from(value: u32) -> Self {
		(value as u64).into()
	}
}

impl<const N: usize> From<u64> for Uint<N> {
	fn from(value: u64) -> Self {
		let mut ret = [0; N];
		ret[0] = value;

		Self(ret)
	}
}

impl<const N: usize> From<u128> for Uint<N> {
	fn from(value: u128) -> Self {
		let mut ret = [0u64; N];

		ret[0] = value as u64;
		ret[1] = (value >> 64) as u64;

		Self(ret)
	}
}

impl<const N: usize> Codec for Uint<N> {
	fn codec_serialize<W: io::Write>(&self, dest: &mut W) -> io::Result<()> {
		dest.write_all(&self.to_be_bytes())
	}

	fn codec_deserialize<R: io::Read>(data: &mut R) -> io::Result<Self>
	where
		Self: Sized,
	{
		let mut buffer = vec![0u8; N * 8];
		data.read_exact(&mut buffer)?;

		Self::from_be_bytes(buffer).map_err(|_| {
			io::Error::new(
				io::ErrorKind::InvalidData,
				"Could not deserialize Uint",
			)
		})
	}
}

impl From<DoubleSha256Hasher> for Uint256 {
	fn from(value: DoubleSha256Hasher) -> Self {
		let buffer: [u8; SHA256_LENGTH] = value.as_bytes().try_into().unwrap();

		let mut ret: [u64; 4] = unsafe { transmute(buffer) };
		for x in ret.iter_mut() {
			*x = x.to_le();
		}

		Uint256::from_u64_array(ret)
	}
}

// From conversion is fallible for this type
#[allow(clippy::from_over_into)]
impl<const N: usize> Into<Vec<u64>> for Uint<N> {
	fn into(self) -> Vec<u64> {
		self.0.to_vec()
	}
}

impl<const N: usize> TryFrom<Vec<u64>> for Uint<N> {
	type Error = StacksError;

	fn try_from(value: Vec<u64>) -> Result<Self, Self::Error> {
		Ok(Self(value.as_slice().try_into()?))
	}
}

/// A 256-bit unsigned integer
pub type Uint256 = Uint<4>;
/// A 512-bit unsigned integer
pub type Uint512 = Uint<8>;

#[cfg(test)]
mod tests {
	use super::*;

	impl<const N: usize> Uint<N> {
		fn bit(&self, index: usize) -> bool {
			let Uint(arr) = self;

			arr[index / 64] & (1 << (index % 64)) != 0
		}

		fn bit_slice(&self, start: usize, end: usize) -> Self {
			(*self >> start).mask(end - start)
		}

		fn mask(&self, n: usize) -> Self {
			let Uint(arr) = self;

			let mut ret = [0; N];
			for i in 0..N {
				if n >= 0x40 * (i + 1) {
					ret[i] = arr[i];
				} else {
					ret[i] = arr[i] & ((1 << (n - 0x40 * i)) - 1);
					break;
				}
			}

			Uint(ret)
		}
	}

	#[test]
	fn should_convert_from_u32() {
		assert_eq!(Uint256::from(1337u32), Uint256::from(1337u64));
	}

	#[test]
	pub fn uint256_bits_test() {
		assert_eq!(Uint256::from(255u64).bits(), 8);
		assert_eq!(Uint256::from(256u64).bits(), 9);
		assert_eq!(Uint256::from(300u64).bits(), 9);
		assert_eq!(Uint256::from(60000u64).bits(), 16);
		assert_eq!(Uint256::from(70000u64).bits(), 17);

		// Try to read the following lines out loud quickly
		let mut shl = Uint256::from(70000u64);
		shl = shl << 100;
		assert_eq!(shl.bits(), 117);
		shl = shl << 100;
		assert_eq!(shl.bits(), 217);
		shl = shl << 100;
		assert_eq!(shl.bits(), 0);

		// Bit set check
		assert!(!Uint256::from(10u64).bit(0));
		assert!(Uint256::from(10u64).bit(1));
		assert!(!Uint256::from(10u64).bit(2));
		assert!(Uint256::from(10u64).bit(3));
		assert!(!Uint256::from(10u64).bit(4));
	}

	#[test]
	pub fn uint256_display_test() {
		assert_eq!(
            format!("{}", Uint256::from(0xDEADBEEFu64)),
            "0x00000000000000000000000000000000000000000000000000000000deadbeef"
        );
		assert_eq!(
            format!("{}", Uint256::from(u64::MAX)),
            "0x000000000000000000000000000000000000000000000000ffffffffffffffff"
        );

		let max_val = Uint256::from_u64_array([
			0xFFFFFFFFFFFFFFFF,
			0xFFFFFFFFFFFFFFFF,
			0xFFFFFFFFFFFFFFFF,
			0xFFFFFFFFFFFFFFFF,
		]);

		assert_eq!(
            format!("{}", max_val),
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );
	}

	#[test]
	pub fn uint256_comp_test() {
		let small = Uint256::from_u64_array([10u64, 0, 0, 0]);
		let big = Uint256::from_u64_array([
			0x8C8C3EE70C644118u64,
			0x0209E7378231E632,
			0,
			0,
		]);
		let bigger = Uint256::from_u64_array([
			0x9C8C3EE70C644118u64,
			0x0209E7378231E632,
			0,
			0,
		]);
		let biggest = Uint256::from_u64_array([
			0x5C8C3EE70C644118u64,
			0x0209E7378231E632,
			0,
			1,
		]);

		dbg!(&bigger, &biggest);

		assert!(small < big);
		assert!(big < bigger);
		assert!(bigger < biggest);
		assert!(bigger <= biggest);
		assert!(biggest <= biggest);
		assert!(bigger >= big);
		assert!(bigger >= small);
		assert!(small <= small);
	}

	#[test]
	pub fn uint256_arithmetic_test() {
		let init = Uint256::from(0xDEADBEEFDEADBEEFu64);
		let copy = init;

		let add = init + copy;
		assert_eq!(
			add,
			Uint256::from_u64_array([0xBD5B7DDFBD5B7DDEu64, 1, 0, 0])
		);
		// Bitshifts
		let shl = add << 88;
		assert_eq!(
			shl,
			Uint256::from_u64_array([0u64, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0])
		);
		let shr = shl >> 40;
		assert_eq!(
			shr,
			Uint256::from_u64_array([
				0x7DDE000000000000u64,
				0x0001BD5B7DDFBD5B,
				0,
				0
			])
		);
		// Increment
		let mut incr = shr;
		incr.increment();
		assert_eq!(
			incr,
			Uint256::from_u64_array([
				0x7DDE000000000001u64,
				0x0001BD5B7DDFBD5B,
				0,
				0
			])
		);
		// Subtraction
		let sub = incr - init;
		assert_eq!(
			sub,
			Uint256::from_u64_array([
				0x9F30411021524112u64,
				0x0001BD5B7DDFBD5A,
				0,
				0
			])
		);
		// Multiplication
		let mult = sub.mul_u32(300);
		assert_eq!(
			mult,
			Uint256::from_u64_array([
				0x8C8C3EE70C644118u64,
				0x0209E7378231E632,
				0,
				0
			])
		);
		// Division
		assert_eq!(
			Uint256::from(105u64) / Uint256::from(5u64),
			Uint256::from(21u64)
		);
		let div = mult / Uint256::from(300u64);

		dbg!(mult, Uint256::from(300u64), div);

		assert_eq!(
			div,
			Uint256::from_u64_array([
				0x9F30411021524112u64,
				0x0001BD5B7DDFBD5A,
				0,
				0
			])
		);
		// TODO: bit inversion
	}

	#[test]
	pub fn mul_u32_test() {
		let u64_val = Uint256::from(0xDEADBEEFDEADBEEFu64);

		let u96_res = u64_val.mul_u32(0xFFFFFFFF);
		let u128_res = u96_res.mul_u32(0xFFFFFFFF);
		let u160_res = u128_res.mul_u32(0xFFFFFFFF);
		let u192_res = u160_res.mul_u32(0xFFFFFFFF);
		let u224_res = u192_res.mul_u32(0xFFFFFFFF);
		let u256_res = u224_res.mul_u32(0xFFFFFFFF);

		assert_eq!(
			u96_res,
			Uint256::from_u64_array([0xffffffff21524111u64, 0xDEADBEEE, 0, 0])
		);
		assert_eq!(
			u128_res,
			Uint256::from_u64_array([
				0x21524111DEADBEEFu64,
				0xDEADBEEE21524110,
				0,
				0
			])
		);
		assert_eq!(
			u160_res,
			Uint256::from_u64_array([
				0xBD5B7DDD21524111u64,
				0x42A4822200000001,
				0xDEADBEED,
				0
			])
		);
		assert_eq!(
			u192_res,
			Uint256::from_u64_array([
				0x63F6C333DEADBEEFu64,
				0xBD5B7DDFBD5B7DDB,
				0xDEADBEEC63F6C334,
				0
			])
		);
		assert_eq!(
			u224_res,
			Uint256::from_u64_array([
				0x7AB6FBBB21524111u64,
				0xFFFFFFFBA69B4558,
				0x854904485964BAAA,
				0xDEADBEEB
			])
		);
		assert_eq!(
			u256_res,
			Uint256::from_u64_array([
				0xA69B4555DEADBEEFu64,
				0xA69B455CD41BB662,
				0xD41BB662A69B4550,
				0xDEADBEEAA69B455C
			])
		);
	}

	#[test]
	pub fn multiplication_test() {
		let u64_val = Uint256::from(0xDEADBEEFDEADBEEFu64);

		let u128_res = u64_val * u64_val;

		assert_eq!(
			u128_res,
			Uint256::from_u64_array([
				0x048D1354216DA321u64,
				0xC1B1CD13A4D13D46,
				0,
				0
			])
		);

		let u256_res = u128_res * u128_res;

		assert_eq!(
			u256_res,
			Uint256::from_u64_array([
				0xF4E166AAD40D0A41u64,
				0xF5CF7F3618C2C886u64,
				0x4AFCFF6F0375C608u64,
				0x928D92B4D7F5DF33u64
			])
		);
	}

	#[test]
	pub fn uint256_bitslice_test() {
		let init = Uint256::from(0xDEADBEEFDEADBEEFu64);
		let add = init + (init << 64);
		assert_eq!(add.bit_slice(64, 128), init);
		assert_eq!(add.mask(64), init);
	}

	#[test]
	pub fn uint256_extreme_bitshift_test() {
		// Shifting a u64 by 64 bits gives an undefined value, so make sure that
		// we're doing the Right Thing here
		let init = Uint256::from(0xDEADBEEFDEADBEEFu64);

		assert_eq!(
			init << 64,
			Uint256::from_u64_array([0, 0xDEADBEEFDEADBEEF, 0, 0])
		);
		let add = (init << 64) + init;
		assert_eq!(
			add,
			Uint256::from_u64_array([
				0xDEADBEEFDEADBEEF,
				0xDEADBEEFDEADBEEF,
				0,
				0
			])
		);
		assert_eq!(
			add >> 0,
			Uint256::from_u64_array([
				0xDEADBEEFDEADBEEF,
				0xDEADBEEFDEADBEEF,
				0,
				0
			])
		);
		assert_eq!(
			add << 0,
			Uint256::from_u64_array([
				0xDEADBEEFDEADBEEF,
				0xDEADBEEFDEADBEEF,
				0,
				0
			])
		);
		assert_eq!(
			add >> 64,
			Uint256::from_u64_array([0xDEADBEEFDEADBEEF, 0, 0, 0])
		);
		assert_eq!(
			add << 64,
			Uint256::from_u64_array([
				0,
				0xDEADBEEFDEADBEEF,
				0xDEADBEEFDEADBEEF,
				0
			])
		);
	}

	#[test]
	pub fn hex_codec() {
		let init = Uint256::from(0xDEADBEEFDEADBEEFu64) << 64
			| Uint256::from(0x0102030405060708u64);

		// little-endian representation
		let hex_init =
			"0807060504030201efbeaddeefbeadde00000000000000000000000000000000";
		assert_eq!(
			Uint256::from_le_bytes(hex::decode(hex_init).unwrap()).unwrap(),
			init
		);
		assert_eq!(hex::encode(init.to_le_bytes()), hex_init);
		assert_eq!(Uint256::from_le_bytes(init.to_le_bytes()).unwrap(), init);

		// big-endian representation
		let hex_init =
			"00000000000000000000000000000000deadbeefdeadbeef0102030405060708";
		assert_eq!(
			Uint256::from_be_bytes(hex::decode(hex_init).unwrap()).unwrap(),
			init
		);
		assert_eq!(hex::encode(init.to_be_bytes()), hex_init);
		assert_eq!(Uint256::from_be_bytes(init.to_be_bytes()).unwrap(), init);
	}

	#[test]
	pub fn uint_increment_test() {
		let mut value = Uint256::from_u64_array([0xffffffffffffffff, 0, 0, 0]);
		value.increment();
		assert_eq!(value, Uint256::from_u64_array([0, 1, 0, 0]));

		value = Uint256::from_u64_array([
			0xffffffffffffffff,
			0xffffffffffffffff,
			0,
			0,
		]);
		value.increment();
		assert_eq!(value, Uint256::from_u64_array([0, 0, 1, 0]));

		value = Uint256::from_u64_array([
			0xffffffffffffffff,
			0xffffffffffffffff,
			0xffffffffffffffff,
			0,
		]);
		value.increment();
		assert_eq!(value, Uint256::from_u64_array([0, 0, 0, 1]));

		value = Uint256::from_u64_array([
			0xffffffffffffffff,
			0xffffffffffffffff,
			0xffffffffffffffff,
			0xffffffffffffffff,
		]);
		value.increment();
		assert_eq!(value, Uint256::from_u64_array([0, 0, 0, 0]));
	}
}
