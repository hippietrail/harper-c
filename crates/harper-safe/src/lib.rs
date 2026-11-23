use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};

use harper_core::{
    core_version,
    linting::{Lint, LintGroup, Linter, Suggestion},
    spell::FstDictionary,
    Document,
};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

// --- Strongly-typed generational handles for C ---

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HarperDoc {
    pub id: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HarperLintGroup {
    pub id: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HarperLint {
    pub id: u64,
}

fn to_key(id: u64) -> DefaultKey {
    DefaultKey::from(KeyData::from_ffi(id))
}

fn from_key(key: DefaultKey) -> u64 {
    key.data().as_ffi()
}

// --- Global thread-safe object registries ---

fn docs() -> &'static Mutex<SlotMap<DefaultKey, Document>> {
    static DOCS: OnceLock<Mutex<SlotMap<DefaultKey, Document>>> = OnceLock::new();
    DOCS.get_or_init(|| Mutex::new(SlotMap::new()))
}

fn lint_groups() -> &'static Mutex<SlotMap<DefaultKey, LintGroup>> {
    static GROUPS: OnceLock<Mutex<SlotMap<DefaultKey, LintGroup>>> = OnceLock::new();
    GROUPS.get_or_init(|| Mutex::new(SlotMap::new()))
}

fn lints() -> &'static Mutex<SlotMap<DefaultKey, Lint>> {
    static LINTS: OnceLock<Mutex<SlotMap<DefaultKey, Lint>>> = OnceLock::new();
    LINTS.get_or_init(|| Mutex::new(SlotMap::new()))
}

// --- Helper: write a Rust &str safely into a C-owned buffer ---

unsafe fn write_str_to_c_buf(s: &str, buf: *mut c_char, buf_cap: usize) -> i32 {
    let Ok(full_len) = i32::try_from(s.len()) else {
        return -1;
    };

    if buf.is_null() || buf_cap == 0 {
        return full_len;
    }

    let max_copy = buf_cap.saturating_sub(1);
    let mut copy_len = s.len().min(max_copy);
    // Avoid splitting a multi-byte UTF-8 character at the boundary
    while copy_len > 0 && !s.is_char_boundary(copy_len) {
        copy_len -= 1;
    }

    std::ptr::copy_nonoverlapping(s.as_ptr(), buf.cast::<u8>(), copy_len);
    *buf.add(copy_len) = 0;

    full_len
}

// --- Library Version ---

#[no_mangle]
pub extern "C" fn harper_safe_get_version(buf: *mut c_char, buf_cap: usize) -> i32 {
    unsafe { write_str_to_c_buf(core_version(), buf, buf_cap) }
}

// --- Document Management ---

#[no_mangle]
pub extern "C" fn harper_safe_create_document(text: *const c_char) -> HarperDoc {
    if text.is_null() {
        return HarperDoc { id: 0 };
    }

    let Ok(text_str) = (unsafe { CStr::from_ptr(text) }).to_str() else {
        return HarperDoc { id: 0 };
    };

    let doc = Document::new_plain_english_curated(text_str);
    let Ok(mut map) = docs().lock() else {
        return HarperDoc { id: 0 };
    };

    HarperDoc {
        id: from_key(map.insert(doc)),
    }
}

#[no_mangle]
pub extern "C" fn harper_safe_free_document(doc: HarperDoc) -> bool {
    let Ok(mut map) = docs().lock() else {
        return false;
    };
    map.remove(to_key(doc.id)).is_some()
}

