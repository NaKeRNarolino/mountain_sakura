# Enums
Enums in MoSa are declared with the `enum` keyword.
```mosa
enum EnumName {
    EnumEntry1,
    EnumEntry2,
    ...
}
```
To access an entry, use the `/>` operator. 
Note, that every enum entry has type of `<EnumName>`, and it's not a primitive type.
```mosa
let a = EnumName/>EnumEntryName;

typeof a // will return "EnumName"
```
MoSa' enums can't store any data, contrary to their rust analogue.