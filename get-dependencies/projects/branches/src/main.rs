/*

fn main() {
    let number = 7;

    if number < 5 {
        println!("condition was true");
    } else {
        println!("condition was false");
    }
}

    */


 /*fn main() {
    let number = 6;

    if number % 4 == 0 {
        println!("number is divisible by 4");
    } else if number % 3 == 0 {
        println!("number is divisible by 3");
    } else if number % 2 == 0 {
        println!("number is divisible by 2");
    } else {
        println!("number is not divisible by 4, 3, or 2");
    }
} 

*/


/*

When this program executes, it checks each if expression in turn and 
executes the first body for which the condition evaluates to true. 
Note that even though 6 is divisible by 2, we don’t see the output 
number is divisible by 2, nor do we see the number is not divisible 
by 4, 3, or 2 text from the else block. That’s because Rust only executes 
the block for the first true condition, and once it finds one, it doesn’t
even check the rest.

Using too many else if expressions can clutter your code, so if you have 
more than one, you might want to refactor your code. Chapter 6 describes a 
powerful Rust branching construct called match for these cases.

*/

fn main() {
    let condition = true;
    let number = if condition { 5 } else { 6 };

    println!("The value of number is: {number}");
}

/*

The number variable will be bound to a value based on the outcome of the if 
expression

*/