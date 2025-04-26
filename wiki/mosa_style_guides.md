# MoSa code Style Guides

## Naming
Variables and functions follow the lowerCamelCase pattern.
```mosa
immut let a: num = 0; // ok
immut let var_name: num = 0; // nope
immut let VarName: num = 0; // meh
immut let varName: num = 0; // ideal

fn doSomething() -> null { } // perfect
```
Layout fields follow the same rules, as variables.
Layout and Enum names, as well as Enum entries use UpperCamelCase.

### `immut`
Immut should be generally applied to all variables, that are not going to be modified.

### Types
It's a good practice to always mention the variable type. But, if the type is *obvious*, like in `let a = Example/>Value` it's obvious that the type is the enum `Example`, you should *avoid* specifying the type, so:
```mosa
immut let a = 0; // ok
immut let b = true; // not bad
immut let c: ExampleEnum = ExampleEnum/>Value; // why
immut let d = ExampleEnum/>Value; // perfect
immut let e = ExampleLayout { }; // also perfect
```

### DAS
It's a good practice to use DAS where possible.
```mosa
let a: Int = countLen("Hello"); // why?
let b = "Hi" ->> countLen; // nice
```

### Self assign operator (:=)
Use the self assign operator everywhere, where possible.
```mosa
let a = 0;
a = a + 10; // why?
:=a + 10; // nice!
a = someFn(a); // why?
:=a ->> someFn; // perfect!
```

### Repeat operator (?:)
Use it everywhere, where possible (where you don't need the value of the iterable)
```mosa
"hi" ->> printLn;
"hi" ->> printLn;
"hi" ->> printLn; // why?
for 0..3 {
    "hi" ->> printLn;
} // better
("hi" ->> printLn)?:3; // perfect!

for 0..5 {
    someFnCall(0);
    someOtherFnCall(1);
} // not bad, but no reason to keep the value of the iterable
(block {
    someFnCall(0);
    someOtherFnCall(1);
})?:5; // perfect
```

### Native Functions
`use native fn` declarations should ideally be on the top of the file, but you can also keep them in any scope you want.
Native function paths follow the pattern `x~>y~>maybeZ~>maybeEvenW`, separating with ~>. But, the paths are just strings, so you can use any pattern you want.