# THIS IS A WIP FEATURE
# Imports and Exports
As of July 18, 2025, MoSa supports importing and exporting functions.
Let's say you have the following hierarchy.

```
test/
    test.mosa
other.mosa
main.mosa
```
In `other.mosa`, we can define a function `hi`, that'll return a string.
The file can also return a value, we'll talk about it later. Now we'll make it return `10`.
```mosa
// other.mosa
// We use `exp` for exporting
exp fn hi() -> str {
    "hi"
}

// We'll return 10 from this file
10
```
Now, we'll make a similar function on `test.mosa` too.
```mosa
// test.mosa
exp fn x() -> num {
    12
}

"hello, world!" // we'll return this string here. Notice that it's not a requirement to return something.
```
We can now import this functions in `main.mosa`.
```mosa
// main.mosa
use other~>hi;
use test:test~>x;

hi(); // this will return "hi"
x(); // this will return 12
```

## What about the file returns?
The file returns allow you to make `use` statements return stuff.
If we consider the example above
```mosa
let h = use other~>hi;
let f = use test:test~>x;

hi(); // "hi", the functions are still being imported
x(); 12

h; // 10, it outputs the thing that the file returned.
f; // "hello, world!"
```
Yes, this does mean that MoSa **executes** the imported files.
Notice that every file runs **once**, and their output is cached, so if you import file `x` in `y` and `z`, `x` will run only once, and the value it returns is cached, and not recalculated.