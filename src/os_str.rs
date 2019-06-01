#[cfg(feature = "alloc")]
use crate::inner::inner_alloc::Buf;
use crate::inner::Slice;
#[allow(unused_imports)]
use crate::sys_common::{AsInner, FromInner, IntoInner};
#[cfg(feature = "alloc")]
use alloc::borrow::Cow;
#[cfg(feature = "alloc")]
use alloc::borrow::ToOwned;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::rc::Rc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::sync::Arc;
#[cfg(feature = "alloc")]
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::str;
#[allow(unused_imports)]
use core::{cmp, fmt, ops};

/// A type that can represent owned, mutable platform-native strings, but is
/// cheaply inter-convertible with Rust strings.
///
/// The need for this type arises from the fact that:
///
/// * On Unix systems, strings are often arbitrary sequences of non-zero
///   bytes, in many cases interpreted as UTF-8.
///
/// * On Windows, strings are often arbitrary sequences of non-zero 16-bit
///   values, interpreted as UTF-16 when it is valid to do so.
///
/// * In Rust, strings are always valid UTF-8, which may contain zeros.
///
/// `OsString` and [`OsStr`] bridge this gap by simultaneously representing Rust
/// and platform-native string values, and in particular allowing a Rust string
/// to be converted into an "OS" string with no cost if possible. A consequence
/// of this is that `OsString` instances are *not* `NUL` terminated; in order
/// to pass to e.g., Unix system call, you should create a [`CStr`].
///
/// `OsString` is to [`&OsStr`] as [`String`] is to [`&str`]: the former
/// in each pair are owned strings; the latter are borrowed
/// references.
///
/// Note, `OsString` and `OsStr` internally do not necessarily hold strings in
/// the form native to the platform; While on Unix, strings are stored as a
/// sequence of 8-bit values, on Windows, where strings are 16-bit value based
/// as just discussed, strings are also actually stored as a sequence of 8-bit
/// values, encoded in a less-strict variant of UTF-8. This is useful to
/// understand when handling capacity and length values.
///
/// # Creating an `OsString`
///
/// **From a Rust string**: `OsString` implements
/// [`From`]`<`[`String`]`>`, so you can use `my_string.from` to
/// create an `OsString` from a normal Rust string.
///
/// **From slices:** Just like you can start with an empty Rust
/// [`String`] and then [`push_str`][String.push_str] `&str`
/// sub-string slices into it, you can create an empty `OsString` with
/// the [`new`] method and then push string slices into it with the
/// [`push`] method.
///
/// # Extracting a borrowed reference to the whole OS string
///
/// You can use the [`as_os_str`] method to get an `&`[`OsStr`] from
/// an `OsString`; this is effectively a borrowed reference to the
/// whole string.
///
/// # Conversions
///
/// See the [module's toplevel documentation about conversions][conversions] for a discussion on
/// the traits which `OsString` implements for [conversions] from/to native representations.
///
/// [`&OsStr`]: OsStr
/// [`CStr`]: crate::CStr
/// [`&str`]: str
/// [String.push_str]: String::push_str
/// [`new`]: #method.new
/// [`push`]: #method.push
/// [`as_os_str`]: #method.as_os_str
/// [conversions]: index.html#conversions
#[cfg(feature = "alloc")]
#[derive(Clone)]
pub struct OsString {
	inner: Buf,
}

/// Borrowed reference to an OS string (see [`OsString`]).
///
/// This type represents a borrowed reference to a string in the operating system's preferred
/// representation.
///
/// `&OsStr` is to [`OsString`] as [`&str`] is to [`String`]: the former in each pair are borrowed
/// references; the latter are owned strings.
///
/// See the [module's toplevel documentation about conversions][conversions] for a discussion on
/// the traits which `OsStr` implements for [conversions] from/to native representations.
///
/// [`OsString`]: struct.OsString.html
/// [`&str`]: str
/// [conversions]: index.html#conversions
pub struct OsStr {
	inner: Slice,
}

