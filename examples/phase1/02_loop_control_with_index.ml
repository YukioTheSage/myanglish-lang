// Phase 1 Example 2
// Break/continue + for-in loop with index.
// Skip first item
// Stop once cost is too large

loke main() -> kain {
    su<kain> costs = [
        1200,
        2500,
        3900,
        4200
    ];
    kain total = 0;

    pat (idx, cost) htae costs {
        hlyin (idx == 0) {
            shar;
        }

        hlyin (cost > 4000) {
            yut;
        }

        total = total + cost;
    }

    pya(total);
    pyan 0;
}
