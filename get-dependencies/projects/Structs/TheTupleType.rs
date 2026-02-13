/*

Structs are similar to tuples, discussed in “The Tuple Type” section, in 
that both hold multiple related values. Like tuples, the pieces of a struct 
can be different types. Unlike with tuples, in a struct you’ll name each 
piece of data so it’s clear what the values mean. Adding these names means 
that structs are more flexible than tuples: You don’t have to rely on the 
order of the data to specify or access the values of an instance.

*/

/*
To define a struct, we enter the keyword struct and name the entire struct. 
A struct’s name should describe the significance of the pieces of data being 
grouped together. Then, inside curly brackets, we define the names and types 
of the pieces of data, which we call fields . Each field has a name and a 
type, just like variables. The fields of a struct are separated by commas, and the entire struct definition ends with a
semicolon. Here’s an example of a struct:

*/

struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}


fn main() {// To create an instance of a struct, we specify the name of the 
    //struct and then provide values for each of the fields in curly 
    //brackets. The syntax is similar to that of a tuple struct, but we 
    //need to use the field name: value format and the order doesn't matter.
    //We must specify the name of each struct field, followed by a colon and 
    //yje value we want to assign to that field. The fields and their values 
    //are separated by commas. 
    //Here’s how we can create an instance of the User struct we defined above: 
    let user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };
}

//shows how to change the value in the email field of a mutable User 
//instance.

fn main() {
    let mut user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };

    user1.email = String::from("anotheremail@example.com");
}

// shows a build_user function that returns a User instance with the given 
//email and username. The active field gets the value true, and the 
//sign_in_count gets a value of 1.

fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username: username,
        email: email,
        sign_in_count: 1,
    }
}

//Because the parameter names and the struct field names are exactly the 
//same in Listing 5-4, we can use the field init shorthand syntax to rewrite 
//build_user so that it behaves exactly the same but doesn’t have the 
//repetition of username and email

fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,
        email,
        sign_in_count: 1,
    }
}
