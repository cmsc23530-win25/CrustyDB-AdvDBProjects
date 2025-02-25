use super::array_non_op::ArrayNonOPDictionary;
use super::array_op::ArrayOPDictionary;
use common::CrustyError;

pub const MAX_STRING_LENGTH: usize = 10;

pub enum DictEncoding {
    /// Here the dictionary is stored as Vec<Vec<u8>>. This is the most basic encoding. Provided as a reference
    NonOPArray,
    /// Here the dictionary is stored as Vec<[u8; MAX_STRING_LENGTH]>. This is the most basic encoding.
    /// Each value in the nested array is a byte of MAX_STRING_LENGTH, you should use the null terminator (0) to pad the string.
    Array,
    /// Here the dictionary is stored as Vec<u8>. You should store offsets in a separate array or vec
    /// (and do this for all non-array encodings). You can use an offset and length, or just an offset and terminator character.
    Dense,
    /// Here the dictionary is stored as Vec<u8>. You should store offsets in a separate array or vec
    /// You will use the ascii values 128-245 for storing sequences of common characters (2+).
    RePair,
    /// Here the dictionary is stored as Vec<u8>. You should store offsets in a separate array or vec
    /// You will use the ascii values 245-255 for encoding the shared prefix length of the front encoding (shared prior prefix)
    Front,
    /// Here the dictionary is stored as Vec<u8>. You should store offsets in a separate array or vec
    /// You will use the ascii values 245-255 for encoding the shared prefix length and use ascci values 128-245 for
    /// encoding the common sequences of characters (2+).
    RePairFront,
}

/// OPDictionaryTrait is a trait that defines the methods that a dictionary must implement.
/// Values will come in as a vector of ASCII codes, with characters only coming from the first 127 ASCII values.
/// This means you can use the extended ASCII values for your own purposes (128-255).
/// See https://www.ascii-code.com/ for a list of ASCII values.
pub trait OPDictionaryTrait {
    /// Create an encode a new order preserving dictionary. The dictionary encoding is specified by the dict_encoding parameter.
    ///
    /// # Arguments
    /// * `dict_encoding` - The encoding to use for the dictionary (how to store the values).
    /// * `values` - The values to store in the dictionary. encoded as ASCII codes.
    ///
    /// # Returns
    ///
    /// * Result of the new dictionary that has been encoded (on the heap as a box).
    fn encode_keys(&mut self, values: Vec<&[u8]>) -> Result<(), CrustyError>;

    /// Given a key, decode it to the original value.
    ///
    /// # Arguments
    /// * `key` - The key to decode.
    ///
    /// # Returns
    /// * The original value that was encoded. Empty vec if the key is not found.
    fn decode_key(&self, key: usize) -> Vec<u8>;

    /// Given a value, get the key that represents it.
    ///
    /// # Arguments
    /// * `value` - The value to get the key for.
    ///
    /// # Returns
    /// * The key that represents the value. None if the value is not found.
    fn get_key(&self, value: &[u8]) -> Option<usize>;

    /// Given a start and end key, get the range of keys that are in the dictionary.
    ///
    /// # Arguments
    /// * `start_inclusive` - The start key (inclusive).
    /// * `end_inclusive` - The end key (inclusive).
    ///
    /// # Returns
    /// * A tuple of the start and end key that are in the dictionary.
    fn get_key_range(&self, start_inclusive: &[u8], end_inclusive: &[u8]) -> (usize, usize);

    /// Get the size of the dictionary encoded array.
    /// This does not need to include any meta data, such offsets or lengths, or the huffman code mapping.
    fn get_size_of_dictionary_encoded_array(&self) -> usize;
}

#[allow(dead_code)]
pub fn dictionary_factory(
    dict_encoding: DictEncoding,
    values: &Vec<Vec<u8>>,
) -> Result<Box<dyn OPDictionaryTrait>, CrustyError> {
    let values_ref: Vec<&[u8]> = values.iter().map(|v| v.as_slice()).collect();

    match dict_encoding {
        DictEncoding::NonOPArray => {
            let mut dict = ArrayNonOPDictionary::new();
            dict.encode_keys(values_ref)?;
            Ok(Box::new(dict))
        }
        DictEncoding::Array => {
            let mut dict = ArrayOPDictionary::new();
            dict.encode_keys(values_ref)?;
            Ok(Box::new(dict))
        }
        DictEncoding::Dense => {
            let mut dict = ArrayOPDictionary::new();
            dict.encode_keys(values_ref)?;
            Ok(Box::new(dict))
        }
        DictEncoding::RePair => {
            let mut dict = ArrayOPDictionary::new();
            dict.encode_keys(values_ref)?;
            Ok(Box::new(dict))
        }
        DictEncoding::Front => {
            let mut dict = ArrayOPDictionary::new();
            dict.encode_keys(values_ref)?;
            Ok(Box::new(dict))
        }
        DictEncoding::RePairFront => {
            let mut dict = ArrayOPDictionary::new();
            dict.encode_keys(values_ref)?;
            Ok(Box::new(dict))
        }
    }
}
