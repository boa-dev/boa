use crate::{
    builtins::temporal::{self, TemporalFields},
    JsNativeError, JsResult, JsString,
};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct IsoDateRecord {
    year: i32,
    month: i32,
    day: i32,
}

// TODO: determine whether the below is neccessary.
impl IsoDateRecord {
    pub(crate) const fn year(&self) -> i32 {
        self.year
    }
    pub(crate) const fn month(&self) -> i32 {
        self.month
    }
    pub(crate) const fn day(&self) -> i32 {
        self.day
    }
}

impl IsoDateRecord {
    /// 3.5.2 CreateISODateRecord
    pub(crate) fn new(year: i32, month: i32, day: i32) -> Self {
        Self { year, month, day }
    }

    /// 3.5.6 RegulateISODate
    pub(crate) fn from_unregulated(
        year: i32,
        month: i32,
        day: i32,
        overflow: &JsString,
    ) -> JsResult<Self> {
        match overflow.to_std_string_escaped().as_str() {
            "constrain" => {
                let m = month.clamp(1, 12);
                let days_in_month = temporal::calendar::utils::iso_days_in_month(year, month);
                let d = day.clamp(1, days_in_month);
                Ok(Self::new(year, m, d))
            }
            "reject" => {
                let date = Self::new(year, month, day);
                if !date.is_valid() {
                    return Err(JsNativeError::range()
                        .with_message("not a valid ISO date.")
                        .into());
                }
                Ok(date)
            }
            _ => unreachable!(),
        }
    }

    /// 12.2.35 ISODateFromFields ( fields, overflow )
    ///
    /// Note: fields.month must be resolved prior to using `from_temporal_fields`
    pub(crate) fn from_temporal_fields(
        fields: TemporalFields,
        overflow: &JsString,
    ) -> JsResult<Self> {
        Self::from_unregulated(
            fields.year().expect("Cannot fail per spec"),
            fields.month().expect("cannot fail after resolution"),
            fields.day().expect("cannot fail per spec"),
            overflow,
        )
    }

    pub(crate) fn month_day_from_temporal_fields(
        fields: TemporalFields,
        overflow: &JsString,
    ) -> JsResult<Self> {
        match fields.year() {
            Some(year) => Self::from_unregulated(
                year,
                fields.month().expect("month must exist."),
                fields.day().expect("cannot fail per spec"),
                overflow,
            ),
            None => Self::from_unregulated(
                1972,
                fields.month().expect("cannot fail per spec"),
                fields.day().expect("cannot fail per spec."),
                overflow,
            ),
        }
    }

    /// 3.5.5 DifferenceISODate
    pub(crate) fn diff_iso_date(
        &self,
        o: Self,
        largest_unit: &JsString,
    ) -> temporal::duration::DurationRecord {
        todo!()
    }

    /// 3.5.7 IsValidISODate
    pub(crate) fn is_valid(&self) -> bool {
        if self.month < 1 || self.month > 12 {
            return false;
        }

        let days_in_month =
            temporal::calendar::utils::iso_days_in_month(self.year, i32::from(self.month));

        if self.day < 1 || self.day > days_in_month {
            return false;
        }
        true
    }

    /// 13.2 IsoDateToEpochDays
    pub(crate) fn as_epoch_days(&self) -> i32 {
        // 1. Let resolvedYear be year + floor(month / 12).
        let resolved_year = self.year + (f64::from(self.month) / 12_f64).floor() as i32;
        // 2. Let resolvedMonth be month modulo 12.
        let resolved_month = self.month % 12;

        // 3. Find a time t such that EpochTimeToEpochYear(t) is resolvedYear, EpochTimeToMonthInYear(t) is resolvedMonth, and EpochTimeToDate(t) is 1.
        let year_t = temporal::date_equations::epoch_time_for_year(resolved_year);
        let month_t = temporal::date_equations::epoch_time_for_month_given_year(
            resolved_month,
            resolved_year,
        );

        // 4. Return EpochTimeToDayNumber(t) + date - 1.
        temporal::date_equations::epoch_time_to_day_number(year_t + month_t) + self.day - 1
    }

    // NOTE: Implementing as mut self so balance is applied to self, but TBD.
    /// 3.5.8 BalanceIsoDate
    pub(crate) fn balance(&mut self) {
        let epoch_days = self.as_epoch_days();
        let ms = temporal::epoch_days_to_epoch_ms(epoch_days, 0);

        // Balance current values
        self.year = temporal::date_equations::epoch_time_to_epoch_year(ms);
        self.month = temporal::date_equations::epoch_time_to_month_in_year(ms);
        self.day = temporal::date_equations::epoch_time_to_date(ms);
    }
}
