yu "kainn/http";
yu "json";

loke hello(http.Request req, http.ResponseWriter w) -> amhar {
    sar ua = req.header("User-Agent");
    sar name = req.query("name");
    hlyin (name == "") {
        name = "world";
    }

    amhar status_err = w.status(200);
    hlyin (status_err != bhala) {
        pyan status_err;
    }

    amhar header_err = w.header("X-UA", ua);
    hlyin (header_err != bhala) {
        pyan header_err;
    }

    twe<sar, kain> payload = {"ok": 1, "name_len": ashay(name)};
    amhar json_err = w.json(payload);
    hlyin (json_err != bhala) {
        pyan w.write("hello " + name);
    }
    pyan bhala;
}

loke main() -> kain {
    amhar reg_err = http.handle("/", hello);
    hlyin (reg_err != bhala) {
        pya(reg_err);
        pyan 1;
    }

    amhar listen_err = http.listen(":8080");
    pya(listen_err);
    pyan 0;
}
