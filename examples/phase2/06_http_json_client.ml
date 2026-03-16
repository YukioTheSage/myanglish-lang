// Phase 2 Example 3
// HTTP client usage with JSON payload.

yu "kainn/http";
yu "json";

loke main() -> kain {
    twe<sar, kain> payload_map = {"order_id": 123, "amount": 5000};
    sar payload, amhar encode_err = json.encode(payload_map);

    hlyin (encode_err != bhala) {
        pya(encode_err);
        pyan 1;
    }

    http.Response post_res, amhar post_err = http.post("https://httpbin.org/post", payload);

    hlyin (post_err != bhala) {
        pya(post_err);
        pyan 1;
    }

    http.Response get_res, amhar get_err = http.get("https://httpbin.org/status/200");

    hlyin (get_err != bhala) {
        pya(get_err);
        pyan 1;
    }

    pya(post_res.status);
    pya(get_res.status);
    pyan 0;
}
