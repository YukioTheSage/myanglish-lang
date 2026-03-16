yu "kainn";

loke main() -> kain {
    kainn.TCPListener listener, amhar listen_err = kainn.tcp_listen(":9000");
    hlyin (listen_err != bhala) {
        pya(listen_err);
        pyan 1;
    }

    kainn.TCPConn conn, amhar accept_err = listener.accept();
    hlyin (accept_err != bhala) {
        pya(accept_err);
        listener.close();
        pyan 1;
    }

    sar msg, amhar read_err = conn.read();
    hlyin (read_err == bhala) {
        amhar write_err = conn.write(msg);
        pya(write_err);
    } mo {
        pya(read_err);
    }

    amhar close_conn_err = conn.close();
    amhar close_listener_err = listener.close();
    pya(close_conn_err);
    pya(close_listener_err);
    pyan 0;
}
