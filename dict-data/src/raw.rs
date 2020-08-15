//! Helpers for dealing with the raw database files.

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

	pub fn from_bytes(bytes: &[u8]) -> RawUint32 {
		let (ptr, len) = (bytes.as_ptr(), bytes.len());
		if len != 4 {
			panic!("RawUint32 from_bytes must with invalid size");
		}
		unsafe { *(ptr as *const RawUint32) }
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

#[inline]
pub unsafe fn cast_vec<S, T: Sized>(data: Vec<S>) -> Vec<T> {
	let src_size = std::mem::size_of::<S>();
	let dst_size = std::mem::size_of::<T>();
	let (ptr, len, cap) = data.into_raw_parts();

	let src_len = len * src_size;
	let src_cap = cap * src_size;
	assert!(src_len % dst_size == 0);
	assert!(src_cap % dst_size == 0);

	let new_len = src_len / dst_size;
	let new_cap = src_cap / dst_size;
	Vec::from_raw_parts(ptr as *mut T, new_len, new_cap)
}
