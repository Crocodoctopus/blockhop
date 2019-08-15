use std::time::Instant;

lazy_static! {
    static ref PROGRAM_START: Instant = { Instant::now() };
}

#[allow(dead_code)]
pub fn get_milliseconds_as_u32() -> u32 {
    let start = *PROGRAM_START;
    let dur = Instant::now().duration_since(start);
    (dur.as_secs() * 1_000 + dur.subsec_nanos() as u64 / 1_000_000) as u32
}

#[allow(dead_code)]
pub fn get_milliseconds_as_u64() -> u64 {
    let start = *PROGRAM_START;
    let dur = Instant::now().duration_since(start);
    dur.as_secs() * 1_000 + dur.subsec_nanos() as u64 / 1_000_000
}

#[allow(dead_code)]
pub fn get_microseconds_as_u32() -> u32 {
    let start = *PROGRAM_START;
    let dur = Instant::now().duration_since(start);
    (dur.as_secs() * 1_000_000 + dur.subsec_nanos() as u64 / 1_000) as u32
}

#[allow(dead_code)]
pub fn get_microseconds_as_u64() -> u64 {
    let start = *PROGRAM_START;
    let dur = Instant::now().duration_since(start);
    dur.as_secs() * 1_000_000 + dur.subsec_nanos() as u64 / 1_000
}
