use std::{fs, io::Write};

use rusqlite::{params_from_iter, types::Value as SqlValue, Connection, Row};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{
    checked_page,
    error::{Phase09Error, Phase09Result},
    models::{AuditEventView, AuditRequest, ExportResult, Paged},
    new_id, now_iso, safe_component, Phase09Service,
};

const AUDIT_PAGE_LIMIT: i64 = 200;
const AUDIT_EXPORT_LIMIT: i64 = 100_000;
const REDACTED: &str = "[REDACTED]";
const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "password_hash",
    "token",
    "token_hash",
    "recovery_code",
    "secret",
    "credential",
    "private_key",
];

impl Phase09Service {
    pub fn list_audit_events(&self, request: AuditRequest) -> Phase09Result<Paged<AuditEventView>> {
        let context = self.authorize("audit.view")?;
        let (page, page_size) = checked_page(request.page, request.page_size, AUDIT_PAGE_LIMIT)?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let query = AuditQuery::new(&context.company_id, &request)?;
        let total = query.count(&connection)?;
        let items = query.load(&connection, page_size, (page - 1) * page_size)?;
        Ok(Paged {
            items,
            page,
            page_size,
            total,
        })
    }

    pub fn export_audit_csv(&self, request: AuditRequest) -> Phase09Result<ExportResult> {
        let context = self.authorize("audit.export")?;
        let connection = self.phase05.phase09_open_maintenance()?;
        let query = AuditQuery::new(&context.company_id, &request)?;
        let total = query.count(&connection)?;
        if total > AUDIT_EXPORT_LIMIT {
            return Err(Phase09Error::new(
                "EXPORT_ROW_LIMIT",
                "The audit export exceeds 100000 rows. Narrow the filters and try again.",
                false,
            ));
        }
        let rows = query.load(&connection, AUDIT_EXPORT_LIMIT.max(1), 0)?;
        let generated_at = now_iso()?;
        let relative_directory = "audit";
        let relative_name = format!("audit-{}.csv", safe_component(&new_id())?);
        let relative_path = format!("{relative_directory}/{relative_name}");
        let directory = self.paths.exports.join(relative_directory);
        fs::create_dir_all(&directory)?;
        let final_path = directory.join(&relative_name);
        if final_path.exists() {
            return Err(Phase09Error::internal());
        }
        let temporary_path = directory.join(format!(".{relative_name}.tmp"));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(&[0xEF, 0xBB, 0xBF])?;
        write_csv_row(&mut file, &["POSMAN AUDIT EXPORT"])?;
        write_csv_row(&mut file, &["generated_at", &generated_at])?;
        write_csv_row(&mut file, &["generated_by", &context.user_id])?;
        write_csv_row(&mut file, &["company_id", &context.company_id])?;
        write_csv_row(&mut file, &["filters", &safe_json(&request)?])?;
        write_csv_row(&mut file, &[])?;
        write_csv_row(
            &mut file,
            &[
                "id",
                "occurred_at",
                "user_id",
                "user",
                "domain",
                "action",
                "entity_type",
                "entity_id",
                "outcome",
                "sensitive",
                "details",
            ],
        )?;
        for row in rows {
            let details = row
                .details
                .as_ref()
                .map(safe_json)
                .transpose()?
                .unwrap_or_default();
            write_csv_row(
                &mut file,
                &[
                    &row.id,
                    &row.occurred_at,
                    row.actor_user_id.as_deref().unwrap_or(""),
                    row.actor_display_name.as_deref().unwrap_or(""),
                    &row.domain,
                    &row.action_code,
                    &row.entity_type,
                    &row.entity_id,
                    &row.outcome,
                    if row.sensitive { "true" } else { "false" },
                    &details,
                ],
            )?;
        }
        file.flush()?;
        file.sync_all()?;
        fs::rename(&temporary_path, &final_path).map_err(|error| {
            let _ = fs::remove_file(&temporary_path);
            Phase09Error::from(error)
        })?;
        let bytes = fs::read(&final_path)?;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let size_bytes = i64::try_from(bytes.len()).map_err(|_| Phase09Error::internal())?;
        Ok(ExportResult {
            relative_path,
            sha256,
            size_bytes,
        })
    }
}

struct AuditQuery {
    where_sql: String,
    values: Vec<SqlValue>,
}

impl AuditQuery {
    fn new(company_id: &str, request: &AuditRequest) -> Phase09Result<Self> {
        let mut query = Self {
            where_sql: "a.company_id=?".to_owned(),
            values: vec![SqlValue::Text(company_id.to_owned())],
        };
        if let Some(value) = trimmed(&request.start_at) {
            query.push("a.occurred_at>=?", value);
        } else {
            query
                .where_sql
                .push_str(" AND a.occurred_at>=strftime('%Y-%m-%dT%H:%M:%fZ','now','-7 days')");
        }
        if let Some(value) = trimmed(&request.end_at) {
            query.push("a.occurred_at<=?", value);
        }
        if let Some(value) = trimmed(&request.user_id) {
            query.push("a.actor_user_id=?", value);
        }
        if let Some(value) = trimmed(&request.domain) {
            validate_filter_token(&value, "domain")?;
            query
                .where_sql
                .push_str(" AND (a.action_code LIKE ? OR a.action_code LIKE ?)");
            query.values.push(SqlValue::Text(format!("{value}.%")));
            query.values.push(SqlValue::Text(format!("{value}_%")));
        }
        if let Some(value) = trimmed(&request.action) {
            validate_filter_token(&value, "action")?;
            query.push("a.action_code=?", value);
        }
        if let Some(value) = trimmed(&request.entity_type) {
            query.push("a.entity_type=?", value);
        }
        if let Some(value) = trimmed(&request.entity_id) {
            query.push("a.entity_id=?", value);
        }
        if let Some(value) = trimmed(&request.outcome) {
            if !matches!(value.as_str(), "SUCCESS" | "FAILURE" | "DENIED") {
                return Err(Phase09Error::validation("Invalid audit outcome filter."));
            }
            query.push("a.outcome=?", value);
        }
        if request.sensitive_only == Some(true) {
            query.where_sql.push_str(
                " AND (lower(a.action_code) LIKE '%password%' OR lower(a.action_code) LIKE '%token%' OR lower(a.action_code) LIKE '%recovery%' OR lower(a.action_code) LIKE '%secret%' OR lower(COALESCE(a.details_json,'')) LIKE '%password%' OR lower(COALESCE(a.details_json,'')) LIKE '%token%' OR lower(COALESCE(a.details_json,'')) LIKE '%secret%' OR lower(COALESCE(a.details_json,'')) LIKE '%credential%' OR lower(COALESCE(a.details_json,'')) LIKE '%private_key%')",
            );
        }
        Ok(query)
    }

