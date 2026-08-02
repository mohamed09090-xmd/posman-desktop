# One-shot PHASE 05 source correction; removed after materialization.
from pathlib import Path
import re


def replace_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected exactly one {label}, found {count}")
    return updated


dto = Path("src-tauri/src/phase05/dto.rs")
text = dto.read_text(encoding="utf-8")
for name in ("UpdatePartnerAddressRequest", "UpdatePartnerContactRequest"):
    text = replace_once(
        text,
        rf'\n#\[derive\(Clone, Deserialize\)\]\n#\[serde\(rename_all = "camelCase"\)\]\npub struct {name} \{{.*?\n\}}\n',
        "\n",
        name,
    )
dto.write_text(text, encoding="utf-8")

products = Path("src-tauri/src/phase05/products.rs")
text = products.read_text(encoding="utf-8")
text = replace_once(
    text,
    r'''fn pricing_warning\(purchase_price: i64, sale_price: i64\) -> Option<&'static str> \{\n    if sale_price < purchase_price \{\n        Some\("BELOW_COST"\)\n    \} else if sale_price == purchase_price \{\n        Some\("ZERO_MARGIN"\)\n    \} else \{\n        None\n    \}\n\}''',
    '''fn pricing_warning(purchase_price: i64, sale_price: i64) -> Option<&'static str> {
    match sale_price.cmp(&purchase_price) {
        std::cmp::Ordering::Less => Some("BELOW_COST"),
        std::cmp::Ordering::Equal => Some("ZERO_MARGIN"),
        std::cmp::Ordering::Greater => None,
    }
}''',
    "pricing warning comparison chain",
)
products.write_text(text, encoding="utf-8")

config = Path("src-tauri/src/phase05/config.rs")
text = config.read_text(encoding="utf-8")
constant = '''const UPDATE_COMPANY_SETTINGS_SQL: &str = r#"
UPDATE company_settings
SET default_margin_rate_scaled=?1,
    below_cost_policy=?2,
    session_idle_timeout_minutes=?3,
    updated_at=?4,
    updated_by=?5,
    row_version=row_version+1
WHERE company_id=?6
"#;

'''
if constant not in text:
    marker = "use super::{\n"
    if text.count(marker) != 1:
        raise SystemExit("expected config import marker exactly once")
    text = text.replace(marker, constant + marker, 1)

text = replace_once(
    text,
    r'''        transaction\.execute\(\n            r#"\n            UPDATE company_settings SET default_margin_rate_scaled=\?1,\n                session_idle_timeout_minutes=\?2, updated_at=\?3, updated_by=\?4,\n                row_version=row_version\+1 WHERE company_id=\?5\n            "#,\n            params!\[\n                request\.default_margin_rate_scaled,\n                request\.below_cost_policy,\n                request\.session_idle_timeout_minutes,\n                now_iso\(\)\?,\n                context\.user_id,\n                context\.company_id\n            \],\n        \)\?;''',
    '''        transaction.execute(
            UPDATE_COMPANY_SETTINGS_SQL,
            params![
                request.default_margin_rate_scaled,
                request.below_cost_policy,
                request.session_idle_timeout_minutes,
                now_iso()?,
                context.user_id,
                context.company_id
            ],
        )?;''',
    "company settings update",
)

tail = '''    #[test]
    fn numbering_format_has_year_and_six_digit_default_padding() {
        assert_eq!(sequence_number("FAC", "2026", 1, 6), "FAC-2026-000001");
    }
}'''
replacement = '''    #[test]
    fn numbering_format_has_year_and_six_digit_default_padding() {
        assert_eq!(sequence_number("FAC", "2026", 1, 6), "FAC-2026-000001");
    }

    #[test]
    fn company_settings_update_persists_policy_and_timeout() {
        let connection = rusqlite::Connection::open_in_memory().expect("SQLite");
        connection
            .execute_batch(
                r#"
                CREATE TABLE company_settings (
                    company_id TEXT PRIMARY KEY,
                    default_margin_rate_scaled INTEGER NOT NULL,
                    below_cost_policy TEXT NOT NULL,
                    session_idle_timeout_minutes INTEGER NOT NULL,
                    updated_at TEXT NOT NULL,
                    updated_by TEXT NOT NULL,
                    row_version INTEGER NOT NULL
                );
                INSERT INTO company_settings VALUES (
                    'company-1', 100000, 'WARNING_ONLY', 15,
                    '2026-01-01T00:00:00Z', 'user-1', 1
                );
                "#,
            )
            .expect("settings fixture");

        let changed = connection
            .execute(
                UPDATE_COMPANY_SETTINGS_SQL,
                rusqlite::params![
                    250000_i64,
                    "ADMIN_OVERRIDE",
                    45_i64,
                    "2026-08-02T12:00:00Z",
                    "user-2",
                    "company-1"
                ],
            )
            .expect("settings update");
        assert_eq!(changed, 1);

        let actual = connection
            .query_row(
                "SELECT default_margin_rate_scaled, below_cost_policy, session_idle_timeout_minutes, row_version FROM company_settings WHERE company_id='company-1'",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                },
            )
            .expect("updated settings");
        assert_eq!(actual, (250000, "ADMIN_OVERRIDE".to_owned(), 45, 2));
    }
}'''
if text.count(tail) != 1:
    raise SystemExit("expected config test module tail exactly once")
config.write_text(text.replace(tail, replacement, 1), encoding="utf-8")
