# M-Lang Language Cheatsheet

Quick reference for all M-Lang syntax.

---

## Variable Declaration

```
kain name = value;     // integer
sar name = "text";     // string
sit name = hman;       // boolean (true)
sit name = hmar;       // boolean (false)
```

## Arrays

```
su<kain> numbers = [1, 2, 3];
kain first = numbers[0];
```

## HashMaps

```
twe<sar, kain> dict = {"key": 1, "key2": 2};
kain val = dict["key"];
```

## Functions

```
loke function_name(kain param1, sar param2) -> kain {
    pyan param1;
}
```

Entry point (main):

```
loke main() -> kain {
    pyan 0;
}
```

## If / Else If / Else

```
hlyin (condition) {
    // ...
} mo hlyin (condition2) {
    // ...
} mo {
    // ...
}
```

## While Loop

```
pat (condition) {
    // ...
}
```

## For-In Loop (Python-style)

```
su<kain> numbers = [1, 2, 3];
pat item htae numbers {
    pya(item);
}
```

## Print

```
pya("Hello");
pya(variable);
```

## Read Input

```
sar input = phat("Enter something: ");
```

## Import

```
yu module_name;
```

## Comments

```
// This is a comment
kain x = 10; // Inline comment
```

## Operators

```
+  -  *  /          // Arithmetic
== != > < >= <=     // Comparison
+                   // String concatenation (on sar types)
```

## Myanmar ↔ ASCII Digits

```
၀=0  ၁=1  ၂=2  ၃=3  ၄=4  ၅=5  ၆=6  ၇=7  ၈=8  ၉=9
```
