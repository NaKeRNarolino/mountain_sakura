# Enums
Enums in MoSa are declared with the `enum` keyword.
```mosa
enum EnumName {
    EnumEntry1,
    EnumEntry2,
    ...
}
```
To access an entry, use the `->` operator. 
Note, that every enum entry has type of `<EnumName>`, and it's not a primitive type.
```mosa
let a = EnumName->EnumEntryName;

typeof a // will return "EnumName"
```
MoSa' enums can't store any data, contrary to their rust analogue.

# Layouts
Layouts are a way to store structured data, they are similar to Rust `struct`s.
Layouts are declared with the `layout` keyword.
````mosa
layout ExampleLayout {
    value1: num, // properties in layouts should have a type defined.
    value2: str = "hi" // properties in layouts can also have default values.
}

let v = ExampleLayout {
    value1 = 12, // we use `=` to define the value,
    value2 = "hi!!", // if we define a property with a default value, it gets overridden.
};

printLn(v.value1); // 12
printLn(v.value2); // hi!!

v.value2 = "hello!"; // all properties in layouts are mutable.

printLn(v.value2); // hello!

printLn(typeof v); // ExampleLayout
````
## Mix statements
We can also use `mix` statements to attach functions to the layout structure.
```mosa
layout ExampleLayout {
    value1: num, 
    value2: str = "hi"
}

mix ExampleLayout {
    fn hello() -> ExampleLayout { // in the future, you'll be able to use `@self` instead of the layout name
        ExampleLayout {
            value1 = 1,
            value2 = "hello"  
        }
    }
}

let v = ExampleLayout->hello();

printLn(v.value2); // hello
```
### Tied functions
In MoSa, `tied` functions are functions that are tied to an instance of a layout.
```mosa
layout ExampleLayout {
    value1: num, 
    value2: str = "hi"
}

mix ExampleLayout {
    fn hello() -> ExampleLayout { // in the future, you'll be able to use `@self` instead of the layout name
        ExampleLayout {
            value1 = 1,
            value2 = "hello"  
        }
    }
    
    tied fn add(v: num)
}

let v = ExampleLayout->hello();

printLn(v.value2); // hello
```