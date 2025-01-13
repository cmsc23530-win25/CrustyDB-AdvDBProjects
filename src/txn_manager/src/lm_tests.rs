#[cfg(test)]
mod test {
    use crate::lm_trait::LockManagerTrait;
    use crate::lockmanager::*;
    use common::prelude::*;
    use common::testutil::init;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Arc, Barrier};
    use std::thread::{self, sleep};
    use std::time::Duration;
    use rand::Rng;
    const TIMEOUT_MS: u64 = 500;

    struct Wrapper2 {
        l1: Arc<LockManager>,
        l2: Arc<LockManager>,
        vid1: ValueId,
        vid2: ValueId,
        txn1: TransactionId,
        txn2: TransactionId,
        b1: Arc<Barrier>,
        b2: Arc<Barrier>,
    }

    struct WrapperN {
        l: Vec<Arc<LockManager>>,
        vid: Vec<ValueId>,
        t: Vec<TransactionId>,
        b: Vec<Arc<Barrier>>
    }

    impl Wrapper2 {
        fn new() -> Self {
            let vid1 = ValueId::new_page(1, 1);
            let vid2 = ValueId::new_page(1, 2);
            let txn1 = TransactionId::new();
            let txn2 = TransactionId::new();
            let lm = LockManager::new(TIMEOUT_MS);
            let l1 = Arc::new(lm);
            let l2 = l1.clone();
            let barrier = Arc::new(Barrier::new(2));
            let b1 = Arc::clone(&barrier);
            let b2 = Arc::clone(&barrier);
            Self {
                l1,
                l2,
                vid1,
                vid2,
                txn1,
                txn2,
                b1,
                b2,
            }
        }
    }

    impl WrapperN {
        /// Create a wrapper for n txns on t threads for
        /// c containers with p pages.
        fn new(txns: usize, threads: usize, c: ContainerId, p: PageId) -> Self {
            let mut l = Vec::new();
            let mut vid = Vec::new();
        
            for i in 0..c {
                for j in 0..p {
                    vid.push(ValueId::new_page(i, j));
                }
            }
            let mut t = Vec::new();
            let mut b = Vec::new();
            let lm = Arc::new(LockManager::new(TIMEOUT_MS));
            let barrier = Arc::new(Barrier::new(threads));
            for _ in 0..txns {
                t.push(TransactionId::new());
            }
            for _ in 0..threads {
                l.push(Arc::clone(&lm));
                b.push(Arc::clone(&barrier));
            }
            Self {
                l,
                vid,
                t,
                b
            }
        }
    }

    #[test]
    fn test_simple_locks() {
        init();
        // Set up value, txns, and lock manager.
        let vid1 = ValueId::new_page(1, 1);
        let vid2 = ValueId::new_page(2, 2);
        let vid3 = ValueId::new_page(2, 1);
        let vid4 = ValueId::new_page(2, 3);

        let txn = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Acquire locks.
        assert!(lm.acquire_lock(txn, vid1, Permissions::ReadOnly).is_ok());
        assert!(lm.acquire_lock(txn, vid2, Permissions::ReadWrite).is_ok());

        // Check locks held.
        assert_eq!(lm.locks_held(txn).len(), 2);
        assert!(lm.locks_held(txn).contains(&vid1));
        assert!(lm.locks_held(txn).contains(&vid2));

        // Try to acquire lock on same value.
        assert!(lm.acquire_lock(txn, vid1, Permissions::ReadOnly).is_ok());
        assert_eq!(lm.locks_held(txn).len(), 2);

        // Release and acquire locks.
        assert_eq!(lm.release_lock(txn, vid2),Ok(()));
        assert!(lm.acquire_lock(txn, vid3, Permissions::ReadWrite).is_ok());
        assert!(lm.acquire_lock(txn, vid4, Permissions::ReadWrite).is_ok());
        assert_eq!(lm.locks_held(txn).len(), 3);

        // Release all locks.
        assert_eq!(lm.release_all_locks(txn),Ok(()));
        assert_eq!(lm.locks_held(txn).len(), 0);

        // Reacquire locks.
        assert!(lm.acquire_lock(txn, vid3, Permissions::ReadWrite).is_ok());
        assert_eq!(lm.locks_held(txn), vec![vid3]);
    }

    #[test]
    fn test_simple_shared_lock() {
        init();
        // Set up value, txns, and lock manager.
        let vid = ValueId::new_page(1, 1);
        let txn1 = TransactionId::new();
        let txn2 = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Both threads read.
        assert!(lm.acquire_lock(txn1, vid, Permissions::ReadOnly).is_ok());
        assert!(lm.acquire_lock(txn2, vid, Permissions::ReadOnly).is_ok());

        assert!(lm.locks_held(txn1).contains(&vid));
        assert!(lm.locks_held(txn2).contains(&vid));
    }

    #[test]
    fn test_simple_errors() {
        init();
        let lm = LockManager::new(TIMEOUT_MS);
        let vid = ValueId::new_page(1, 1);
        let txn = TransactionId::new();
        assert!(lm.release_lock(txn, vid).is_err());
        assert!(lm.release_all_locks(txn).is_err());

    }
    

    #[test]
    fn test_simple_release_shared_lock() {
        init();
        // Set up value, txns, and lock manager.
        let vid = ValueId::new_page(1, 1);
        let txn1 = TransactionId::new();
        let txn2 = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Both threads read.
        assert!(lm.acquire_lock(txn1, vid, Permissions::ReadOnly).is_ok());
        assert!(lm.acquire_lock(txn2, vid, Permissions::ReadOnly).is_ok());

        // Both threads release.
        lm.release_lock(txn1, vid).unwrap();
        lm.release_lock(txn2, vid).unwrap();
        assert_eq!(lm.locks_held(txn1).len(), 0);
        assert_eq!(lm.locks_held(txn2).len(), 0);
    }

    #[test]
    fn test_simple_release_missing_lock() {
        init();
        // Set up value, txns, and lock manager.
        let vid = ValueId::new_page(1, 1);
        let txn1 = TransactionId::new();
        let txn2 = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // txn1 read.
        assert!(lm.acquire_lock(txn1, vid, Permissions::ReadOnly).is_ok());

        // Both threads release, but txn2 never had lock.
        assert!(lm.release_lock(txn1, vid).is_ok());
        assert!(lm.release_lock(txn2, vid).is_err());
    }

    #[test]
    #[should_panic]
    fn test_release_missing_value() {
        init();
        // Set up value, txns, and lock manager.
        let vid1 = ValueId::new_page(1, 1);
        let vid2 = ValueId::new_page(2, 2);
        let txn = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Txn acquire lock on vid1.
        assert!(lm.acquire_lock(txn, vid1, Permissions::ReadOnly).is_ok());

        // Txn release, but never had lock on vid2.
        lm.release_lock(txn, vid2).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_simple_release_missing_txn() {
        init();
        // Set up value, txns, and lock manager.
        let vid = ValueId::new_page(1, 1);
        let txn = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Txn release, but never had lock.
        lm.release_lock(txn, vid).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_simple_release_locks_missing_txn() {
        init();
        // Set up value, txns, and lock manager.
        let txn = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Txn release, but never had lock.
        lm.release_all_locks(txn).unwrap();
    }

    #[test]
    fn test_upgrade_lock_simple() {
        init();
        // Set up value, txns, and lock manager.
        let vid = ValueId::new_page(1, 1);
        let failvid = ValueId::new_page(2, 4);

        let txn1 = TransactionId::new();
        let txn2 = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Read.
        assert!(lm.acquire_lock(txn1, vid, Permissions::ReadOnly).is_ok());

        // Upgrade to write.
        assert!(lm.upgrade_lock(txn1, vid).is_ok());

        // Fail on upgrading non-existent lock.
        assert!(lm.upgrade_lock(txn1, failvid).is_err());

        // Fail on a txn with no locks
        assert!(lm.upgrade_lock(txn2, failvid).is_err());
    }

    #[test]
    fn test_upgrade_lock() {
        init();
        // Set up value, txns, and lock manager.
        let vid = ValueId::new_page(1, 1);
        let txn1 = TransactionId::new();
        let txn2 = TransactionId::new();
        let lm = LockManager::new(TIMEOUT_MS);

        // Both threads read.
        assert!(lm.acquire_lock(txn1, vid, Permissions::ReadOnly).is_ok());
        assert!(lm.acquire_lock(txn2, vid, Permissions::ReadOnly).is_ok());

        // Try to upgrade without releasing read.
        assert!(lm.upgrade_lock(txn1, vid).is_err(), "Txn1 should not be able to upgrade");

        // txn2 releases to allow txn1 to write.
        lm.release_lock(txn2, vid).unwrap();
        // assert_eq!(lm.locks_held(txn2).len(), 0);
        // assert!(lm.acquire_lock(txn1, vid, Permissions::ReadWrite).is_ok());
    }


    #[test]
    fn test_conc_release_all_queued() -> Result<(), CrustyError> {
        // Check that locks are blocked, have a txn release all locks and see if others
        // can acquire the locks that were waiting.
        init();
        // check that an aborted txn releases all locks and requests
        let mut w = WrapperN::new(3, 3, 1, 3);
        let vid1 = w.vid[0];
        let vid2 = w.vid[1];
        let vid3 = w.vid[2];
        let (txn1, txn2, txn3) = (w.t[0], w.t[1], w.t[2]);
        let (l1, l2, l3) = (w.l.pop().unwrap(), w.l.pop().unwrap(), w.l.pop().unwrap());
        let (b1, b2, b3) = (w.b.pop().unwrap(), w.b.pop().unwrap(), w.b.pop().unwrap());
        let t1 = thread::spawn(move || {
            assert!(l1.acquire_lock(txn1, vid1, Permissions::ReadWrite).is_ok());
            b1.wait();
            assert!(l1.acquire_lock(txn1, vid2, Permissions::ReadWrite).is_err());
            b1.wait();
            // Queue up on v3
            assert!(l1.acquire_lock(txn1, vid3, Permissions::ReadOnly).is_ok());
            let t1locks = l1.locks_held(txn1);
            assert!(t1locks.contains(&vid1));
            assert!(t1locks.contains(&vid3));

        });
        let t2 = thread::spawn(move || {
            assert!(l2.acquire_lock(txn2, vid2, Permissions::ReadWrite).is_ok());
            assert!(l2.acquire_lock(txn2, vid3, Permissions::ReadWrite).is_ok());
            b2.wait();
            b2.wait();
            thread::sleep(Duration::from_millis(TIMEOUT_MS/6));
            //Mock abort
            assert!(l2.release_all_locks(txn2).is_ok());
            assert!(l2.locks_held(txn2).is_empty());

        });
        let t3 = thread::spawn(move || {
            b3.wait();
            assert!(l3.acquire_lock(txn3, vid3, Permissions::ReadOnly).is_err());
            b3.wait();
            // Queue up on v3
            assert!(l3.acquire_lock(txn3, vid3, Permissions::ReadOnly).is_ok());
            let t3locks = l3.locks_held(txn3);
            assert!(t3locks.contains(&vid3));
        });
        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_conc_read_shared_queued() -> Result<(), CrustyError> {
        init();
        let mut w = WrapperN::new(3, 3, 1, 2);
        let vid1 = w.vid[0];
        let vid2 = w.vid[1];
        let (txn1, txn2, txn3) = (w.t[0], w.t[1], w.t[2]);
        let (l1, l2, l3) = (w.l.pop().unwrap(), w.l.pop().unwrap(), w.l.pop().unwrap());
        let (b1, b2, b3) = (w.b.pop().unwrap(), w.b.pop().unwrap(), w.b.pop().unwrap());

        let t1 = thread::spawn(move || {
            assert!(l1.acquire_lock(txn1, vid1, Permissions::ReadOnly).is_ok());
            assert!(l1.acquire_lock(txn1, vid2, Permissions::ReadWrite).is_ok());
            b1.wait();
            b1.wait();
            thread::sleep(Duration::from_millis(TIMEOUT_MS/10));
            assert!(l1.release_all_locks(txn1).is_ok());
        });
        let t2 = thread::spawn(move || {
            b2.wait();
            assert!(l2.acquire_lock(txn2, vid1, Permissions::ReadOnly).is_ok());
            b2.wait();
            assert!(l2.acquire_lock(txn2, vid2, Permissions::ReadOnly).is_ok());

        });
        let t3 = thread::spawn(move || {
            b3.wait();
            assert!(l3.acquire_lock(txn3, vid1, Permissions::ReadOnly).is_ok());
            b3.wait();
            assert!(l3.acquire_lock(txn3, vid2, Permissions::ReadOnly).is_ok());
        });
        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_conc_wait() -> Result<(), CrustyError> {
        init();
        let w = Wrapper2::new();

        let t1 = thread::spawn(move || {
            {
                assert!(w.l1.acquire_lock(w.txn1, w.vid1, Permissions::ReadWrite).is_ok());
                assert!(w.l1.acquire_lock(w.txn1, w.vid2, Permissions::ReadOnly).is_ok());
                w.b1.wait();
            }
            {
                sleep(Duration::from_millis(TIMEOUT_MS/4));
                w.b1.wait();
                if w.l1.release_all_locks(w.txn1).is_err() {
                    unreachable!("Should not fail");
                }
                w.b1.wait();
            }
        });
        let t2 = thread::spawn(move || {
            {
                assert!(w.l2.acquire_lock(w.txn2, w.vid2, Permissions::ReadOnly).is_ok());
                w.b2.wait();
                assert!(!w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadWrite).is_ok());
            }
            {
                w.b2.wait();
                w.b2.wait();
                assert!(w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadWrite).is_ok());
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        Ok(())
    }

    #[test]
    #[allow(clippy::unnecessary_wraps)]
    fn test_conc_lm() -> Result<(), CrustyError> {
        init();
        let w = Wrapper2::new();

        let t1 = thread::spawn(move || {
            {
                assert!(w.l1.acquire_lock(w.txn1, w.vid1, Permissions::ReadWrite).is_ok());
                assert!(w.l1.acquire_lock(w.txn1, w.vid1, Permissions::ReadWrite).is_ok());
                assert!(w.l1.acquire_lock(w.txn1, w.vid2, Permissions::ReadOnly).is_ok());
                w.b1.wait();
            }
            {
                w.b1.wait();
                if w.l1.release_all_locks(w.txn1).is_err() {
                    unreachable!("Should not fail");
                }
                w.b1.wait();
            }
        });
        let t2 = thread::spawn(move || {
            {
                w.b2.wait();
                assert!(!w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadWrite).is_ok());
                assert!(w.l2.acquire_lock(w.txn2, w.vid2, Permissions::ReadOnly).is_ok());
            }
            {
                w.b2.wait();
                w.b2.wait();
                assert!(w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadWrite).is_ok());
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_timeout() -> Result<(), CrustyError> {
        init();
        let w = Wrapper2::new();
        let t1 = thread::spawn(move || {
            {
                assert!(w.l1.acquire_lock(w.txn1, w.vid1, Permissions::ReadWrite).is_ok());
                w.b1.wait();
            }
            {
                sleep(Duration::from_millis(TIMEOUT_MS));
                w.b1.wait();
                if w.l1.release_all_locks(w.txn1).is_err() {
                    unreachable!("Should not fail");
                }
            }
        });
        let t2 = thread::spawn(move || {
            {
                w.b2.wait();
                assert!(w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadWrite).is_err(),
                        "Should fail to acquire lock due to timeout");
                w.b2.wait();
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_deadlock() -> Result<(), CrustyError> {
        init();
        let w = Wrapper2::new();
        let t1 = thread::spawn(move || {
            {
                assert!(w.l1.acquire_lock(w.txn1, w.vid1, Permissions::ReadWrite).is_ok());
                w.b1.wait();
            }
            {
                assert!(w.l1.acquire_lock(w.txn1, w.vid2, Permissions::ReadWrite).is_err(),
                    "Should fail to acquire lock due to deadlock");
            }
        });
        let t2 = thread::spawn(move || {
            {
                assert!(w.l2.acquire_lock(w.txn2, w.vid2, Permissions::ReadWrite).is_ok());
                w.b2.wait();
                assert!(w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadWrite).is_err(),
                        "Should fail to acquire lock due to deadlock");
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_lm_no_starvation() -> Result<(), CrustyError> {
        init();
        let mut w = WrapperN::new(3,3,1,1);
        let vid1 = w.vid[0];
        let (txn1, txn2, txn3) = (w.t[0], w.t[1], w.t[2]);
        let (l1, l2, l3) = (w.l.pop().unwrap(), w.l.pop().unwrap(), w.l.pop().unwrap());
        let (b1, b2, b3) = (w.b.pop().unwrap(), w.b.pop().unwrap(), w.b.pop().unwrap());
        
        let t1 = thread::spawn(move || {
            assert!(l1.acquire_lock(txn1, vid1, Permissions::ReadOnly).is_ok());
            b1.wait();
        });
        let t2 = thread::spawn(move || {
            b2.wait();
            assert!(l2.acquire_lock(txn2, vid1, Permissions::ReadWrite).is_err());
        });
        let t3 = thread::spawn(move || {
            b3.wait();
            thread::sleep(Duration::from_millis(50));
            assert!(l3.acquire_lock(txn3, vid1, Permissions::ReadOnly).is_err());

        });
        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_conc_upgrade_blocked() -> Result<(), CrustyError> {
        init();
        //SL, SL, Upgrade
        let w = Wrapper2::new();

        let t1 = thread::spawn(move || {
            assert!(w.l1.acquire_lock(w.txn1, w.vid1, Permissions::ReadOnly).is_ok());
            w.b1.wait();
        });
        let t2 = thread::spawn(move || {
            assert!(w.l2.acquire_lock(w.txn2, w.vid1, Permissions::ReadOnly).is_ok());
            w.b2.wait();
            // Upgrade should fail as txn1 has the shared lock
            assert!(w.l2.upgrade_lock(w.txn2, w.vid1).is_err());
        });
        t1.join().unwrap();
        t2.join().unwrap();
        Ok(())
    }


    #[test]
    fn test_conc_downgrade_shared() -> Result<(), CrustyError> {
        init();
        // XL, SL, SL Downgrade should give shared locks to others
        let mut w = WrapperN::new(3,3,1,1);
        let vid1 = w.vid[0];
        let (txn1, txn2, txn3) = (w.t[0], w.t[1], w.t[2]);
        let (l1, l2, l3) = (w.l.pop().unwrap(), w.l.pop().unwrap(), w.l.pop().unwrap());
        let (b1, b2, b3) = (w.b.pop().unwrap(), w.b.pop().unwrap(), w.b.pop().unwrap());
        
        let t1 = thread::spawn(move || {
            assert!(l1.acquire_lock(txn1, vid1, Permissions::ReadWrite).is_ok());
            b1.wait();
            thread::sleep(Duration::from_millis(TIMEOUT_MS/10));
            assert!(l1.downgrade_lock(txn1, vid1).is_ok());
            b1.wait();
            assert!(l1.release_lock(txn1, vid1).is_ok());
        });
        let t2 = thread::spawn(move || {
            b2.wait();
            assert!(l2.acquire_lock(txn2, vid1, Permissions::ReadOnly).is_ok());
            assert!(l2.locks_held(txn2).contains(&vid1));
            b2.wait();
        });
        let t3 = thread::spawn(move || {
            b3.wait();
            assert!(l3.acquire_lock(txn3, vid1, Permissions::ReadOnly).is_ok());
            assert!(l3.locks_held(txn3).contains(&vid1));
            b3.wait();

        });
        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        Ok(())
    }

    #[test]
    fn test_conc_stress() -> Result<(), CrustyError> {
        init();
        let fails = Arc::new(AtomicUsize::new(0));

        let thread_cnt = 8;
        let c_count = 8;
        let p_count = 1000;

        let txns = 1000;
        let txn_per_thread = txns / thread_cnt;
        let min_txn_len = 5;
        let max_txn_len = 10;

        let fail_rate = 0.005;
        let avg_ops = (max_txn_len+min_txn_len)/2 * txns;
        let max_fails = fail_rate * avg_ops as f64;
        
        let mut w = WrapperN::new(txns, thread_cnt, c_count, p_count);
        let mut threads = Vec::new();
        for _i in 0..thread_cnt {
            let lm = w.l.pop().unwrap();
            let tfails = Arc::clone(&fails);
            let t = thread::spawn(move || {
                let mut rng = rand::thread_rng();    
                for _ in 0..txn_per_thread {
                    let txn = TransactionId::new();
                    let len = rng.gen_range(min_txn_len..=max_txn_len);
                    for _ in 0..len {
                        let perm = if rng.gen_bool(0.5) {
                            Permissions::ReadOnly
                        } else {
                            Permissions::ReadWrite
                        };
                        let vid = ValueId::new_page(rng.gen_range(0..c_count), rng.gen_range(0..p_count));
                        if lm.acquire_lock(txn, vid, perm).is_err() {
                            tfails.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    lm.release_all_locks(txn).unwrap();
                }
            });
            threads.push(t);
        }
        for t in threads {
            t.join().unwrap();
        }
        let failures = fails.load(std::sync::atomic::Ordering::Relaxed);
        assert!(failures <= (max_fails as usize), "Too many failures. Max {max_fails} had {failures}");
        Ok(())
    }

}
