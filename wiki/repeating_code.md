# Repeat operator
In MoSa we can use the *repeat operator* (`?:`) to repeatedly call some code.
As there are no loops in MoSa *for now*, we can only call it a static amount of times(we cannot stop the loop at some point).

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