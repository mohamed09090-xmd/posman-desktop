use rusqlite::{params, OptionalExtension};

use super::{
    dto::{
        CreateRoleRequest, CreateUserRequest, Page, PageRequest, ResetUserPasswordRequest,
        RoleView, SetRolePermissionsRequest, SetUserRolesRequest, UpdateRoleRequest,
        UpdateUserRequest, UserView,
    },
    error::{Phase05Error, Phase05Result},
    state::{audit, new_id, normalize_username, now_iso, trim_required, Phase05Service},
};

impl Phase05Service {
    pub fn list_users(&self, request: PageRequest) -> Phase05Result<Page<UserView>> {
        let context = self.require_session(Some("security.users.view"))?;
        let page = request.page.unwrap_or(1).max(1);
        let page_size = request.page_size.unwrap_or(25).clamp(1, 100);
        let offset = i64::from(page.saturating_sub(1)) * i64::from(page_size);
        let search = request.search.unwrap_or_default().trim().to_lowercase();
        let pattern = format!("%{search}%");
        let connection = self.open()?;
        let total: i64 = connection.query_row(
            "SELECT COUNT(*) FROM users WHERE company_id=?1 AND (?2='' OR lower(username) LIKE ?3 OR lower(display_name) LIKE ?3)",
            params![context.company_id, search, pattern], |row| row.get(0))?;
        let mut statement = connection.prepare(
            r#"
            SELECT id, username, display_name, preferred_language, is_active,
                   failed_login_count, locked_until, row_version
            FROM users WHERE company_id=?1 AND (
                ?2='' OR lower(username) LIKE ?3 OR lower(display_name) LIKE ?3
            ) ORDER BY is_active DESC, username LIMIT ?4 OFFSET ?5
            "#,
        )?;
        let rows = statement.query_map(params![context.company_id, search, pattern, page_size, offset], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)? == 1, row.get::<_, i64>(5)?, row.get::<_, Option<String>>(6)?, row.get::<_, i64>(7)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(UserView { role_ids: load_user_role_ids(&connection, &context.company_id, &row.0)?, id: row.0, username: row.1, display_name: row.2, preferred_language: row.3, is_active: row.4, failed_login_count: row.5, locked_until: row.6, row_version: row.7 });
        }
        Ok(Page { items, page, page_size, total: u64::try_from(total).unwrap_or(0) })
    }

    pub fn create_user(&self, request: CreateUserRequest) -> Phase05Result<UserView> {
        if request.password != request.password_confirmation || !matches!(request.preferred_language.as_str(), "ar" | "fr") { return Err(Phase05Error::invalid("user")); }
        let context = self.require_session(Some("security.users.manage"))?;
        let username = normalize_username(&request.username)?;
        let display_name = trim_required(&request.display_name, "displayName")?;
        let password_hash = self.password_engine.hash(&request.password)?;
        let mut connection = self.open()?;
        validate_roles(&connection, &context.company_id, &request.role_ids)?;
        let id = new_id();
        let timestamp = now_iso()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            INSERT INTO users (id, company_id, username, display_name, password_hash,
                preferred_language, created_at, created_by, updated_at, updated_by)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7, ?8)
            "#,
            params![id, context.company_id, username, display_name, password_hash, request.preferred_language, timestamp, context.user_id],
        )?;
        replace_user_roles(&transaction, &context.company_id, &id, &request.role_ids, &context.user_id, &timestamp)?;
        audit(&transaction, &context, "security.user.create", "users", &id, None)?;
        transaction.commit()?;
        self.get_user(&id)
    }

    pub fn update_user(&self, request: UpdateUserRequest) -> Phase05Result<UserView> {
        if !matches!(request.preferred_language.as_str(), "ar" | "fr") { return Err(Phase05Error::invalid("preferredLanguage")); }
        let context = self.require_session(Some("security.users.manage"))?;
        if request.id == context.user_id && !request.is_active { return Err(Phase05Error::new("CURRENT_USER_DEACTIVATION_FORBIDDEN", "The current user cannot be deactivated during the active session.")); }
        let mut connection = self.open()?;
        if !request.is_active && is_system_administrator(&connection, &context.company_id, &request.id)? && active_system_administrators(&connection, &context.company_id)? <= 1 { return Err(last_admin_error()); }
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            r#"
            UPDATE users SET display_name=?1, preferred_language=?2, is_active=?3,
                updated_at=?4, updated_by=?5, row_version=row_version+1
            WHERE id=?6 AND company_id=?7 AND row_version=?8
            "#,
            params![trim_required(&request.display_name, "displayName")?, request.preferred_language, if request.is_active {1_i64} else {0_i64}, now_iso()?, context.user_id, request.id, context.company_id, request.row_version],
        )?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        if !request.is_active { transaction.execute("UPDATE sessions SET revoked_at=?1 WHERE user_id=?2 AND company_id=?3 AND revoked_at IS NULL", params![now_iso()?, request.id, context.company_id])?; }
        audit(&transaction, &context, "security.user.update", "users", &request.id, None)?;
        transaction.commit()?;
        self.get_user(&request.id)
    }

    pub fn set_user_roles(&self, request: SetUserRolesRequest) -> Phase05Result<UserView> {
        let context = self.require_session(Some("security.roles.manage"))?;
        let mut connection = self.open()?;
        validate_roles(&connection, &context.company_id, &request.role_ids)?;
        if is_system_administrator(&connection, &context.company_id, &request.user_id)?
            && !role_list_contains_system_administrator(&connection, &context.company_id, &request.role_ids)?
            && active_system_administrators(&connection, &context.company_id)? <= 1 { return Err(last_admin_error()); }
        let transaction = connection.transaction()?;
        let timestamp = now_iso()?;
        replace_user_roles(&transaction, &context.company_id, &request.user_id, &request.role_ids, &context.user_id, &timestamp)?;
        audit(&transaction, &context, "security.user.roles.set", "users", &request.user_id, None)?;
        transaction.commit()?;
        self.get_user(&request.user_id)
    }

    pub fn reset_user_password(&self, request: ResetUserPasswordRequest) -> Phase05Result<()> {
        if request.new_password != request.new_password_confirmation { return Err(Phase05Error::new("PASSWORD_CONFIRMATION_MISMATCH", "The password confirmation does not match.")); }
        let context = self.require_session(Some("security.users.manage"))?;
        let hash = self.password_engine.hash(&request.new_password)?;
        let mut connection = self.open()?;
        let transaction = connection.transaction()?;
        let changed = transaction.execute(
            r#"
            UPDATE users SET password_hash=?1, failed_login_count=0, locked_until=NULL,
                updated_at=?2, updated_by=?3, row_version=row_version+1
            WHERE id=?4 AND company_id=?5
            "#,
            params![hash, now_iso()?, context.user_id, request.user_id, context.company_id],
        )?;
        if changed != 1 { return Err(Phase05Error::new("NOT_FOUND", "The user was not found.")); }
        transaction.execute("UPDATE sessions SET revoked_at=?1 WHERE user_id=?2 AND company_id=?3 AND revoked_at IS NULL", params![now_iso()?, request.user_id, context.company_id])?;
        audit(&transaction, &context, "security.user.password.reset", "users", &request.user_id, None)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn list_roles(&self) -> Phase05Result<Vec<RoleView>> {
        let context = self.require_session(Some("security.users.view"))?;
        let connection = self.open()?;
        let mut statement = connection.prepare("SELECT id, code, name_ar, name_fr, is_system, is_active, row_version FROM roles WHERE company_id=?1 ORDER BY is_system DESC, code")?;
        let rows = statement.query_map([context.company_id.as_str()], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, i64>(4)? == 1, row.get::<_, i64>(5)? == 1, row.get::<_, i64>(6)?)))?.collect::<Result<Vec<_>, _>>()?;
        let mut roles = Vec::with_capacity(rows.len());
        for row in rows { roles.push(RoleView { permission_ids: load_role_permission_ids(&connection, &context.company_id, &row.0)?, id: row.0, code: row.1, name_ar: row.2, name_fr: row.3, is_system: row.4, is_active: row.5, row_version: row.6 }); }
        Ok(roles)
    }

    pub fn create_role(&self, request: CreateRoleRequest) -> Phase05Result<RoleView> {
        let context = self.require_session(Some("security.roles.manage"))?;
        let code = trim_required(&request.code, "code")?.to_uppercase();
        let mut connection = self.open()?;
        validate_permissions(&connection, &request.permission_ids)?;
        let id = new_id();
        let timestamp = now_iso()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            r#"
            INSERT INTO roles (id, company_id, code, name_ar, name_fr, is_system,
                created_at, created_by, updated_at, updated_by)
            VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?6, ?7)
            "#,
            params![id, context.company_id, code, trim_required(&request.name_ar, "nameAr")?, trim_required(&request.name_fr, "nameFr")?, timestamp, context.user_id],
        )?;
        replace_role_permissions(&transaction, &context.company_id, &id, &request.permission_ids, &context.user_id, &timestamp)?;
        audit(&transaction, &context, "security.role.create", "roles", &id, None)?;
        transaction.commit()?;
        self.get_role(&id)
    }

    pub fn update_role(&self, request: UpdateRoleRequest) -> Phase05Result<RoleView> {
        let context = self.require_session(Some("security.roles.manage"))?;
        let mut connection = self.open()?;
        let system = connection.query_row("SELECT is_system FROM roles WHERE id=?1 AND company_id=?2", params![request.id, context.company_id], |row| row.get::<_, i64>(0)).optional()?.ok_or_else(|| Phase05Error::new("NOT_FOUND", "The role was not found."))? == 1;
        if system && !request.is_active { return Err(Phase05Error::new("SYSTEM_ADMINISTRATOR_ROLE_PROTECTED", "The System Administrator role cannot be deactivated or deleted.")); }
        let transaction = connection.transaction()?;
        let changed = transaction.execute("UPDATE roles SET name_ar=?1,name_fr=?2,is_active=?3,updated_at=?4,updated_by=?5,row_version=row_version+1 WHERE id=?6 AND company_id=?7 AND row_version=?8", params![trim_required(&request.name_ar, "nameAr")?, trim_required(&request.name_fr, "nameFr")?, if request.is_active {1_i64} else {0_i64}, now_iso()?, context.user_id, request.id, context.company_id, request.row_version])?;
        if changed != 1 { return Err(Phase05Error::concurrency()); }
        audit(&transaction, &context, "security.role.update", "roles", &request.id, None)?;
        transaction.commit()?;
        self.get_role(&request.id)
    }

    pub fn set_role_permissions(&self, request: SetRolePermissionsRequest) -> Phase05Result<RoleView> {
        let context = self.require_session(Some("security.roles.manage"))?;
        let mut connection = self.open()?;
        validate_permissions(&connection, &request.permission_ids)?;
        let system: bool = connection.query_row("SELECT is_system FROM roles WHERE id=?1 AND company_id=?2", params![request.role_id, context.company_id], |row| Ok(row.get::<_, i64>(0)? == 1))?;
        if system {
            let all = all_permission_ids(&connection)?;
            if request.permission_ids.len() != all.len() || !all.iter().all(|id| request.permission_ids.contains(id)) { return Err(Phase05Error::new("SYSTEM_ADMINISTRATOR_PERMISSIONS_PROTECTED", "The System Administrator role must retain every current permission.")); }
        }
        let transaction = connection.transaction()?;
        replace_role_permissions(&transaction, &context.company_id, &request.role_id, &request.permission_ids, &context.user_id, &now_iso()?)?;
        audit(&transaction, &context, "security.role.permissions.set", "roles", &request.role_id, None)?;
        transaction.commit()?;
        self.get_role(&request.role_id)
    }

    fn get_user(&self, id: &str) -> Phase05Result<UserView> {
        let context = self.require_session(Some("security.users.view"))?;
        let connection = self.open()?;
        let mut user = connection.query_row("SELECT id,username,display_name,preferred_language,is_active,failed_login_count,locked_until,row_version FROM users WHERE id=?1 AND company_id=?2", params![id, context.company_id], |row| Ok(UserView { id: row.get(0)?, username: row.get(1)?, display_name: row.get(2)?, preferred_language: row.get(3)?, is_active: row.get::<_, i64>(4)? == 1, failed_login_count: row.get(5)?, locked_until: row.get(6)?, role_ids: Vec::new(), row_version: row.get(7)? })).optional()?.ok_or_else(|| Phase05Error::new("NOT_FOUND", "The user was not found."))?;
        user.role_ids = load_user_role_ids(&connection, &context.company_id, id)?;
        Ok(user)
    }

    fn get_role(&self, id: &str) -> Phase05Result<RoleView> {
        let context = self.require_session(Some("security.users.view"))?;
        let connection = self.open()?;
        let mut role = connection.query_row("SELECT id,code,name_ar,name_fr,is_system,is_active,row_version FROM roles WHERE id=?1 AND company_id=?2", params![id, context.company_id], |row| Ok(RoleView { id: row.get(0)?, code: row.get(1)?, name_ar: row.get(2)?, name_fr: row.get(3)?, is_system: row.get::<_, i64>(4)? == 1, is_active: row.get::<_, i64>(5)? == 1, permission_ids: Vec::new(), row_version: row.get(6)? })).optional()?.ok_or_else(|| Phase05Error::new("NOT_FOUND", "The role was not found."))?;
        role.permission_ids = load_role_permission_ids(&connection, &context.company_id, id)?;
        Ok(role)
    }
}

