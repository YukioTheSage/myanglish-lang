// Phase 1 Example 3
// Tuple error handling + closure callback.

loke safe_div(kain a, kain b) -> (kain, amhar) {
    hlyin (b == 0) {
        pyan (0, amhar("division by zero"));
    }

    pyan (a / b, bhala);
}

loke apply_fee(loke(kain) -> kain transform, kain amount) -> kain {
    pyan transform(amount);
}

loke main() -> kain {
    kain result, amhar err = safe_div(10, 0);

    hlyin (err != bhala) {
        pya("Cannot divide");
        pya(err);
    } mo {
        pya(result);
    }

    kain fee = 250;
    kain after_fee = apply_fee(loke(kain amount) -> kain {
            pyan amount - fee;
        }, 5000);
    pya(after_fee);
    pyan 0;
}
