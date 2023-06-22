use bstr::ByteSlice;

pub struct HighlightIndex {
    pub offset: usize,
    pub length: usize,
}

pub enum HighLightLocation {
    Data,
    Text,
    DataAndText
}

pub trait TextHighligher {
    fn match_pattern(&self, buffer: &Vec<u8>) -> Vec<HighlightIndex>;
    fn get_highlight_location(&self) -> HighLightLocation;

    fn index_matches_highlight_index(&self, index : usize, highlight_indexes: &[HighlightIndex]) -> bool {
        for highlight_index in highlight_indexes {
            if index >= highlight_index.offset && index <= highlight_index.offset + highlight_index.length {
                return true;
            }
        }
        false   
    }
}

pub struct FindOnText{
    pub text_to_find : String,
}

impl TextHighligher for FindOnText {
    fn match_pattern(&self, buffer: &Vec<u8>) -> Vec<HighlightIndex> {
        let matches : Vec<usize> = buffer.find_iter(self.text_to_find.as_bytes()).collect();
        
        let mut highlights = Vec::<HighlightIndex>::new();

        for offset in matches  {
            highlights.push(HighlightIndex { offset: offset, length: self.text_to_find.as_bytes().len() })
        }

        highlights
    }

    fn get_highlight_location(&self) -> HighLightLocation {
        HighLightLocation::Text
    }
}

pub struct FindOnHexValues {
    pub hex_values : Vec<[u8; 2]>,
}

impl  TextHighligher for FindOnHexValues {
    fn match_pattern(&self, buffer: &Vec<u8>) -> Vec<HighlightIndex> {
        let mut highlights = Vec::<HighlightIndex>::new();

        highlights
    }

    fn get_highlight_location(&self) -> HighLightLocation {
        HighLightLocation::Data
    }
}