#[cfg(feature = "alloc")]
impl OsString {
	/// Constructs a new empty `OsString`.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let os_string = OsString::new();
	/// ```
	pub fn new() -> OsString {
		OsString {
			inner: Buf::from_string(String::new()),
		}
	}

	/// Converts to an [`OsStr`] slice.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::{OsString, OsStr};
	///
	/// let os_string = OsString::from("foo");
	/// let os_str = OsStr::new("foo");
	/// assert_eq!(os_string.as_os_str(), os_str);
	/// ```
	pub fn as_os_str(&self) -> &OsStr {
		self
	}

	/// Converts the `OsString` into a [`String`] if it contains valid Unicode data.
	///
	/// On failure, ownership of the original `OsString` is returned.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let os_string = OsString::from("foo");
	/// let string = os_string.into_string();
	/// assert_eq!(string, Ok(String::from("foo")));
	/// ```
	pub fn into_string(self) -> Result<String, OsString> {
		self.inner
			.into_string()
			.map_err(|buf| OsString { inner: buf })
	}

	/// Extends the string with the given [`&OsStr`] slice.
	///
	/// [`&OsStr`]: OsStr
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut os_string = OsString::from("foo");
	/// os_string.push("bar");
	/// assert_eq!(&os_string, "foobar");
	/// ```
	pub fn push<T: AsRef<OsStr>>(&mut self, s: T) {
		self.inner.push_slice(&s.as_ref().inner)
	}

	/// Creates a new `OsString` with the given capacity.
	///
	/// The string will be able to hold exactly `capacity` length units of other
	/// OS strings without reallocating. If `capacity` is 0, the string will not
	/// allocate.
	///
	/// See main `OsString` documentation information about encoding.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut os_string = OsString::with_capacity(10);
	/// let capacity = os_string.capacity();
	///
	/// // This push is done without reallocating
	/// os_string.push("foo");
	///
	/// assert_eq!(capacity, os_string.capacity());
	/// ```
	pub fn with_capacity(capacity: usize) -> OsString {
		OsString {
			inner: Buf::with_capacity(capacity),
		}
	}

	/// Truncates the `OsString` to zero length.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut os_string = OsString::from("foo");
	/// assert_eq!(&os_string, "foo");
	///
	/// os_string.clear();
	/// assert_eq!(&os_string, "");
	/// ```
	pub fn clear(&mut self) {
		self.inner.clear()
	}

	/// Returns the capacity this `OsString` can hold without reallocating.
	///
	/// See `OsString` introduction for information about encoding.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut os_string = OsString::with_capacity(10);
	/// assert!(os_string.capacity() >= 10);
	/// ```
	pub fn capacity(&self) -> usize {
		self.inner.capacity()
	}

	/// Reserves capacity for at least `additional` more capacity to be inserted
	/// in the given `OsString`.
	///
	/// The collection may reserve more space to avoid frequent reallocations.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut s = OsString::new();
	/// s.reserve(10);
	/// assert!(s.capacity() >= 10);
	/// ```
	pub fn reserve(&mut self, additional: usize) {
		self.inner.reserve(additional)
	}

	/// Reserves the minimum capacity for exactly `additional` more capacity to
	/// be inserted in the given `OsString`. Does nothing if the capacity is
	/// already sufficient.
	///
	/// Note that the allocator may give the collection more space than it
	/// requests. Therefore, capacity can not be relied upon to be precisely
	/// minimal. Prefer reserve if future insertions are expected.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut s = OsString::new();
	/// s.reserve_exact(10);
	/// assert!(s.capacity() >= 10);
	/// ```
	pub fn reserve_exact(&mut self, additional: usize) {
		self.inner.reserve_exact(additional)
	}

