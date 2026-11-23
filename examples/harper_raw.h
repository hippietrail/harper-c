#ifndef HARPER_RAW_H
#define HARPER_RAW_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque Rust types
typedef struct Document Document;
typedef struct Lint Lint;
typedef struct LintGroup LintGroup;

// Borrowed UTF-8 byte slice (NOT null-terminated!).
// Valid only while the parent Rust object remains alive.
typedef struct {
    const char* ptr;
    size_t len;
} HarperSlice;

// Free any heap-allocated `char*` returned by `harper_raw_*` functions.
void harper_raw_free_string(char* s);

HarperSlice harper_raw_get_version(void);

Document* harper_raw_create_document(const char* text);
void harper_raw_free_document(Document* doc);
int32_t harper_raw_get_token_count(const Document* doc);

LintGroup* harper_raw_create_lint_group(void);
void harper_raw_free_lint_group(LintGroup* group);

Lint** harper_raw_get_lints(const Document* doc, LintGroup* group, int32_t* count);
void harper_raw_free_lints(Lint** lints, int32_t count);

// Zero-copy slice into `lint->message` (do not use after `harper_raw_free_lints`!)
HarperSlice harper_raw_get_lint_message(const Lint* lint);
bool harper_raw_get_lint_range(const Lint* lint, int32_t* start, int32_t* end);

// Newly allocated UTF-8 string of the linted span (must be freed with `harper_raw_free_string`)
char* harper_raw_get_lint_text(const Document* doc, const Lint* lint);

int32_t harper_raw_get_suggestion_count(const Lint* lint);
// Newly allocated suggestion string (must be freed with `harper_raw_free_string`)
char* harper_raw_get_suggestion_text(const Lint* lint, int32_t index);

#ifdef __cplusplus
}
#endif

#endif // HARPER_RAW_H
