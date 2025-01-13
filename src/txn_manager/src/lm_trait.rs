use common::ids::Permissions;
use common::ids::TransactionId;
use common::ids::ValueId;
use common::CrustyError;

pub trait LockManagerTrait {

    fn new(timeout_ms: u64) -> Self;

    fn clear(&self);

    fn acquire_lock(&self, tid: TransactionId, vid: ValueId, perm: Permissions) -> Result<(), CrustyError>;

    fn release_lock(&self, tid: TransactionId, vid: ValueId) -> Result<(), CrustyError>;

    fn release_all_locks(&self, tid: TransactionId) -> Result<(), CrustyError>;
    
    fn locks_held(&self, tid: TransactionId) -> Vec<ValueId>;

    fn upgrade_lock(&self, tid: TransactionId, vid: ValueId) -> Result<(), CrustyError>;

    fn downgrade_lock(&self, tid: TransactionId, vid: ValueId) -> Result<(), CrustyError>;
}