fn replace_user_roles(transaction: &rusqlite::Transaction<'_>, company_id: &str, user_id: &str, role_ids: &[String], actor_id: &str, timestamp: &str) -> Phase05Result<()> {
    transaction.execute("DELETE FROM user_roles WHERE company_id=?1 AND user_id=?2", params![company_id, user_id])?;
    for role_id in role_ids { transaction.execute("INSERT INTO user_roles (id,company_id,user_id,role_id,assigned_at,assigned_by) VALUES (?1,?2,?3,?4,?5,?6)", params![new_id(), company_id, user_id, role_id, timestamp, actor_id])?; }
    Ok(())
}

fn replace_role_permissions(transaction: &rusqlite::Transaction<'_>, company_id: &str, role_id: &str, permission_ids: &[String], actor_id: &str, timestamp: &str) -> Phase05Result<()> {
    transaction.execute("DELETE FROM role_permissions WHERE company_id=?1 AND role_id=?2", params![company_id, role_id])?;
    for permission_id in permission_ids { transaction.execute("INSERT INTO role_permissions (id,company_id,role_id,permission_id,granted_at,granted_by) VALUES (?1,?2,?3,?4,?5,?6)", params![new_id(), company_id, role_id, permission_id, timestamp, actor_id])?; }
    Ok(())
}

