# Compression : OP Dictionary Variants

For this milestone you will be building out several implementations/variants of order-preserving dictionaries. These dictionaries will all implement the same trait. These dictionaries will be used to compress data of ASCII strings up to `MAX_STRING_LENGTH` characters. The ASCII characters will only come from codes 32-127 (see https://www.ascii-code.com/). The code keys 128-255 you will use for special encodings as described below. The strings encoded in the dictionary will be unique and the dictionary will be used to encode and decode the strings, both for single key looks up and range look ups.  All keys will be given in a single encode function call.  

The variants and their respective points are as follows. These are defined in the `DictEncoding` enum.  All are OP except the first.
 - NonOPArray (0) - given as a simple reference to help you get started
 - Array (10) - a simple array based dictionary where you store the values  in Vec of arrays. Each array value is padding to the `MAX_STRING_LENGTH`.
 - Dense (10) - a dense array based dictionary where you store the values in a single `Vec<u8>`. You should store the key offset mapping in a separate variable.  You can optionally encode the length in this variable or you can write out a separator character between values. For example if we had two values `apple` and `cherry` in the dictionary, the dense array would look like `applecherry` or `apple\0cherry\0` depending on how you store the values (this is not showing the ASCII codes for simplicity).  The key offset mapping would be `[(0,5), (5,6)]` in the first case  and `[0, 6]` in the second case. This offset mapping will be used in the following methods.  **Nothing else can be stored in the offset key mapping. Doing so will result in 50% penalty.**
 - RePair (20) - not a true 'repair' implementation but a variant inspired by it where you store the values in a single `Vec<u8>` with the key-offset mapping. Here, when encoding the keys your code should replace 2+ character sequences with a single character from the ASCII code space 128-245. For example, if you had the strings `apple`, `application`, `appleton` you could choose to encode `app` with the integer code \128 and `on` with the integer code \129. In doing so we would have the dictionary stored as `\128le\128let\129\128licati\129` (this is not showing the ASCII codes for simplicity and assumes lengths are encoded in the key-offset mapping).  The key-offset mapping would be `[(0,3) (3, 5), (8, 8)]`.  You have leeway in how you choose to select the sequences to replace, but you must justify your approach and it should be based on attempting to maximize the compression ratio by considering frequency and length of the sequences. I strongly suggest you cap the possible sequence length to be 3 or 4. 'Counting ngrams' maybe a useful search term to find simple and efficient algorithms to find the most common sequences. You will need to consider how to compare encoded values as you likely cannot directly compared encoded values.
 - Front (20) a dictionary where you encode how much of the prefix should come from the prior value. This will be stored in a single `Vec<u8>` and you will use the ASCII code space 245-255 to encode prefix lengths. For example, consider we are storing values of `app`, `apple`, `appleton`, and `application`. The prefix of the four keys would be 0, 3, 5, 4. Assuming we map 245 to be a prefix length of 1, 246 to be 2, etc, the stored values would look like `app\247le\249ton\248ication` (this is not showing the ASCII codes for simplicity).  Note that this means random access/decoding of stored keys may not be possible.
 - RePairFront (20) - a combination of RePair and Front where you encode how much of the prefix should come from the prior value and replace 2+ character sequences with a single character from the ASCII code space 128-245. This will be stored in a single `Vec<u8>` and you will use the ASCII code space 245-255 to encode prefix lengths.  This is effectively combining the two approaches. The only caveat/requirement is that when considering sequences to encode for repair you should not count sequences that are 'covered' by prefix. From the prior example, if `app` only appeared as a prefix, you should not encode it as a sequence as it is already encoded in the prefix. If `app` appeared in other keys, you could encode it as a sequence if the frequency was 'high enough'.

Note you should **not** modify Cargo.toml, lib.rs, dictionary_trait.rs, tests.rs, ascii.rs, encode.rs or add new files. 


## Suggested Steps 
#### Step 1  - Gain Understanding and read tests.
Read through the trait `OPDictionaryTrait` to understand the trait function requirements.

#### Step 2 - Implement ArrayOP. 
You should be able to pass all `array_op_` tests with this implementation. 
This (and subsequent tests) iterates through a series of files and checks that the encoding and decoding of the strings 
is correct. These checks for individual key lookups and range lookups (both with existing and non-existing keys). You 
could (temporarily modify) this test to only use the first file. ALL CAP comments highlight the changes.

```rust
#[test]
fn array_op_lookups() {
    init();
    let mut rng = SmallRng::seed_from_u64(23530);
    // CHANGING TO ONLY USE FIRST FILE
    for file in &TEST_FILES[..1] { 
        let data = convert_file_wrapper(file).unwrap();
        let d = dictionary_factory(DictEncoding::Array, &data).unwrap();
        assert!(d.get_size_of_dictionary_encoded_array() == data.len() * MAX_STRING_LENGTH);
        test_keys_random_order(&d, &data, &mut rng);
    }
}
```

#### Step 3 - Implement DenseOP. 
You should be able to pass `dense_op_` tests with this implementation.

#### Step 4 - Implement Front, and RePair. 
You should be able to pass `front_op_`, and `repair_op_` with these implementations. You will likely need to add some new unit tests for your code here.

#### Step 5 - Implement RePairFront. 
You should be able to pass `repair_front_op_` with this implementation.

## Notes

1) There will be a lot of unused variables to start. You can suppress these warnings by prefixing your cargo command with the following:
`RUSTFLAGS="-A unused"`

