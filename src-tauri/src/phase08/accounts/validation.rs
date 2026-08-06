fn validate_account(input:&AccountInput)->Phase08Result<()> {
    if input.code.trim().is_empty()||input.name_ar.trim().is_empty(){return Err(Phase08Error::validation("Account code and Arabic name are required."));}
    if !["ASSET","LIABILITY","EQUITY","REVENUE","EXPENSE","OFF_BALANCE"].contains(&input.account_type.as_str()){return Err(Phase08Error::validation("Unsupported account type."));}
    if !["DEBIT","CREDIT"].contains(&input.normal_side.as_str()){return Err(Phase08Error::validation("Unsupported normal side."));}
    Ok(())
}