fn validate_roles(connection: &rusqlite::Connection, company_id: &str, role_ids: &[String]) -> Phase05Result<()> {
    for role_id in role_ids { if connection.query_row("SELECT 1 FROM roles WHERE id=?1 AND company_id=?2 AND is_active=1", params![role_id, company_id], |_| Ok(())).optional()?.is_none() { return Err(Phase05Error::invalid("roleIds")); } }
    Ok(())
}

fn validate_permissions(connection: &rusqlite::Connection, permission_ids: &[String]) -> Phase05Result<()> {
    for id in permission_ids { if connection.query_row("SELECT 1 FROM permissions WHERE id=?1", [id], |_| Ok(())).optional()?.is_none() { return Err(Phase05Error::invalid("permissionIds")); } }
    Ok(())
}

fn load_user_role_ids(connection: &rusqlite::Connection, company_id: &str, user_id: &str) -> Phase05Result<Vec<String>> {
    let mut statement = connection.prepare("SELECT role_id FROM user_roles WHERE company_id=?1 AND user_id=?2 ORDER BY role_id")?;
    let rows = statement.query_map(params![company_id, user_id], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Phase05Error::from)
}

fn load_role_permission_ids(connection: &rusqlite::Connection, company_id: &str, role_id: &str) -> Phase05Result<Vec<String>> {
    let mut statement = connection.prepare("SELECT permission_id FROM role_permissions WHERE company_id=?1 AND role_id=?2 ORDER BY permission_id")?;
    let rows = statement.query_map(params![company_id, role_id], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Phase05Error::from)
}

