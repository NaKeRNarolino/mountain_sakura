# Native Functions and `use native fn`
Native functions are a way to call Rust code from MoSa.
A *native* function is a basic Rust function, it takes `Vec<RuntimeValue` as input and should return `RuntimeValue` as an output.
A native function can be defined in a scope *from the Rust side* of MoSa, however there's going to be a way to add them dynamically to your projects in the future.

## Adding a native function
Here's an example of adding a `printLn` native function.
```rust
// When `scope` is the RuntimeScope the program is using
scope.add_native_function(String::from("mosa-native~>printLn"), Arc::new(|args| {
    if let RuntimeValue::String(str) = &args[0] {
        println!("{}", str);
    }
    RuntimeValue::Null
}));
```
A `path` for a native function is any string, but the general, MoSa way for them is `"x~>y~>z"`.

## Using a native function
To use a *native* function, we can use the `use native fn` syntax.

```mosa
use native fn <Identifier> # <String>;
```
Where `<Identifier>` stands for any identifier, and `<String>` for the path String.

For example, to use the code above we'll use this code below.
```mosa
use native fn printLn # "mosa-native~>printLn";

printLn("Hello, world!");
```

### Priority question
**In general, MoSa *prefers* *native* functions, not regular ones.**
So if you have a native function and a regular function with the same name, MoSa will call the *native* one.