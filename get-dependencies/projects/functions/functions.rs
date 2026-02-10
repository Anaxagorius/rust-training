/*

fn main() {
    println!("Hello, world!");

    another_function();
    another_function2(10);
}

fn another_function() {
    println!("Another function.");
}

    */

/*

fn main2() {
    another_function2(5);
}

fn another_function2(x: i32) {
    println!("The value of x is: {x}");
}



fn main() {
    print_labeled_measurement(5, 'h');
}

fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}

    */

fn main() {
    let x = plus_one(5);

    println!("The value of x is: {x}");
}

fn plus_one(x: i32) -> i32 {
    x + 1
}

