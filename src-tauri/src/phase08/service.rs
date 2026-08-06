use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
use uuid::Uuid;

use crate::phase05::Phase06AuthContext;

use super::{
    dto::{
        AccountInput, AccountView, EntityVersion, FiscalPeriodView, InstallAccountingTemplateRequest,
        JournalInput, PeriodActionInput, PostingRuleInput,
    },
    error::{Phase08Error, Phase08Result},
    Phase08Service,
};

const PERMISSIONS: &[(&str, &str, &str, bool)] = &[
    ("perm-accounting-read", "accounting.read", "accounting", false),
    ("perm-accounting-configure", "accounting.configure", "accounting", true),
    ("perm-accounting-manual-post", "accounting.manual.post", "accounting", true),
    ("perm-accounting-reverse", "accounting.reverse", "accounting", true),
    ("perm-accounting-period", "accounting.period.manage", "accounting", true),
    ("perm-payment-receipt", "payment.receipt.post", "payments", true),
    ("perm-payment-disbursement", "payment.disbursement.post", "payments", true),
    ("perm-payment-allocate", "payment.allocate", "payments", false),
    ("perm-payment-receipt-allocate", "payment.receipt.allocate", "payments", false),
    ("perm-payment-disbursement-allocate", "payment.disbursement.allocate", "payments", false),
];

pub(crate) fn new_id() -> String { Uuid::now_v7().to_string() }

pub(crate) fn now_iso() -> Phase08Result<String> {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| Phase08Error::internal())
}

pub(crate) fn boolean(value: bool) -> i64 { if value { 1 } else { 0 } }

impl Phase08Service {
    pub fn new(phase05: crate::phase05::Phase05Service) -> Phase08Result<Self> {
        let service = Self { phase05 };
        service.provision_permissions()?;
        Ok(service)
    }

    fn provision_permissions(&self) -> Phase08Result<()> {
        let mut connection = self.phase05.phase06_open().map_err(|_| Phase08Error::internal())?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now = now_iso()?;
        for (id, code, domain, sensitive) in PERMISSIONS {
            tx.execute(
                "INSERT OR IGNORE INTO permissions (id,code,domain,description_ar,description_fr,is_sensitive,created_at) VALUES (?1,?2,?3,?2,?2,?4,?5)",
                params![id, code, domain, boolean(*sensitive), now],
            )?;
        }
        for role in ["OWNER", "SYSTEM_ADMINISTRATOR"] {
            for (_, code, _, _) in PERMISSIONS {
                tx.execute(
                    r#"INSERT OR IGNORE INTO role_permissions(id,company_id,role_id,permission_id,granted_at,granted_by)
                       SELECT ?1 || '-' || r.id || '-' || p.id,r.company_id,r.id,p.id,?2,NULL
                       FROM roles r JOIN permissions p ON p.code=?3
                       WHERE r.code=?4 AND r.is_active=1"#,
                    params!["rp-p08", now, code, role],
                )?;
            }
        }
        for code in ["payment.receipt.post", "payment.receipt.allocate", "accounting.read"] {
            grant_role(&tx, "SALES", code, &now)?;
        }
        for code in ["payment.disbursement.post", "payment.disbursement.allocate", "accounting.read"] {
            grant_role(&tx, "PURCHASING", code, &now)?;
        }
        grant_role(&tx, "AUDITOR", "accounting.read", &now)?;
        tx.commit()?;
        Ok(())
    }

    pub(crate) fn context(&self, permission: Option<&str>) -> Phase08Result<Phase06AuthContext> {
        self.phase05.phase06_authorize(permission).map_err(|_| Phase08Error::permission())
    }

    pub(crate) fn immediate<T>(&self, operation: impl FnOnce(&Transaction<'_>) -> Phase08Result<T>) -> Phase08Result<T> {
        let mut connection = self.phase05.phase06_open().map_err(|_| Phase08Error::internal())?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = operation(&tx)?;
        tx.commit()?;
        Ok(result)
    }

    pub(crate) fn read<T>(&self, operation: impl FnOnce(&rusqlite::Connection) -> Phase08Result<T>) -> Phase08Result<T> {
        let connection = self.phase05.phase06_open().map_err(|_| Phase08Error::internal())?;
        operation(&connection)
    }
}

fn grant_role(tx:&Transaction<'_>,role:&str,permission:&str,now:&str)->Phase08Result<()> {
    tx.execute(r#"INSERT OR IGNORE INTO role_permissions(id,company_id,role_id,permission_id,granted_at,granted_by)
      SELECT 'rp-p08-'||r.id||'-'||p.id,r.company_id,r.id,p.id,?1,NULL FROM roles r JOIN permissions p ON p.code=?2
      WHERE r.code=?3 AND r.is_active=1"#,params![now,permission,role])?;
    Ok(())
}

pub(crate) fn require_company_row(tx:&Transaction<'_>,table:&str,id:&str,company:&str,code:&str)->Phase08Result<()> {
    let sql=format!("SELECT 1 FROM {table} WHERE id=?1 AND company_id=?2");
    if tx.query_row(&sql,params![id,company],|r|r.get::<_,i64>(0)).optional()?.is_none(){
        return Err(Phase08Error::new(code,"The requested company-scoped record was not found.",false));
    }
    Ok(())
}

pub(crate) fn require_active_postable_account(tx:&Transaction<'_>,company:&str,id:&str)->Phase08Result<()> {
    let state=tx.query_row("SELECT is_active,allow_posting FROM accounts WHERE id=?1 AND company_id=?2",params![id,company],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?))).optional()?;
    match state {
        None=>Err(Phase08Error::new("ACCOUNT_NOT_FOUND","The configured account was not found for this company.",false)),
        Some((0,_))=>Err(Phase08Error::new("ACCOUNT_INACTIVE","The configured account is inactive.",false)),
        Some((_,0))=>Err(Phase08Error::new("ACCOUNT_NOT_POSTABLE","The configured account does not accept postings.",false)),
        _=>Ok(())
    }
}
