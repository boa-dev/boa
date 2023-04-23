#![allow(dead_code)]

#[derive(Debug)]
pub struct TimeZone {
    pub(crate) initialized_temporal_time_zone: bool,
    pub(crate) identifier: String,
    pub(crate) offset_nanoseconds: Option<i64>,
}
