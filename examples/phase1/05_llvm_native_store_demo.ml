// LLVM native compiler demo.
// Uses only the Phase 1 surface currently supported by --target llvm.

pone Order {
    sar customer;
    kain subtotal;
}

loke safe_bonus(kain subtotal) -> (kain, amhar) {
    hlyin (subtotal < 1000) {
        pyan (0, amhar("subtotal too low"));
    }

    pyan (subtotal / 10, bhala);
}

loke apply_adjustment(loke(kain) -> kain adjust, kain amount) -> kain {
    pyan adjust(amount);
}

loke main() -> kain {
    Order order = Order { customer: "Aye Aye", subtotal: 5000 };
    order.subtotal = 5400;

    su<sar> items = ["tea", "rice"];
    items.push("cake");

    twe<sar, kain> prices = {"tea": 1500};
    prices["rice"] = 2500;

    kain bonus, amhar err = safe_bonus(order.subtotal);
    hlyin (err != bhala) {
        pya(err);
        pyan 1;
    }

    kain service_fee = 300;
    kain final_total = apply_adjustment(loke(kain amount) -> kain {
            pyan amount - service_fee;
        }, order.subtotal + bonus);

    pya(order.customer);
    pya(items);
    pya(prices["rice"]);
    pya(bonus);
    pya(final_total);
    pyan 0;
}