	/// Shrinks the capacity of the `OsString` to match its length.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsString;
	///
	/// let mut s = OsString::from("foo");
	///
	/// s.reserve(100);
	/// assert!(s.capacity() >= 100);
	///
	/// s.shrink_to_fit();
	/// assert_eq!(3, s.capacity());
	/// ```
	pub fn shrink_to_fit(&mut self) {
		self.inner.shrink_to_fit()
	}

	/*/// Shrinks the capacity of the `OsString` with a lower bound.
	///
	///
	/// The capacity will remain at least as large as both the length
	/// and the supplied value.
	///
	/// Panics if the current capacity is smaller than the supplied
	/// minimum capacity.
	///
	/// # Examples
	///
	/// ```
	/// #![feature(shrink_to)]
	/// use std::ffi::OsString;
	///
	/// let mut s = OsString::from("foo");
	///
	/// s.reserve(100);
	/// assert!(s.capacity() >= 100);
	///
	/// s.shrink_to(10);
	/// assert!(s.capacity() >= 10);
	/// s.shrink_to(0);
	/// assert!(s.capacity() >= 3);
	/// ```
	#[inline]
	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.inner.shrink_to(min_capacity)
	}*/

	/// Converts this `OsString` into a boxed [`OsStr`].
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::{OsString, OsStr};
	///
	/// let s = OsString::from("hello");
	///
	/// let b: Box<OsStr> = s.into_boxed_os_str();
	/// ```
	pub fn into_boxed_os_str(self) -> Box<OsStr> {
		let rw = Box::into_raw(self.inner.into_box()) as *mut OsStr;
		unsafe { Box::from_raw(rw) }
	}
}

#[cfg(feature = "alloc")]
impl From<String> for OsString {
	/// Converts a [`String`] into a [`OsString`].
	///
	/// The conversion copies the data, and includes an allocation on the heap.
	///
	/// [`OsString`]: crate::OsString
	fn from(s: String) -> OsString {
		OsString {
			inner: Buf::from_string(s),
		}
	}
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + AsRef<OsStr>> From<&T> for OsString {
	fn from(s: &T) -> OsString {
		s.as_ref().to_os_string()
	}
}

#[cfg(feature = "alloc")]
impl ops::Index<ops::RangeFull> for OsString {
	type Output = OsStr;

	#[inline]
	fn index(&self, _index: ops::RangeFull) -> &OsStr {
		OsStr::from_inner(self.inner.as_slice())
	}
}

#[cfg(feature = "alloc")]
impl ops::Deref for OsString {
	type Target = OsStr;

	#[inline]
	fn deref(&self) -> &OsStr {
		&self[..]
	}
}

#[cfg(feature = "alloc")]
impl Default for OsString {
	/// Constructs an empty `OsString`.
	#[inline]
	fn default() -> OsString {
		OsString::new()
	}
}

#[cfg(feature = "alloc")]
impl fmt::Debug for OsString {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&**self, formatter)
	}
}

#[cfg(feature = "alloc")]
impl PartialEq for OsString {
	fn eq(&self, other: &OsString) -> bool {
		self == other
	}
}

#[cfg(feature = "alloc")]
impl PartialEq<str> for OsString {
	fn eq(&self, other: &str) -> bool {
		&**self == other
	}
}

#[cfg(feature = "alloc")]
impl PartialEq<OsString> for str {
	fn eq(&self, other: &OsString) -> bool {
		&**other == self
	}
}

#[cfg(feature = "alloc")]
impl PartialEq<&str> for OsString {
	fn eq(&self, other: &&str) -> bool {
		**self == **other
	}
}

#[cfg(feature = "alloc")]
impl<'a> PartialEq<OsString> for &'a str {
	fn eq(&self, other: &OsString) -> bool {
		**other == **self
	}
}

#[cfg(feature = "alloc")]
impl Eq for OsString {}

