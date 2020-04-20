fn main() {
    flexi_logger::Logger::with_str("trace")
        .format(flexi_logger::colored_opt_format)
        .start()
        .unwrap();

    log::trace!("Starting sample_sandbox");

    mat_engine::temporary_main();
}
