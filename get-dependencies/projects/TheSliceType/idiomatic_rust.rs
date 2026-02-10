fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(idx) => &s[..idx],
        None => s,
    }
}

/*

How it works

s.find(' ') searches for the first space in the string.
If a space is found, we return the substring from the start up to (but not 
including) the space.
If no space is found, the entire string is returned because it must be a 
single word.

*/

//Example

fn main() {
    let s1 = "hello world from rust";
    let s2 = "singleword";

    println!("{}", first_word(s1)); // "hello"
    println!("{}", first_word(s2)); // "singleword"
}

//Usage with String

fn first_word(s: &String) -> usize {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return i;
        }
    }

    s.len()
}

//More examples

fn main() {
    let mut s = String::from("hello world");

    let word = first_word(&s); // word will get the value 5

    s.clear(); // this empties the String, making it equal to ""

    // word still has the value 5 here, but s no longer has any content that we
    // could meaningfully use with the value 5, so word is now totally invalid!
}

/*

A string slice is a reference to a contiguous sequence of the elements of a 
String, and it looks like this:

    let s = String::from("hello world");

    let hello = &s[0..5];
    let world = &s[6..11];

    Rather than a reference to the entire String, hello is a reference 
    to a portion of the String, specified in the extra [0..5] bit

The type of hello and world is &str, which is a string slice. A string 
slice is a reference to a portion of a String, and it allows you to work 
with parts of a String without taking ownership of the entire String.

*/

/*

With Rust’s .. range syntax, if you want to start at index 0, you can drop 
the value before the two periods. In other words, these are equal:

let s = String::from("hello");

let slice = &s[0..2];
let slice = &s[..2];

By the same token, if your slice includes the last byte of the String, 
you can drop the trailing number. That means these are equal:

let s = String::from("hello");

let len = s.len();

let slice = &s[3..len];
let slice = &s[3..];

You can also drop both values to take a slice of the entire string. 
So, these are equal:

let s = String::from("hello");

let len = s.len();

let slice = &s[0..len];
let slice = &s[..];

*/