#[cfg(feature = "alloc")]
impl PartialOrd for OsString {
	#[inline]
	fn partial_cmp(&self, other: &OsString) -> Option<cmp::Ordering> {
		(&**self).partial_cmp(&**other)
	}
	#[inline]
	fn lt(&self, other: &OsString) -> bool {
		self < other
	}
	#[inline]
	fn le(&self, other: &OsString) -> bool {
		self <= other
	}
	#[inline]
	fn gt(&self, other: &OsString) -> bool {
		self > other
	}
	#[inline]
	fn ge(&self, other: &OsString) -> bool {
		self >= other
	}
}

#[cfg(feature = "alloc")]
impl PartialOrd<str> for OsString {
	#[inline]
	fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
		(&**self).partial_cmp(other)
	}
}

#[cfg(feature = "alloc")]
impl Ord for OsString {
	#[inline]
	fn cmp(&self, other: &OsString) -> cmp::Ordering {
		(&**self).cmp(&**other)
	}
}

#[cfg(feature = "alloc")]
impl Hash for OsString {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		(&**self).hash(state)
	}
}

impl OsStr {
	/// Coerces into an `OsStr` slice.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsStr;
	///
	/// let os_str = OsStr::new("foo");
	/// ```
	pub fn new<S: AsRef<OsStr> + ?Sized>(s: &S) -> &OsStr {
		s.as_ref()
	}

	fn from_inner(inner: &Slice) -> &OsStr {
		unsafe { &*(inner as *const Slice as *const OsStr) }
	}

	/// Yields a [`&str`] slice if the `OsStr` is valid Unicode.
	///
	/// This conversion may entail doing a check for UTF-8 validity.
	///
	/// [`&str`]: str
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsStr;
	///
	/// let os_str = OsStr::new("foo");
	/// assert_eq!(os_str.to_str(), Some("foo"));
	/// ```
	pub fn to_str(&self) -> Option<&str> {
		self.inner.to_str()
	}

