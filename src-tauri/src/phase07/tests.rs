use rusqlite::Connection;

use crate::phase05::Phase06AuthContext;

use super::{
    pricing::{base_line, PricedLine},
    service::{insert_transform_links, PreparedSalesLine},
};

const NOW: &str = "2026-08-05T00:00:00Z";

fn prepared(source: &str, quantity: i64) -> PreparedSalesLine {
    PreparedSalesLine {
        source_line_id: Some(source.to_owned()),
        product_id: "product-1".to_owned(),
        warehouse_id: "warehouse-1".to_owned(),
        quantity_scaled: quantity,
        unit_price_scaled: 15_000,
        unit_cost_scaled: 10_000,
        discount_rate_scaled: 0,
        tax_rate_scaled: 190_000,
        product_code: "P-1".to_owned(),
        product_name: "Article".to_owned(),
        unit_id: "unit-1".to_owned(),
        unit_code: "PC".to_owned(),
        tax_code: Some("TVA19".to_owned()),
        priced: base_line(quantity, 15_000, 0, 190_000, "HT").unwrap(),
    }
}

fn context() -> Phase06AuthContext {
    Phase06AuthContext {
        company_id: "company-1".to_owned(),
        user_id: "user-1".to_owned(),
        session_id: "session-1".to_owned(),
    }
}

fn fixture() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    connection.execute_batch(include_str!("../../../database/schema.sql")).unwrap();
    connection.execute(
        "INSERT INTO companies (id,code,legal_name,name_ar,name_fr,created_at,updated_at)
         VALUES ('company-1','C1','Company','شركة','Société',?1,?1)",
        [NOW],
    ).unwrap();
    connection.execute(
        "INSERT INTO fiscal_years (id,company_id,code,name,starts_on,ends_on,status,created_at,updated_at)
         VALUES ('fy-1','company-1','2026','2026','2026-01-01','2026-12-31','OPEN',?1,?1)",
        [NOW],
    ).unwrap();
    connection.execute(
        "INSERT INTO units (id,company_id,code,name_ar,name_fr,decimal_scale,created_at,updated_at)
         VALUES ('unit-1','company-1','PC','قطعة','Pièce',6,?1,?1)",
        [NOW],
    ).unwrap();
    connection.execute(
        "INSERT INTO warehouses (id,company_id,code,name_ar,name_fr,created_at,updated_at)
         VALUES ('warehouse-1','company-1','W1','مخزن','Dépôt',?1,?1)",
        [NOW],
    ).unwrap();
    connection.execute(
        "INSERT INTO products (id,company_id,unit_id,code,name_ar,name_fr,product_kind,stock_tracked,created_at,updated_at)
         VALUES ('product-1','company-1','unit-1','P1','منتج','Produit','STOCK_ITEM',1,?1,?1)",
        [NOW],
    ).unwrap();
    for (id, kind, number) in [
        ("order-1", "SALES_ORDER", "CMD000001"),
        ("delivery-1", "DELIVERY_NOTE", "BL000001"),
        ("delivery-2", "DELIVERY_NOTE", "BL000002"),
        ("delivery-3", "DELIVERY_NOTE", "BL000003"),
    ] {
        let status = if kind == "SALES_ORDER" { "CONFIRMED" } else { "DRAFT" };
        connection.execute(
            "INSERT INTO commercial_documents (
               id,company_id,fiscal_year_id,document_type,document_number,workflow_status,
               commercial_date,created_at,updated_at
             ) VALUES (?1,'company-1','fy-1',?2,?3,?4,'2026-08-05',?5,?5)",
            rusqlite::params![id,kind,number,status,NOW],
        ).unwrap();
    }
    for (id, document, line_number, quantity) in [
        ("source-line", "order-1", 1, 20_000_000_i64),
        ("target-8", "delivery-1", 1, 8_000_000_i64),
        ("target-12", "delivery-2", 1, 12_000_000_i64),
        ("target-extra", "delivery-3", 1, 1_000_000_i64),
    ] {
        connection.execute(
            "INSERT INTO commercial_document_lines (
               id,company_id,document_id,product_id,warehouse_id,unit_id,line_number,
               product_code_snapshot,description_snapshot,unit_code_snapshot,quantity_scaled,
               created_at,updated_at
             ) VALUES (?1,'company-1',?2,'product-1','warehouse-1','unit-1',?3,'P1','Produit','PC',?4,?5,?5)",
            rusqlite::params![id,document,line_number,quantity,NOW],
        ).unwrap();
    }
    connection
}

#[test]
fn sales_transformation_accepts_eight_plus_twelve_and_rejects_twenty_one() {
    let mut connection = fixture();
    let transaction = connection.transaction().unwrap();
    insert_transform_links(&transaction, &context(), &[prepared("source-line", 8_000_000)], &["target-8".to_owned()], "ORDER_TO_DELIVERY").unwrap();
    insert_transform_links(&transaction, &context(), &[prepared("source-line", 12_000_000)], &["target-12".to_owned()], "ORDER_TO_DELIVERY").unwrap();
    let extra = insert_transform_links(&transaction, &context(), &[prepared("source-line", 1_000_000)], &["target-extra".to_owned()], "ORDER_TO_DELIVERY");
    assert!(matches!(extra, Err(error) if error.code == "TRANSFORMATION_LIMIT_EXCEEDED"));
}

#[test]
fn ttc_input_is_normalized_to_ht_without_binary_floating_point() {
    let priced: PricedLine = base_line(1_000_000, 11_900, 0, 190_000, "TTC").unwrap();
    assert_eq!(priced.taxable_ht_minor, 100);
    assert_eq!(priced.tax_minor, 19);
    assert_eq!(priced.ttc_minor, 119);
}
