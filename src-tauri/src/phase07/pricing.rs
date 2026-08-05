use crate::phase06::{
    error::{Phase06Error, Phase06Result},
    fixed_point::{
        round_half_up_non_negative, MONEY_SCALE, PERCENT_DENOMINATOR, QUANTITY_SCALE,
        UNIT_VALUE_SCALE,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PricedLine {
    pub line_discount_minor: i64,
    pub before_header_ht_minor: i64,
    pub allocated_header_discount_minor: i64,
    pub taxable_ht_minor: i64,
    pub tax_minor: i64,
    pub ttc_minor: i64,
}

fn checked_i64(value: i128) -> Phase06Result<i64> {
    i64::try_from(value).map_err(|_| Phase06Error::numeric_overflow())
}

pub(crate) fn base_line(
    quantity_scaled: i64,
    unit_price_scaled: i64,
    discount_rate_scaled: i64,
    tax_rate_scaled: i64,
    price_mode: &str,
) -> Phase06Result<PricedLine> {
    if quantity_scaled <= 0
        || unit_price_scaled < 0
        || !(0..=1_000_000).contains(&discount_rate_scaled)
        || !(0..=1_000_000).contains(&tax_rate_scaled)
        || !matches!(price_mode, "HT" | "TTC")
    {
        return Err(Phase06Error::invalid("salesLine"));
    }
    let gross = round_half_up_non_negative(
        i128::from(quantity_scaled)
            .checked_mul(i128::from(unit_price_scaled))
            .ok_or_else(Phase06Error::numeric_overflow)?,
        QUANTITY_SCALE * UNIT_VALUE_SCALE / MONEY_SCALE,
    )?;
    let discount = round_half_up_non_negative(
        gross
            .checked_mul(i128::from(discount_rate_scaled))
            .ok_or_else(Phase06Error::numeric_overflow)?,
        PERCENT_DENOMINATOR,
    )?;
    let discounted = gross
        .checked_sub(discount)
        .ok_or_else(|| Phase06Error::invalid("discountRateScaled"))?;
    let before_header_ht = if price_mode == "HT" {
        discounted
    } else {
        round_half_up_non_negative(
            discounted
                .checked_mul(PERCENT_DENOMINATOR)
                .ok_or_else(Phase06Error::numeric_overflow)?,
            PERCENT_DENOMINATOR
                .checked_add(i128::from(tax_rate_scaled))
                .ok_or_else(Phase06Error::numeric_overflow)?,
        )?
    };
    let tax = round_half_up_non_negative(
        before_header_ht
            .checked_mul(i128::from(tax_rate_scaled))
            .ok_or_else(Phase06Error::numeric_overflow)?,
        PERCENT_DENOMINATOR,
    )?;
    Ok(PricedLine {
        line_discount_minor: checked_i64(discount)?,
        before_header_ht_minor: checked_i64(before_header_ht)?,
        allocated_header_discount_minor: 0,
        taxable_ht_minor: checked_i64(before_header_ht)?,
        tax_minor: checked_i64(tax)?,
        ttc_minor: checked_i64(
            before_header_ht
                .checked_add(tax)
                .ok_or_else(Phase06Error::numeric_overflow)?,
        )?,
    })
}

pub(crate) fn allocate_header_discount(
    lines: &mut [(PricedLine, i64)],
    header_rate_scaled: i64,
) -> Phase06Result<(i64, i64, i64, i64)> {
    if !(0..=1_000_000).contains(&header_rate_scaled) || lines.is_empty() {
        return Err(Phase06Error::invalid("headerDiscountRateScaled"));
    }
    let base_total = lines.iter().try_fold(0_i128, |sum, (line, _)| {
        sum.checked_add(i128::from(line.before_header_ht_minor))
            .ok_or_else(Phase06Error::numeric_overflow)
    })?;
    let header_total = round_half_up_non_negative(
        base_total
            .checked_mul(i128::from(header_rate_scaled))
            .ok_or_else(Phase06Error::numeric_overflow)?,
        PERCENT_DENOMINATOR,
    )?;
    let mut allocated = 0_i128;
    let last = lines.len() - 1;
    for (index, (line, tax_rate)) in lines.iter_mut().enumerate() {
        let share = if index == last {
            header_total
                .checked_sub(allocated)
                .ok_or_else(Phase06Error::numeric_overflow)?
        } else if base_total == 0 {
            0
        } else {
            round_half_up_non_negative(
                header_total
                    .checked_mul(i128::from(line.before_header_ht_minor))
                    .ok_or_else(Phase06Error::numeric_overflow)?,
                base_total,
            )?
        };
        allocated = allocated
            .checked_add(share)
            .ok_or_else(Phase06Error::numeric_overflow)?;
        let taxable = i128::from(line.before_header_ht_minor)
            .checked_sub(share)
            .ok_or_else(|| Phase06Error::invalid("headerDiscountRateScaled"))?;
        let tax = round_half_up_non_negative(
            taxable
                .checked_mul(i128::from(*tax_rate))
                .ok_or_else(Phase06Error::numeric_overflow)?,
            PERCENT_DENOMINATOR,
        )?;
        line.allocated_header_discount_minor = checked_i64(share)?;
        line.taxable_ht_minor = checked_i64(taxable)?;
        line.tax_minor = checked_i64(tax)?;
        line.ttc_minor = checked_i64(
            taxable
                .checked_add(tax)
                .ok_or_else(Phase06Error::numeric_overflow)?,
        )?;
    }
    let ht = lines.iter().try_fold(0_i64, |sum, (line, _)| {
        sum.checked_add(line.taxable_ht_minor)
            .ok_or_else(Phase06Error::numeric_overflow)
    })?;
    let tax = lines.iter().try_fold(0_i64, |sum, (line, _)| {
        sum.checked_add(line.tax_minor)
            .ok_or_else(Phase06Error::numeric_overflow)
    })?;
    let ttc = ht
        .checked_add(tax)
        .ok_or_else(Phase06Error::numeric_overflow)?;
    Ok((checked_i64(header_total)?, ht, tax, ttc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_discount_is_deterministic_and_totals_reconcile() {
        let mut lines = vec![
            (
                base_line(8_000_000, 12_500, 0, 190_000, "HT").unwrap(),
                190_000,
            ),
            (
                base_line(12_000_000, 12_500, 0, 190_000, "HT").unwrap(),
                190_000,
            ),
        ];
        let (discount, ht, tax, ttc) = allocate_header_discount(&mut lines, 100_000).unwrap();
        assert_eq!(discount, 250);
        assert_eq!(ht, 2_250);
        assert_eq!(tax, 428);
        assert_eq!(ttc, 2_678);
        assert_eq!(
            lines
                .iter()
                .map(|item| item.0.allocated_header_discount_minor)
                .sum::<i64>(),
            discount
        );
    }
}
