// Phase 2 Example 2
// file.read/file.write + su_nit.env/su_nit.args.

yu "file";
yu "su_nit";

loke main() -> kain {
    sar token = su_nit.env("BOT_TOKEN");
    su<sar> args = su_nit.args();
    amhar write_err = file.write("phase2_note.txt", "token=" + token);

    hlyin (write_err != bhala) {
        pya(write_err);
        pyan 1;
    }

    sar content, amhar read_err = file.read("phase2_note.txt");

    hlyin (read_err != bhala) {
        pya(read_err);
        pyan 1;
    }

    pya(content);
    pya(args);
    pyan 0;
}
