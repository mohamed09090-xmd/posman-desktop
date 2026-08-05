use rusqlite::{Connection, TransactionBehavior};

use crate::phase05::Phase06AuthContext;

use super::{
    begin_idempotency,
    error::Phase06Error,
    finish_idempotency,
    fixed_point::{extended_cost_minor, line_totals, weighted_average_cost},
    projections::{apply_movement, balance, set_reserved, MovementSpec},
    validate_idempotency_key, IdempotencyStart,
};

const NOW: &str = "2026-08-05T00:00:00Z";

fn fixture() -> Connection {
    let connection = Connection::open_in_memory().expect("in-memory SQLite");
    connection.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    connection
        .execute_batch(include_str!("../../../database/schema.sql"))
        .expect("accepted schema");
    connection
        .execute(
            "INSERT INTO companies (id, code, legal_name, name_ar, name_fr, created_at, updated_at)
         VALUES ('c1', 'C1', 'Company One', 'شركة', 'Société', ?1, ?1)",
            [NOW],
        )
        .unwrap();
    connection.execute(
        "INSERT INTO company_settings (id, company_id, negative_stock_policy, created_at, updated_at)
         VALUES ('cs1', 'c1', 'PRIVILEGED_OVERRIDE', ?1, ?1)",
        [NOW],
    ).unwrap();
    connection.execute(
        "INSERT INTO users (id, company_id, username, display_name, password_hash, created_at, updated_at)
         VALUES ('u1', 'c1', 'admin', 'Administrator',
                 '$argon2id$v=19$m=65536,t=3,p=1$abcdefghijklmnop$abcdefghijklmnopqrstuv', ?1, ?1)",
        [NOW],
    ).unwrap();
    for (id, code) in [("w1", "W1"), ("w2", "W2")] {
        connection.execute(
            "INSERT INTO warehouses (id, company_id, code, name_ar, name_fr, created_at, updated_at)
             VALUES (?1, 'c1', ?2, ?2, ?2, ?3, ?3)",
            rusqlite::params![id, code, NOW],
        ).unwrap();
    }
    for (id, code) in [("l1", "L1"), ("l2", "L2")] {
        connection
            .execute(
                "INSERT INTO warehouse_locations (
                id, company_id, warehouse_id, code, name_ar, name_fr, created_at, updated_at
             ) VALUES (?1, 'c1', 'w1', ?2, ?2, ?2, ?3, ?3)",
                rusqlite::params![id, code, NOW],
            )
            .unwrap();
    }
    connection.execute(
        "INSERT INTO units (id, company_id, code, name_ar, name_fr, decimal_scale, created_at, updated_at)
         VALUES ('unit1', 'c1', 'PC', 'قطعة', 'Pièce', 6, ?1, ?1)",
        [NOW],
    ).unwrap();
    connection
        .execute(
            "INSERT INTO products (id, company_id, unit_id, code, name_ar, name_fr,
                               product_kind, stock_tracked, created_at, updated_at)
         VALUES ('p1', 'c1', 'unit1', 'P1', 'منتج', 'Produit', 'STOCK_ITEM', 1, ?1, ?1)",
            [NOW],
        )
        .unwrap();
    connection
}

fn context() -> Phase06AuthContext {
    Phase06AuthContext {
        company_id: "c1".to_owned(),
        user_id: "u1".to_owned(),
        session_id: "s1".to_owned(),
    }
}

fn movement<'a>(
    warehouse_id: &'a str,
    location_id: Option<&'a str>,
    movement_type: &'a str,
    quantity_delta: i64,
    inbound_cost: Option<i64>,
    recalculate_average: bool,
    event: (&'a str, bool),
) -> MovementSpec<'a> {
    MovementSpec {
        product_id: "p1",
        warehouse_id,
        location_id,
        source_document_id: None,
        source_line_id: None,
        movement_type,
        business_date: "2026-08-05",
        quantity_delta,
        inbound_cost,
        recalculate_average,
        posting_event_key: event.0,
        transfer_group_id: None,
        notes: None,
        allow_negative: event.1,
    }
}

#[test]
fn cump_and_rounding_are_exact_integer_results() {
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
    assert_eq!(extended_cost_minor(1_500_000, 12_345).unwrap(), 1_852);
    assert_eq!(
        line_totals(2_000_000, 10_000, 100_000, 190_000).unwrap(),
        (20, 180, 34, 214)
    );
}

#[test]
fn opening_and_receipt_update_aggregate_and_location_cump() {
    let mut connection = fixture();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let context = context();
    apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            Some("l1"),
            "OPENING",
            10_000_000,
            Some(10_000),
            true,
            ("open-1", false),
        ),
    )
    .unwrap();
    let receipt = apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            Some("l1"),
            "PURCHASE_RECEIPT",
            5_000_000,
            Some(16_000),
            true,
            ("receipt-1", false),
        ),
    )
    .unwrap();
    assert_eq!(receipt.quantity_after, 15_000_000);
    assert_eq!(receipt.average_cost_after, 12_000);
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", None)
            .unwrap()
            .average_cost,
        12_000
    );
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", Some("l1"))
            .unwrap()
            .average_cost,
        12_000
    );
}

#[test]
fn negative_stock_block_and_override_are_distinct() {
    let mut connection = fixture();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let context = context();
    let blocked = apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            None,
            "ADJUSTMENT_OUT",
            -1_000_000,
            None,
            false,
            ("negative-block", false),
        ),
    );
    assert!(matches!(blocked, Err(Phase06Error { code, .. }) if code == "INSUFFICIENT_STOCK"));
    let applied = apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            None,
            "ADJUSTMENT_OUT",
            -1_000_000,
            None,
            false,
            ("negative-override", true),
        ),
    )
    .unwrap();
    assert_eq!(applied.quantity_after, -1_000_000);
}

