// harper.h - Header file for the Rust library

#ifndef HARPER_H // Include guard to prevent multiple inclusions
#define HARPER_H

#include <stdint.h>

#ifdef __cplusplus // Check if we're compiling with a C++ compiler
extern "C" { // If so, use C linkage for the following functions
#endif

// Opaque types
typedef struct Document Document;
typedef struct Lint Lint;
typedef struct LintGroup LintGroup;

// Create a new document from plain English text
// Returns NULL on error
Document* harper_create_document(const char* text);

// Free a document created by harper_create_document
void harper_free_document(Document* doc);

// Get the full text content of the document
// Returns a newly allocated string that must be freed by the caller using free()
// Returns NULL on error
char* harper_get_document_text(const Document* doc);

// Get the number of tokens in the document
// Returns -1 on error
int32_t harper_get_token_count(const Document* doc);

// Get the text of a specific token in the document
// Returns a newly allocated string that must be freed by the caller using free()
// Returns NULL on error
char* harper_get_token_text(const Document* doc, int32_t index);

// Create a new lint group with curated rules
// Returns NULL on error
LintGroup* harper_create_lint_group(void);

// Free a lint group created by harper_create_lint_group
void harper_free_lint_group(LintGroup* lint_group);

// Get all lints for a document using a lint group
// Returns an array of Lint pointers, and sets count to the number of lints
// The caller is responsible for freeing both the array and each Lint using harper_free_lints
// Returns NULL on error
Lint** harper_get_lints(const Document* doc, LintGroup* lint_group, int32_t* count);

// Free an array of lints created by harper_get_lints
void harper_free_lints(Lint** lints, int32_t count);

// Get the message for a lint
// Returns a newly allocated string that must be freed by the caller using free()
// Returns NULL on error
char* harper_get_lint_message(const Lint* lint);

// Get the start position of a lint in the document
// Returns -1 on error
int32_t harper_get_lint_start(const Lint* lint);

// Get the end position of a lint in the document
// Returns -1 on error
int32_t harper_get_lint_end(const Lint* lint);

#ifdef __cplusplus
} // End of extern "C"
#endif

#endif // End of include guard
