#![allow(clippy::match_same_arms, clippy::too_many_lines)]

pub mod locales;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Auto,
    Ar,
    Ca,
    #[serde(alias = "zh-Hans", alias = "zh_cn", alias = "zh-CN")]
    ZhHans,
    #[serde(alias = "zh-Hant", alias = "zh_tw", alias = "zh-TW", alias = "zh-HK")]
    ZhHant,
    Hr,
    Cs,
    Da,
    Nl,
    #[serde(alias = "en-AU")]
    EnAu,
    #[serde(alias = "en-CA")]
    EnCa,
    #[serde(alias = "en-IN")]
    EnIn,
    #[serde(alias = "en-JP")]
    EnJp,
    #[serde(alias = "en-GB", alias = "en_uk", alias = "en-UK")]
    EnGb,
    #[serde(alias = "en-US", alias = "en")]
    EnUs,
    Fi,
    #[serde(alias = "fr-CA")]
    FrCa,
    #[serde(alias = "fr-FR", alias = "fr")]
    FrFr,
    #[serde(alias = "de-DE")]
    De,
    El,
    He,
    Hi,
    Hu,
    Id,
    It,
    #[serde(alias = "ja-JP")]
    Ja,
    #[serde(alias = "ko-KR")]
    Ko,
    Ms,
    #[serde(alias = "no")]
    Nb,
    Pl,
    #[serde(alias = "pt-BR")]
    PtBr,
    #[serde(alias = "pt-PT", alias = "pt")]
    PtPt,
    Ro,
    Ru,
    Sk,
    #[serde(alias = "es-CL")]
    EsCl,
    #[serde(rename = "es_419", alias = "es419", alias = "es-419")]
    Es419,
    #[serde(alias = "es-MX")]
    EsMx,
    #[serde(alias = "es-ES", alias = "es")]
    EsEs,
    #[serde(alias = "es-US")]
    EsUs,
    Sv,
    Th,
    Tr,
    Uk,
    Vi,
}

pub const ALL_LANGUAGES: &[Language] = &[
    Language::Auto,
    Language::Ar,
    Language::Ca,
    Language::ZhHans,
    Language::ZhHant,
    Language::Hr,
    Language::Cs,
    Language::Da,
    Language::Nl,
    Language::EnAu,
    Language::EnCa,
    Language::EnIn,
    Language::EnJp,
    Language::EnGb,
    Language::EnUs,
    Language::Fi,
    Language::FrCa,
    Language::FrFr,
    Language::De,
    Language::El,
    Language::He,
    Language::Hi,
    Language::Hu,
    Language::Id,
    Language::It,
    Language::Ja,
    Language::Ko,
    Language::Ms,
    Language::Nb,
    Language::Pl,
    Language::PtBr,
    Language::PtPt,
    Language::Ro,
    Language::Ru,
    Language::Sk,
    Language::EsCl,
    Language::Es419,
    Language::EsMx,
    Language::EsEs,
    Language::EsUs,
    Language::Sv,
    Language::Th,
    Language::Tr,
    Language::Uk,
    Language::Vi,
];

