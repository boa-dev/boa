#[cfg(test)]
mod tests;

mod utils;

use icu_collator::CaseFirst;
use icu_datetime::options::preferences::HourCycle;
use icu_locid::Locale;
pub(super) use utils::*;

use crate::JsString;

pub(crate) struct JsLocale {
    locale: Locale,
    calendar: JsString,
    hour_cycle: HourCycle,
    case_first: CaseFirst,
    numeric: bool,
    numbering: JsString,
}
