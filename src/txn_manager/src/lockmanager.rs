use common::ids::Permissions;
use common::ids::TransactionId;
use common::ids::ValueId;
use common::CrustyError;

use crate::lm_trait::LockManagerTrait;

/// Implementation of the lock manager. This holds any state needed by your
/// lock manager. This struct should implement the LockManagerTrait.
pub struct LockManager {
    // Add any needed state here
    timeout_ms: u64,
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new(2000)
    }
}

impl LockManagerTrait for LockManager {
    fn new(timeout_ms: u64) -> Self {
        //TODO Add any initialization code here
        Self { timeout_ms }
    }

    fn clear(&self) {
        panic!("TODO milestone lm");
    }

    fn acquire_lock(
        &self,
        tid: TransactionId,
        vid: ValueId,
        perm: Permissions,
    ) -> Result<(), CrustyError> {
        panic!("TODO milestone lm");
    }

    fn locks_held(&self, tid: TransactionId) -> Vec<ValueId> {
        panic!("TODO milestone lm");
    }

    fn release_lock(&self, tid: TransactionId, vid: ValueId) -> Result<(), CrustyError> {
        panic!("TODO milestone lm");
    }

    fn release_all_locks(&self, tid: TransactionId) -> Result<(), CrustyError> {
        panic!("TODO milestone lm");
    }

    fn upgrade_lock(&self, tid: TransactionId, vid: ValueId) -> Result<(), CrustyError> {
        panic!("TODO milestone lm");
    }

    fn downgrade_lock(&self, tid: TransactionId, vid: ValueId) -> Result<(), CrustyError> {
        panic!("TODO milestone lm");
    }
}
