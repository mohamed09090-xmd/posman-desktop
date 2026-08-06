fn add_rule(connection: &Connection, company: &str, event: &str, lines: &[(&str, &str, &str)]) {
    let rule = format!("rule-{company}-{event}");
    let debit = format!("account-{company}-{}", lines.iter().find(|line| line.0 == "DEBIT").unwrap().1);
    let credit = format!("account-{company}-{}", lines.iter().find(|line| line.0 == "CREDIT").unwrap().1);
    connection.execute(
        "INSERT INTO posting_rules(id,company_id,accounting_journal_id,debit_account_id,credit_account_id,code,source_event_type,priority,valid_from,is_active,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?6,100,'2026-01-01',1,?7,?7)",
        params![rule, company, format!("journal-{company}"), debit, credit, event, NOW],
    ).unwrap();
    for (index, (side, role, component)) in lines.iter().enumerate() {
        connection.execute(
            "INSERT INTO posting_rule_lines(id,company_id,posting_rule_id,line_number,side,account_id,account_role_code,amount_component,description_ar,partner_dimension,product_dimension,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,NULL,?6,?7,?7,1,0,?8,?8)",
            params![format!("line-{rule}-{index}"), company, rule, (index+1) as i64, side, role, component, NOW],
        ).unwrap();
    }
}

fn envelope<T: serde::Serialize>(key: &str, payload: T) -> Idempotent<T> {
    let request_hash_sha256 = request_hash(&payload).expect("hash test payload");
    Idempotent {
        idempotency_key: key.to_owned(),
        request_hash_sha256,
        payload,
    }
}

fn rehash<T: serde::Serialize>(request: &mut Idempotent<T>) {
    request.request_hash_sha256 = request_hash(&request.payload).expect("rehash test payload");
}

fn event(kind: &str, id: &str, components: &[(&str, i64)]) -> Idempotent<SourceEventRequest> {
    envelope(
        &format!("idem-{id}"),
        SourceEventRequest {
            source_event_type: kind.to_owned(),
            source_event_id: id.to_owned(),
            source_document_id: None,
            event_date: "2026-08-06".to_owned(),
            partner_id: Some("partner-company-1".to_owned()),
            product_id: None,
            payment_method_id: None,
            memo: None,
            components_minor: components.iter().map(|(key, value)| ((*key).to_owned(), *value)).collect::<BTreeMap<_,_>>(),
            inject_failure_after_header: false,
        },
    )
}

fn post(connection: &mut Connection, request: &Idempotent<SourceEventRequest>) -> super::dto::PostingResult {
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate).unwrap();
    let result = post_source_event_in_tx(&tx, &context("company-1"), request).unwrap();
    tx.commit().unwrap();
    result
}

fn assert_balanced(connection: &Connection, entry_id: &str, expected: i64) {
    let values: (i64, i64, i64) = connection.query_row(
        "SELECT COUNT(*),SUM(debit_minor),SUM(credit_minor) FROM journal_entry_lines WHERE journal_entry_id=?1",
        [entry_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).unwrap();
    assert!(values.0 >= 2);
    assert_eq!(values.1, expected);
    assert_eq!(values.2, expected);
}
