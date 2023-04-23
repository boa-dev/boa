pub(crate) struct TimeZone {
    initialized_temporal_time_zone: bool,
    identifier: String,
    offset_nanoseconds: u64,
}
