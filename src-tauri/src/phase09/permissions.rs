use rusqlite::{params, Transaction, TransactionBehavior};

use super::{error::Phase09Result, now_iso, Phase09Service};

pub const PERMISSIONS: &[(&str, &str, &str, bool)] = &[
    (
        "perm-p09-doc-tpl-view",
        "documents.templates.view",
        "documents",
        false,
    ),
    (
        "perm-p09-doc-tpl-manage",
        "documents.templates.manage",
        "documents",
        true,
    ),
    (
        "perm-p09-doc-render",
        "documents.render",
        "documents",
        false,
    ),
    ("perm-p09-doc-print", "documents.print", "documents", false),
    (
        "perm-p09-doc-export",
        "documents.export",
        "documents",
        false,
    ),
    ("perm-p09-reports-view", "reports.view", "reports", false),
    (
        "perm-p09-reports-export",
        "reports.export",
        "reports",
        false,
    ),
    ("perm-p09-audit-view", "audit.view", "audit", false),
    ("perm-p09-audit-export", "audit.export", "audit", true),
    ("perm-p09-backup-view", "backup.view", "backup", false),
    ("perm-p09-backup-create", "backup.create", "backup", false),
    ("perm-p09-backup-restore", "backup.restore", "backup", true),
    ("perm-p09-backup-manage", "backup.manage", "backup", true),
];

impl Phase09Service {
    pub(crate) fn provision_permissions(&self) -> Phase09Result<()> {
        let mut connection = self.phase05.phase06_open()?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_iso()?;
        for (id, code, domain, sensitive) in PERMISSIONS {
            transaction.execute(
                "INSERT OR IGNORE INTO permissions(id,code,domain,description_ar,description_fr,is_sensitive,created_at) VALUES(?1,?2,?3,?2,?2,?4,?5)",
                params![id, code, domain, if *sensitive { 1_i64 } else { 0_i64 }, now],
            )?;
        }
        for role in ["OWNER", "SYSTEM_ADMINISTRATOR"] {
            for (_, code, _, _) in PERMISSIONS {
                grant(&transaction, role, code, &now)?;
            }
        }
        for role in ["SALES", "PURCHASING"] {
            for code in [
                "documents.templates.view",
                "documents.render",
                "documents.print",
                "documents.export",
                "reports.view",
                "reports.export",
            ] {
                grant(&transaction, role, code, &now)?;
            }
        }
        for role in ["ACCOUNTANT", "AUDITOR"] {
            for code in [
                "documents.templates.view",
                "reports.view",
                "reports.export",
                "audit.view",
            ] {
                grant(&transaction, role, code, &now)?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
}

fn grant(
    transaction: &Transaction<'_>,
    role: &str,
    permission: &str,
    now: &str,
) -> Phase09Result<()> {
    transaction.execute(
        r#"INSERT OR IGNORE INTO role_permissions(id,company_id,role_id,permission_id,granted_at,granted_by)
           SELECT 'rp-p09-'||r.id||'-'||p.id,r.company_id,r.id,p.id,?1,NULL
           FROM roles r JOIN permissions p ON p.code=?2
           WHERE r.code=?3 AND r.is_active=1"#,
        params![now, permission, role],
    )?;
    Ok(())
}