#[test]
fn cross_warehouse_transfer_pair_carries_source_cump() {
    let mut connection = fixture();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let context = context();
    apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            None,
            "OPENING",
            10_000_000,
            Some(12_000),
            true,
            ("opening-w1", false),
        ),
    )
    .unwrap();
    let mut out = movement(
        "w1",
        None,
        "TRANSFER_OUT",
        -4_000_000,
        None,
        false,
        ("transfer-out", false),
    );
    out.transfer_group_id = Some("group-1");
    apply_movement(&transaction, &context, out).unwrap();
    let mut incoming = movement(
        "w2",
        None,
        "TRANSFER_IN",
        4_000_000,
        Some(12_000),
        true,
        ("transfer-in", false),
    );
    incoming.transfer_group_id = Some("group-1");
    apply_movement(&transaction, &context, incoming).unwrap();
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", None)
            .unwrap()
            .on_hand,
        6_000_000
    );
    let target = balance(&transaction, "c1", "p1", "w2", None).unwrap();
    assert_eq!((target.on_hand, target.average_cost), (4_000_000, 12_000));
    let pair: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM stock_movements WHERE transfer_group_id='group-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(pair, 2);
}

#[test]
fn same_warehouse_location_transfer_preserves_cump() {
    let mut connection = fixture();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let context = context();
    apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            Some("l1"),
            "OPENING",
            8_000_000,
            Some(9_500),
            true,
            ("loc-opening", false),
        ),
    )
    .unwrap();
    apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            Some("l1"),
            "TRANSFER_OUT",
            -3_000_000,
            None,
            false,
            ("loc-out", false),
        ),
    )
    .unwrap();
    apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            Some("l2"),
            "TRANSFER_IN",
            3_000_000,
            Some(9_500),
            false,
            ("loc-in", false),
        ),
    )
    .unwrap();
    let aggregate = balance(&transaction, "c1", "p1", "w1", None).unwrap();
    assert_eq!(
        (aggregate.on_hand, aggregate.average_cost),
        (8_000_000, 9_500)
    );
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", Some("l1"))
            .unwrap()
            .on_hand,
        5_000_000
    );
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", Some("l2"))
            .unwrap()
            .on_hand,
        3_000_000
    );
}

#[test]
fn reservations_never_exceed_available_and_support_partial_release() {
    let mut connection = fixture();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let context = context();
    apply_movement(
        &transaction,
        &context,
        movement(
            "w1",
            None,
            "OPENING",
            5_000_000,
            Some(10_000),
            true,
            ("reserve-opening", false),
        ),
    )
    .unwrap();
    set_reserved(&transaction, &context, "p1", "w1", None, 3_000_000).unwrap();
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", None)
            .unwrap()
            .available()
            .unwrap(),
        2_000_000
    );
    assert!(set_reserved(&transaction, &context, "p1", "w1", None, 3_000_000).is_err());
    set_reserved(&transaction, &context, "p1", "w1", None, -1_000_000).unwrap();
    assert_eq!(
        balance(&transaction, "c1", "p1", "w1", None)
            .unwrap()
            .reserved,
        2_000_000
    );
}

#[test]
fn idempotency_replays_same_hash_and_rejects_conflict() {
    let mut connection = fixture();
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    let context = context();
    let key = "phase06-idempotency-key";
    validate_idempotency_key(key).unwrap();
    assert!(matches!(
        begin_idempotency(
            &transaction,
            &context,
            "stock.test",
            key,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        )
        .unwrap(),
        IdempotencyStart::New
    ));
    finish_idempotency(
        &transaction,
        &context,
        "stock.test",
        key,
        "commercial_document",
        "doc-1",
    )
    .unwrap();
    assert!(
        matches!(begin_idempotency(&transaction, &context, "stock.test", key, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap(), IdempotencyStart::Replayed(id) if id == "doc-1")
    );
    assert!(
        matches!(begin_idempotency(&transaction, &context, "stock.test", key, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"), Err(Phase06Error { code, .. }) if code == "IDEMPOTENCY_CONFLICT")
    );
}

#[test]
fn stock_movement_ledger_is_append_only_and_events_unique() {
    let mut connection = fixture();
    let movement_id;
    {
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let context = context();
        movement_id = apply_movement(
            &transaction,
            &context,
            movement(
                "w1",
                None,
                "OPENING",
                1_000_000,
                Some(10_000),
                true,
                ("immutable-event", false),
            ),
        )
        .unwrap()
        .movement_id;
        transaction.commit().unwrap();
    }
    assert!(connection
        .execute(
            "UPDATE stock_movements SET notes='tampered' WHERE id=?1",
            [&movement_id]
        )
        .is_err());
    assert!(connection
        .execute("DELETE FROM stock_movements WHERE id=?1", [&movement_id])
        .is_err());
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .unwrap();
    assert!(apply_movement(
        &transaction,
        &context(),
        movement(
            "w1",
            None,
            "OPENING",
            1_000_000,
            Some(10_000),
            true,
            ("immutable-event", false)
        )
    )
    .is_err());
}

#[test]
fn normalized_errors_are_safe() {
    let serialized = serde_json::to_string(&Phase06Error::invalid("quantityScaled")).unwrap();
    assert!(serialized.contains("VALIDATION_FAILED"));
    assert!(!serialized.to_ascii_lowercase().contains("select "));
    assert!(!serialized.contains("/mnt/"));
    assert!(!serialized.to_ascii_lowercase().contains("sqlite"));
}
