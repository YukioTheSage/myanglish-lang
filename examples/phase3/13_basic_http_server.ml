yu "kainn/http";

loke hello(http.Request req, http.ResponseWriter w) -> amhar {
    pyan w.write("Mingalabar from M-Lang!");
}

loke main() -> kain {
    amhar handle_err = http.handle("/", hello);

    hlyin (handle_err != bhala) {
        pya(handle_err);
        pyan 1;
    }

    pya("Server running on :18081");
    amhar listen_err = http.listen(":18081");

    hlyin (listen_err != bhala) {
        pya(listen_err);
        pyan 1;
    }

    pyan 0;
}
