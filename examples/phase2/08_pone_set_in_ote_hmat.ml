// Phase 2 Example 4
// pone_set.pon_san + in_ote stdin/stdout + hmat logging levels.

yu "pone_set";
yu "in_ote";
yu "hmat";

loke main() -> kain {
    amhar info_err = hmat.mhat_chet("Phase 2 stdlib demo started.");
    hlyin (info_err != bhala) {
        pya(info_err);
        pyan 1;
    }

    sar prompt = pone_set.pon_san("%s", "Enter your name: ");
    amhar write_prompt_err = in_ote.htote_yay(prompt);
    hlyin (write_prompt_err != bhala) {
        pya(write_prompt_err);
        pyan 1;
    }

    sar name, amhar read_err = in_ote.twin_phat();
    hlyin (read_err != bhala) {
        amhar warn_err = hmat.mhat_thati("Failed to read stdin line.");
        pya(warn_err);
        pya(read_err);
        pyan 1;
    }

    sar message = pone_set.pon_san("Mingalabar %s\n", name);
    amhar write_msg_err = in_ote.htote_yay(message);
    hlyin (write_msg_err != bhala) {
        pya(write_msg_err);
        pyan 1;
    }

    amhar err_level_log = hmat.mhat_amhar("Demo complete (sample error-level log).");
    hlyin (err_level_log != bhala) {
        pya(err_level_log);
        pyan 1;
    }

    pyan 0;
}
