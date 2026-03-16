loke producer(laung<kain> ch) -> amhar {
    naut_sone ch.close();
    ch.send(100);
    ch.send(200);
    ch.send(300);
    pyan bhala;
}

loke main() -> kain {
    laung<kain> ch = laung<kain>(2);
    kyoe producer(ch);

    kain first = ch.recv();
    kain second = ch.recv();
    kain third = ch.recv();

    pya(first);
    pya(second);
    pya(third);
    pyan 0;
}
