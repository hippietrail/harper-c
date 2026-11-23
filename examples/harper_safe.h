#ifndef HARPER_SAFE_H
#define HARPER_SAFE_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Strongly-typed generational handles (id == 0 means invalid/null handle)
typedef struct { uint64_t id; } HarperDoc;
typedef struct { uint64_t id; } HarperLintGroup;
typedef struct { uint64_t id; } HarperLint;

// String buffer functions write a null-terminated UTF-8 string into `buf` (up to `buf_cap` bytes)
// and return the full untruncated byte length, or -1 if the handle/index is invalid.
// Pass `buf = NULL, buf_cap = 0` to query the required length.

int32_t harper_safe_get_version(char* buf, size_t buf_cap);

HarperDoc harper_safe_create_document(const char* text);
bool harper_safe_free_document(HarperDoc doc);
int32_t harper_safe_get_token_count(HarperDoc doc);
int32_t harper_safe_get_token_text(HarperDoc doc, int32_t index, char* buf, size_t buf_cap);

HarperLintGroup harper_safe_create_lint_group(void);
bool harper_safe_free_lint_group(HarperLintGroup group);

// Pass `out_handles = NULL, out_cap = 0` to query how many lints exist,
// or pass a preallocated array `out_handles` of capacity `out_cap` to populate handles.
int32_t harper_safe_get_lints(HarperDoc doc, HarperLintGroup group, HarperLint* out_handles, size_t out_cap);
bool harper_safe_free_lint(HarperLint lint);

int32_t harper_safe_get_lint_message(HarperLint lint, char* buf, size_t buf_cap);
bool harper_safe_get_lint_range(HarperLint lint, int32_t* start, int32_t* end);
int32_t harper_safe_get_lint_text(HarperDoc doc, HarperLint lint, char* buf, size_t buf_cap);
int32_t harper_safe_get_suggestion_count(HarperLint lint);
int32_t harper_safe_get_suggestion_text(HarperLint lint, int32_t index, char* buf, size_t buf_cap);

#ifdef __cplusplus
}
#endif

#endif // HARPER_SAFE_H
