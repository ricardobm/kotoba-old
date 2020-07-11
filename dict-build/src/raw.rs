/// Unsigned 32 bit integer in LE (little endian) byte order.
///
/// Both Raw integer types are used for platform independent persistence of the
/// database:
///
/// - During database write the integer is converted to LE byte order (a no-op
///   on most common platforms).
///
/// - For loading the database is memory mapped, so conversion happens only when
///   values are used (again a no-op on most platforms).
///
///   - For the rare BE platform case we keep the integer in LE format and pay
///     the conversion price for every use, instead of trying to map to the
///     native integer format on load (which would be more efficient but would
///     also mean a long delay when loading the database).
#[derive(Copy, Clone)]
pub struct RawUint32(u32);

impl RawUint32 {
	pub fn bytes(self) -> [u8; 4] {
		self.0.to_le_bytes()
	}
}

impl std::convert::From<u32> for RawUint32 {
	#[inline]
	fn from(item: u32) -> Self {
		Self(item.to_le())
	}
}

impl std::convert::From<usize> for RawUint32 {
	#[inline]
	fn from(item: usize) -> Self {
		Self((item as u32).to_le())
	}
}

impl std::convert::Into<u32> for RawUint32 {
	#[inline]
	fn into(self) -> u32 {
		u32::from_le(self.0)
	}
}

impl std::convert::Into<usize> for RawUint32 {
	#[inline]
	fn into(self) -> usize {
		let index: u32 = self.into();
		index as usize
	}
}