impl Language {
    #[must_use]
    pub fn all() -> &'static [Language] {
        ALL_LANGUAGES
    }

    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Auto => "System Default (Auto)",
            Language::Ar => "Arabic",
            Language::Ca => "Catalan",
            Language::ZhHans => "Chinese (Simplified)",
            Language::ZhHant => "Chinese (Traditional)",
            Language::Hr => "Croatian",
            Language::Cs => "Czech",
            Language::Da => "Danish",
            Language::Nl => "Dutch",
            Language::EnAu => "English (Australia)",
            Language::EnCa => "English (Canada)",
            Language::EnIn => "English (India)",
            Language::EnJp => "English (Japan)",
            Language::EnGb => "English (UK)",
            Language::EnUs => "English (US)",
            Language::Fi => "Finnish",
            Language::FrCa => "French (Canada)",
            Language::FrFr => "French (France)",
            Language::De => "German",
            Language::El => "Greek",
            Language::He => "Hebrew",
            Language::Hi => "Hindi",
            Language::Hu => "Hungarian",
            Language::Id => "Indonesian",
            Language::It => "Italian",
            Language::Ja => "Japanese",
            Language::Ko => "Korean",
            Language::Ms => "Malay",
            Language::Nb => "Norwegian Bokmål",
            Language::Pl => "Polish",
            Language::PtBr => "Portuguese (Brazil)",
            Language::PtPt => "Portuguese (Portugal)",
            Language::Ro => "Romanian",
            Language::Ru => "Russian",
            Language::Sk => "Slovak",
            Language::EsCl => "Spanish (Chile)",
            Language::Es419 => "Spanish (Latin America)",
            Language::EsMx => "Spanish (Mexico)",
            Language::EsEs => "Spanish (Spain)",
            Language::EsUs => "Spanish (United States)",
            Language::Sv => "Swedish",
            Language::Th => "Thai",
            Language::Tr => "Turkish",
            Language::Uk => "Ukrainian",
            Language::Vi => "Vietnamese",
        }
    }

    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Language::Auto => "auto",
            Language::Ar => "ar",
            Language::Ca => "ca",
            Language::ZhHans => "zh-Hans",
            Language::ZhHant => "zh-Hant",
            Language::Hr => "hr",
            Language::Cs => "cs",
            Language::Da => "da",
            Language::Nl => "nl",
            Language::EnAu => "en-AU",
            Language::EnCa => "en-CA",
            Language::EnIn => "en-IN",
            Language::EnJp => "en-JP",
            Language::EnGb => "en-GB",
            Language::EnUs => "en-US",
            Language::Fi => "fi",
            Language::FrCa => "fr-CA",
            Language::FrFr => "fr-FR",
            Language::De => "de",
            Language::El => "el",
            Language::He => "he",
            Language::Hi => "hi",
            Language::Hu => "hu",
            Language::Id => "id",
            Language::It => "it",
            Language::Ja => "ja",
            Language::Ko => "ko",
            Language::Ms => "ms",
            Language::Nb => "nb",
            Language::Pl => "pl",
            Language::PtBr => "pt-BR",
            Language::PtPt => "pt-PT",
            Language::Ro => "ro",
            Language::Ru => "ru",
            Language::Sk => "sk",
            Language::EsCl => "es-CL",
            Language::Es419 => "es-419",
            Language::EsMx => "es-MX",
            Language::EsEs => "es-ES",
            Language::EsUs => "es-US",
            Language::Sv => "sv",
            Language::Th => "th",
            Language::Tr => "tr",
            Language::Uk => "uk",
            Language::Vi => "vi",
        }
    }

    #[must_use]
    pub fn base_language(&self) -> Option<Language> {
        match self {
            Language::EnAu
            | Language::EnCa
            | Language::EnIn
            | Language::EnJp
            | Language::EnGb => Some(Language::EnUs),
            Language::FrCa => Some(Language::FrFr),
            Language::EsCl
            | Language::Es419
            | Language::EsMx
            | Language::EsUs => Some(Language::EsEs),
            Language::PtBr => Some(Language::PtPt),
            _ => None,
        }
    }

    #[must_use]
    pub fn from_code(raw_code: &str) -> Option<Language> {
        let trimmed = raw_code.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Clean up charset suffixes like .UTF-8 or @modifier
        let base = trimmed
            .split('.')
            .next()?
            .split('@')
            .next()?
            .trim();

        let lower = base.to_ascii_lowercase();
        let normalized = lower.replace('_', "-");

        match normalized.as_str() {
            "auto" => Some(Language::Auto),
            "ar" | "ar-sa" | "ar-ae" | "ar-eg" => Some(Language::Ar),
            "ca" | "ca-es" => Some(Language::Ca),
            "zh-hans" | "zh-cn" | "zh-sg" | "zh-hans-cn" | "zh-hans-us" => Some(Language::ZhHans),
            "zh-hant" | "zh-tw" | "zh-hk" | "zh-mo" | "zh-hant-tw" | "zh-hant-hk" => {
                Some(Language::ZhHant)
            }
            "zh" => Some(Language::ZhHans),
            "hr" | "hr-hr" => Some(Language::Hr),
            "cs" | "cs-cz" => Some(Language::Cs),
            "da" | "da-dk" => Some(Language::Da),
            "nl" | "nl-nl" | "nl-be" => Some(Language::Nl),
            "en-au" => Some(Language::EnAu),
            "en-ca" => Some(Language::EnCa),
            "en-in" => Some(Language::EnIn),
            "en-jp" => Some(Language::EnJp),
            "en-gb" | "en-uk" => Some(Language::EnGb),
            "en-us" | "en" => Some(Language::EnUs),
            "fi" | "fi-fi" => Some(Language::Fi),
            "fr-ca" => Some(Language::FrCa),
            "fr-fr" | "fr" | "fr-be" | "fr-ch" => Some(Language::FrFr),
            "de" | "de-de" | "de-at" | "de-ch" => Some(Language::De),
            "el" | "el-gr" => Some(Language::El),
            "he" | "he-il" | "iw" | "iw-il" => Some(Language::He),
            "hi" | "hi-in" => Some(Language::Hi),
            "hu" | "hu-hu" => Some(Language::Hu),
            "id" | "id-id" | "in" => Some(Language::Id),
            "it" | "it-it" | "it-ch" => Some(Language::It),
            "ja" | "ja-jp" => Some(Language::Ja),
            "ko" | "ko-kr" => Some(Language::Ko),
            "ms" | "ms-my" => Some(Language::Ms),
            "nb" | "nb-no" | "no" | "no-no" | "nn" | "nn-no" => Some(Language::Nb),
            "pl" | "pl-pl" => Some(Language::Pl),
            "pt-br" => Some(Language::PtBr),
            "pt-pt" | "pt" => Some(Language::PtPt),
            "ro" | "ro-ro" => Some(Language::Ro),
            "ru" | "ru-ru" => Some(Language::Ru),
            "sk" | "sk-sk" => Some(Language::Sk),
            "es-cl" => Some(Language::EsCl),
            "es-419" | "es419" | "es-latinamerica" => Some(Language::Es419),
            "es-mx" => Some(Language::EsMx),
            "es-us" => Some(Language::EsUs),
            "es-es" | "es" => Some(Language::EsEs),
            "sv" | "sv-se" => Some(Language::Sv),
            "th" | "th-th" => Some(Language::Th),
            "tr" | "tr-tr" => Some(Language::Tr),
            "uk" | "uk-ua" => Some(Language::Uk),
            "vi" | "vi-vn" => Some(Language::Vi),
            _ => {
                // Secondary check for prefixes
                if let Some(prefix) = normalized.split('-').next() {
                    match prefix {
                        "ar" => Some(Language::Ar),
                        "ca" => Some(Language::Ca),
                        "hr" => Some(Language::Hr),
                        "cs" => Some(Language::Cs),
                        "da" => Some(Language::Da),
                        "nl" => Some(Language::Nl),
                        "en" => Some(Language::EnUs),
                        "fi" => Some(Language::Fi),
                        "fr" => Some(Language::FrFr),
                        "de" => Some(Language::De),
                        "el" => Some(Language::El),
                        "he" | "iw" => Some(Language::He),
                        "hi" => Some(Language::Hi),
                        "hu" => Some(Language::Hu),
                        "id" => Some(Language::Id),
                        "it" => Some(Language::It),
                        "ja" => Some(Language::Ja),
                        "ko" => Some(Language::Ko),
                        "ms" => Some(Language::Ms),
                        "nb" | "no" | "nn" => Some(Language::Nb),
                        "pl" => Some(Language::Pl),
                        "pt" => Some(Language::PtPt),
                        "ro" => Some(Language::Ro),
                        "ru" => Some(Language::Ru),
                        "sk" => Some(Language::Sk),
                        "es" => Some(Language::EsEs),
                        "sv" => Some(Language::Sv),
                        "th" => Some(Language::Th),
                        "tr" => Some(Language::Tr),
                        "uk" => Some(Language::Uk),
                        "vi" => Some(Language::Vi),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }
}

#[must_use]
pub fn detect_system_language() -> Language {
    // 1. Check environment variables
    let env_vars = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"];
    for var in env_vars {
        if let Ok(val) = std::env::var(var) {
            let val = val.trim();
            if !val.is_empty()
                && val != "C"
                && val != "POSIX"
                && let Some(lang) = Language::from_code(val)
                && lang != Language::Auto
            {
                return lang;
            }
        }
    }

    // 2. On macOS query AppleLanguages defaults
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleLanguages"])
            .output()
            && output.status.success()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim().trim_matches(|c| c == '(' || c == ')' || c == ',');
                let cleaned = trimmed.trim().trim_matches('"');
                if !cleaned.is_empty()
                    && let Some(lang) = Language::from_code(cleaned)
                    && lang != Language::Auto
                {
                    return lang;
                }
            }
        }
    }

    Language::EnUs
}

#[must_use]
pub fn t(key: &str, lang: Language) -> &'static str {
    let effective_lang = if lang == Language::Auto {
        let detected = detect_system_language();
        if detected == Language::Auto {
            Language::EnUs
        } else {
            detected
        }
    } else {
        lang
    };

    // 1. Direct translation
    if let Some(val) = locales::lookup(effective_lang, key) {
        return val;
    }

    // 2. Base language fallback (e.g. es-MX -> es-ES)
    if let Some(base) = effective_lang.base_language()
        && let Some(val) = locales::lookup(base, key)
    {
        return val;
    }

    // 3. English (US) fallback
    if let Some(val) = locales::lookup(Language::EnUs, key) {
        return val;
    }

    // 4. Return key as fallback
    Box::leak(key.to_string().into_boxed_str())
}
