impl Phase06Service {
    pub fn new(phase05: Phase05Service) -> Phase06Result<Self> {
        let service = Self { phase05 };
        service.provision_permissions()?;
        Ok(service)
    }

    fn provision_permissions(&self) -> Phase06Result<()> {
        let mut connection = self.phase05.phase06_open()?;
        let transaction =
            connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_iso()?;

        for (id, code, domain, description_ar, description_fr, sensitive) in
            PHASE06_PERMISSIONS
        {
            transaction.execute(
                r#"
                INSERT OR IGNORE INTO permissions (
                    id, code, domain, description_ar, description_fr,
                    is_sensitive, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    id,
                    code,
                    domain,
                    description_ar,
                    description_fr,
                    if *sensitive { 1_i64 } else { 0_i64 },
                    now
                ],
            )?;
        }

        grant_template_permissions(&transaction, "OWNER", ALL_PHASE06_CODES, &now, false)?;
        grant_template_permissions(&transaction, "STOCK", STOCK_ROLE_CODES, &now, false)?;
        grant_template_permissions(&transaction, "PURCHASING", PURCHASING_ROLE_CODES, &now, false)?;
        grant_template_permissions(&transaction, "AUDITOR", AUDITOR_ROLE_CODES, &now, false)?;

        grant_template_permissions(
            &transaction,
            "SYSTEM_ADMINISTRATOR",
            ALL_PHASE06_CODES,
            &now,
            true,
        )?;
        grant_template_permissions(&transaction, "OWNER", ALL_PHASE06_CODES, &now, true)?;
        grant_template_permissions(&transaction, "STOCK", STOCK_ROLE_CODES, &now, true)?;
        grant_template_permissions(&transaction, "PURCHASING", PURCHASING_ROLE_CODES, &now, true)?;
        grant_template_permissions(&transaction, "AUDITOR", AUDITOR_ROLE_CODES, &now, true)?;

        transaction.commit()?;
        Ok(())
    }

    fn context(&self, permission: Option<&str>) -> Phase06Result<Phase06AuthContext> {
        self.phase05.phase06_authorize(permission).map_err(Into::into)
    }

    fn immediate<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Phase06Result<T>,
    ) -> Phase06Result<T> {
        let mut connection = self.phase05.phase06_open()?;
        let transaction =
            connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        match operation(&transaction) {
            Ok(result) => {
                transaction.commit()?;
                Ok(result)
            }
            Err(mut error) => {
                drop(transaction);
                if let Some(failure) = error.accounting_failure.take() {
                    crate::phase08::record_failed_posting_attempt(&mut connection, &failure)
                        .map_err(|_| Phase06Error::internal())?;
                }
                Err(error)
            }
        }
    }

    fn read<T>(
        &self,
        operation: impl FnOnce(&rusqlite::Connection) -> Phase06Result<T>,
    ) -> Phase06Result<T> {
        let connection = self.phase05.phase06_open()?;
        operation(&connection)
    }
}

fn grant_template_permissions(
    transaction: &Transaction<'_>,
    role_code: &str,
    permission_codes: &str,
    timestamp: &str,
    company_scoped: bool,
) -> Phase06Result<()> {
    let scope_predicate = if company_scoped {
        "role.company_id IS NOT NULL"
    } else {
        "role.company_id IS NULL"
    };
    let sql = format!(
        r#"
        INSERT OR IGNORE INTO role_permissions (
            id, company_id, role_id, permission_id, granted_at, granted_by
        )
        SELECT
            'rp-p06-' || role.id || '-' || permission.id,
            role.company_id,
            role.id,
            permission.id,
            ?1,
            NULL
        FROM roles AS role
        CROSS JOIN permissions AS permission
        WHERE {scope_predicate}
          AND role.is_system=1
          AND role.is_active=1
          AND role.code=?2
          AND permission.code IN ({permission_codes})
        "#
    );
    transaction.execute(&sql, params![timestamp, role_code])?;
    Ok(())
}

pub(crate) fn authorize_transaction(
    transaction: &Transaction<'_>,
    context: &Phase06AuthContext,
    permission: &str,
) -> Phase06Result<()> {
    let authorized = transaction
        .query_row(
            r#"
            SELECT 1
            FROM sessions AS session
            JOIN users AS user
              ON user.id=session.user_id AND user.company_id=session.company_id
            JOIN user_roles AS assignment
              ON assignment.user_id=user.id AND assignment.company_id=user.company_id
            JOIN roles AS role
              ON role.id=assignment.role_id
             AND role.company_id=assignment.company_id
            JOIN role_permissions AS grant_row
              ON grant_row.role_id=role.id
            JOIN permissions AS permission_row
              ON permission_row.id=grant_row.permission_id
            WHERE session.id=?1 AND session.company_id=?2 AND session.user_id=?3
              AND session.revoked_at IS NULL AND session.expires_at>?4
              AND user.is_active=1 AND role.is_active=1
              AND permission_row.code=?5
            LIMIT 1
            "#,
            params![
                context.session_id,
                context.company_id,
                context.user_id,
                now_iso()?,
                permission
            ],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some();
    if !authorized {
        return Err(Phase06Error::permission());
    }
    Ok(())
}
