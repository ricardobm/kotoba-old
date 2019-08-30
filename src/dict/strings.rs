use std::borrow::Cow;
use std::collections::HashMap;

/// Provides a table for interned strings.
pub struct StringTable {
	str_table: Vec<String>,
	str_index: HashMap<InternalString, StringIndex>,
}

/// Index for an interned string in a `StringTable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StringIndex(usize);

impl StringTable {
	/// Create a new empty `StringTable` instance.
	pub fn new() -> StringTable {
		let mut result = StringTable {
			str_table: Vec::new(),
			str_index: HashMap::new(),
		};
		result.intern(String::new());
		result
	}

	/// Number of unique entries in the table.
	///
	/// Note that the empty string is not counted.
	#[allow(dead_code)]
	pub fn len(&self) -> usize {
		self.str_table.len()
	}

	/// Iterator for all entries in the table.
	///
	/// Note that the empty string is not returned.
	#[allow(dead_code)]
	pub fn entries(&self) -> impl Iterator<Item = &String> {
		self.str_table.iter()
	}

	/// Retrieves the string for a `StringIndex`.
	///
	/// Note that the empty string is registered by default and always zero.
	pub fn get<'a>(&'a self, index: StringIndex) -> &'a str {
		let StringIndex(index) = index;
		if index == 0 {
			return "";
		} else {
			self.str_table[index - 1].as_str()
		}
	}

	/// Interns a string into a `StringIndex`.
	pub fn intern<'a, S: Into<Cow<'a, str>>>(&mut self, value: S) -> StringIndex {
		let cow = value.into();
		if cow.len() == 0 {
			return StringIndex(0);
		}

		let key = InternalString::from(&cow);
		if let Some(&index) = self.str_index.get(&key) {
			index
		} else {
			self.push(cow)
		}
	}

	/// Appends a string to the table directly, using the current position as
	/// its index. This should be used only to deserialize a string table.
	pub fn push<'a, S: Into<String>>(&mut self, value: S) -> StringIndex {
		self.str_table.push(value.into());
		let key = InternalString::from(self.str_table.last().unwrap().as_str());
		let index = StringIndex(self.str_table.len());
		self.str_index.insert(key, index);
		index
	}
}

/// Used only internally as keys for the HashMap used by `StringTable`.
#[derive(Copy, Clone)]
struct InternalString {
	ptr: *const str,
}

impl InternalString {
	fn from<S>(value: S) -> InternalString
	where
		S: AsRef<str>,
	{
		let ptr = value.as_ref() as *const str;
		InternalString { ptr }
	}
}

impl std::cmp::PartialEq for InternalString {
	fn eq(&self, other: &Self) -> bool {
		if self.ptr == other.ptr {
			true
		} else {
			unsafe { (*self.ptr) == (*other.ptr) }
		}
	}
}

impl std::cmp::Eq for InternalString {}

impl std::hash::Hash for InternalString {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		unsafe { (*self.ptr).hash(state) }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_string_table() {
		let mut tb = StringTable::new();

		assert_eq!("", tb.get(StringIndex(0)));
		assert_eq!(StringIndex(0), tb.intern(""));
		assert_eq!(StringIndex(0), tb.intern(String::new()));

		let idx1 = tb.intern("string1");
		let idx2 = tb.intern("string2");
		let idx3 = tb.intern("string3");

		assert_eq!(idx1, tb.intern("string1"));
		assert_eq!(idx2, tb.intern("string2"));
		assert_eq!(idx3, tb.intern("string3"));

		assert_eq!("string1", tb.get(idx1));
		assert_eq!("string2", tb.get(idx2));
		assert_eq!("string3", tb.get(idx3));

		assert_eq!(3, tb.len());
	}
}
