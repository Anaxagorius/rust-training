/*

fn main() {
    loop {
        println!("again!");
    }
}//Endless Again!!!!

*/

/*

fn main() {
    let mut counter = 0;

    let result = loop {
        counter += 1;

        if counter == 10 {
            break counter * 2;
        }
    };

    println!("The result is {result}");
}



fn main() {
    let mut count = 0;
    'counting_up: loop {
        println!("count = {count}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break;
            }
            if count == 2 {
                break 'counting_up;
            }
            remaining -= 1;
        }

        count += 1;
    }
    println!("End count = {count}");
}



If you have loops within loops, break and continue apply to the innermost 
loop at that point. You can optionally specify a loop label on a loop that 
you can then use with break or continue to specify that those keywords apply 
to the labeled loop instead of the innermost loop. Loop labels must begin 
with a single quote

The outer loop has the label 'counting_up, and it will count up from 0 to 
2. The inner loop without a label counts down from 10 to 9. The first break 
that doesn’t specify a label will exit the inner loop only. The break 
'counting_up; statement will exit the outer loop. 



fn main() {
    let mut number = 3;

    while number != 0 {
        println!("{number}!");

        number -= 1;
    }

    println!("LIFTOFF!!!");
}//This construct eliminates a lot of nesting that would be necessary if 
//you used loop, if, else, and break, and it’s clearer. While a condition 
//evaluates to true, the code runs; otherwise, it exits the loop.



//You can choose to use the while construct to loop over the elements of a collection, such as an array

fn main() {
    let a = [10, 20, 30, 40, 50];
    let mut index = 0;

    while index < 5 {
        println!("the value is: {}", a[index]);

        index += 1;
    }
}//Here, the code counts up through the elements in the array. It starts at index 0 and then loops until 
//it reaches the final index in the array (that is, when index < 5 is no longer true).
//However, this approach is more error-prone and less idiomatic than using a for loop, which we cover next.

*/

fn main() {
    let a = [10, 20, 30, 40, 50];

    for element in a {
        println!("the value is: {element}");
    }
}//This code creates a variable named element that takes the value of each 
//element in the array a on each iteration through the loop. The for loop
//is more concise and less error-prone than the while loop we used to
//achieve the same functionality earlier.

