/*!
Rust String & Slice Study Guide (Single-File)
================================================

**What you’ll learn (TL;DR):**
- `&str` is a **string slice**; string literals have type `&str`.
- Slices borrow data (no ownership), so they’re lightweight and safe.
- Idiomatic APIs accept `&str` so they work with both `String` and string literals.
- Useful slice forms: `[start..end]`, `[..end]`, `[start..]`, and `[..]` (the whole value).

This file consolidates several small examples into one **compile-ready** guide
with clear names, tests, and a single `main()`.
*/

// -----------------------------
// Section 1: Core Concepts
// -----------------------------

/// A string **slice** (`&str`) borrows a view into UTF-8 bytes owned elsewhere (e.g., a `String` or a
/// string literal). Slices never own, so they’re cheap to copy and pass around.
///
/// Common slice syntax shortcuts:
/// - `&s[0..2]`  == `&s[..2]`  (start at 0)
/// - `&s[3..len]` == `&s[3..]` (go to the end)
/// - `&s[0..len]` == `&s[..]`  (entire string)
///
/// **Note:** Slicing indices operate on **bytes**, not Unicode scalar values; slicing
/// between code point boundaries will panic at runtime. Prefer methods like `split_whitespace()`
/// when working with human text.
mod core_concepts {}

// -----------------------------
// Section 2: Two "first word" helpers
// -----------------------------

/// Returns the **byte index** where the first word ends (space or end-of-string).
///
/// This mirrors the classic example that scans raw bytes.
/// Use this if you genuinely need the index. For most ergonomics, see `first_word_slice` below.
pub fn first_word_index(s: &String) -> usize {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b' ' { return i; }
    }
    s.len()
}

/// Returns the **first word slice** from any `&str` (works with both `String` and string literals).
///
/// Accepting `&str` is idiomatic because it’s more general than `&String` and avoids unnecessary
/// conversions when the caller already has a `&str`.
pub fn first_word_slice(s: &str) -> &str {
    match s.find(' ') {
        Some(idx) => &s[..idx],
        None => s,
    }
}

// -----------------------------
// Section 3: Demonstrations
// -----------------------------

fn main() {
    // Working with a String
    let mut s = String::from("hello world from rust");

    // Index-based result
    let idx = first_word_index(&s); // -> 5 (the space after "hello")
    println!("first_word_index: {}", idx);

    // Slice-based result (borrowed view)
    let w = first_word_slice(&s); // -> "hello"
    println!("first_word_slice: {}", w);

    // Clearing the String invalidates **indices and slices** pointing inside it,
    // so never hold slices across mutations that could reallocate.
    s.clear();

    // Working with a string literal (type: &str)
    let lit: &str = "singleword"; // string literals are slices already
    println!("first of literal: {}", first_word_slice(lit));

    // Slice forms (by value length):
    let s2 = String::from("hello");
    let len = s2.len();
    let _a = &s2[..2];   // == &s2[0..2]
    let _b = &s2[3..];   // == &s2[3..len]
    let _c = &s2[..];    // == &s2[0..len]

    // Slices also work for other sequences (e.g., arrays)
    let arr = [1, 2, 3, 4, 5];
    let sub: &[i32] = &arr[1..3];
    assert_eq!(sub, &[2, 3]);
}

// -----------------------------
// Section 4: Tests (run with `cargo test`)
// -----------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_finds_space_or_len() {
        let s = String::from("hello world");
        assert_eq!(first_word_index(&s), 5);
        let s2 = String::from("single");
        assert_eq!(first_word_index(&s2), s2.len());
    }

    #[test]
    fn slice_returns_first_word() {
        assert_eq!(first_word_slice("hello world"), "hello");
        assert_eq!(first_word_slice("single"), "single");
        let s = String::from("hi rustaceans");
        assert_eq!(first_word_slice(&s), "hi");
    }

    #[test]
    fn array_slicing_demo() {
        let a = [1, 2, 3, 4, 5];
        let sl = &a[1..3];
        assert_eq!(sl, &[2, 3]);
    }
}
