use crate::{
    builtins::{
        date::utils,
        temporal::{self, plain_date::iso::IsoDateRecord},
    },
    JsBigInt,
};

#[derive(Default, Debug, Clone)]
pub(crate) struct IsoDateTimeRecord {
    iso_date: IsoDateRecord,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
    microsecond: i32,
    nanosecond: i32,
}

impl IsoDateTimeRecord {
    pub(crate) const fn iso_date(&self) -> IsoDateRecord {
        self.iso_date
    }
}

// ==== `IsoDateTimeRecord` methods ====

impl IsoDateTimeRecord {
    pub(crate) const fn with_date(mut self, year: i32, month: i32, day: i32) -> Self {
        let iso_date = IsoDateRecord::new(year, month, day);
        self.iso_date = iso_date;
        self
    }

    pub(crate) const fn with_time(
        mut self,
        hour: i32,
        minute: i32,
        second: i32,
        ms: i32,
        mis: i32,
        ns: i32,
    ) -> Self {
        self.hour = hour;
        self.minute = minute;
        self.second = second;
        self.millisecond = ms;
        self.microsecond = mis;
        self.nanosecond = ns;
        self
    }

    /// 5.5.1 `ISODateTimeWithinLimits ( year, month, day, hour, minute, second, millisecond, microsecond, nanosecond )`
    pub(crate) fn is_valid(&self) -> bool {
        self.iso_date.is_valid();
        let ns = self.get_utc_epoch_ns(None).to_f64();

        if ns <= temporal::ns_min_instant().to_f64() - (temporal::NS_PER_DAY as f64)
            || ns >= temporal::ns_max_instant().to_f64() + (temporal::NS_PER_DAY as f64)
        {
            return false;
        }
        true
    }

    /// 14.8.1 `GetUTCEpochNanoseconds`
    pub(crate) fn get_utc_epoch_ns(&self, offset_ns: Option<i64>) -> JsBigInt {
        let day = utils::make_day(
            i64::from(self.iso_date.year()),
            i64::from(self.iso_date.month()),
            i64::from(self.iso_date.day()),
        )
        .unwrap_or_default();
        let time = utils::make_time(
            i64::from(self.hour),
            i64::from(self.minute),
            i64::from(self.second),
            i64::from(self.millisecond),
        )
        .unwrap_or_default();

        let ms = utils::make_date(day, time).unwrap_or_default();

        let epoch_ns = match offset_ns {
            Some(offset) if offset != 0 => {
                let ns = (ms * 1_000_000_i64)
                    + (i64::from(self.microsecond) * 1_000_i64)
                    + i64::from(self.nanosecond);
                ns - offset
            }
            _ => {
                (ms * 1_000_000_i64)
                    + (i64::from(self.microsecond) * 1_000_i64)
                    + i64::from(self.nanosecond)
            }
        };

        JsBigInt::from(epoch_ns)
    }
}
