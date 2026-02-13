/*

Tuple structs have the added meaning the struct name provides but don’t have 
names associated with their fields; rather, they just have the types of the 
fields. Tuple structs are useful when you want to give the whole tuple a 
name and make the tuple a different type from other tuples, and when naming 
each field as in a regular struct would be verbose or redundant.

To define a tuple struct, start with the struct keyword and the struct name 
followed by the types in the tuple.

*/

struct Color(i32, i32, i32);// A tuple struct to hold the RGB values of a color
struct Point(i32, i32, i32);

fn main() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
}

/*

Defining Unit-like Structs
There are structs that don’t have any fields! These are called unit-like
structs because they behave similarly to the unit type (). Unit-like structs
can be useful in situations where you need to implement a trait on some type but
don’t have any data that you want to store in the type itself. For example, the
standard library has a unit-like struct called PhantomData that is used when you
need to tell the compiler about a type that you want to use in a struct, but
you don’t have any value of that type to put in the struct. We’ll talk about
PhantomData in more detail in Chapter 19. Here’s how we can define a unit-like
struct:

*/

struct AlwaysEqual;

fn main() {
    let subject = AlwaysEqual;
}

//To define AlwaysEqual, we use the struct keyword, the name we want, and 
//then a semicolon. No need for curly brackets or parentheses! Then, we can
//get an instance of AlwaysEqual in the subject variable in a similar way: 
//using the name we defined, without any curly brackets or parentheses.

/*

Ownership of Struct Data
The ownership rules that apply to other Rust data types also apply to the data
in a struct. For example, if we have a struct that has a String field, the
ownership of the String data is moved to the struct when we create an instance of
the struct. If we want to use that String data elsewhere, we can create a
reference to the data in the struct. We can also create a new instance of the
struct with a reference to the data in the original instance.

*/

struct User {
    active: bool,
    username: &str,
    email: &str,
    sign_in_count: u64,
}

fn main() {
    let user1 = User {
        active: true,
        username: "someusername123",
        email: "someone@example.com",
        sign_in_count: 1,
    };
}