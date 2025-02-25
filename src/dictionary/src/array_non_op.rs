use super::dictionary_trait::OPDictionaryTrait;

pub struct ArrayNonOPDictionary {
    dictionary: Box<Vec<Vec<u8>>>,
}

impl ArrayNonOPDictionary {
    pub fn new() -> Self {
        ArrayNonOPDictionary {
            dictionary: Box::new(Vec::new()),
        }
    }
}

impl OPDictionaryTrait for ArrayNonOPDictionary {
    fn encode_keys(&mut self, values: Vec<&[u8]>) -> Result<(), common::CrustyError> {
        for value in values {
            self.dictionary.push(value.to_vec());
        }
        Ok(())
    }

    fn decode_key(&self, key: usize) -> Vec<u8> {
        self.dictionary[key].clone()
    }

    fn get_key(&self, value: &[u8]) -> Option<usize> {
        for (i, val) in self.dictionary.iter().enumerate() {
            if val == value {
                return Some(i);
            }
        }
        None
    }

    fn get_key_range(&self, _start_inclusive: &[u8], _end_inclusive: &[u8]) -> (usize, usize) {
        panic!("Not supported for this dictionary type");
    }

    fn get_size_of_dictionary_encoded_array(&self) -> usize {
        // Get the size of the dictionary
        let mut size = 0;
        for val in self.dictionary.iter() {
            size += val.len();
        }
        size
    }
}
