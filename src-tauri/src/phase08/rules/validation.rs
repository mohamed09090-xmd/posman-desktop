fn validate_rule(input:&PostingRuleInput)->Phase08Result<()> {
    if input.code.trim().is_empty()||input.source_event_type.trim().is_empty(){return Err(Phase08Error::validation("Rule code and source event type are required."));}
    if input.lines.len()<2{return Err(Phase08Error::new("POSTING_RULE_LINES_REQUIRED","A posting rule requires at least two lines.",false));}
    let mut numbers=std::collections::BTreeSet::new();
    let mut has_debit = false;
    let mut has_credit = false;
    for line in &input.lines {
        if !numbers.insert(line.line_number){return Err(Phase08Error::validation("Posting-rule line numbers must be unique."));}
        if (line.account_id.is_some())==(line.account_role_code.is_some()){return Err(Phase08Error::validation("Each rule line must select either an account or an account role."));}
        if !["DEBIT","CREDIT"].contains(&line.side.as_str()){return Err(Phase08Error::validation("Unsupported rule-line side."));}
        has_debit |= line.side == "DEBIT";
        has_credit |= line.side == "CREDIT";
    }
    if !has_debit || !has_credit {
        return Err(Phase08Error::new("POSTING_RULE_LINES_REQUIRED", "A posting rule requires at least one debit and one credit line.", false));
    }
    Ok(())
}
