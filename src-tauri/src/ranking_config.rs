/// Centralised ranking constants for the search pipeline.
///
/// Every magic number that was previously scattered across `lib.rs`,
/// `matcher.rs`, and helper functions now lives here with documentation.

// ── Score weight clamping ────────────────────────────────────────────
pub const WEIGHT_MIN: u32 = 10;
pub const WEIGHT_MAX: u32 = 300;

// ── Calculator ───────────────────────────────────────────────────────
pub const CALCULATOR_BASE_SCORE: u32 = 900_000;

// ── Store / Settings (search) ────────────────────────────────────────
pub const STORE_SEARCH_SCORE: u32 = 2_600;
pub const SETTINGS_BASE_SCORE: u32 = 1_100;
pub const SETTINGS_FUZZY_MULTIPLIER: u32 = 8;

// ── Store / Settings (suggestions) ──────────────────────────────────
pub const STORE_SUGGESTION_SCORE: u32 = 800_000;
pub const SETTINGS_SUGGESTION_SCORE: u32 = 1_200_000;
pub const EXTENSION_SUGGESTION_SCORE: u32 = 850_000;
pub const APP_SUGGESTION_WEIGHT: u32 = 100;

// ── Intent workflow ──────────────────────────────────────────────────
pub const INTENT_MIN_QUERY_LEN: usize = 8;
pub const INTENT_MIN_STEPS: usize = 2;
pub const INTENT_BASE_SCORE: u32 = 970_000;

// ── Window results ───────────────────────────────────────────────────
pub const WINDOW_NO_QUERY_BASE: u32 = 650;
pub const WINDOW_FUZZY_TITLE_BASE: u32 = 850;
pub const WINDOW_FUZZY_TITLE_MULTIPLIER: u32 = 8;
pub const WINDOW_FUZZY_CLASS_BASE: u32 = 760;
pub const WINDOW_FUZZY_CLASS_MULTIPLIER: u32 = 6;
pub const WINDOW_NO_MATCH_BASE: u32 = 500;
pub const WINDOW_RECENCY_MULTIPLIER: u32 = 8;
pub const WINDOW_RECENCY_CAP: u32 = 180;

// ── Extension command results ────────────────────────────────────────
pub const EXTENSION_FUZZY_BASE: u32 = 700;
pub const EXTENSION_FUZZY_MULTIPLIER: u32 = 6;
pub const EXTENSION_EXACT_SCORE: u32 = 1_050;

// ── Clipboard results ────────────────────────────────────────────────
pub const CLIPBOARD_EXACT_BASE: u32 = 900;
pub const CLIPBOARD_FUZZY_BASE: u32 = 700;
pub const CLIPBOARD_FUZZY_MULTIPLIER: u32 = 5;
pub const CLIPBOARD_PINNED_BONUS: u32 = 220;
pub const CLIPBOARD_SCAN_LIMIT: usize = 80;

// ── Profile results ──────────────────────────────────────────────────
pub const PROFILE_FUZZY_BASE: u32 = 1_000;
pub const PROFILE_FUZZY_MULTIPLIER: u32 = 10;
pub const PROFILE_FALLBACK_BASE: u32 = 900;

// ── Fuzzy app matching (matcher.rs) ──────────────────────────────────
pub const APP_EXACT_NAME_BONUS: u32 = 800;
pub const APP_PREFIX_NAME_BONUS: u32 = 260;
pub const APP_SECONDARY_PENALTY: u32 = 10;
pub const APP_TERTIARY_PENALTY: u32 = 20;

// ── Usage relevance (matcher.rs) ─────────────────────────────────────
pub const USAGE_LN_MULTIPLIER: f64 = 130.0;
pub const USAGE_HARD_CAP: u32 = 1_400;
pub const USAGE_RELEVANCE_DIVISOR: u32 = 3;
pub const USAGE_RELEVANCE_ADDEND: u32 = 180;

// ── query_relevance_bonus ────────────────────────────────────────────
pub const QR_TITLE_EXACT: u32 = 18_000;
pub const QR_TITLE_PREFIX: u32 = 12_000;
pub const QR_TITLE_CONTAINS: u32 = 7_000;
pub const QR_SUBTITLE_CONTAINS: u32 = 3_000;
pub const QR_EXEC_CONTAINS: u32 = 2_500;
pub const QR_MULTI_TOKEN_ALL: u32 = 4_000;

// ── source_intent_bonus ──────────────────────────────────────────────
pub const SI_APPLICATION: u32 = 8_000;
pub const SI_FILE: u32 = 4_500;
pub const SI_WINDOW: u32 = 4_500;
pub const SI_CALCULATOR: u32 = 4_000;
pub const SI_EXTENSION: u32 = 3_500;
pub const SI_CLIPBOARD: u32 = 3_200;

// ── app_entity_bonus ─────────────────────────────────────────────────
pub const AE_EXACT: u32 = 8_000;
pub const AE_PREFIX: u32 = 5_000;
pub const AE_CONTAINS: u32 = 2_000;

// ── Negative scoring ─────────────────────────────────────────────────
/// Minimum score below which a result is suppressed from output.
pub const NEGATIVE_SCORE_THRESHOLD: u32 = 50;
/// Penalty applied to stale clipboard entries older than this many days.
pub const CLIPBOARD_STALE_DAYS: u64 = 30;
pub const CLIPBOARD_STALE_PENALTY: u32 = 200;
/// Penalty for extremely short fuzzy matches (likely noise).
pub const SHORT_MATCH_PENALTY: u32 = 150;
pub const SHORT_MATCH_THRESHOLD: usize = 2;

// ── Documents in suggestions ─────────────────────────────────────────
pub const SUGGESTION_DOC_LIMIT: usize = 12;

// ── Typo tolerance ───────────────────────────────────────────────────
/// Score for "Did you mean?" suggestions shown when results are sparse.
pub const TYPO_SUGGESTION_SCORE: u32 = 500;
