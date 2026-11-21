use crate::dict::{self, Dictionary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Convert,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Script {
    Latin,
    Cyrillic,
}

const MIN_CHARS: usize = 2;
const MAX_CHARS: usize = 40;
const MIN_SCORE: f32 = 0.6;

const RU_CORE: &[&str] = &[
    "ага", "ай", "ах", "ба", "бы", "ведь", "во", "вот", "вон", "все", "да", "даже",
    "дескать", "же", "ка", "как", "конечно", "кстати", "ладно", "ли", "лишь", "мне",
    "меня", "мол", "мой", "нас", "не", "ни", "но", "ну", "ой", "ок", "он", "она",
    "оно", "они", "ох", "по", "пожалуй", "раз", "разве", "сам", "сама", "свой",
    "то", "тот", "та", "те", "ты", "увы", "уж", "уже", "уф", "фу", "хм", "чё", "чо",
    "что", "это", "вы", "вас", "вам", "вами", "тебе", "тебя", "ему", "ей", "их",
    "твой", "наш", "ваш", "кто", "где", "куда", "когда", "зачем", "почему", "сколько",
    "чей", "эта", "этот", "эти", "весь", "вся", "всё", "один", "одна", "одно", "два",
    "две", "три", "именно", "просто", "типа", "вообще", "вроде", "кажется", "наверное",
    "значит", "итак", "однако", "зато", "будто", "словно", "ясно", "точно", "который",
    "которая", "которые", "тобой", "мной", "нами", "вами",
];

const EN_CORE: &[&str] = &[
    "ah", "actually", "alas", "anyway", "boo", "duh", "eh", "even", "gonna", "gosh",
    "gotta", "hey", "hi", "hmm", "huh", "indeed", "jeez", "just", "kinda", "like",
    "literally", "maybe", "mhm", "nah", "nope", "oh", "ooh", "okay", "only", "ouch",
    "phew", "perhaps", "prolly", "really", "right", "so", "sorta", "still", "surely",
    "uh", "um", "wanna", "well", "whatever", "whoa", "wow", "yeah", "yep", "yknow",
    "yet", "you", "yay",
];

pub fn evaluate(typed: &str, converted: &str) -> Verdict {
    let typed_len = typed.chars().count();
    if !(MIN_CHARS..=MAX_CHARS).contains(&typed_len) {
        return Verdict::Keep;
    }

    let Some(target_script) = script_of(converted) else {
        return Verdict::Keep;
    };
    if letters_of(converted).len() < 2 {
        return Verdict::Keep;
    }

    let Some(source_script) = script_of(typed) else {
        if typed.chars().any(|c| c.is_alphabetic()) {
            return Verdict::Keep;
        }
        let dict = match target_script {
            Script::Latin => dict::english(),
            Script::Cyrillic => dict::russian(),
        };
        return if score(converted, dict) >= MIN_SCORE {
            Verdict::Convert
        } else {
            Verdict::Keep
        };
    };
    if source_script == target_script {
        return Verdict::Keep;
    }

    let (source_dict, target_dict, core) = match source_script {
        Script::Latin => (dict::english(), dict::russian(), RU_CORE),
        Script::Cyrillic => (dict::russian(), dict::english(), EN_CORE),
    };

    let source_score = score(typed, source_dict);
    let target_score = score(converted, target_dict);
    let converted_letters = letters_of(converted);

    if target_score >= MIN_SCORE
        && (target_score > source_score
            || (target_score == source_score && core_hit(core, &converted_letters)))
    {
        Verdict::Convert
    } else {
        Verdict::Keep
    }
}

fn core_hit(list: &[&str], word: &str) -> bool {
    let normalized = dict::normalize(word);
    list.iter().any(|w| *w == normalized)
}

fn letters_of(text: &str) -> String {
    text.chars().filter(|c| c.is_alphabetic()).collect()
}

fn script_of(text: &str) -> Option<Script> {
    let mut script = None;
    for c in text.chars().filter(|c| c.is_alphabetic()) {
        let current = if c.is_ascii_alphabetic() {
            Script::Latin
        } else if ('\u{0400}'..='\u{04FF}').contains(&c) {
            Script::Cyrillic
        } else {
            return None;
        };
        match script {
            None => script = Some(current),
            Some(s) if s != current => return None,
            _ => {}
        }
    }
    script
}

fn score(word: &str, dict: &Dictionary) -> f32 {
    if dict.contains(word) {
        return 1.0;
    }
    let chars: Vec<char> = word.chars().collect();
    for cut in 1..=3 {
        if chars.len() > cut + 2 {
            let stem: String = chars[..chars.len() - cut].iter().collect();
            if dict.contains(&stem) {
                return 0.6;
            }
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_wrong_layout_russian() {
        assert_eq!(evaluate("ghbdtn", "привет"), Verdict::Convert);
        assert_eq!(evaluate("cfujnjx", "словарь"), Verdict::Convert);
    }

    #[test]
    fn keeps_real_english() {
        assert_eq!(evaluate("hello", "руддщ"), Verdict::Keep);
        assert_eq!(evaluate("world", "цщкдв"), Verdict::Keep);
    }

    #[test]
    fn converts_wrong_layout_english() {
        assert_eq!(evaluate("руддщ", "hello"), Verdict::Convert);
    }

    #[test]
    fn keeps_real_russian() {
        assert_eq!(evaluate("привет", "ghbdtn"), Verdict::Keep);
    }

    #[test]
    fn converts_two_letter_words() {
        assert_eq!(evaluate("gh", "пу"), Verdict::Convert);
        assert_eq!(evaluate("to", "ещ"), Verdict::Keep);
        assert_eq!(evaluate("не", "yt"), Verdict::Keep);
        assert_eq!(evaluate("yt", "не"), Verdict::Convert);
        assert_eq!(evaluate("ye", "ну"), Verdict::Convert);
        assert_eq!(evaluate("vs", "мы"), Verdict::Keep);
    }

    #[test]
    fn converts_punctuation_position_letters() {
        assert_eq!(evaluate(",;", "бы"), Verdict::Convert);
        assert_eq!(evaluate(";t", "же"), Verdict::Convert);
        assert_eq!(evaluate("e;t", "уже"), Verdict::Convert);
        assert_eq!(evaluate("'nj", "это"), Verdict::Convert);
        assert_eq!(evaluate("djj,ot", "вообще"), Verdict::Convert);
    }

    #[test]
    fn keeps_short_and_unknown() {
        assert_eq!(evaluate("qzxw", "яче"), Verdict::Keep);
        assert_eq!(evaluate("12345", "12345"), Verdict::Keep);
    }
}
