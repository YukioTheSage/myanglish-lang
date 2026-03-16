// Phase 2 Example 1
// json.encode + json.decode.

yu "json";

loke main() -> kain {
    twe<sar, kain> order = {"price": 5000, "qty": 2};
    sar payload, amhar encode_err = json.encode(order);

    hlyin (encode_err != bhala) {
        pya(encode_err);
        pyan 1;
    }

    pya(payload);
    twe<sar, kain> parsed, amhar decode_err = json.decode(payload);

    hlyin (decode_err != bhala) {
        pya(decode_err);
        pyan 1;
    }

    pya(parsed["price"]);
    pyan 0;
}
