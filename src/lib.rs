#![forbid(intra_doc_link_resolution_failure)]

//! Utilities related to FFI bindings, for embedded platforms that use
//! Unix-like conventions. This is mostly copy & pasted from the Rust
//! standard library.
//!
//! Note that OsString and CString require the `alloc` feature enabled
//! in your Cargo.toml.
//!
//! This module provides utilities to handle data across non-Rust
//! interfaces, like other programming languages and the underlying
//! operating system. It is mainly of use for FFI (Foreign Function
//! Interface) bindings and code that needs to exchange C-like strings
//! with other languages.
//!
//! # Overview
//!
//! Rust represents owned strings with the [`String`] type, and
//! borrowed slices of strings with the [`str`] primitive. Both are
//! always in UTF-8 encoding, and may contain nul bytes in the middle,
//! i.e., if you look at the bytes that make up the string, there may
//! be a `\0` among them. Both `String` and `str` store their length
//! explicitly; there are no nul terminators at the end of strings
//! like in C.
//!
//! C strings are different from Rust strings:
//!
//! * **Encodings** - Rust strings are UTF-8, but C strings may use
//! other encodings. If you are using a string from C, you should
//! check its encoding explicitly, rather than just assuming that it
//! is UTF-8 like you can do in Rust.
//!
//! * **Character size** - C strings may use `char` or `wchar_t`-sized
//! characters; please **note** that C's `char` is different from Rust's.
//! The C standard leaves the actual sizes of those types open to
//! interpretation, but defines different APIs for strings made up of
//! each character type. Rust strings are always UTF-8, so different
//! Unicode characters will be encoded in a variable number of bytes
//! each. The Rust type [`char`] represents a '[Unicode scalar
//! value]', which is similar to, but not the same as, a '[Unicode
//! code point]'.
//!
//! * **Nul terminators and implicit string lengths** - Often, C
//! strings are nul-terminated, i.e., they have a `\0` character at the
//! end. The length of a string buffer is not stored, but has to be
//! calculated; to compute the length of a string, C code must
//! manually call a function like `strlen()` for `char`-based strings,
//! or `wcslen()` for `wchar_t`-based ones. Those functions return
//! the number of characters in the string excluding the nul
//! terminator, so the buffer length is really `len+1` characters.
//! Rust strings don't have a nul terminator; their length is always
//! stored and does not need to be calculated. While in Rust
//! accessing a string's length is a O(1) operation (because the
//! length is stored); in C it is an O(length) operation because the
//! length needs to be computed by scanning the string for the nul
//! terminator.
//!
//! * **Internal nul characters** - When C strings have a nul
//! terminator character, this usually means that they cannot have nul
//! characters in the middle — a nul character would essentially
//! truncate the string. Rust strings *can* have nul characters in
//! the middle, because nul does not have to mark the end of the
//! string in Rust.
//!
//! # Representations of non-Rust strings
//!
//! [`CString`] and [`CStr`] are useful when you need to transfer
//! UTF-8 strings to and from languages with a C ABI, like Python.
//!
//! * **From Rust to C:** [`CString`] represents an owned, C-friendly
//! string: it is nul-terminated, and has no internal nul characters.
//! Rust code can create a [`CString`] out of a normal string (provided
//! that the string doesn't have nul characters in the middle), and
//! then use a variety of methods to obtain a raw `*mut `[`u8`] that can
//! then be passed as an argument to functions which use the C
//! conventions for strings.
//!
//! * **From C to Rust:** [`CStr`] represents a borrowed C string; it
//! is what you would use to wrap a raw `*const `[`u8`] that you got from
//! a C function. A [`CStr`] is guaranteed to be a nul-terminated array
//! of bytes. Once you have a [`CStr`], you can convert it to a Rust
//! [`&str`][`str`] if it's valid UTF-8, or lossily convert it by adding
//! replacement characters.
//!
//! [`OsString`] and [`OsStr`] are useful when you need to transfer
//! strings to and from the operating system itself, or when capturing
//! the output of external commands. Conversions between [`OsString`],
//! [`OsStr`] and Rust strings work similarly to those for [`CString`]
//! and [`CStr`].
//!
//! * [`OsString`] represents an owned string in whatever
//! representation the operating system prefers. In the Rust standard
//! library, various APIs that transfer strings to/from the operating
//! system use [`OsString`] instead of plain strings.
//!
//! * [`OsStr`] represents a borrowed reference to a string in a
//! format that can be passed to the operating system. It can be
//! converted into an UTF-8 Rust string slice in a similar way to
//! [`OsString`].
//!
//! # Conversions
//!
//! ## On Unix
//!
//! On Unix, [`OsStr`] implements the
//! [`OsStrExt`] trait, which
//! augments it with two methods, [`from_bytes`] and [`as_bytes`].
//! These do inexpensive conversions from and to UTF-8 byte slices.
//!
//! Additionally, on Unix [`OsString`] implements the
//! [`OsStringExt`] trait,
//! which provides [`from_vec`] and [`into_vec`] methods that consume
//! their arguments, and take or produce vectors of [`u8`].
//!
//! [`String`]: alloc::string::String
//! [Unicode scalar value]: http://www.unicode.org/glossary/#unicode_scalar_value
//! [Unicode code point]: http://www.unicode.org/glossary/#code_point
//! [`from_vec`]: OsStringExt::from_vec
//! [`into_vec`]: OsStringExt::into_vec
//! [`from_bytes`]: OsStrExt::from_bytes
//! [`as_bytes`]: OsStrExt::as_bytes
#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;

