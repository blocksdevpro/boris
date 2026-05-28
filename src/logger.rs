use env_logger::Builder;

pub fn setup_logger() {
    let mut builder = Builder::new();
    builder.filter_level(log::LevelFilter::Info);

    builder.filter_module("boris", log::LevelFilter::Debug);
    builder.init();
}
