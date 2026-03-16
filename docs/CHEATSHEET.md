# M-Lang Cheatsheet

Quick syntax reference for current `mlang` (v0.1.0).

## Types

```ml
kain n = 10;         // int64
sar s = "text";      // string
sit ok = hman;       // bool
da_tha pi = 3.14;    // float64
amhar err = bhala;   // error or nil
```

```ml
su<kain> nums = [1, 2, 3];
twe<sar, kain> price = {"tea": 1500};
laung<kain> jobs = laung<kain>();     // unbuffered channel
laung<sar> queue = laung<sar>(100);   // buffered channel
```

## Concurrency (Phase 3)

```ml
loke worker(laung<kain> ch) -> amhar {
    naut_sone ch.close();
    ch.send(1);
    pyan bhala;
}

kyoe worker(jobs);      // concurrent call
kain v = jobs.recv();    // receive value
jobs.close();            // close channel
```

## Functions and Closures

```ml
loke add(kain a, kain b) -> kain {
    pyan a + b;
}

loke apply(loke(kain) -> kain fn, kain x) -> kain {
    pyan fn(x);
}
```

```ml
kain out = apply(loke(kain v) -> kain {
    pyan v + 1;
}, 41);
```

## Tuple Returns + Destructuring

```ml
loke safe_div(kain a, kain b) -> (kain, amhar) {
    hlyin (b == 0) {
        pyan (0, amhar("division by zero"));
    }
    pyan (a / b, bhala);
}

kain q, amhar err = safe_div(10, 2);
```

## Conditionals and Loops

```ml
hlyin (ok) {
    pya("ok");
} mo hlyin (n > 0) {
    pya("positive");
} mo {
    pya("other");
}
```

```ml
pat (n > 0) {
    n = n - 1;
}

pat (kain i = 0; i < 3; i = i + 1) {
    pya(i);
}
```

```ml
pat item htae nums {
    pya(item);
}

pat (idx, item) htae nums {
    hlyin (idx == 0) { shar; }
    hlyin (item > 100) { yut; }
}
```

## Structs, Methods, Interfaces

```ml
pone Cart {
    sar customer;
    kain item_count;
}

nee (Cart c) summary() -> sar {
    pyan c.customer;
}

myat Summarizer {
    loke summary() -> sar;
}
```

```ml
Cart c = Cart { customer: "Aye Aye", item_count: 1 };
c.item_count = 2;
pya(c.summary());
```

## Collections and Methods

```ml
nums.push(4);
nums.remove(0);
kain l1 = nums.len();

sar text = "a,b,c";
su<sar> parts = text.khwae(",");
sit has_a = text.swal("a");
sar lower = text.ayaik();
kain l2 = text.ashay();
```

```ml
su<kain> more = htae(nums, 99);
kain len_any = ashay(price); // array, map, or string
```

## Index, Slice, Convert

```ml
kain first = nums[0];
su<kain> mid = nums[1:3];
su<kain> tail = nums[2:];
su<kain> head = nums[:2];
```

```ml
kain a = pyaung_kain("42");
sar b = pyaung_sar(42);
da_tha c = pyaung_da_tha("3.14");
```

## I/O and Imports

```ml
pya("hello");
sar name = phat("name: ");
```

```ml
yu "json";
yu "file";
yu "su_nit";
yu "kainn/http";
yu "kainn";
yu "pone_set";
yu "in_ote";
yu "hmat";

atote main;
pay loke exported_fn() -> kain {
    pyan 0;
}
```

```ml
sar msg = pone_set.pon_san("hi %s", "mlang");
sar line, amhar read_err = in_ote.twin_phat();
amhar write_err = in_ote.htote_yay(msg);
amhar info_err = hmat.mhat_chet(msg);
```

## HTTP Server and Sockets (Phase 3)

```ml
yu "kainn/http";

loke handler(http.Request req, http.ResponseWriter w) -> amhar {
    sar q = req.query("q");
    w.header("X-Query", q);
    w.status(200);
    pyan w.write("ok");
}

http.handle("/", handler);
http.listen(":8080");
```

```ml
yu "kainn";

kainn.TCPListener listener, amhar listen_err = kainn.tcp_listen(":9000");
kainn.TCPConn conn, amhar accept_err = listener.accept();
sar msg, amhar read_err = conn.read();
amhar write_err = conn.write(msg);
amhar close_err = conn.close();
```

## Comments and Digits

```ml
// comment
kain mixed = ၂0; // Myanmar + ASCII digits both supported
```
