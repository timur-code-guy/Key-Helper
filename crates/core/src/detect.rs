use crate::dict::{self, Dictionary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Convert,
    Keep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Russian,
    Ukrainian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Script {
    Latin,
    Cyrillic,
}

const MIN_CHARS: usize = 2;
const MAX_CHARS: usize = 40;
const MIN_SCORE: f32 = 0.6;

const RU_SINGLE: &[&str] = &["а", "в", "и", "к", "о", "с", "у", "я"];
const EN_SINGLE: &[&str] = &["a", "i"];
const UK_SINGLE: &[&str] = &["а", "в", "і", "у", "я"];

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

const UK_CORE: &[&str] = &[
    "а", "ага", "ай", "але", "би", "бо", "вас", "вам", "вами", "весь", "вже",
    "він", "вона", "воно", "всі", "все", "всього", "геть", "для", "до", "дуже",
    "е", "ее", "ей", "є", "ж", "же", "за", "зараз", "значить", "звідки", "звідти",
    "звідусіль", "і", "й", "його", "ї", "її", "їх", "к", "каже", "коли", "котрий",
    "куди", "лише", "людина", "ми", "моє", "можна", "мой", "мою", "на", "навіть",
    "не", "нема", "немає", "нижче", "ни", "ні", "но", "ну", "один", "одна", "оно",
    "онт", "от", "ота", "оте", "отже", "ох", "по", "привіт", "просто", "проти",
    "проте", "про", "раз", "сам", "сама", "скільки", "со", "так", "також", "там",
    "твій", "тебе", "теж", "тільки", "тобі", "тощо", "трохи", "ту", "тут", "ти",
    "у", "увесь", "уже", "уся", "хай", "хіба", "хто", "це", "цей", "ця", "чи",
    "число", "чому", "що", "щоб", "щойно", "як", "який", "якщо", "якось", "які",
    "вже", "добре", "ок", "дякую", "будь", "ласка", "треба", "було", "бути", "є",
    "був", "була", "буде", "адже", "наче", "мов", "мовляв", "хоч", "хоча", "тобто",
    "ось", "он", "таки", "биж", "нехай", "досі", "завжди", "свій", "своя", "наш",
    "ваш", "мій", "моя", "така", "такі", "ви", "вони", "я", "та",
];

const EN_CORE: &[&str] = &[
    "a", "ah", "am", "an", "and", "are", "as", "at", "be", "but", "by", "do", "for",
    "go", "he", "if", "in", "is", "it", "me", "my", "no", "of", "on", "or", "so",
    "to", "up", "us", "we", "actually", "alas", "anyway", "boo", "duh", "eh",
    "even", "gonna", "gosh", "gotta", "hey", "hi", "hmm", "huh", "indeed", "jeez",
    "just", "kinda", "like", "literally", "maybe", "mhm", "nah", "nope", "oh", "ooh",
    "okay", "only", "ouch", "phew", "perhaps", "prolly", "really", "right",
    "sorta", "still", "surely", "uh", "um", "wanna", "well", "whatever", "whoa", "wow",
    "yeah", "yep", "yknow", "yet", "you", "yay",
];

pub fn evaluate(typed: &str, converted: &str, source: Language, target: Language) -> Verdict {
    let typed_len = typed.chars().count();
    if typed_len == 1 {
        return evaluate_single(typed, converted, target);
    }
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
        let dict = dict_for(target_script, target);
        return if score(converted, dict) >= MIN_SCORE {
            Verdict::Convert
        } else {
            Verdict::Keep
        };
    };
    let cyrillic_cross = source_script == Script::Cyrillic
        && source != target
        && matches!(target, Language::Russian | Language::Ukrainian);
    if source_script == target_script && !cyrillic_cross {
        return Verdict::Keep;
    }

    let source_dict = dict_for(source_script, source);
    let target_dict = dict_for(target_script, target);
    let core = core_for(target);
    let source_score = if source_script == Script::Cyrillic {
        score(typed, dict::russian()).max(score(typed, dict::ukrainian()))
    } else {
        score(typed, source_dict)
    };
    let target_score = score(converted, target_dict);
    let converted_letters = letters_of(converted);
    let cyrillic_to_latin = source_script == Script::Cyrillic && target_script == Script::Latin;

    if target_score >= MIN_SCORE
        && (target_score > source_score
            || (!cyrillic_to_latin
                && target_score == source_score
                && core_hit(core, &converted_letters)))
    {
        Verdict::Convert
    } else {
        Verdict::Keep
    }
}

fn evaluate_single(typed: &str, converted: &str, target: Language) -> Verdict {
    let Some(source_script) = script_of(typed) else {
        return Verdict::Keep;
    };
    let letters = letters_of(converted);
    if letters.chars().count() != 1 {
        return Verdict::Keep;
    }
    let Some(target_script) = script_of(&letters) else {
        return Verdict::Keep;
    };
    if source_script == target_script {
        return Verdict::Keep;
    }
    let singles = match (target_script, target) {
        (Script::Latin, _) => EN_SINGLE,
        (Script::Cyrillic, Language::Ukrainian) => UK_SINGLE,
        (Script::Cyrillic, _) => RU_SINGLE,
    };
    let normalized = dict::normalize(&letters);
    if singles.contains(&normalized.as_str()) {
        Verdict::Convert
    } else {
        Verdict::Keep
    }
}

