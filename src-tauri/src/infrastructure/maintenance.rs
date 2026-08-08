use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Condvar, Mutex},
};

use rusqlite::Connection;

#[derive(Clone, Debug)]
pub struct MaintenanceGate {
    inner: Arc<GateInner>,
}

#[derive(Debug)]
struct GateInner {
    state: Mutex<GateState>,
    idle: Condvar,
}

#[derive(Debug, Default)]
struct GateState {
    restore_active: bool,
    active_database_operations: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaintenanceGateError {
    RestoreActive,
    Poisoned,
}

pub struct DatabaseOperationPermit {
    gate: MaintenanceGate,
    released: bool,
}

pub struct RestoreMaintenancePermit {
    gate: MaintenanceGate,
    released: bool,
}

pub struct GuardedConnection {
    connection: Connection,
    _permit: DatabaseOperationPermit,
}

impl MaintenanceGate {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(GateInner {
                state: Mutex::new(GateState::default()),
                idle: Condvar::new(),
            }),
        }
    }

    pub fn ensure_available(&self) -> Result<(), MaintenanceGateError> {
        let state = self
            .inner
            .state
            .lock()
            .map_err(|_| MaintenanceGateError::Poisoned)?;
        if state.restore_active {
            Err(MaintenanceGateError::RestoreActive)
        } else {
            Ok(())
        }
    }

    pub fn enter_database_operation(
        &self,
    ) -> Result<DatabaseOperationPermit, MaintenanceGateError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| MaintenanceGateError::Poisoned)?;
        if state.restore_active {
            return Err(MaintenanceGateError::RestoreActive);
        }
        state.active_database_operations = state
            .active_database_operations
            .checked_add(1)
            .ok_or(MaintenanceGateError::Poisoned)?;
        Ok(DatabaseOperationPermit {
            gate: self.clone(),
            released: false,
        })
    }

    pub fn begin_restore(&self) -> Result<RestoreMaintenancePermit, MaintenanceGateError> {
        let mut state = self
            .inner
            .state
            .lock()
            .map_err(|_| MaintenanceGateError::Poisoned)?;
        if state.restore_active {
            return Err(MaintenanceGateError::RestoreActive);
        }
        state.restore_active = true;
        while state.active_database_operations > 0 {
            state = self
                .inner
                .idle
                .wait(state)
                .map_err(|_| MaintenanceGateError::Poisoned)?;
        }
        Ok(RestoreMaintenancePermit {
            gate: self.clone(),
            released: false,
        })
    }

    #[cfg(test)]
    pub fn active_database_operations(&self) -> Result<usize, MaintenanceGateError> {
        self.inner
            .state
            .lock()
            .map(|state| state.active_database_operations)
            .map_err(|_| MaintenanceGateError::Poisoned)
    }

    #[cfg(test)]
    pub fn restore_active(&self) -> Result<bool, MaintenanceGateError> {
        self.inner
            .state
            .lock()
            .map(|state| state.restore_active)
            .map_err(|_| MaintenanceGateError::Poisoned)
    }
}

impl Default for MaintenanceGate {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseOperationPermit {
    pub fn guard(self, connection: Connection) -> GuardedConnection {
        GuardedConnection {
            connection,
            _permit: self,
        }
    }

    fn release(&mut self) {
        if self.released {
            return;
        }
        if let Ok(mut state) = self.gate.inner.state.lock() {
            state.active_database_operations = state.active_database_operations.saturating_sub(1);
            self.gate.inner.idle.notify_all();
        }
        self.released = true;
    }
}

impl Drop for DatabaseOperationPermit {
    fn drop(&mut self) {
        self.release();
    }
}

impl RestoreMaintenancePermit {
    fn release(&mut self) {
        if self.released {
            return;
        }
        if let Ok(mut state) = self.gate.inner.state.lock() {
            state.restore_active = false;
            self.gate.inner.idle.notify_all();
        }
        self.released = true;
    }
}

impl Drop for RestoreMaintenancePermit {
    fn drop(&mut self) {
        self.release();
    }
}

impl Deref for GuardedConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl DerefMut for GuardedConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_waits_for_database_operations_and_releases_on_drop() {
        let gate = MaintenanceGate::new();
        let permit = gate.enter_database_operation().unwrap();
        assert_eq!(gate.active_database_operations().unwrap(), 1);
        drop(permit);
        let restore = gate.begin_restore().unwrap();
        assert!(gate.restore_active().unwrap());
        assert_eq!(
            gate.enter_database_operation().err(),
            Some(MaintenanceGateError::RestoreActive)
        );
        drop(restore);
        assert!(!gate.restore_active().unwrap());
        assert!(gate.enter_database_operation().is_ok());
    }
}
