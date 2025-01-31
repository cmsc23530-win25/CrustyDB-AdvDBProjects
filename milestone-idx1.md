# Index 1 

For this milestone you will be building out either a hash index file or B+ tree index file. For both structures the implementation will be similar. The main difference will be in the way you handle the search key and how you organize the index structure. Both approaches will require that you implement the index page and the index file.  For the hash index, each page will represent a bucket in the hash table. For the B+ tree index, each page will represent a node in the tree (inner or leaf). The pages will be based on the FixedPage implementation (see below) and changes should only be made within the `fixed_index_page.rs` and `fixed_index_file.rs` files (and potentially changing the flag in `index/mod.rs`). For the `fixed_index_page.rs` you will need to write your own trait, implementation, and tests. For the `fixed_index_file.rs` you will **need to implement the `IndexFileTrait` trait for the `FixedIndexFile` struct**. A few tests are provided but you should add additional tests to ensure your implementation is correct.

Note that the `FixedPage` is slightly different than the page that you built for CrustyDB as each page will have a defined key and value size. The page's metadata (such as free slots) is stored outside of the data array. The `FixedPage` struct is defined in the `fixed_page.rs` file and should *not* be modified. The `FixedPage` struct is used for both the data and index pages and should provide all the necessary functionality for the index file implementation.  For this milestone we assume fixed size records, which will be made up of a key (byte array) and a value (byte array) for the data heap files. We will assume that the key's for data records are unique, but nothing will validate this. For the index files, we will also be using the same fixed size page structure, but each index entry will be made up of a search key (byte array) and a value_id (encoded into a byte array of 10 bytes). For our workloads the last `SEARCH_KEY_SIZE` bytes of the record value will provide the search key. *Search keys are not assumed to be unique.* All of the sizes are defined in the lib.rs prelude.

We have provided the HeapFile and HeapPage implementations for you to use as a reference. You should not need to modify these files outside of adding the lock manager calls in the next milestone. **See `fixed_heap_file.rs` and `fixed_heap_page.rs` for reference in building your index file and page.** Note how the FixedHeapFile uses the buffer pool to read and write pages, along with appending a new page to the file.

*Note* that in a real implementation, you would likely encode the first page of an index file to be a header page. This page would contain metadata about the index file, such as the root page of the index or configuration options. For simplicity, we are not requiring this for this milestone and you can keep all metadata for a file inside the `FixedIndexFile` struct.

## Data Structures
 - FixedPage: The fixed page struct for representing data and index pages for fixed size records. Each record is assumed to have two parts a key and a value. For the data page, the key is assumed (but not checked) to be unique and the value is the data associated with the key. For the index page, the key is the search key and the value is the pointer to a data or index page (e.g. for an inner node of a B+Tree the pointer will be another page in the index file). For simplicity we store the page metadata outside of the data array. All necessary metadata should already exist and you should not make any modifications to the struct. If you feel something is missing reach out on Ed.
 - PagePointer: A byte slice of size 10 to hold a ValueId encoded using `to_fixed_bytes()`. You can reverse this by calling `ValueId::from_fixed_bytes(encoded_bytes)`.
 - BufferPool: A struct to manage pages (frames) in the database. All reads and writes of pages will go through this. All read and write operations should use the lock manager first. This structure begins use of unsafe which disables some compiler checks.  
 - FixedHeapFile/Page (`heap/mod.rs`): A struct to manage the heap file. This will be used to store the data pages. The data pages will be stored in a container of consecutive pages. The file is a new struct and impl for managing the FixedHeapFile. The page is a trait for using the underlying FixedPage.
 - FixedIndexFile/Page (`index/mod.rs`): A struct to manage the index file. This where you will be doing all of your work for this milestone. The index file will be used to store the index pages. The index pages will be stored in a container of consecutive pages. The file is a new struct and impl for managing the FixedIndexFile. The page is a trait for using the underlying FixedPage.
