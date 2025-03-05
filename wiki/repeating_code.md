# Repeat operator
In MoSa we can use the *repeat operator* (`?:`) to repeatedly call some code.

The repeat operator uses this syntax.
```mosa
(<Experssion>)?:<Expression>
```
Where `<Expression>` stands for any expression (including any code blocks). An important thing is that the expression on the *right* of the operator should evaluate to a valid number. If it's a floating point number, it'll loop until it hits the floored value of the number.
```mosa
let a = 10;
(:=a + 1)?:5;
a // returns 15
```

## Code blocks
Code blocks are parts of the code. Any code block has it's own scope, parented by the scope it was created in.
For example, code in { } after `if`, `else`, of function declaration is a code block.
You can also create a code block in any place of the code, using the `block` keyword, as so
```mosa
block {
    // code here...
}
```

### Using code blocks in the repeat operator
You should place a code block in the parentheses, so
```mosa
let a = 10;

(block {
    :=a + 1;
    :=a * 2;
})?:2;

a // returns 46
```

### Bindings
In MoSa you can obtain pre-calculated values using *bindings*. Bindings are accessed using `^<BindingName>` syntax, and cannot be modified / defined by the user.
While using the repeat operator, you are able to access the `index` binding, which shows the count of iteration, starting from 0.
So
```mosa
let a = 10;

(block {
    :=a + ^index;
})?:2;

a // returns 11 (added 0 and 1 on first and second iterations)
```

# Loops
### Iterables
In MoSa, you can define iterables by using `<Expression>..<Expression>` syntax, where both expressions should be evaluated to valid `num`s.
E.g.
```mosa
let a = 0..5; // represents an iterable of [0, 1, 2, 3, 4]
// iterables have their own primitive type `iterable`
```

## For loop
A for loop is defined using the `for <Expression> { }` syntax, where expression should evaluate to a valid `iterable`.
In the for loop you have access to 2 bindings: `index` and `value`.
Index is the absolute distance from the start of the iterator, and value is the value at that point. So, e.g., if you have an iterable 3..7 it's second iteration's index will be 1, while the value will be 4.
E.g.
```mosa
use native fn printLn#"mosa-native~>printLn"; // ensuring the native function printLn is defined on the rust-side

for 10..13 {
    ^index ->> printLn;
    ^value ->> printLn; 
}

// Will result in:
// 0
// 10
// 1
// 11
// 2
// 12
```