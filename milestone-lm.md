# Lock Manager

For this milestone you will be implementing a lock manager that would be used a  2PL protocol.  The lock manager will be responsible for managing locks on a per valueId basis. Even though valueIds are hierarchical, the lock manager does not need to consider this hierarchy and lock valueIds at the granularity of the valueId itself (e.g. no hierarchical/intention locks).

The lock manager will be responsible for granting and releasing locks. See the documentation at https://classes.cs.uchicago.edu/archive/2025/winter/23530-1/crusty/txn_manager/lm_trait/trait.LockManagerTrait.html for details and requirements on the implementation and the individual functions. In addition, Chapter 18.1.4 in the book gives a reference on one approach to implementing a lock manager.

For this assignment you will be implementing the lock manager in the `lockmanager.rs` file. You can add any new struct or helper methods inside of this file.  You should not modify any other files. This means you cannot change the function signatures in the `lm_trait.rs` file (and as a result you cannot change the required functions in the `lockmanager.rs` file). This also means you cannot change the cargo.toml file to add any new dependencies. You can import anything from the standard library. The unit/correctness tests for the lock manager are in the `lm_tests.rs` file.  If you want to add any additional tests you can add them to `lockmanager.rs`.

Note you will use interior mutability to implement the lock manager and you would assume that multiple threads would share the lock manager via `Arc<LockManager>`.

To run the tests for the lock manager you can use the following command:
```
cargo test -p txn_manager
```

If you want to run a single test, you suffix the command with the test name. For example, to only run the test `test_upgrade_lock_simple` you would use the following command:

```
cargo test -p txn_manager test_upgrade_lock_simple
```

If you want to run the tests multiple times (eg 10) you can use the following command:
```
for i in 0..10; do cargo test -p txn_manager;done;
```

## Scoring and Requirements
80% of your grade will be based on the tests in `lm_tests.rs`. The other 20% will be based on the quality of your code and your write up.

### Quality
15% of your score is based on code quality (following good coding conventions, comments, well organized functions, etc). You should only use debug statements and not print!.  Comments are only needed for non-obvious code.  You should not have comments for every line of code.  You should have comments for any non-obvious code or any code that is doing something tricky.

**We will run `cargo fmt --check` and `cargo clippy` on your code (txn_manager only), if either fails or reports issues on your code, you will receive a loss on code quality.** 

These are a formatter and linter. You can easily run `cargo fmt` to format your code in the right "style" anc clippy gives you warnings about your code, for either performance reasons or code quality. 

### Write Up
5% of your score is based on the write up.  The write up should be a markdown file named `my-lm.md`.
- A brief description of your solution, in particular what design decisions you made and why. This is only needed for the parts of your solution that involved some significant work (e.g. just returning a counter or a pass through function isn't a design decision).
- How long you roughly spent on the milestone, and what you liked/disliked on the milestone.
- If you know some part of the milestone is incomplete, write up what parts are not working, how close you think you are, and what part(s) you got stuck on.
- What references / questions you may have asked a LLM or classmate.

### Incremental Git Commits and Pushes
As part of our grading we will look at your git history.  We expect to see at least 10 check-ins where we can see incremental changes (and highly expect several commits to replace prior work/fix bugs). These changes should also be pushed to github so we can see them.

**Important:** If there is only one commit, or the commit history is not meaningful, **you will lose 50% of your grade.** This is to encourage incremental progress and ensure that all work is done by the student.

Git commit messages should be meaningful, but can be light. For example, "Release lock working" or "fixing shared bug" are fine. The occasional "WIP" is also fine, especially if you are frequently committing and pushing your changes to github.

## Tips

- Figure out how to read and understand the tests.
- You will need to use interior mutability to implement the lock manager.  Think about what data you need to store and how you can use synchronization primitives to protect that data.
- Avoid if let and match with locks.  Think about acquiring locks with proper scoping for the lock's lifetime.
- Think about how to get timeouts to work. You will likely need to use `std::time::Instant` and `std::time::Duration` to implement timeouts.
- Think about write/read vs try_write/try_read.  
- Barriers are used in the tests to ensure that the tests are run in a specific order.  You should not need to use barriers in your implementation.
- The lock manager is not responsible for enforcing the 2PL protocol.  It is only responsible for managing locks.  A transaction manager would be responsible for enforcing the 2PL protocol.
- The amount of code you need to write is not that large.  The difficulty is in understanding the requirements and implementing the lock manager correctly. 


## Logging / Logging Tests (repeated from pg)

CrustyDB uses the [env_logger](https://docs.rs/env_logger/0.8.2/env_logger/) crate for logging. Per the docs on the log crate:
```
The basic use of the log crate is through the five logging macros: error!, warn!, info!, debug! and trace! 
where error! represents the highest-priority log messages and trace! the lowest.  
The log messages are filtered by configuring the log level to exclude messages with 
a lower priority. Each of these macros accept format strings similarly to println!.
```

The logging level is set by an environmental variable, `RUST_LOG`. The easiest way to set the level is when running a cargo command you set the logging level in the same command. EG : `RUST_LOG=debug cargo run --bin server`. However, when running unit tests the logging/output is suppressed and the logger is not initialized. So if you want to use logging for a test you must:
 - Make sure the test in question calls `init()` which is defined in `common::testutils` that initializes the logger. It can safely be called multiple times.
 - You **may** make a change to common::testutils to set the log level to a specific level. Do not check this in though.
 - Tell cargo to not capture the output. For example, setting the level to DEBUG: `RUST_LOG=debug cargo test -- --nocapture [opt_test_name]`  **note the -- before --nocapture**

## Suggested Steps
First read all of the requirements and the documentation for the lock manager.  Then read the tests in `lm_tests.rs` to understand what is being tested. I would suggest starting with sketching out the solution on paper before implementing it. Then start with the "simple" tests and then move to the complex concurrent (`conc`) tests.  The simple tests are a good way to get a handle on the basic functionality of the lock manager.  The complex tests are more about testing the edge cases and the more complex interactions between transactions.

To run the suite of simple tests you can use the following command:
```
cargo test -p txn_manager simple
```

For example the test `test_simple_locks` is a good starting point.  It tests the basic functionality of the lock manager without concerns of concurrency, timeouts, or upgrades/downgrades.  It is a good starting point to understand how the lock manager works. After move on to the `conc` tests to test the more complex interactions between transactions. Ensure that the stress test works at the end to ensure that your lock manager is aborting too many requests.