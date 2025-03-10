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

Here's a small example of MoSa code.
```mosa
// Calculate a power of 2

fn power_of_two ->> Num -> Num { // define a function using DAS. (You can use a regular function too)
    let buf = 1; // create a variable `buf` and assign 1
    // argument in a DAS function is always called `it`
    (:=buf * 2)?:it; // use the repeat `?:` operator to repeat the expression with the self-assign `:=` operator
    buf // return the variable buf
}

fn power_of_two(power: Num) -> Num {
// here's an example of creating a regular function. 
// MoSa's DAS function definition is a syntactic sugar,
// the interpreter see's them both as regular functions,
// and a regular function can be called with the `->>` operator,
// and a DAS function can be called with `()`
    let buf = 1;
    (:=buf * 2)?:power;
    buf
}

immut let power = 8; // MoSa has no constants in their regular form, so we use `immut let` to create a variable that cannot be modified in the future
let res = power ->> power_of_two; // using DAS to pass `power` as an argument to `power_of_two`
res // returning `res`; 256 is the result
```

You can check the docs at the wiki: https://github.com/NaKeRNarolino/mountain_sakura/wiki