For example, if you were running `cargo clippy` you would run:
`RUSTFLAGS="-A unused" cargo clippy`

2) You can also limit clippy to this workspace only by running:
`cargo clippy --lib -p idx_fixed_store -- --no-deps`

3) You can convert a [u8] to an ASCII string by using the following code:
```rust
String::from_utf8(VAR.to_vec()).unwrap();
```

## Scoring and Requirements
80% of your grade will be based on the provided tests. The other 20% will be based on the quality of your code and your write up. We are working on getting a leaderboard up for the milestone, and this would only give bonus points. Details coming soon on this.

### Quality
15% of your score is based on code quality (following good coding conventions, comments, well organized functions, etc). You should only use debug statements and not print!.  Comments are only needed for non-obvious code.  You should not have comments for every line of code.  You should have comments for any non-obvious code or any code that is doing something tricky.

**We will run `cargo fmt --check` and `cargo clippy` on your code, if either fails or reports issues on your code, you will receive a loss on code quality.** 

These are a formatter and linter. You can easily run `cargo fmt` to format your code in the right "style" anc clippy gives you warnings about your code, for either performance reasons or code quality. 

### Write Up
5% of your score is based on the write up.  The write up should be a markdown file named `my-comp.md`.
- A brief description of your solution, in particular what design decisions you made and why. This is only needed for the parts of your solution that involved some significant work (e.g. just returning a counter or a pass through function isn't a design decision).
- How long you roughly spent on the milestone, and what you liked/disliked on the milestone.
- If you know some part of the milestone is incomplete, write up what parts are not working, how close you think you are, and what part(s) you got stuck on.
- What references / questions you may have asked a LLM or classmate.

### Incremental Git Commits and Pushes
As part of our grading we will look at your git history.  We expect to see at least 10 check-ins where we can see incremental changes (and highly expect several commits to replace prior work/fix bugs). These changes should also be pushed to github so we can see them.

**Important:** If there is only one commit, or the commit history is not meaningful, **you will lose 50% of your grade.** This is to encourage incremental progress and ensure that all work is done by the student.

Git commit messages should be meaningful, but can be light. For example, "Release lock working" or "fixing shared bug" are fine. The occasional "WIP" is also fine, especially if you are frequently committing and pushing your changes to github.