#[no_mangle]
pub extern "C" fn harper_safe_get_token_count(doc: HarperDoc) -> i32 {
    let Ok(map) = docs().lock() else {
        return -1;
    };
    let Some(doc_ref) = map.get(to_key(doc.id)) else {
        return -1;
    };
    i32::try_from(doc_ref.get_tokens().len()).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn harper_safe_get_token_text(
    doc: HarperDoc,
    index: i32,
    buf: *mut c_char,
    buf_cap: usize,
) -> i32 {
    if index < 0 {
        return -1;
    }
    let Ok(map) = docs().lock() else {
        return -1;
    };
    let Some(doc_ref) = map.get(to_key(doc.id)) else {
        return -1;
    };
    let Some(token) = doc_ref.get_tokens().get(index as usize) else {
        return -1;
    };

    let text = doc_ref.get_span_content_str(&token.span);
    unsafe { write_str_to_c_buf(&text, buf, buf_cap) }
}

// --- Lint Groups ---

#[no_mangle]
pub extern "C" fn harper_safe_create_lint_group() -> HarperLintGroup {
    let dictionary = FstDictionary::curated();
    let group = LintGroup::new_curated(Arc::new(dictionary), harper_core::Dialect::American);
    let Ok(mut map) = lint_groups().lock() else {
        return HarperLintGroup { id: 0 };
    };
    HarperLintGroup {
        id: from_key(map.insert(group)),
    }
}

#[no_mangle]
pub extern "C" fn harper_safe_free_lint_group(group: HarperLintGroup) -> bool {
    let Ok(mut map) = lint_groups().lock() else {
        return false;
    };
    map.remove(to_key(group.id)).is_some()
}

// --- Linting & Lint Handles ---

#[no_mangle]
pub extern "C" fn harper_safe_get_lints(
    doc: HarperDoc,
    group: HarperLintGroup,
    out_handles: *mut HarperLint,
    out_cap: usize,
) -> i32 {
    let Ok(docs_map) = docs().lock() else {
        return -1;
    };
    let Some(doc_ref) = docs_map.get(to_key(doc.id)) else {
        return -1;
    };

    let Ok(mut groups_map) = lint_groups().lock() else {
        return -1;
    };
    let Some(group_ref) = groups_map.get_mut(to_key(group.id)) else {
        return -1;
    };

    let found_lints = group_ref.lint(doc_ref);
    let Ok(total) = i32::try_from(found_lints.len()) else {
        return -1;
    };

    // If out_handles is NULL, just return the count without storing handles
    if out_handles.is_null() || out_cap == 0 {
        return total;
    }

    let Ok(mut lints_map) = lints().lock() else {
        return -1;
    };

    let to_store = found_lints.len().min(out_cap);
    for (i, lint) in found_lints.into_iter().take(to_store).enumerate() {
        let handle = HarperLint {
            id: from_key(lints_map.insert(lint)),
        };
        unsafe {
            *out_handles.add(i) = handle;
        }
    }

    i32::try_from(to_store).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn harper_safe_free_lint(lint: HarperLint) -> bool {
    let Ok(mut map) = lints().lock() else {
        return false;
    };
    map.remove(to_key(lint.id)).is_some()
}

#[no_mangle]
pub extern "C" fn harper_safe_get_lint_message(
    lint: HarperLint,
    buf: *mut c_char,
    buf_cap: usize,
) -> i32 {
    let Ok(map) = lints().lock() else {
        return -1;
    };
    let Some(lint_ref) = map.get(to_key(lint.id)) else {
        return -1;
    };
    unsafe { write_str_to_c_buf(&lint_ref.message, buf, buf_cap) }
}

/// Returns char-index range [start, end) of the lint. Returns false if handle is invalid.
#[no_mangle]
pub extern "C" fn harper_safe_get_lint_range(
    lint: HarperLint,
    start: *mut i32,
    end: *mut i32,
) -> bool {
    if start.is_null() || end.is_null() {
        return false;
    }
    let Ok(map) = lints().lock() else {
        return false;
    };
    let Some(lint_ref) = map.get(to_key(lint.id)) else {
        unsafe {
            *start = -1;
            *end = -1;
        }
        return false;
    };

    unsafe {
        *start = i32::try_from(lint_ref.span.start).unwrap_or(-1);
        *end = i32::try_from(lint_ref.span.end).unwrap_or(-1);
    }
    true
}

/// Safely extracts the linted text span directly from `doc` into `buf` (handles multi-byte UTF-8!).
#[no_mangle]
pub extern "C" fn harper_safe_get_lint_text(
    doc: HarperDoc,
    lint: HarperLint,
    buf: *mut c_char,
    buf_cap: usize,
) -> i32 {
    let Ok(docs_map) = docs().lock() else {
        return -1;
    };
    let Some(doc_ref) = docs_map.get(to_key(doc.id)) else {
        return -1;
    };
    let Ok(lints_map) = lints().lock() else {
        return -1;
    };
    let Some(lint_ref) = lints_map.get(to_key(lint.id)) else {
        return -1;
    };

    let span_text = doc_ref.get_span_content_str(&lint_ref.span);
    unsafe { write_str_to_c_buf(&span_text, buf, buf_cap) }
}

#[no_mangle]
pub extern "C" fn harper_safe_get_suggestion_count(lint: HarperLint) -> i32 {
    let Ok(map) = lints().lock() else {
        return -1;
    };
    let Some(lint_ref) = map.get(to_key(lint.id)) else {
        return -1;
    };
    i32::try_from(lint_ref.suggestions.len()).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn harper_safe_get_suggestion_text(
    lint: HarperLint,
    index: i32,
    buf: *mut c_char,
    buf_cap: usize,
) -> i32 {
    if index < 0 {
        return -1;
    }
    let Ok(map) = lints().lock() else {
        return -1;
    };
    let Some(lint_ref) = map.get(to_key(lint.id)) else {
        return -1;
    };
    let Some(suggestion) = lint_ref.suggestions.get(index as usize) else {
        return -1;
    };

    let formatted = match suggestion {
        Suggestion::ReplaceWith(chars) => {
            format!("Replace with: \"{}\"", chars.iter().collect::<String>())
        }
        Suggestion::InsertAfter(chars) => {
            format!("Insert \"{}\"", chars.iter().collect::<String>())
        }
        Suggestion::Remove => "Remove error".to_string(),
    };

    unsafe { write_str_to_c_buf(&formatted, buf, buf_cap) }
}
