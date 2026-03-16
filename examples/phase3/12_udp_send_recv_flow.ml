yu "kainn";

loke main() -> kain {
    kainn.UDPConn udp, amhar bind_err = kainn.udp_bind(":9100");
    hlyin (bind_err != bhala) {
        pya(bind_err);
        pyan 1;
    }

    sar packet, sar from_addr, amhar recv_err = udp.recv();
    hlyin (recv_err == bhala) {
        amhar send_err = udp.send_to(from_addr, packet);
        pya(send_err);
    } mo {
        pya(recv_err);
    }

    amhar close_err = udp.close();
    pya(close_err);
    pyan 0;
}
