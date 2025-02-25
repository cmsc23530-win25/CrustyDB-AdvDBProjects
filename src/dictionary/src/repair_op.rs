use super::dictionary_trait::OPDictionaryTrait;

pub struct RepairOPDictionary {
    dictionary: Vec<u8>,
    offsets: Vec<(usize, usize)>,// You can change this
}

impl RepairOPDictionary {
    pub fn new() -> Self {
        RepairOPDictionary {
            dictionary: Vec::new(),
            offsets: Vec::new(),
        }
    }
}

impl OPDictionaryTrait for RepairOPDictionary {
    fn encode_keys(&mut self, values: Vec<&[u8]>) -> Result<(), common::CrustyError> {
        panic!("TODO milestone comp");
    }

    fn decode_key(&self, key: usize) -> Vec<u8> {
        panic!("TODO milestone comp");
    }

    fn get_key(&self, value: &[u8]) -> Option<usize> {
        panic!("TODO milestone comp");
    }

    fn get_key_range(&self, start_inclusive: &[u8], end_inclusive: &[u8]) -> (usize, usize) {
        panic!("TODO milestone comp");
    }

    fn get_size_of_dictionary_encoded_array(&self) -> usize {
        panic!("TODO milestone comp");
    }
}
