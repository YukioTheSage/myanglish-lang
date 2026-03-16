// Phase 1 Example 1
// Struct field assignment + array/map mutation.
// Struct field assignment
// Array index assignment + push
// Map index assignment

pone Cart {
    sar customer;
    kain item_count;
}

loke main() -> kain {
    Cart cart = Cart { customer: "Aye Aye", item_count: 1 };
    cart.customer = "Ko Ko";
    cart.item_count = 2;
    su<sar> items = ["tea", "coffee"];
    items[1] = "latte";
    items.push("cake");
    twe<sar, kain> prices = {"tea": 1500};
    prices["coffee"] = 2200;
    pya(cart.customer);
    pya(cart.item_count);
    pya(items);
    pya(prices["coffee"]);
    pyan 0;
}
