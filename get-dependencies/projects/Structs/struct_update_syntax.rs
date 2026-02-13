/*

It’s often useful to create a new instance of a struct that includes most 
of the values from another instance of the same type, but changes some of 
them. You can do this using struct update syntax.

*/

//create a new User instance in user2 in the regular way, without the update syntax. We set a new value for email
//and use the values of the other fields from user1. This is a lot of repetition,
//and it’s easy to make a mistake and forget to copy one of the fields or to
//copy a field but forget to change its value. We can use struct update syntax to
//avoid this repetition. Struct update syntax uses .. followed by the name of the
//instance we want to use as the base. This syntax will specify that the remaining
//fields not explicitly set should have the same value as the fields in the given
//instance. Here’s how we can use struct update syntax to create user2 based on
//user1:

fn main() {
    // --snip--

    let user2 = User {
        active: user1.active,
        username: user1.username,
        email: String::from("another@example.com"),
        sign_in_count: user1.sign_in_count,
    };
}

//Using struct update syntax, we can achieve the same effect with less code
//and less repetition. We specify the new value for email and then use ..user1 to
//fill in the rest of the fields with the values from user1. The struct update syntax
//must come last in the struct definition, because it specifies that the remaining
//fields not explicitly set should have the same value as the fields in the given
//instance. Here’s how we can rewrite the code to use struct update syntax:

fn main() {
    // --snip--
    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
}