    fn push(&mut self, expression: &str, value: String) {
        self.where_sql.push_str(" AND ");
        self.where_sql.push_str(expression);
        self.values.push(SqlValue::Text(value));
    }

    fn count(&self, connection: &Connection) -> Phase09Result<i64> {
        connection
            .query_row(
                &format!("SELECT COUNT(*) FROM audit_logs a WHERE {}", self.where_sql),
                params_from_iter(self.values.iter()),
                |row| row.get(0),
            )
            .map_err(Phase09Error::from)
    }

    fn load(
        &self,
        connection: &Connection,
        limit: i64,
        offset: i64,
    ) -> Phase09Result<Vec<AuditEventView>> {
        let sql = format!(
            "SELECT a.id,a.actor_user_id,u.display_name,a.action_code,a.entity_type,a.entity_id,a.occurred_at,a.outcome,a.details_json FROM audit_logs a LEFT JOIN users u ON u.id=a.actor_user_id AND u.company_id=a.company_id WHERE {} ORDER BY a.occurred_at DESC,a.id DESC LIMIT {limit} OFFSET {offset}",
            self.where_sql
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(self.values.iter()), map_audit_event)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    }
}

fn map_audit_event(row: &Row<'_>) -> rusqlite::Result<AuditEventView> {
    let action_code: String = row.get(3)?;
    let details_text: Option<String> = row.get(8)?;
    let mut details = details_text
        .as_deref()
        .and_then(|value| serde_json::from_str::<Value>(value).ok());
    if let Some(value) = details.as_mut() {
        redact(value);
    }
    let sensitive = is_sensitive(&action_code, details_text.as_deref());
    Ok(AuditEventView {
        id: row.get(0)?,
        actor_user_id: row.get(1)?,
        actor_display_name: row.get(2)?,
        domain: audit_domain(&action_code),
        action_code,
        entity_type: row.get(4)?,
        entity_id: row.get(5)?,
        occurred_at: row.get(6)?,
        outcome: row.get(7)?,
        sensitive,
        details,
    })
}

fn redact(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map.iter_mut() {
                if is_sensitive_key(key) {
                    *nested = Value::String(REDACTED.to_owned());
                } else {
                    redact(nested);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact),
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase().replace('-', "_");
    SENSITIVE_KEYS.iter().any(|candidate| {
        normalized == *candidate
            || normalized.ends_with(&format!("_{candidate}"))
            || normalized.contains(candidate)
    })
}

fn is_sensitive(action: &str, details: Option<&str>) -> bool {
    let text = format!(
        "{} {}",
        action.to_ascii_lowercase(),
        details.unwrap_or("").to_ascii_lowercase()
    );
    SENSITIVE_KEYS.iter().any(|key| text.contains(key))
}

fn audit_domain(action: &str) -> String {
    action
        .split(['.', '_'])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("system")
        .to_ascii_lowercase()
}

fn trimmed(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn validate_filter_token(value: &str, field: &str) -> Phase09Result<()> {
    if value.len() > 100
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
    {
        return Err(Phase09Error::validation(&format!(
            "Invalid audit {field} filter."
        )));
    }
    Ok(())
}

fn safe_json<T: serde::Serialize>(value: &T) -> Phase09Result<String> {
    serde_json::to_string(value).map_err(|_| Phase09Error::internal())
}

fn write_csv_row(file: &mut fs::File, cells: &[&str]) -> Phase09Result<()> {
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            file.write_all(b";")?;
        }
        file.write_all(csv_cell(cell).as_bytes())?;
    }
    file.write_all(b"\r\n")?;
    Ok(())
}

fn csv_cell(value: &str) -> String {
    let cleaned = value
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\t' | '\n' | '\r'))
        .collect::<String>()
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let first = cleaned.trim_start().chars().next();
    let neutralized = if matches!(first, Some('=' | '+' | '-' | '@')) {
        format!("'{cleaned}")
    } else {
        cleaned
    };
    format!("\"{}\"", neutralized.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_redaction_removes_sensitive_values_before_serialization() {
        let mut value = serde_json::json!({
            "password": "secret",
            "nested": {"tokenHash": "abc", "safe": "visible"},
            "items": [{"private_key": "key"}],
        });
        redact(&mut value);
        let serialized = value.to_string();
        assert!(!serialized.contains("secret"));
        assert!(!serialized.contains("abc"));
        assert!(!serialized.contains("key\""));
        assert!(serialized.contains("visible"));
        assert!(serialized.contains(REDACTED));
    }

    #[test]
    fn csv_formula_prefix_is_neutralized() {
        assert_eq!(csv_cell(" =SUM(A1:A2)"), "\"' =SUM(A1:A2)\"");
        assert_eq!(csv_cell("safe"), "\"safe\"");
    }
}
