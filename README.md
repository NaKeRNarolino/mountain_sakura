# Mountain Sakura - `a language that actually makes sense.`

A small code example
```mosa
let two_power = 1; // note that comments are not currently actually supported.
// this program is to calculate 2^16
let power = 16;
(:=two_power * 2)?:power; // here we're using the self-assign(:=) operator to immediately assign `two_power * 2` and we're also using the repeat operator (?:) to repeat the code `power` times.
two_power // returning two_power
```