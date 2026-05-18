use std::collections::HashMap;

pub struct TransliterationEngine {
    single_map: HashMap<char, String>,
    multi_map: HashMap<String, String>,
    buffer: String,
}

impl TransliterationEngine {
    pub fn new() -> Self {
        let mut single_map = HashMap::new();
        let mut multi_map = HashMap::new();

        // Basic Latin -> Cyrillic (from TRANSLATION_TABLE.md)
        let singles = [
            ('a', "а"), ('b', "б"), ('c', "к"), ('d', "д"), ('e', "е"),
            ('f', "ф"), ('g', "г"), ('h', "х"), ('i', "и"), ('j', "дж"),
            ('k', "к"), ('l', "л"), ('m', "м"), ('n', "н"), ('o', "о"),
            ('p', "п"), ('q', "к"), ('r', "р"), ('s', "с"), ('t', "т"),
            ('u', "у"), ('v', "в"), ('w', "в"), ('x', "кс"), ('y', "й"),
            ('z', "з"),
            ('A', "А"), ('B', "Б"), ('C', "К"), ('D', "Д"), ('E', "Е"),
            ('F', "Ф"), ('G', "Г"), ('H', "Х"), ('I', "И"), ('J', "Дж"),
            ('K', "К"), ('L', "Л"), ('M', "М"), ('N', "Н"), ('O', "О"),
            ('P', "П"), ('Q', "К"), ('R', "Р"), ('S', "С"), ('T', "Т"),
            ('U', "У"), ('V', "В"), ('W', "В"), ('X', "Кс"), ('Y', "Й"),
            ('Z', "З"),
        ];

        for (lat, cyr) in singles {
            single_map.insert(lat, cyr.to_string());
        }

        // Multi-character rules
        let multis = [
            ("sch", "щ"), ("Sch", "Щ"), ("SCH", "Щ"),
            ("shch", "щ"), ("Shch", "Щ"), ("SHCH", "Щ"),
            ("yo", "ё"), ("Yo", "Ё"), ("YO", "Ё"),
            ("zh", "ж"), ("Zh", "Ж"), ("ZH", "Ж"),
            ("kh", "х"), ("Kh", "Х"), ("KH", "Х"),
            ("ts", "ц"), ("Ts", "Ц"), ("TS", "Ц"),
            ("ch", "ч"), ("Ch", "Ч"), ("CH", "Ч"),
            ("sh", "ш"), ("Sh", "Ш"), ("SH", "Ш"),
            ("yu", "ю"), ("Yu", "Ю"), ("YU", "Ю"),
            ("ya", "я"), ("Ya", "Я"), ("YA", "Я"),
        ];

        for (lat, cyr) in multis {
            multi_map.insert(lat.to_string(), cyr.to_string());
        }

        Self {
            single_map,
            multi_map,
            buffer: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    pub fn process(&mut self, c: char) -> TransliterationAction {
        if !c.is_alphabetic() {
            self.buffer.clear();
            return TransliterationAction::None;
        }

        self.buffer.push(c);

        for len in (2..=4).rev() {
            if self.buffer.len() >= len {
                let tail = &self.buffer[self.buffer.len() - len..];
                if let Some(cyr) = self.multi_map.get(tail) {
                    let backspaces = len - 1;
                    return TransliterationAction::Replace(backspaces, cyr.clone());
                }
            }
        }

        if let Some(cyr) = self.single_map.get(&c) {
            return TransliterationAction::Convert(cyr.clone());
        }

        TransliterationAction::None
    }
}

pub enum TransliterationAction {
    None,
    Convert(String),
    Replace(usize, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singles() {
        let mut engine = TransliterationEngine::new();
        if let TransliterationAction::Convert(s) = engine.process('a') {
            assert_eq!(s, "а");
        } else {
            panic!("Expected Convert");
        }
    }

    #[test]
    fn test_multis() {
        let mut engine = TransliterationEngine::new();
        engine.process('s');
        if let TransliterationAction::Replace(len, s) = engine.process('h') {
            assert_eq!(len, 1);
            assert_eq!(s, "ш");
        } else {
            panic!("Expected Replace");
        }
    }

    #[test]
    fn test_longest_match() {
        let mut engine = TransliterationEngine::new();
        engine.process('s');
        engine.process('h');
        if let TransliterationAction::Replace(_len, s) = engine.process('c') {
            assert_eq!(s, "к");
        }

        engine.reset();
        engine.process('s');
        engine.process('c');
        if let TransliterationAction::Replace(len, s) = engine.process('h') {
            assert_eq!(len, 2);
            assert_eq!(s, "щ");
        }
    }
}