fn all_permission_ids(connection: &rusqlite::Connection) -> Phase05Result<Vec<String>> {
    let mut statement = connection.prepare("SELECT id FROM permissions ORDER BY id")?;
    let rows = statement.query_map([], |row| row.get(0))?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Phase05Error::from)
}

fn is_system_administrator(connection: &rusqlite::Connection, company_id: &str, user_id: &str) -> Phase05Result<bool> {
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM user_roles ur JOIN roles r ON r.id=ur.role_id AND r.company_id=ur.company_id WHERE ur.company_id=?1 AND ur.user_id=?2 AND r.code='SYSTEM_ADMINISTRATOR' AND r.is_active=1", params![company_id, user_id], |row| row.get(0))?;
    Ok(count > 0)
}

fn active_system_administrators(connection: &rusqlite::Connection, company_id: &str) -> Phase05Result<i64> {
    connection.query_row("SELECT COUNT(DISTINCT u.id) FROM users u JOIN user_roles ur ON ur.user_id=u.id AND ur.company_id=u.company_id JOIN roles r ON r.id=ur.role_id AND r.company_id=u.company_id WHERE u.company_id=?1 AND u.is_active=1 AND r.code='SYSTEM_ADMINISTRATOR' AND r.is_active=1", [company_id], |row| row.get(0)).map_err(Phase05Error::from)
}

fn role_list_contains_system_administrator(connection: &rusqlite::Connection, company_id: &str, role_ids: &[String]) -> Phase05Result<bool> {
    for role_id in role_ids { let code: Option<String> = connection.query_row("SELECT code FROM roles WHERE id=?1 AND company_id=?2", params![role_id, company_id], |row| row.get(0)).optional()?; if code.as_deref() == Some("SYSTEM_ADMINISTRATOR") { return Ok(true); } }
    Ok(false)
}

fn last_admin_error() -> Phase05Error {
    Phase05Error::new("LAST_SYSTEM_ADMINISTRATOR_PROTECTED", "The last active System Administrator cannot be deactivated or unassigned.")
}
