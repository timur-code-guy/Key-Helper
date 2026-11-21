use std::sync::OnceLock;

const RU_WORDS: &str = include_str!("../data/ru.txt");
const EN_WORDS: &str = include_str!("../data/en.txt");
const UK_WORDS: &str = include_str!("../data/uk.txt");

pub struct Dictionary {
    words: Vec<&'static str>,
}

impl Dictionary {
    fn from_source(source: &'static str) -> Self {
        let mut words: Vec<&'static str> = source.lines().collect();
        words.sort_unstable();
        words.dedup();
        Self { words }
    }

    pub fn contains(&self, word: &str) -> bool {
        let normalized = normalize(word);
        self.words
            .binary_search(&normalized.as_str())
            .is_ok()
    }
}

pub(crate) fn normalize(word: &str) -> String {
    word.chars()
        .flat_map(|c| match c {
            'ё' | 'Ё' => "е".chars().collect::<Vec<_>>(),
            _ => c.to_lowercase().collect(),
        })
        .collect()
}

static RU: OnceLock<Dictionary> = OnceLock::new();
static EN: OnceLock<Dictionary> = OnceLock::new();
static UK: OnceLock<Dictionary> = OnceLock::new();

pub fn russian() -> &'static Dictionary {
    RU.get_or_init(|| Dictionary::from_source(RU_WORDS))
}

pub fn english() -> &'static Dictionary {
    EN.get_or_init(|| Dictionary::from_source(EN_WORDS))
}

pub fn ukrainian() -> &'static Dictionary {
    UK.get_or_init(|| Dictionary::from_source(UK_WORDS))
}
