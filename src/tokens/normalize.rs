use regex::Regex;
use std::sync::LazyLock;

// Each pair is (pattern, replacement). Applied in order, so compound forms
// (e.g. "twenty first") must appear before their component words.
static NORMALIZATIONS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Vosk emits AM/PM as separate letters: "a m" / "p m"
        (Regex::new(r"(?i)\ba\s+m\b").unwrap(), "am"),
        (Regex::new(r"(?i)\bp\s+m\b").unwrap(), "pm"),

        // Compound ordinals — before simple ordinals and cardinals
        (Regex::new(r"(?i)\btwenty[\s-]first\b").unwrap(),   "21st"),
        (Regex::new(r"(?i)\btwenty[\s-]second\b").unwrap(),  "22nd"),
        (Regex::new(r"(?i)\btwenty[\s-]third\b").unwrap(),   "23rd"),
        (Regex::new(r"(?i)\btwenty[\s-]fourth\b").unwrap(),  "24th"),
        (Regex::new(r"(?i)\btwenty[\s-]fifth\b").unwrap(),   "25th"),
        (Regex::new(r"(?i)\btwenty[\s-]sixth\b").unwrap(),   "26th"),
        (Regex::new(r"(?i)\btwenty[\s-]seventh\b").unwrap(), "27th"),
        (Regex::new(r"(?i)\btwenty[\s-]eighth\b").unwrap(),  "28th"),
        (Regex::new(r"(?i)\btwenty[\s-]ninth\b").unwrap(),   "29th"),
        (Regex::new(r"(?i)\bthirty[\s-]first\b").unwrap(),   "31st"),
        (Regex::new(r"(?i)\bthirtieth\b").unwrap(),           "30th"),
        (Regex::new(r"(?i)\btwentieth\b").unwrap(),           "20th"),

        // Simple ordinals 1st–19th (longer words before shorter where needed)
        (Regex::new(r"(?i)\bnineteenth\b").unwrap(),  "19th"),
        (Regex::new(r"(?i)\beighteenth\b").unwrap(),  "18th"),
        (Regex::new(r"(?i)\bseventeenth\b").unwrap(), "17th"),
        (Regex::new(r"(?i)\bsixteenth\b").unwrap(),   "16th"),
        (Regex::new(r"(?i)\bfifteenth\b").unwrap(),   "15th"),
        (Regex::new(r"(?i)\bfourteenth\b").unwrap(),  "14th"),
        (Regex::new(r"(?i)\bthirteenth\b").unwrap(),  "13th"),
        (Regex::new(r"(?i)\btwelfth\b").unwrap(),     "12th"),
        (Regex::new(r"(?i)\beleventh\b").unwrap(),    "11th"),
        (Regex::new(r"(?i)\btenth\b").unwrap(),       "10th"),
        (Regex::new(r"(?i)\bninth\b").unwrap(),        "9th"),
        (Regex::new(r"(?i)\beighth\b").unwrap(),       "8th"),
        (Regex::new(r"(?i)\bseventh\b").unwrap(),      "7th"),
        (Regex::new(r"(?i)\bsixth\b").unwrap(),        "6th"),
        (Regex::new(r"(?i)\bfifth\b").unwrap(),        "5th"),
        (Regex::new(r"(?i)\bfourth\b").unwrap(),       "4th"),
        (Regex::new(r"(?i)\bthird\b").unwrap(),        "3rd"),
        (Regex::new(r"(?i)\bsecond\b").unwrap(),       "2nd"),
        (Regex::new(r"(?i)\bfirst\b").unwrap(),        "1st"),

        // Cardinals — longer/more-specific before shorter to avoid partial overlaps
        // e.g. "nineteen" must precede "nine", "thirteen" must precede "three"
        (Regex::new(r"(?i)\bnineteen\b").unwrap(),  "19"),
        (Regex::new(r"(?i)\beighteen\b").unwrap(),  "18"),
        (Regex::new(r"(?i)\bseventeen\b").unwrap(), "17"),
        (Regex::new(r"(?i)\bsixteen\b").unwrap(),   "16"),
        (Regex::new(r"(?i)\bfifteen\b").unwrap(),   "15"),
        (Regex::new(r"(?i)\bfourteen\b").unwrap(),  "14"),
        (Regex::new(r"(?i)\bthirteen\b").unwrap(),  "13"),
        (Regex::new(r"(?i)\btwelve\b").unwrap(),    "12"),
        (Regex::new(r"(?i)\beleven\b").unwrap(),    "11"),
        (Regex::new(r"(?i)\bthirty\b").unwrap(),    "30"),
        (Regex::new(r"(?i)\btwenty\b").unwrap(),    "20"),
        (Regex::new(r"(?i)\bten\b").unwrap(),       "10"),
        (Regex::new(r"(?i)\bnine\b").unwrap(),       "9"),
        (Regex::new(r"(?i)\beight\b").unwrap(),      "8"),
        (Regex::new(r"(?i)\bseven\b").unwrap(),      "7"),
        (Regex::new(r"(?i)\bsix\b").unwrap(),        "6"),
        (Regex::new(r"(?i)\bfive\b").unwrap(),       "5"),
        (Regex::new(r"(?i)\bfour\b").unwrap(),       "4"),
        (Regex::new(r"(?i)\bthree\b").unwrap(),      "3"),
        (Regex::new(r"(?i)\btwo\b").unwrap(),        "2"),
        (Regex::new(r"(?i)\bone\b").unwrap(),        "1"),
        (Regex::new(r"(?i)\bzero\b").unwrap(),       "0"),
    ]
});

pub fn normalize(text: &str) -> String {
    let mut result = text.to_owned();
    for (re, replacement) in NORMALIZATIONS.iter() {
        result = re.replace_all(&result, *replacement).into_owned();
    }
    result
}
