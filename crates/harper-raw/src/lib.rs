use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;

use harper_core::{
    core_version,
    linting::{Lint, LintGroup, Linter, Suggestion},
    spell::FstDictionary,
    Document,
};

/// Borrowed UTF-8 byte slice pointing directly into Rust-owned memory.
/// NOT null-terminated! Only valid while the owning Rust object remains alive.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HarperSlice {
    pub ptr: *const c_char,
    pub len: usize,
}

impl HarperSlice {
    const fn empty() -> Self {
        Self {
            ptr: ptr::null(),
            len: 0,
        }
    }

    fn from_str(s: &str) -> Self {
        Self {
            ptr: s.as_ptr().cast::<c_char>(),
            len: s.len(),
        }
    }
}

/// Frees a heap-allocated C string returned by `harper_raw_*` functions.
#[no_mangle]
pub extern "C" fn harper_raw_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Returns a zero-copy slice pointing to the static Harper Core version string.
#[no_mangle]
pub extern "C" fn harper_raw_get_version() -> HarperSlice {
    HarperSlice::from_str(core_version())
}

/// Creates a new document from plain English UTF-8 text.
/// Caller owns the returned pointer and must free it with `harper_raw_free_document`.
#[no_mangle]
pub extern "C" fn harper_raw_create_document(text: *const c_char) -> *mut Document {
    if text.is_null() {
        return ptr::null_mut();
    }
    let Ok(text_str) = (unsafe { CStr::from_ptr(text) }).to_str() else {
        return ptr::null_mut();
    };
    Box::into_raw(Box::new(Document::new_plain_english_curated(text_str)))
}

#[no_mangle]
pub extern "C" fn harper_raw_free_document(doc: *mut Document) {
    if !doc.is_null() {
        unsafe {
            let _ = Box::from_raw(doc);
        }
    }
}

#[no_mangle]
pub extern "C" fn harper_raw_get_token_count(doc: *const Document) -> i32 {
    if doc.is_null() {
        return -1;
    }
    let doc = unsafe { &*doc };
    i32::try_from(doc.get_tokens().len()).unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn harper_raw_create_lint_group() -> *mut LintGroup {
    let dictionary = FstDictionary::curated();
    let group = LintGroup::new_curated(Arc::new(dictionary), harper_core::Dialect::American);
    Box::into_raw(Box::new(group))
}

#[no_mangle]
pub extern "C" fn harper_raw_free_lint_group(group: *mut LintGroup) {
    if !group.is_null() {
        unsafe {
            let _ = Box::from_raw(group);
        }
    }
}

/// Returns a heap-allocated array of `Lint*` pointers and writes its length to `count`.
/// Caller must free with `harper_raw_free_lints`.
#[no_mangle]
pub extern "C" fn harper_raw_get_lints(
    doc: *const Document,
    group: *mut LintGroup,
    count: *mut i32,
) -> *mut *mut Lint {
    if doc.is_null() || group.is_null() || count.is_null() {
        return ptr::null_mut();
    }

    let doc = unsafe { &*doc };
    let group = unsafe { &mut *group };
    let found = group.lint(doc);

    let Ok(len) = i32::try_from(found.len()) else {
        return ptr::null_mut();
    };

    // Convert into a boxed slice so capacity == len is guaranteed when freeing later
    let raw_lints: Vec<*mut Lint> = found
        .into_iter()
        .map(|lint| Box::into_raw(Box::new(lint)))
        .collect();
    let mut boxed_slice = raw_lints.into_boxed_slice();

    let ptr = boxed_slice.as_mut_ptr();
    std::mem::forget(boxed_slice);

    unsafe {
        *count = len;
    }
    ptr
}

#[no_mangle]
pub extern "C" fn harper_raw_free_lints(lints: *mut *mut Lint, count: i32) {
    if lints.is_null() || count <= 0 {
        return;
    }
    let len = count as usize;
    unsafe {
        let slice = std::ptr::slice_from_raw_parts_mut(lints, len);
        let boxed_slice = Box::from_raw(slice);
        for &lint_ptr in boxed_slice.iter() {
            if !lint_ptr.is_null() {
                let _ = Box::from_raw(lint_ptr);
            }
        }
    }
}

/// Zero-copy slice into `lint->message`.
/// DANGER (Lifetime): Valid only until `lint` is freed via `harper_raw_free_lints`.
#[no_mangle]
pub extern "C" fn harper_raw_get_lint_message(lint: *const Lint) -> HarperSlice {
    if lint.is_null() {
        return HarperSlice::empty();
    }
    let lint = unsafe { &*lint };
    HarperSlice::from_str(&lint.message)
}

/// Writes the char-index range `[start, end)` of `lint`.
#[no_mangle]
pub extern "C" fn harper_raw_get_lint_range(
    lint: *const Lint,
    start: *mut i32,
    end: *mut i32,
) -> bool {
    if start.is_null() || end.is_null() {
        return false;
    }
    if lint.is_null() {
        unsafe {
            *start = -1;
            *end = -1;
        }
        return false;
    }
    let lint = unsafe { &*lint };
    unsafe {
        *start = i32::try_from(lint.span.start).unwrap_or(-1);
        *end = i32::try_from(lint.span.end).unwrap_or(-1);
    }
    true
}

/// Extracts the linted text span as an allocated C string (handles multi-byte UTF-8 safely).
/// Caller must free with `harper_raw_free_string`.
#[no_mangle]
pub extern "C" fn harper_raw_get_lint_text(doc: *const Document, lint: *const Lint) -> *mut c_char {
    if doc.is_null() || lint.is_null() {
        return ptr::null_mut();
    }
    let doc = unsafe { &*doc };
    let lint = unsafe { &*lint };
    let text = doc.get_span_content_str(&lint.span);
    match CString::new(text) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn harper_raw_get_suggestion_count(lint: *const Lint) -> i32 {
    if lint.is_null() {
        return -1;
    }
    let lint = unsafe { &*lint };
    i32::try_from(lint.suggestions.len()).unwrap_or(-1)
}

/// Formats a suggestion as a newly allocated C string.
/// Caller must free with `harper_raw_free_string`.
#[no_mangle]
pub extern "C" fn harper_raw_get_suggestion_text(lint: *const Lint, index: i32) -> *mut c_char {
    if lint.is_null() || index < 0 {
        return ptr::null_mut();
    }
    let lint = unsafe { &*lint };
    let Some(suggestion) = lint.suggestions.get(index as usize) else {
        return ptr::null_mut();
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

    match CString::new(formatted) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