	/// Converts an `OsStr` to a [`Cow`]`<`[`str`]`>`.
	///
	/// Any non-Unicode sequences are replaced with
	/// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].
	///
	/// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
	///
	/// # Examples
	///
	/// Calling `to_string_lossy` on an `OsStr` with invalid unicode:
	///
	/// ```
	/// // Note, due to differences in how Unix and Windows represent strings,
	/// // we are forced to complicate this example, setting up example `OsStr`s
	/// // with different source data and via different platform extensions.
	/// // Understand that in reality you could end up with such example invalid
	/// // sequences simply through collecting user command line arguments, for
	/// // example.
	///
	/// #[cfg(any(unix, target_os = "redox"))] {
	///     use std::ffi::OsStr;
	///     use std::os::unix::ffi::OsStrExt;
	///
	///     // Here, the values 0x66 and 0x6f correspond to 'f' and 'o'
	///     // respectively. The value 0x80 is a lone continuation byte, invalid
	///     // in a UTF-8 sequence.
	///     let source = [0x66, 0x6f, 0x80, 0x6f];
	///     let os_str = OsStr::from_bytes(&source[..]);
	///
	///     assert_eq!(os_str.to_string_lossy(), "fo�o");
	/// }
	/// #[cfg(windows)] {
	///     use std::ffi::OsString;
	///     use std::os::windows::prelude::*;
	///
	///     // Here the values 0x0066 and 0x006f correspond to 'f' and 'o'
	///     // respectively. The value 0xD800 is a lone surrogate half, invalid
	///     // in a UTF-16 sequence.
	///     let source = [0x0066, 0x006f, 0xD800, 0x006f];
	///     let os_string = OsString::from_wide(&source[..]);
	///     let os_str = os_string.as_os_str();
	///
	///     assert_eq!(os_str.to_string_lossy(), "fo�o");
	/// }
	/// ```
	#[cfg(feature = "alloc")]
	pub fn to_string_lossy(&self) -> Cow<'_, str> {
		self.inner.to_string_lossy()
	}

	/// Copies the slice into an owned [`OsString`].
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::{OsStr, OsString};
	///
	/// let os_str = OsStr::new("foo");
	/// let os_string = os_str.to_os_string();
	/// assert_eq!(os_string, OsString::from("foo"));
	/// ```
	#[cfg(feature = "alloc")]
	pub fn to_os_string(&self) -> OsString {
		OsString {
			inner: self.inner.to_owned(),
		}
	}

	/// Checks whether the `OsStr` is empty.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsStr;
	///
	/// let os_str = OsStr::new("");
	/// assert!(os_str.is_empty());
	///
	/// let os_str = OsStr::new("foo");
	/// assert!(!os_str.is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.inner.inner.is_empty()
	}

	/// Returns the length of this `OsStr`.
	///
	/// Note that this does **not** return the number of bytes in the string in
	/// OS string form.
	///
	/// The length returned is that of the underlying storage used by `OsStr`;
	/// As discussed in the [`OsString`] introduction, [`OsString`] and `OsStr`
	/// store strings in a form best suited for cheap inter-conversion between
	/// native-platform and Rust string forms, which may differ significantly
	/// from both of them, including in storage size and encoding.
	///
	/// This number is simply useful for passing to other methods, like
	/// [`OsString::with_capacity`] to avoid reallocations.
	///
	/// # Examples
	///
	/// ```
	/// use std::ffi::OsStr;
	///
	/// let os_str = OsStr::new("");
	/// assert_eq!(os_str.len(), 0);
	///
	/// let os_str = OsStr::new("foo");
	/// assert_eq!(os_str.len(), 3);
	/// ```
	pub fn len(&self) -> usize {
		self.inner.inner.len()
	}

	/// Converts a [`Box`]`<OsStr>` into an [`OsString`] without copying or allocating.
	#[cfg(feature = "alloc")]
	pub fn into_os_string(self: Box<OsStr>) -> OsString {
		let boxed = unsafe { Box::from_raw(Box::into_raw(self) as *mut Slice) };
		OsString {
			inner: Buf::from_box(boxed),
		}
	}

	/// Gets the underlying byte representation.
	///
	/// Note: it is *crucial* that this API is private, to avoid
	/// revealing the internal, platform-specific encodings.
	fn bytes(&self) -> &[u8] {
		unsafe { &*(&self.inner as *const _ as *const [u8]) }
	}

	pub fn display(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.inner, formatter)
	}
}

#[cfg(feature = "alloc")]
impl From<&OsStr> for Box<OsStr> {
	fn from(s: &OsStr) -> Box<OsStr> {
		let rw = Box::into_raw(s.inner.into_box()) as *mut OsStr;
		unsafe { Box::from_raw(rw) }
	}
}

#[cfg(feature = "alloc")]
impl From<Box<OsStr>> for OsString {
	/// Converts a `Box<OsStr>` into a `OsString` without copying or allocating.
	fn from(boxed: Box<OsStr>) -> OsString {
		boxed.into_os_string()
	}
}

#[cfg(feature = "alloc")]
impl From<OsString> for Box<OsStr> {
	/// Converts a [`OsString`] into a [`Box`]`<OsStr>` without copying or allocating.
	fn from(s: OsString) -> Box<OsStr> {
		s.into_boxed_os_str()
	}
}

#[cfg(feature = "alloc")]
impl Clone for Box<OsStr> {
	#[inline]
	fn clone(&self) -> Self {
		self.to_os_string().into_boxed_os_str()
	}
}

#[cfg(feature = "alloc")]
impl From<OsString> for Arc<OsStr> {
	/// Converts a [`OsString`] into a [`Arc`]`<OsStr>` without copying or allocating.
	#[inline]
	fn from(s: OsString) -> Arc<OsStr> {
		let arc = s.inner.into_arc();
		unsafe { Arc::from_raw(Arc::into_raw(arc) as *const OsStr) }
	}
}

