loke main() -> kain {
    kain age = 20;
    sar name = "Aung Aung";
    sar greeting = "Hello, " + name + "!";
    sar h = "hi";
    pya(h);
    sar hau = "nay kaug lrr";
    pya(hau);

    hlyin (age > 18) {
        pya("Hello World! ");
        pya(name);
    } mo hlyin (age == 18) {
        pya("Too young!");
    } mo {
        pya("Too young!");
    }

    su<kain> numbers = [
        1,
        2,
        3,
        4,
        5
    ];

    pat item htae numbers {
        pya(item);
    }

    Person person = Person { name: "Aung Aung", age: 30 };
    sar fff = person.greet();
    pya(greeting);
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
