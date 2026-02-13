/*

Methods
Methods are similar to functions: We declare them with the fn keyword and a 
name, they can have parameters and a return value, and they contain some 
code that’s run when the method is called from somewhere else. Unlike 
functions, methods are defined within the context of a struct (or an enum 
or a trait object), and their first parameter is always self, which 
represents the instance of the struct the method is being called on.
Methods are used to specify the behavior of a struct, and they’re an important part of the object-oriented programming features of Rust. We’ll talk
about how to define methods in Chapter 5, but we’ll cover how to use 
methods in this chapter. We’ll also talk about how to use associated 
functions, which are similar to methods but don’t take self as their first 
parameter. 

*/

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!(
        "The area of the rectangle is {} square pixels.",
        rect1.area()
    );
}/*In this code, we define a method named area on the Rectangle struct. 
The method takes an immutable reference to self as its parameter and 
returns a u32 value that is the area of the rectangle. We can call the area 
method on an instance of Rectangle using dot notation, just like we would 
call a function. The method has access to the fields of the struct through 
self, which is a reference to the instance of the struct that the method is 
being called on. In this case, we use self.width and self.height to calculate
the area of the rectangle.*/

impl Rectangle {
    fn width(&self) -> bool {
        self.width > 0
    }
}

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    if rect1.width() {
        println!("The rectangle has a nonzero width; it is {}", rect1.width);
    }
}/*In this code, we define another method named width on the Rectangle 
struct. This method also takes an immutable reference to self as its 
parameter and returns a bool value that indicates whether the width of
the rectangle is greater than 0. We can call this method on an instance of
Rectangle using dot notation, just like we would call a function. The method
has access to the fields of the struct through self, which is a reference 
tothe instance of the struct that the method is being called on. In this 
case, we use self.width to check if the width of the rectangle is greater than 0 and return a
boolean value accordingly. In the main function, we call the width method on
the rect1 instance and print a message if the rectangle has a nonzero width.*/

//-----------------------------------------------------------------------------

/*

Where’s the -> Operator?
In C and C++, two different operators are used for calling methods: You use 
. if you’re calling a method on the object directly and -> if you’re calling 
the method on a pointer to the object and need to dereference the pointer 
first. In other words, if object is a pointer, object->something() is 
similar to (*object).something().

Rust doesn’t have an equivalent to the -> operator; instead, Rust has a 
feature called automatic referencing and dereferencing. Calling methods is 
one of the few places in Rust with this behavior.

Here’s how it works: When you call a method with object.something(), Rust 
automatically adds in &, &mut, or * so that object matches the signature of 
the method. In other words, the following are the same:

p1.distance(&p2);
(&p1).distance(&p2);
The first one looks much cleaner. This automatic referencing behavior works 
because methods have a clear receiver—the type of self. Given the receiver 
and name of a method, Rust can figure out definitively whether the method is 
reading (&self), mutating (&mut self), or consuming (self). The fact that 
Rust makes borrowing implicit for method receivers is a big part of making 
ownership ergonomic in practice.

*/

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}/*In this code, we define two methods on the Rectangle struct: area and
can_hold. The area method takes an immutable reference to self and returns 
the area of the rectangle. The can_hold method takes an immutable reference 
to self and another Rectangle as a parameter and returns a boolean value 
indicating whether the rectangle can hold the other rectangle. We can call 
these methods on instances of Rectangle using dot notation, and Rust will 
automatically handle the referencing and dereferencing for us.*/

//-----------------------------------------------------------------------------

/*

Methods with More Parameters

*/

fn main() {
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };
    let rect2 = Rectangle {
        width: 10,
        height: 40,
    };
    let rect3 = Rectangle {
        width: 60,
        height: 45,
    };

    println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2));
    println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3));
}/*In this code, we create three instances of the Rectangle struct: rect1,
rect2, and rect3. We then call the can_hold method on rect1, passing in
rect2 and rect3 as parameters. The can_hold method checks if rect1 can hold
rect2 and rect3 by comparing their widths and heights. The output will indicate
whether rect1 can hold rect2 and whether rect1 can hold rect3 based on the
dimensions of the rectangles.

Can rect1 hold rect2? true
Can rect1 hold rect3? false
In this output, we see that rect1 can hold rect2 because rect1 has a greater 
width and height than rect2. However, rect1 cannot hold rect3 because rect3 
has a greater width than rect1, even though its height is less than rect1's 
height.*/


impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}/*In this code, we define two methods on the Rectangle struct: area and
can_hold. The area method takes an immutable reference to self and returns
the area of the rectangle. The can_hold method takes an immutable reference
to self and another Rectangle as a parameter and returns a boolean value
indicating whether the rectangle can hold the other rectangle. We can call
these methods on instances of Rectangle using dot notation, and Rust will
automatically handle the referencing and dereferencing for us.*/

//-----------------------------------------------------------------------------

/*

Associated Functions

All functions defined within an impl block are called associated functions 
because they’re associated with the type named after the impl. We can define 
associated functions that don’t have self as their first parameter (and thus 
are not methods) because they don’t need an instance of the type to work 
with. We’ve already used one function like this: the String::from function 
hat’s defined on the String type.

Associated functions that aren’t methods are often used for constructors 
that will return a new instance of the struct. These are often called new, 
but new isn’t a special name and isn’t built into the language. For example, 
we could choose to provide an associated function named square that would 
have one dimension parameter and use that as both width and height, thus 
making it easier to create a square Rectangle rather than having to specify 
the same value twice:

*/

impl Rectangle {
    fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }
}
/*In this code, we define an associated function named square on the 
Rectangle struct. This function takes one parameter, size, and returns a new
instance of Rectangle with both width and height set to size. This allows us
to easily create a square Rectangle without having to specify the same value
twice. We can call this function using the syntax Rectangle::square(size),
where size is the desired dimension of the square. The Self keyword refers
to the type that the impl block is for, which in this case is Rectangle.*/

//-----------------------------------------------------------------------------

/*

Multiple Blocks

Each struct is allowed to have multiple impl blocks, and the impl blocks can 
be separated by other code, and the methods defined in each impl block are
interspersed with the struct definition or even be in different files. This 
is useful when you want to organize your code by grouping related methods 
into separate impl blocks. For example, you might have one impl block that 
contains methods that only read data from the struct and another impl block that contains methods that mutate the struct. You can also have an impl block that 
contains associated functions and another impl block that contains methods. 
The Rust compiler will treat all of the impl blocks as if they were one big 
impl block, so it doesn’t matter how you organize your code into impl blocks.

*/

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}
/*In this code, we have two separate impl blocks for the Rectangle struct. The first impl block defines the area method, and the second impl block defines the
can_hold method. The Rust compiler will treat these two impl blocks as if they were one big impl block, so both methods will be available for instances of the 
Rectangle struct. This allows us to organize our code in a way that makes 
sense to us, grouping related methods together while still keeping all the 
functionality of the Rectangle struct intact.*/
