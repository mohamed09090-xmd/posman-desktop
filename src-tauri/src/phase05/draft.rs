use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde_json::Value;

use super::{
    dto::{SaveSetupDraftRequest, SetupDraft, SetupStatus},
    error::{Phase05Error, Phase05Result},
    pricing::fiscal_year_default,
    state::{new_id, now_iso, Phase05Service},
};

const SETUP_DRAFT_SCHEMA_VERSION: i64 = 1;

impl Phase05Service {
    pub fn get_setup_status(&self) -> Phase05Result<SetupStatus> {
        let connection = self.open()?;
        let companies: i64 =
            connection.query_row("SELECT COUNT(*) FROM companies", [], |row| row.get(0))?;
        let drafts: i64 = connection.query_row(
            "SELECT COUNT(*) FROM setup_drafts WHERE is_active=1",
            [],
            |row| row.get(0),
        )?;
        let (starts_on, ends_on) = fiscal_year_default();
        Ok(SetupStatus {
            setup_required: companies == 0,
            has_draft: drafts == 1,
            schema_version: "0005".to_owned(),
            default_fiscal_starts_on: starts_on,
            default_fiscal_ends_on: ends_on,
        })
    }

    pub fn load_setup_draft(&self) -> Phase05Result<Option<SetupDraft>> {
        let connection = self.open()?;
        connection
            .query_row(
                r#"
                SELECT draft_schema_version, validated_json, row_version
                FROM setup_drafts WHERE is_active=1 LIMIT 1
                "#,
                [],
                |row| {
                    let json: String = row.get(1)?;
                    let data = serde_json::from_str::<Value>(&json).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            json.len(),
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Ok(SetupDraft {
                        draft_schema_version: row.get(0)?,
                        data,
                        row_version: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Phase05Error::from)
    }

    pub fn save_setup_draft(&self, request: SaveSetupDraftRequest) -> Phase05Result<SetupDraft> {
        if request.draft_schema_version != SETUP_DRAFT_SCHEMA_VERSION {
            return Err(invalid_draft(
                "The setup draft schema version is unsupported.",
            ));
        }
        reject_sensitive_json(&request.data)?;
        let serialized = serde_json::to_string(&request.data)
            .map_err(|_| invalid_draft("The setup draft is not valid JSON."))?;
        let timestamp = now_iso()?;
        let mut connection = self.open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = transaction
            .query_row(
                "SELECT id, row_version FROM setup_drafts WHERE is_active=1 LIMIT 1",
                [],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;
        let row_version = if let Some((id, current_version)) = existing {
            if request.row_version != Some(current_version) {
                return Err(Phase05Error::concurrency());
            }
            let updated = transaction.execute(
                r#"
                UPDATE setup_drafts
                SET draft_schema_version=?1, validated_json=?2, updated_at=?3,
                    row_version=row_version+1
                WHERE id=?4 AND row_version=?5 AND is_active=1
                "#,
                params![
                    request.draft_schema_version,
                    serialized,
                    timestamp,
                    id,
                    current_version
                ],
            )?;
            if updated != 1 {
                return Err(Phase05Error::concurrency());
            }
            current_version + 1
        } else {
            if request.row_version.is_some() {
                return Err(Phase05Error::concurrency());
            }
            transaction.execute(
                r#"
                INSERT INTO setup_drafts (
                    id, draft_schema_version, validated_json, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?4)
                "#,
                params![
                    new_id(),
                    request.draft_schema_version,
                    serialized,
                    timestamp
                ],
            )?;
            1
        };
        transaction.commit()?;
        Ok(SetupDraft {
            draft_schema_version: request.draft_schema_version,
            data: request.data,
            row_version,
        })
    }

    pub fn discard_setup_draft(&self) -> Phase05Result<()> {
        let mut connection = self.open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            r#"
            UPDATE setup_drafts
            SET is_active=0, updated_at=?1, row_version=row_version+1
            WHERE is_active=1
            "#,
            [now_iso()?],
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn reject_sensitive_json(value: &Value) -> Phase05Result<()> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key
                    .chars()
                    .filter(|character| character.is_alphanumeric())
                    .flat_map(char::to_lowercase)
                    .collect::<String>();
                if normalized.contains("password")
                    || normalized.contains("recoverycode")
                    || normalized.contains("sessiontoken")
                    || normalized.contains("secret")
                {
                    return Err(invalid_draft(
                        "Sensitive values cannot be saved in a setup draft.",
                    ));
                }
                reject_sensitive_json(child)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                reject_sensitive_json(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn invalid_draft(message: &str) -> Phase05Error {
    Phase05Error::new("SETUP_INVALID_DRAFT", message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nested_passwords_are_rejected_from_setup_drafts() {
        let error = reject_sensitive_json(&json!({
            "company": {"name": "POSMAN"},
            "administrator": {"password": "not persisted"}
        }))
        .expect_err("secret should be rejected");
        assert_eq!(error.code, "SETUP_INVALID_DRAFT");
    }
}