fn dict_for(script: Script, lang: Language) -> &'static Dictionary {
    match script {
        Script::Latin => dict::english(),
        Script::Cyrillic => match lang {
            Language::Ukrainian => dict::ukrainian(),
            _ => dict::russian(),
        },
    }
}

fn core_for(lang: Language) -> &'static [&'static str] {
    match lang {
        Language::English => EN_CORE,
        Language::Russian => RU_CORE,
        Language::Ukrainian => UK_CORE,
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
    use Language::*;

    #[test]
    fn converts_wrong_layout_russian() {
        assert_eq!(evaluate("ghbdtn", "привет", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("cfujnjx", "словарь", English, Russian), Verdict::Convert);
    }

    #[test]
    fn keeps_real_english() {
        assert_eq!(evaluate("hello", "руддщ", Russian, English), Verdict::Keep);
        assert_eq!(evaluate("world", "цщкдв", Russian, English), Verdict::Keep);
    }

    #[test]
    fn converts_wrong_layout_english() {
        assert_eq!(evaluate("руддщ", "hello", Russian, English), Verdict::Convert);
    }

    #[test]
    fn keeps_real_russian() {
        assert_eq!(evaluate("привет", "ghbdtn", Russian, English), Verdict::Keep);
    }

    #[test]
    fn converts_two_letter_words() {
        assert_eq!(evaluate("gh", "пу", English, Russian), Verdict::Keep);
        assert_eq!(evaluate("to", "ещ", English, Russian), Verdict::Keep);
        assert_eq!(evaluate("не", "yt", Russian, English), Verdict::Keep);
        assert_eq!(evaluate("yt", "не", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("ye", "ну", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("vs", "мы", English, Russian), Verdict::Keep);
    }

    #[test]
    fn converts_punctuation_position_letters() {
        assert_eq!(evaluate(",;", "бы", English, Russian), Verdict::Convert);
        assert_eq!(evaluate(";t", "же", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("e;t", "уже", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("'nj", "это", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("djj,ot", "вообще", English, Russian), Verdict::Convert);
    }

    #[test]
    fn converts_single_letter_words() {
        assert_eq!(evaluate("z", "я", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("b", "и", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("d", "в", English, Russian), Verdict::Convert);
        assert_eq!(evaluate("Z", "Я", English, Russian), Verdict::Convert);
    }

    #[test]
    fn keeps_single_non_words() {
        assert_eq!(evaluate("a", "ф", English, Russian), Verdict::Keep);
        assert_eq!(evaluate("i", "ш", English, Russian), Verdict::Keep);
        assert_eq!(evaluate("s", "ы", English, Russian), Verdict::Keep);
        assert_eq!(evaluate("ф", "a", Russian, English), Verdict::Convert);
        assert_eq!(evaluate("ш", "i", Russian, English), Verdict::Convert);
        assert_eq!(evaluate("я", "z", Russian, English), Verdict::Keep);
    }

    #[test]
    fn keeps_cyrillic_word_on_tie() {
        assert_eq!(evaluate("тщ", "no", Russian, English), Verdict::Keep);
        assert_eq!(evaluate("що", "що", Russian, Ukrainian), Verdict::Convert);
    }

    #[test]
    fn converts_wrong_layout_ukrainian() {
        assert_eq!(evaluate("gbdtn", "дякую", English, Ukrainian), Verdict::Convert);
    }

    #[test]
    fn converts_wrong_layout_english_from_ukrainian() {
        assert_eq!(evaluate("руддщ", "hello", Ukrainian, English), Verdict::Convert);
        assert_eq!(evaluate("дякую", "руддщ", Ukrainian, English), Verdict::Keep);
    }

    #[test]
    fn converts_between_russian_and_ukrainian() {
        assert_eq!(evaluate("вякую", "дякую", Russian, Ukrainian), Verdict::Convert);
        assert_eq!(evaluate("дякую", "вякую", Ukrainian, Russian), Verdict::Keep);
    }

    #[test]
    fn converts_ukrainian_single_letters() {
        assert_eq!(evaluate("s", "і", English, Ukrainian), Verdict::Convert);
        assert_eq!(evaluate("i", "і", English, Ukrainian), Verdict::Convert);
        assert_eq!(evaluate("і", "s", Ukrainian, English), Verdict::Keep);
    }

    #[test]
    fn keeps_real_cyrillic_on_tie() {
        assert_eq!(evaluate("ща", "of", Russian, English), Verdict::Keep);
        assert_eq!(evaluate("ща", "of", Ukrainian, English), Verdict::Keep);
        assert_eq!(evaluate("не", "yt", Russian, English), Verdict::Keep);
    }

    #[test]
    fn keeps_short_and_unknown() {
        assert_eq!(evaluate("qzxw", "яче", English, Russian), Verdict::Keep);
        assert_eq!(evaluate("12345", "12345", English, Russian), Verdict::Keep);
    }
}
