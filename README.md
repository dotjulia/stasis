# Stasis Lang
An interpreted toy language with minimal syntax.

## Syntax
The syntax only consists of a small set of symbols $\Sigma=\\{$ { $, $} $, $( $, $) $, $=> $, $` $\\}$.
Everything in stasis is either a function or a number.
 ### Expressions
All expressions in stasis consist of function calls and values. Values are number literals or function definitions (see below).
A function can be called by giving it an argument like that: `function argument;` (eg `+ 1`).
Functions which take no arguments still need to be given an argument to call them otherwise the expression
just returns the function definition as a value (eg. `ret4` just returns 4. `ret4;` -> Function, `ret4 0;` -> 4).

Functions only take one argument at a time and are evaluated left to right. For example:

`+ 1 2;` = `(+ 1) 2;`

`+` = Function taking two arguments `+ 1;` = Function taking one argument, `+ 1 2;` = 3

A function can also be called postfix with the backtick ``` ` ```.

`+ 1;` = ```1 `+;```

This allows us to write additions more naturally: ``` 1 `+ 2 ``` = ```(1 `+) 2``` = `(+ 1) 2` = `+ 1 2`.
This can be done with all functions: ```1 `not``` = `not 1`.
### Functions
A function is a first-class citizens and can be created by writing code within
curly braces `'{' (expression';')+ '}'`. For example:
```
{
  + 4 3;
}
```
This expression creates a function which returns the number `7`.
The last statement of a function always specifies it's return value.

A function can take any amount of arguments. Arguments are specified before the function body using `=>` as a delimiter
`'{' (token+ '=>')? (expression';')+ '}'`.
```
{ op0 op1 =>
  not (op0 `- op1);
}
```
This function takes two arguments (`op0` and `op1`), subtracts them and applies the logical not operation (`not` and `-` are built in functions).
The built-in logical `not` function takes one argument and returns `1` if the argument is `0` and `0` otherwise.
Therefore this functions checks if the two arguments are equal.

## Built-In Functions
### `+`
Takes two numeric arguments and adds them.
```
+ 3 4;  // 7
3 `+ 4; // 7
```
### `-`
Takes two numeric arguments and subtracts them.
```
- 4 3;  // 1
4 `- 3; // 1
- 3 4;  // 18446744073709551614 (numbers are unsigned)
```
### `not`
Returns `1` if the argument is `0` and `0` otherwise.
```
not 0;  // 1
not 3;  // 0
not 1;  // 0
0 `not; // 1
```
### `print`
Prints the argument and returns `0`.
```
print 4;     // Number(4)
print print; // Function(ValueFunction { func: BuiltInFunction(print, 1), bound_variables: [] })
```
### `let`
Takes two arguments. The first one has to be a function with a single function reference as a statement.
The second one is the value that is a assigned to the token in the first function.
```
let { a; } 4;
print a; // 4
```
This sounds confusing but to keep syntax to a minimum there is no special way of declaring/defining a new function or variable.
Instead `let` is a built-in function, but why am I not doing it like this:
```
let a = 4;
```
This expression would lead us to a runtime error because at the time we are calling the `let`-function the runtime would try to resolve the `a` token.

Instead by defining a new function like this `{ a; }` the `a` token is only resolved when the function is called.
Due to the `let` function being a built-in function it can inspect the function at runtime and figure out the token without ever calling the function.

This means that the syntax `let { a; } = 4;` would also be possible if we define `=` and just ignore the second parameter.

Defining functions with `let` is also possible:
```
{
  let { print4; } { a0 => print 4; print a0; };
  print4 12; // Number(4), Number(12)
}
```
### `if`
`If` takes two parameters. The first one is a value or a function and the second one a function. It calls the first one if it is a function and executes the second function if the return value is not 0.
```
{
  let { =; } {a0 a1 => not (a0 `- a1);};
  let { a; } 4;
  if (a `= 4) {
    print 5;
  };
  if {a `= 5;} {
    print 4;
  };
  print a;
} // Number(5), Number(4)
```