- StorageManager: This struct puts all the components together and acts as the "entry point" for this module.  This struct's implementation is provided for you and should not be modified. It would allow a DBMS to interact with the storage layer. It assumes every table is implementing with a heap file and every index and has a corresponding index associated with it.

## Implementation Details
Depending on the index your choose, a few implementation details must be followed.

### Hash Index
 - You must either use chained hashing, linear probing, or extendible hashing for your hash table organization. 
 - For this implementation you will not need to worry about concurrency and can assume a single thread/transaction will exclusively access the index file. This will change for the next milestone.
 - Each hash bucket will be stored in a FixedPage and hold multiple records. You will write a custom trait and implementation for your hash page and use the FixedPage struct to store the data.
 - Your hash table will be constructed of `initial_page_capacity` pages and will grow as needed. E
 - If you hash table grows, you must continue to use the same container and re-purpose existing pages (e.g. you can clear out a page and reuse it after reading the index entries).
 - If you do not implicitly map the buckets to a page, your write up needs to justify why.
 - If you use chained hashing or linear probing, you will need to rehash the table when the hash index is filled or when either the `MAX_CHAIN_LENGTH` or `MAX_PROBE_LENGTH is exceeded for adding an index entry.  For rehashing, it is acceptable to read all the entries from the old hash table into a temporary variable/struct and reinsert them into the new hash table. 
 - If you use extendible hashing, you will need to split the directory and reorganize the hash table when a hash bucket/page is filled. This will likely require you explicitly track the bucket to page mapping (along with page specific metadata not stored in the FixedPage struct).
 - You can use the default hasher from Rust to hash the search key. Reach out if you need a new non-std crate for a different hash function. See below for an example on hashing a search key. Note you will need to "wrap" the hash output to fit the current pages/buckets.
 - You do not need a "smart" or optimized bulk load for this milestone, you can just iteratively call the insert method for each key/value pair. For the next milestone this will change.

 ```rust
 let mut hasher = DefaultHasher::new();
 search_key.hash(&mut hasher);
 let hash_value = hasher.finish();
 ```


### B+ Tree Index
- You will implement a B+ tree index where each node (inner and leaf) corresponds to a FixedPage with a custom trait and implementation.
- For this implementation you will not need to worry about concurrency and can assume a single thread/transaction will exclusively access the index file. This will change for the next milestone.
- For the B+ tree index, you will need to implement a B+ tree with a fixed branching factor/degree/fan out. You should (1) determine this value to maximize the number of index entries in a page, and (2) encode the branching factor in the FixedIndexFile struct as a formula using defined constants. (e.g. `const DEGREE: usize = 14 * INDEX_ENTRY_SIZE;`).
- There is a variable `page_pointer: PagePointer` in FixedPage that you should use to store the "last pointer" of a page (e.g. greater than the last search key or sibling of a leaf).
- A boolean `is_leaf: bool` in FixedPage should be used to determine if the page is a leaf or inner node. 
- For leaf nodes, you will need to store the search key and the ValueId. For inner nodes, you will need to store the search key and the page number of the child node.
- For leaf nodes, you will need to store the search key and value (encoded value id). You will use the `page_pointer` to store the sibling of the right leaf node. You will use overflow pages to only store duplicate key entries as we do not want to change the key size. For using an overflow, you will want to separate the next sibling pointer from overflow pointer, so you know when to stop looking (e.g., in some cases you should never navigate to the sibling ). 
- For deletion you do not need to pivot data or merge nodes. It is fine to violate the B+ tree properties for this implementation.
- You do not need a "smart" or optimized bulk load for this milestone, you can just iteratively call the insert method for each key/value pair. For the next milestone this will change.


## Suggested Steps 
## Step 1  - Gain Understanding
Read and understand the FixedPage struct along with the FixedHeapFile/Page implementations. This will greatly simplify your life (and mine).

## Step 2 - Decide on Index Structure
Decide your index structure implementation. The hash implementation is likely easier, but is harder than it seems.  The tree implementation will likely require some recursive methods. For the next milestone you will continue to build out parts of your implementation  (e.g. bulk load, concurrency, etc.).

`index/mod.rs` should have one of the following definitions depending on your choice.  
```rust
pub const INDEX_TYPE: StateType = StateType::HashTable;
```
or
```rust
pub const INDEX_TYPE: StateType = StateType::Tree;
```

## Step 3 - Index Page Trait and Tests
Write your trait for your FixedIndexPage and unit tests. 

See `fn sample_util_page_test() {` in `src/data_page.rs` for an example of generating synthetic data to use for a test.
This includes some comments on using the testutil to generate random data with controlled randomness, both for the the records and the search keys. You may need to generate mock ValueIds to test this.  

## Step 4 - Index Page Implementation 
Implement the trait defined in the prior step. 

## Step 5 - Index File Trait and Tests
Implement the `IndexFileTrait`for `FixedIndexFile` and write tests for the index file. Note depending on your decision, you may ignore one init variable (e.g. `init_capacity` for the B+ tree index). The `supports_range` will be set based on the index type flag and you will need to reference it when determining if `get_pointers_for_key_range` should return an error. 

## Notes

1) There will be a lot of unused variables to start. You can suppress these warnings by prefixing your cargo command with the following:
`RUSTFLAGS="-A unused"`

For example, if you were running `cargo clippy` you would run:
`RUSTFLAGS="-A unused" cargo clippy`

2) You can also limit clippy to this workspace only by running:
`cargo clippy --lib -p idx_fixed_store -- --no-deps`

3) You can convert a Vec to a fixed size slice via try_into and defining the size. For example:

```rust
let (k, v) = self.get_kv(i).unwrap();
let kb: &[u8; SEARCH_KEY_SIZE] = k[..SEARCH_KEY_SIZE].try_into().unwrap();
```

## Scoring and Requirements
70% of your grade will be based on the provided tests. 10% will come from the quality and coverage of the tests you write and prove. The other 20% will be based on the quality of your code and your write up.

### Quality
15% of your score is based on code quality (following good coding conventions, comments, well organized functions, etc). You should only use debug statements and not print!.  Comments are only needed for non-obvious code.  You should not have comments for every line of code.  You should have comments for any non-obvious code or any code that is doing something tricky.

**We will run `cargo fmt --check` and `cargo clippy` on your code (txn_manager only), if either fails or reports issues on your code, you will receive a loss on code quality.** 

These are a formatter and linter. You can easily run `cargo fmt` to format your code in the right "style" anc clippy gives you warnings about your code, for either performance reasons or code quality. 

### Write Up
5% of your score is based on the write up.  The write up should be a markdown file named `my-idx1.md`.
- A brief description of your solution, in particular what design decisions you made and why. This is only needed for the parts of your solution that involved some significant work (e.g. just returning a counter or a pass through function isn't a design decision).
- How long you roughly spent on the milestone, and what you liked/disliked on the milestone.
- If you know some part of the milestone is incomplete, write up what parts are not working, how close you think you are, and what part(s) you got stuck on.
- What references / questions you may have asked a LLM or classmate.

### Incremental Git Commits and Pushes
As part of our grading we will look at your git history.  We expect to see at least 10 check-ins where we can see incremental changes (and highly expect several commits to replace prior work/fix bugs). These changes should also be pushed to github so we can see them.

**Important:** If there is only one commit, or the commit history is not meaningful, **you will lose 50% of your grade.** This is to encourage incremental progress and ensure that all work is done by the student.

Git commit messages should be meaningful, but can be light. For example, "Release lock working" or "fixing shared bug" are fine. The occasional "WIP" is also fine, especially if you are frequently committing and pushing your changes to github.
