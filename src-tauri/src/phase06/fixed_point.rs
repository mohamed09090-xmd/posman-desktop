use super::error::{Phase06Error, Phase06Result};

pub const QUANTITY_SCALE: i128 = 1_000_000;
pub const UNIT_VALUE_SCALE: i128 = 10_000;
pub const MONEY_SCALE: i128 = 100;
pub const RATE_SCALE: i128 = 10_000;
pub const PERCENT_DENOMINATOR: i128 = 100 * RATE_SCALE;

fn checked_i64(value: i128) -> Phase06Result<i64> {
    i64::try_from(value).map_err(|_| Phase06Error::numeric_overflow())
}

pub fn round_half_up_non_negative(numerator: i128, denominator: i128) -> Phase06Result<i128> {
    if numerator < 0 || denominator <= 0 {
        return Err(Phase06Error::invalid("fixedPoint"));
    }
    numerator
        .checked_add(denominator / 2)
        .map(|value| value / denominator)
        .ok_or_else(Phase06Error::numeric_overflow)
}

pub fn extended_cost_minor(quantity_scaled: i64, unit_cost_scaled: i64) -> Phase06Result<i64> {
    if unit_cost_scaled < 0 {
        return Err(Phase06Error::invalid("unitCostScaled"));
    }
    let numerator = i128::from(quantity_scaled)
        .abs()
        .checked_mul(i128::from(unit_cost_scaled))
        .ok_or_else(Phase06Error::numeric_overflow)?;
    checked_i64(round_half_up_non_negative(
        numerator,
        QUANTITY_SCALE * UNIT_VALUE_SCALE / MONEY_SCALE,
    )?)
}

pub fn weighted_average_cost(
    old_quantity_scaled: i64,
    old_average_cost_scaled: i64,
    received_quantity_scaled: i64,
    receipt_cost_scaled: i64,
) -> Phase06Result<i64> {
    if received_quantity_scaled <= 0 || old_average_cost_scaled < 0 || receipt_cost_scaled < 0 {
        return Err(Phase06Error::invalid("cost"));
    }

    let new_quantity = old_quantity_scaled
        .checked_add(received_quantity_scaled)
        .ok_or_else(Phase06Error::numeric_overflow)?;

    // Preserve the last known CUMP while the stock remains zero or negative.
    if new_quantity <= 0 {
        return Ok(old_average_cost_scaled);
    }

    // A receipt that restores a zero/negative balance to positive establishes
    // the new CUMP using only that restoring batch.
    if old_quantity_scaled <= 0 {
        return Ok(receipt_cost_scaled);
    }

    let old_value = i128::from(old_quantity_scaled)
        .checked_mul(i128::from(old_average_cost_scaled))
        .ok_or_else(Phase06Error::numeric_overflow)?;
    let receipt_value = i128::from(received_quantity_scaled)
        .checked_mul(i128::from(receipt_cost_scaled))
        .ok_or_else(Phase06Error::numeric_overflow)?;
    let weighted_value = old_value
        .checked_add(receipt_value)
        .ok_or_else(Phase06Error::numeric_overflow)?;

    checked_i64(round_half_up_non_negative(
        weighted_value,
        i128::from(new_quantity),
    )?)
}

pub fn line_totals(
    quantity_scaled: i64,
    unit_price_scaled: i64,
    discount_rate_scaled: i64,
    tax_rate_scaled: i64,
) -> Phase06Result<(i64, i64, i64, i64)> {
    if quantity_scaled <= 0
        || unit_price_scaled < 0
        || !(0..=1_000_000).contains(&discount_rate_scaled)
        || !(0..=1_000_000).contains(&tax_rate_scaled)
    {
        return Err(Phase06Error::invalid("lineTotals"));
    }

    let gross_numerator = i128::from(quantity_scaled)
        .checked_mul(i128::from(unit_price_scaled))
        .ok_or_else(Phase06Error::numeric_overflow)?;
    let gross_minor = round_half_up_non_negative(
        gross_numerator,
        QUANTITY_SCALE * UNIT_VALUE_SCALE / MONEY_SCALE,
    )?;
    let discount_numerator = gross_minor
        .checked_mul(i128::from(discount_rate_scaled))
        .ok_or_else(Phase06Error::numeric_overflow)?;
    let discount_minor = round_half_up_non_negative(discount_numerator, PERCENT_DENOMINATOR)?;
    let net_ht_minor = gross_minor
        .checked_sub(discount_minor)
        .ok_or_else(|| Phase06Error::invalid("discountRateScaled"))?;
    let tax_numerator = net_ht_minor
        .checked_mul(i128::from(tax_rate_scaled))
        .ok_or_else(Phase06Error::numeric_overflow)?;
    let tax_minor = round_half_up_non_negative(tax_numerator, PERCENT_DENOMINATOR)?;
    let total_ttc_minor = net_ht_minor
        .checked_add(tax_minor)
        .ok_or_else(Phase06Error::numeric_overflow)?;

    Ok((
        checked_i64(discount_minor)?,
        checked_i64(net_ht_minor)?,
        checked_i64(tax_minor)?,
        checked_i64(total_ttc_minor)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_point_boundaries_are_half_up() {
        assert_eq!(round_half_up_non_negative(5, 10).unwrap(), 1);
        assert_eq!(round_half_up_non_negative(4, 10).unwrap(), 0);
        assert_eq!(extended_cost_minor(1_500_000, 12_345).unwrap(), 1_852);
    }

    #[test]
    fn weighted_average_is_exact_and_deterministic() {
        assert_eq!(
            weighted_average_cost(10_000_000, 10_000, 5_000_000, 16_000).unwrap(),
            12_000
        );
        assert_eq!(
            weighted_average_cost(0, 99_999, 1_000_000, 12_345).unwrap(),
            12_345
        );
        assert_eq!(
            weighted_average_cost(-2_000_000, 10_000, 3_000_000, 15_000).unwrap(),
            15_000
        );
        assert_eq!(
            weighted_average_cost(-4_000_000, 10_000, 3_000_000, 15_000).unwrap(),
            10_000
        );
    }
}
