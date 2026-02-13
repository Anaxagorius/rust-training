/*!
Rust Structs, Tuple Structs & Methods — Study Guide (Single-File)
=================================================================

**What you’ll learn (TL;DR):**
- When to use **named-field structs**, **tuple structs**, and **unit-like structs**.
- How **field init shorthand** and **struct update syntax (`..base`)** reduce boilerplate.
- How to attach **methods** to types with `impl`, including receivers `&self`, `&mut self`, and `self`.
- What **automatic referencing/dereferencing** means for method calls, and how to write **associated functions** like constructors.

This file merges your previous structs/tuples guide with your methods examples into one
compile-ready study guide with a single `main()` and unit tests.
*/

// -----------------------------
// Section 1: Classic structs (named fields)
// -----------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

/// Build a `User` using field init shorthand (parameter names match field names).
fn build_user(email: String, username: String) -> User {
    User { active: true, username, email, sign_in_count: 1 }
}

/// The `..` syntax copies remaining fields from a base instance; it must appear last.
fn update_email(base: &User, new_email: &str) -> User {
    User {
        email: new_email.to_string(),
        ..base.clone()
    }
}

// -----------------------------
// Section 2: Tuple structs & unit-like structs
// -----------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Color(i32, i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point(i32, i32, i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AlwaysEqual; // unit-like struct (zero-sized marker)

// -----------------------------
// Section 3: Methods, associated functions, and impl organization
// -----------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    /// Immutable receiver: read-only access to fields
    fn area(&self) -> u32 { self.width * self.height }

    /// Convenience predicate method
    fn has_nonzero_width(&self) -> bool { self.width > 0 }

    /// Method with an extra parameter (borrows another Rectangle)
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    /// Associated function (no receiver). Often used as a constructor.
    fn square(size: u32) -> Self { Self { width: size, height: size } }
}

// Notes:
// - Rust performs *automatic referencing/dereferencing* for method calls, so
//   `rect.area()` and `(&rect).area()` are equivalent when the method takes `&self`.
// - You may have multiple `impl` blocks for the same type; the compiler treats them
//   as one combined implementation (useful for grouping related methods).

// -----------------------------
// Section 4: Demonstrations
// -----------------------------

fn main() {
    // Classic struct: create, mutate, build, and update
    let mut user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };
    user1.email = String::from("anotheremail@example.com");

    let user2 = build_user("user2@example.com".into(), "user2".into());
    let user3 = update_email(&user2, "user3@example.com");

    // Tuple & unit-like structs
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
    let _marker = AlwaysEqual;

    // Methods on Rectangle
    let rect1 = Rectangle { width: 30, height: 50 };
    let rect2 = Rectangle { width: 10, height: 40 };
    let rect3 = Rectangle { width: 60, height: 45 };

    println!("area(rect1) = {}", rect1.area());
    if rect1.has_nonzero_width() {
        println!("rect1 has nonzero width: {}", rect1.width);
    }
    println!("rect1 can hold rect2? {}", rect1.can_hold(&rect2));
    println!("rect1 can hold rect3? {}", rect1.can_hold(&rect3));

    let sq = Rectangle::square(20);
    println!("square area = {} ({}x{})", sq.area(), sq.width, sq.height);

    // Show that tuple structs are distinct types
    println!("black: {:?}, origin: {:?}", black, origin);
    println!("user1: {:?}\nuser2: {:?}\nuser3: {:?}", user1, user2, user3);
}

// -----------------------------
// Section 5: Tests (run with `cargo test`)
// -----------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- User & update syntax ---
    #[test]
    fn build_user_uses_shorthand() {
        let u = build_user("a@b.com".into(), "tom".into());
        assert!(u.active);
        assert_eq!(u.username, "tom");
        assert_eq!(u.email, "a@b.com");
        assert_eq!(u.sign_in_count, 1);
    }

    #[test]
    fn update_uses_base_fields() {
        let base = build_user("a@b.com".into(), "base".into());
        let upd = update_email(&base, "c@d.com");
        assert_eq!(upd.username, base.username);
        assert_eq!(upd.sign_in_count, base.sign_in_count);
        assert_eq!(upd.email, "c@d.com");
    }

    // --- Tuple structs ---
    #[test]
    fn tuple_structs_are_distinct_types() {
        let c = Color(0, 0, 0);
        let p = Point(0, 0, 0);
        let _same_color: Color = c;
        let _same_point: Point = p;
        assert_eq!(c, Color(0,0,0));
        assert_eq!(p, Point(0,0,0));
    }

    // --- Rectangle methods ---
    #[test]
    fn area_and_predicates() {
        let r = Rectangle { width: 3, height: 4 };
        assert_eq!(r.area(), 12);
        assert!(r.has_nonzero_width());
    }

    #[test]
    fn can_hold_logic() {
        let big = Rectangle { width: 8, height: 7 };
        let small = Rectangle { width: 5, height: 5 };
        assert!(big.can_hold(&small));
        assert!(!small.can_hold(&big));
    }

    #[test]
    fn square_constructor() {
        let s = Rectangle::square(10);
        assert_eq!(s.width, 10);
        assert_eq!(s.height, 10);
        assert_eq!(s.area(), 100);
    }
}