#[doc(no_inline)]
pub use cstr_core::CStr;
#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use cstr_core::CString;

#[cfg(feature = "alloc")]
pub use inner::inner_alloc::OsStringExt;
pub use inner::OsStrExt;
pub use os_str::OsStr;
#[cfg(feature = "alloc")]
pub use os_str::OsString;

mod inner;
mod lossy;
mod os_str;

mod sys_common {
	#[doc(hidden)]
	pub trait AsInner<Inner: ?Sized> {
		fn as_inner(&self) -> &Inner;
	}

	/// A trait for extracting representations from std types
	#[doc(hidden)]
	pub trait IntoInner<Inner> {
		fn into_inner(self) -> Inner;
	}

	/// A trait for creating std types from internal representations
	#[doc(hidden)]
	pub trait FromInner<Inner> {
		fn from_inner(inner: Inner) -> Self;
	}

	pub mod bytestring {
		use core::fmt::{Formatter, Result, Write};

		use crate::lossy::{Utf8Lossy, Utf8LossyChunk};

		pub fn debug_fmt_bytestring(slice: &[u8], f: &mut Formatter<'_>) -> Result {
			// Writes out a valid unicode string with the correct escape sequences
			fn write_str_escaped(f: &mut Formatter<'_>, s: &str) -> Result {
				for c in s.chars().flat_map(|c| c.escape_debug()) {
					f.write_char(c)?
				}
				Ok(())
			}

			f.write_str("\"")?;
			for Utf8LossyChunk { valid, broken } in Utf8Lossy::from_bytes(slice).chunks() {
				write_str_escaped(f, valid)?;
				for b in broken {
					write!(f, "\\x{:02X}", b)?;
				}
			}
			f.write_str("\"")
		}
	}
}

// https://tools.ietf.org/html/rfc3629
#[rustfmt::skip]
static UTF8_CHAR_WIDTH: [u8; 256] = [
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x1F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x3F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x5F
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,
	1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1, // 0x7F
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0x9F
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
	0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // 0xBF
	0,0,2,2,2,2,2,2,2,2,2,2,2,2,2,2,
	2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2, // 0xDF
	3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3, // 0xEF
	4,4,4,4,4,0,0,0,0,0,0,0,0,0,0,0, // 0xFF
];

/// Given a first byte, determines how many bytes are in this UTF-8 character.
#[inline]
fn utf8_char_width(b: u8) -> usize {
	UTF8_CHAR_WIDTH[b as usize] as usize
}
