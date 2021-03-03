use std::io::Write;

use env_logger;

#[allow(unused_must_use)]
pub fn init_logger(test_mode: bool) {
    env_logger::builder()
        .format(|buf, record| writeln!(buf,
                                       "[{} {} {}:{}] {}",
                                       buf.timestamp(),
                                       record.level(),
                                       record.file().unwrap_or(record.target()),
                                       record.line().unwrap_or(0),
                                       record.args()))
        .is_test(test_mode)
        .try_init();
}
