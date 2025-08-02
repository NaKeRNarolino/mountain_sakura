# Mountain Sakura - *a language that actually makes sense.*

MoSa is a work-in-progress interpreted programming languages, that's really easy to integrate with Rust.
It's `use native` allows to make calling rust code as easy as just calling a function.

```mosa
use native fn printLn # "mosa-native~>printLn";

printLn("Hello, world!");
```
And on the Rust side of things
```rust
scope.add_native_function(String::from("mosa-native~>printLn"), Arc::new(|args| {
    println!("{}", &args[0]);
    RuntimeValue::Null
}));
```
---

Here's a small example showcasing a few features of MoSa.
```mosa
// file execution starts from the start, no `main` function is needed
use native fn printLn#"mosa-native~>printLn"; // we'll use a native function-wrapper the for rust println! marco
// as there's no std for mosa yet

let v = 1; // create a variable named v and assign zero to it
let vType: str = typeof v; // here we explicitly define the type for the variable.
// `typeof` gets a string representation of the type from `v`

vType ->> printLn; // using DAS we can call a function with only one argument like this
// prints `num`

// MoSa is a strictly-typed language, so in this case we cannot assign "hi" to `v`, as it's a num

// you can also create your own functions
// the regular syntax is identical to rust
fn f(v: num) -> num {
    v * v * 2 // MoSa has no `return` keyword. It returns whatever the last expression in that code block returned
}

// it's worth mentioning that a function that returns nothing will return `null`
// in MoSa `null` is it's own primitive type, just like str, num, etc.

immut let q = f(v); // regular function calls are also supported
// an `immut` variable (or as I like to call them - invariables) cannot be edited afterwards.
// analogue of `const` in most languages

printLn(q); // prints `2`

// MoSa also has *complex* data structures, enums and layouts. Learn more about them on the wiki!
```

You can check the docs at the wiki: https://github.com/NaKeRNarolino/mountain_sakura/wiki

# JNI
This repository now includes optional module of JNI which cannot be used when application runs on rust.
Currently there are no limiters to JNI and also no JNI library for usage. But it will be implemented!

[JNI Code repo](https://github.com/kofeychi/mosajni)