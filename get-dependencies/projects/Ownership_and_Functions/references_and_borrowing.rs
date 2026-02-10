/*

Here is how you would define and use a calculate_length function that has a 
reference to an object as a parameter instead of taking ownership of the 
value:

*/

fn main() {
    let s1 = String::from("hello");

    let len = calculate_length(&s1);

    println!("The length of '{s1}' is {len}.");
}

fn calculate_length(s: &String) -> usize {
    s.len()
}