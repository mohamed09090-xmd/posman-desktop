use time::{Date, Month, OffsetDateTime};

use super::error::{Phase05Error, Phase05Result};

pub const PERCENT_SCALE: i128 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PricingResult {
    pub sale_ht_scaled: i64,
    pub discounted_ht_scaled: i64,
    pub tax_scaled: i64,
    pub ttc_scaled: i64,
}

pub fn calculate_pricing(
    cost_ht_scaled: i64,
    markup_rate_scaled: i64,
    discount_rate_scaled: i64,
    tax_rate_scaled: i64,
) -> Phase05Result<PricingResult> {
    if cost_ht_scaled < 0
        || !(0..=1_000_000).contains(&markup_rate_scaled)
        || !(0..=1_000_000).contains(&discount_rate_scaled)
        || !(0..=1_000_000).contains(&tax_rate_scaled)
    {
        return Err(Phase05Error::new(
            "PRICING_INPUT_INVALID",
            "The price or percentage is outside the supported range.",
        ));
    }
    let cost = i128::from(cost_ht_scaled);
    let sale = mul_div_half_up(
        cost,
        PERCENT_SCALE
            .checked_add(i128::from(markup_rate_scaled))
            .ok_or_else(overflow)?,
        PERCENT_SCALE,
    )?;
    let discounted = mul_div_half_up(
        sale,
        PERCENT_SCALE
            .checked_sub(i128::from(discount_rate_scaled))
            .ok_or_else(overflow)?,
        PERCENT_SCALE,
    )?;
    let tax = mul_div_half_up(discounted, i128::from(tax_rate_scaled), PERCENT_SCALE)?;
    let ttc = discounted.checked_add(tax).ok_or_else(overflow)?;
    Ok(PricingResult {
        sale_ht_scaled: checked_i64(sale)?,
        discounted_ht_scaled: checked_i64(discounted)?,
        tax_scaled: checked_i64(tax)?,
        ttc_scaled: checked_i64(ttc)?,
    })
}

fn mul_div_half_up(left: i128, right: i128, denominator: i128) -> Phase05Result<i128> {
    if denominator <= 0 || left < 0 || right < 0 {
        return Err(Phase05Error::new(
            "PRICING_INPUT_INVALID",
            "The pricing calculation received an invalid value.",
        ));
    }
    let product = left.checked_mul(right).ok_or_else(overflow)?;
    let half = denominator / 2;
    product
        .checked_add(half)
        .and_then(|value| value.checked_div(denominator))
        .ok_or_else(overflow)
}

fn checked_i64(value: i128) -> Phase05Result<i64> {
    i64::try_from(value).map_err(|_| overflow())
}

fn overflow() -> Phase05Error {
    Phase05Error::new(
        "NUMERIC_OVERFLOW",
        "The calculation is too large to store safely.",
    )
}

pub fn current_device_date() -> Date {
    OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
        .date()
}

pub fn parse_date(value: &str) -> Phase05Result<Date> {
    let mut parts = value.split('-');
    let year = parts.next().and_then(|part| part.parse::<i32>().ok());
    let month = parts.next().and_then(|part| part.parse::<u8>().ok());
    let day = parts.next().and_then(|part| part.parse::<u8>().ok());
    if parts.next().is_some() {
        return Err(Phase05Error::invalid("date"));
    }
    let month = month
        .and_then(|value| Month::try_from(value).ok())
        .ok_or_else(|| Phase05Error::invalid("date"))?;
    Date::from_calendar_date(
        year.ok_or_else(|| Phase05Error::invalid("date"))?,
        month,
        day.ok_or_else(|| Phase05Error::invalid("date"))?,
    )
    .map_err(|_| Phase05Error::invalid("date"))
}

pub fn format_date(date: Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}

pub fn fiscal_year_default() -> (String, String) {
    let year = current_device_date().year();
    (format!("{year:04}-01-01"), format!("{year:04}-12-31"))
}

pub fn fiscal_periods(starts_on: &str, ends_on: &str) -> Phase05Result<Vec<(String, String)>> {
    let start = parse_date(starts_on)?;
    let end = parse_date(ends_on)?;
    let expected_end = add_months(start, 12)?
        .previous_day()
        .ok_or_else(|| Phase05Error::invalid("fiscalYear"))?;
    if end != expected_end {
        return Err(Phase05Error::new(
            "FISCAL_YEAR_INVALID",
            "The fiscal year must contain exactly twelve full months.",
        ));
    }
    let mut periods = Vec::with_capacity(12);
    for offset in 0..12 {
        let period_start = add_months(start, offset)?;
        let period_end = add_months(start, offset + 1)?
            .previous_day()
            .ok_or_else(|| Phase05Error::invalid("fiscalYear"))?;
        periods.push((format_date(period_start), format_date(period_end)));
    }
    Ok(periods)
}

fn add_months(date: Date, months: i32) -> Phase05Result<Date> {
    let base = date
        .year()
        .checked_mul(12)
        .and_then(|value| value.checked_add(i32::from(date.month() as u8) - 1))
        .ok_or_else(|| Phase05Error::invalid("fiscalYear"))?;
    let target = base
        .checked_add(months)
        .ok_or_else(|| Phase05Error::invalid("fiscalYear"))?;
    let year = target.div_euclid(12);
    let month_number =
        u8::try_from(target.rem_euclid(12) + 1).map_err(|_| Phase05Error::invalid("fiscalYear"))?;
    let month = Month::try_from(month_number).map_err(|_| Phase05Error::invalid("fiscalYear"))?;
    let day = date.day().min(days_in_month(year, month));
    Date::from_calendar_date(year, month, day).map_err(|_| Phase05Error::invalid("fiscalYear"))
}

fn days_in_month(year: i32, month: Month) -> u8 {
    match month {
        Month::January
        | Month::March
        | Month::May
        | Month::July
        | Month::August
        | Month::October
        | Month::December => 31,
        Month::April | Month::June | Month::September | Month::November => 30,
        Month::February if is_leap_year(year) => 29,
        Month::February => 28,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_pricing_vector_is_exact() {
        let result = calculate_pricing(1_000_000, 200_000, 100_000, 190_000).expect("pricing");
        assert_eq!(
            result,
            PricingResult {
                sale_ht_scaled: 1_200_000,
                discounted_ht_scaled: 1_080_000,
                tax_scaled: 205_200,
                ttc_scaled: 1_285_200,
            }
        );
    }

    #[test]
    fn pricing_rounds_half_up_and_rejects_overflow() {
        assert_eq!(
            calculate_pricing(1, 500_000, 0, 0)
                .expect("round")
                .sale_ht_scaled,
            2
        );
        assert_eq!(
            calculate_pricing(i64::MAX, 1_000_000, 0, 1_000_000)
                .expect_err("overflow")
                .code,
            "NUMERIC_OVERFLOW"
        );
    }

    #[test]
    fn fiscal_periods_cover_leap_year_without_gaps() {
        let periods = fiscal_periods("2024-01-01", "2024-12-31").expect("periods");
        assert_eq!(periods.len(), 12);
        assert_eq!(periods[1], ("2024-02-01".into(), "2024-02-29".into()));
        for pair in periods.windows(2) {
            assert_eq!(
                parse_date(&pair[0].1).expect("end").next_day(),
                Some(parse_date(&pair[1].0).expect("start"))
            );
        }
    }
}
