// Feature 1: Float type (da_tha)
// Feature 2: Nil (bhala)
// Feature 3: Type conversions (pyaung)
// Feature 4: Slice operations
// Feature 5: String methods
// Feature 6: Structs (pone)
// Feature 7: Methods (nee)
// Feature 8: Interfaces (myat)

loke main() -> kain {
    da_tha pi = 3.14;
    da_tha radius = 2.5;
    sar maybe = bhala;
    sar num_str = pyaung_sar(42);
    kain parsed = pyaung_kain("100");
    da_tha float_val = pyaung_da_tha("3.14");
    su<kain> numbers = [
        1,
        2,
        3,
        4,
        5
    ];
    su<kain> sliced = numbers[1:3];
    kain length = ashay(numbers);
    su<kain> appended = htae(numbers, 6);
    sar text = "hello,world";
    su<sar> parts = text.khwae(",");
    sit has = text.swal("hello");
    pya(pi);
    pya(num_str);
    pya(length);
    pya(sliced);
    pya(appended);
    pya(parts);
    pya(has);
    Person person = Person { name: "Aung Aung", age: 30 };
    sar greeting = person.greet();
    pya(greeting);

    pat item htae numbers {
        pya(item);
    }

    pyan 0;
}

pone Person {
    sar name;
    kain age;
}

nee (Person p) greet() -> sar {
    pyan "Hello, " + p.name + "! You are " + pyaung_sar(p.age) + " years old.";
}

myat Greeter {
    loke greet() -> sar;
}