#[cfg(feature = "alloc")]
impl From<&OsStr> for Arc<OsStr> {
	#[inline]
	fn from(s: &OsStr) -> Arc<OsStr> {
		let arc = s.inner.into_arc();
		unsafe { Arc::from_raw(Arc::into_raw(arc) as *const OsStr) }
	}
}

#[cfg(feature = "alloc")]
impl From<OsString> for Rc<OsStr> {
	/// Converts a [`OsString`] into a [`Rc`]`<OsStr>` without copying or allocating.
	#[inline]
	fn from(s: OsString) -> Rc<OsStr> {
		let rc = s.inner.into_rc();
		unsafe { Rc::from_raw(Rc::into_raw(rc) as *const OsStr) }
	}
}

#[cfg(feature = "alloc")]
impl From<&OsStr> for Rc<OsStr> {
	#[inline]
	fn from(s: &OsStr) -> Rc<OsStr> {
		let rc = s.inner.into_rc();
		unsafe { Rc::from_raw(Rc::into_raw(rc) as *const OsStr) }
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<OsString> for Cow<'a, OsStr> {
	#[inline]
	fn from(s: OsString) -> Cow<'a, OsStr> {
		Cow::Owned(s)
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a OsStr> for Cow<'a, OsStr> {
	#[inline]
	fn from(s: &'a OsStr) -> Cow<'a, OsStr> {
		Cow::Borrowed(s)
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<&'a OsString> for Cow<'a, OsStr> {
	#[inline]
	fn from(s: &'a OsString) -> Cow<'a, OsStr> {
		Cow::Borrowed(s.as_os_str())
	}
}

#[cfg(feature = "alloc")]
impl<'a> From<Cow<'a, OsStr>> for OsString {
	#[inline]
	fn from(s: Cow<'a, OsStr>) -> Self {
		s.into_owned()
	}
}

#[cfg(feature = "alloc")]
impl Default for Box<OsStr> {
	fn default() -> Box<OsStr> {
		let rw = Box::into_raw(Slice::empty_box()) as *mut OsStr;
		unsafe { Box::from_raw(rw) }
	}
}

impl Default for &OsStr {
	/// Creates an empty `OsStr`.
	#[inline]
	fn default() -> Self {
		OsStr::new("")
	}
}

impl PartialEq for OsStr {
	fn eq(&self, other: &OsStr) -> bool {
		self.bytes().eq(other.bytes())
	}
}

impl PartialEq<str> for OsStr {
	fn eq(&self, other: &str) -> bool {
		*self == *OsStr::new(other)
	}
}

impl PartialEq<OsStr> for str {
	fn eq(&self, other: &OsStr) -> bool {
		*other == *OsStr::new(self)
	}
}

impl Eq for OsStr {}

impl PartialOrd for OsStr {
	#[inline]
	fn partial_cmp(&self, other: &OsStr) -> Option<cmp::Ordering> {
		self.bytes().partial_cmp(other.bytes())
	}
	#[inline]
	fn lt(&self, other: &OsStr) -> bool {
		self.bytes().lt(other.bytes())
	}
	#[inline]
	fn le(&self, other: &OsStr) -> bool {
		self.bytes().le(other.bytes())
	}
	#[inline]
	fn gt(&self, other: &OsStr) -> bool {
		self.bytes().gt(other.bytes())
	}
	#[inline]
	fn ge(&self, other: &OsStr) -> bool {
		self.bytes().ge(other.bytes())
	}
}

impl PartialOrd<str> for OsStr {
	#[inline]
	fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
		self.partial_cmp(OsStr::new(other))
	}
}

// FIXME (#19470): cannot provide PartialOrd<OsStr> for str until we
// have more flexible coherence rules.

impl Ord for OsStr {
	#[inline]
	fn cmp(&self, other: &OsStr) -> cmp::Ordering {
		self.bytes().cmp(other.bytes())
	}
}

#[cfg(feature = "alloc")]
macro_rules! impl_cmp {
	($lhs:ty, $rhs: ty) => {
		impl<'a, 'b> PartialEq<$rhs> for $lhs {
			#[inline]
			fn eq(&self, other: &$rhs) -> bool {
				<OsStr as PartialEq>::eq(self, other)
			}
		}

		impl<'a, 'b> PartialEq<$lhs> for $rhs {
			#[inline]
			fn eq(&self, other: &$lhs) -> bool {
				<OsStr as PartialEq>::eq(self, other)
			}
		}

		impl<'a, 'b> PartialOrd<$rhs> for $lhs {
			#[inline]
			fn partial_cmp(&self, other: &$rhs) -> Option<cmp::Ordering> {
				<OsStr as PartialOrd>::partial_cmp(self, other)
			}
		}

		impl<'a, 'b> PartialOrd<$lhs> for $rhs {
			#[inline]
			fn partial_cmp(&self, other: &$lhs) -> Option<cmp::Ordering> {
				<OsStr as PartialOrd>::partial_cmp(self, other)
			}
		}
	};
}

#[cfg(feature = "alloc")]
impl_cmp!(OsString, OsStr);
#[cfg(feature = "alloc")]
impl_cmp!(OsString, &'a OsStr);
#[cfg(feature = "alloc")]
impl_cmp!(Cow<'a, OsStr>, OsStr);
#[cfg(feature = "alloc")]
impl_cmp!(Cow<'a, OsStr>, &'b OsStr);
#[cfg(feature = "alloc")]
impl_cmp!(Cow<'a, OsStr>, OsString);

impl Hash for OsStr {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.bytes().hash(state)
	}
}

impl fmt::Debug for OsStr {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self.inner, formatter)
	}
}

#[cfg(feature = "alloc")]
impl Borrow<OsStr> for OsString {
	fn borrow(&self) -> &OsStr {
		&self[..]
	}
}

#[cfg(feature = "alloc")]
impl ToOwned for OsStr {
	type Owned = OsString;
	fn to_owned(&self) -> OsString {
		self.to_os_string()
	}
	//	fn clone_into(&self, target: &mut OsString) {
	//		target.clear();
	//		target.push(self);
	//	}
}

impl AsRef<OsStr> for OsStr {
	fn as_ref(&self) -> &OsStr {
		self
	}
}

#[cfg(feature = "alloc")]
impl AsRef<OsStr> for OsString {
	fn as_ref(&self) -> &OsStr {
		self
	}
}

impl AsRef<OsStr> for str {
	fn as_ref(&self) -> &OsStr {
		OsStr::from_inner(Slice::from_str(self))
	}
}

#[cfg(feature = "alloc")]
impl AsRef<OsStr> for String {
	fn as_ref(&self) -> &OsStr {
		(&**self).as_ref()
	}
}

#[cfg(feature = "alloc")]
impl FromInner<Buf> for OsString {
	fn from_inner(buf: Buf) -> OsString {
		OsString { inner: buf }
	}
}

#[cfg(feature = "alloc")]
impl IntoInner<Buf> for OsString {
	fn into_inner(self) -> Buf {
		self.inner
	}
}

impl AsInner<Slice> for OsStr {
	#[inline]
	fn as_inner(&self) -> &Slice {
		&self.inner
	}
}

#[cfg(feature = "alloc")]
#[cfg(test)]
mod tests {
	use super::*;
	use crate::sys_common::{AsInner, IntoInner};

	use alloc::rc::Rc;
	use alloc::sync::Arc;

	#[test]
	fn test_os_string_with_capacity() {
		let os_string = OsString::with_capacity(0);
		assert_eq!(0, os_string.inner.into_inner().capacity());

		let os_string = OsString::with_capacity(10);
		assert_eq!(10, os_string.inner.into_inner().capacity());

		let mut os_string = OsString::with_capacity(0);
		os_string.push("abc");
		assert!(os_string.inner.into_inner().capacity() >= 3);
	}

	#[test]
	fn test_os_string_clear() {
		let mut os_string = OsString::from("abc");
		assert_eq!(3, os_string.inner.as_inner().len());

		os_string.clear();
		assert_eq!(&os_string, "");
		assert_eq!(0, os_string.inner.as_inner().len());
	}

	#[test]
	fn test_os_string_capacity() {
		let os_string = OsString::with_capacity(0);
		assert_eq!(0, os_string.capacity());

		let os_string = OsString::with_capacity(10);
		assert_eq!(10, os_string.capacity());

		let mut os_string = OsString::with_capacity(0);
		os_string.push("abc");
		assert!(os_string.capacity() >= 3);
	}

	#[test]
	fn test_os_string_reserve() {
		let mut os_string = OsString::new();
		assert_eq!(os_string.capacity(), 0);

		os_string.reserve(2);
		assert!(os_string.capacity() >= 2);

		for _ in 0..16 {
			os_string.push("a");
		}

		assert!(os_string.capacity() >= 16);
		os_string.reserve(16);
		assert!(os_string.capacity() >= 32);

		os_string.push("a");

		os_string.reserve(16);
		assert!(os_string.capacity() >= 33)
	}

	#[test]
	fn test_os_string_reserve_exact() {
		let mut os_string = OsString::new();
		assert_eq!(os_string.capacity(), 0);

		os_string.reserve_exact(2);
		assert!(os_string.capacity() >= 2);

		for _ in 0..16 {
			os_string.push("a");
		}

		assert!(os_string.capacity() >= 16);
		os_string.reserve_exact(16);
		assert!(os_string.capacity() >= 32);

		os_string.push("a");

		os_string.reserve_exact(16);
		assert!(os_string.capacity() >= 33)
	}

	#[test]
	fn test_os_string_default() {
		let os_string: OsString = Default::default();
		assert_eq!("", &os_string);
	}

	#[test]
	fn test_os_str_is_empty() {
		let mut os_string = OsString::new();
		assert!(os_string.is_empty());

		os_string.push("abc");
		assert!(!os_string.is_empty());

		os_string.clear();
		assert!(os_string.is_empty());
	}

	#[test]
	fn test_os_str_len() {
		let mut os_string = OsString::new();
		assert_eq!(0, os_string.len());

		os_string.push("abc");
		assert_eq!(3, os_string.len());

		os_string.clear();
		assert_eq!(0, os_string.len());
	}

	#[test]
	fn test_os_str_default() {
		let os_str: &OsStr = Default::default();
		assert_eq!("", os_str);
	}

	#[test]
	fn into_boxed() {
		let orig = "Hello, world!";
		let os_str = OsStr::new(orig);
		let boxed: Box<OsStr> = Box::from(os_str);
		let os_string = os_str.to_owned().into_boxed_os_str().into_os_string();
		assert_eq!(os_str, &*boxed);
		assert_eq!(&*boxed, &*os_string);
		assert_eq!(&*os_string, os_str);
	}

	#[test]
	fn boxed_default() {
		let boxed = <Box<OsStr>>::default();
		assert!(boxed.is_empty());
	}

	#[test]
	fn into_rc() {
		let orig = "Hello, world!";
		let os_str = OsStr::new(orig);
		let rc: Rc<OsStr> = Rc::from(os_str);
		let arc: Arc<OsStr> = Arc::from(os_str);

		assert_eq!(&*rc, os_str);
		assert_eq!(&*arc, os_str);

		let rc2: Rc<OsStr> = Rc::from(os_str.to_owned());
		let arc2: Arc<OsStr> = Arc::from(os_str.to_owned());

		assert_eq!(&*rc2, os_str);
		assert_eq!(&*arc2, os_str);
	}
}
