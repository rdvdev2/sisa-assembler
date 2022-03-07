pub struct Flags {
    pub text_section_start: u16,
    pub data_section_start: DataSectionStart,
    pub auto_align_words: bool,
    pub auto_align_sections: bool
}

pub enum DataSectionStart {
    AfterText,
    Absolute(u16),
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            text_section_start: 0,
            data_section_start: DataSectionStart::AfterText,
            auto_align_words: false,
            auto_align_sections: false,
        }
    }
}
