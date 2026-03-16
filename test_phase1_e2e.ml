pone Counter {
    kain value;
}

loke safe_div(kain a, kain b) -> (kain, amhar) {
    hlyin (b == 0) {
        pyan (0, amhar("division by zero"));
    }
    pyan (a / b, bhala);
}

loke apply_twice(loke(kain) -> kain fn, kain x) -> kain {
    kain first = fn(x);
    pyan fn(first);
}

loke main() -> kain {
    Counter c = Counter { value: 1 };
    c.value = 3;

    su<kain> nums = [1, 2, 3];
    nums[0] = 10;
    nums.push(4);

    twe<sar, kain> prices = {"tea": 500};
    prices["coffee"] = 800;

    kain i = 0;
    pat (i < 10) {
        i = i + 1;
        hlyin (i == 2) {
            shar;
        }
        hlyin (i == 5) {
            yut;
        }
    }

    pat (kain idx, kain item) htae nums {
        pya(idx);
        pya(item);
    }

    kain r, amhar err = safe_div(10, 0);
    hlyin (err != bhala) {
        pya("div error");
    } mo {
        pya(r);
    }

    kain out = apply_twice(loke(kain v) -> kain {
        pyan v + c.value;
    }, 2);
    pya(out);

    pyan 0;
}
