// krenth.c - C program using the safe handle-based Rust FFI library

#include <stdio.h>
#include <stdlib.h>
#include "harper_safe.h"

static int lint_cmp(const void* a, const void* b) {
    HarperLint lint_a = *(const HarperLint*)a;
    HarperLint lint_b = *(const HarperLint*)b;

    int32_t start_a = 0, start_b = 0, scratch = 0;
    harper_safe_get_lint_range(lint_a, &start_a, &scratch);
    harper_safe_get_lint_range(lint_b, &start_b, &scratch);

    return (start_a > start_b) - (start_a < start_b);
}

int main(int argc, char* argv[]) {
    char version[64];
    if (harper_safe_get_version(version, sizeof(version)) >= 0) {
        printf("Using Harper Core version: %s\n", version);
    }

    const char* text = argc > 1 ? argv[1] : "Helloo ,Wrld!";

    HarperDoc doc = harper_safe_create_document(text);
    if (doc.id == 0) {
        printf("Failed to create document\n");
        return 1;
    }

    HarperLintGroup group = harper_safe_create_lint_group();
    if (group.id == 0) {
        printf("Failed to create lint group\n");
        harper_safe_free_document(doc);
        return 1;
    }

    // Populate up to 64 lint handles on the stack (zero heap allocations in C!)
    HarperLint lints[64];
    int32_t lint_count = harper_safe_get_lints(doc, group, lints, 64);

    if (lint_count > 0) {
        qsort(lints, (size_t)lint_count, sizeof(HarperLint), lint_cmp);

        printf("%d lints:\n", lint_count);
        for (int32_t i = 0; i < lint_count; i++) {
            char span_text[256];
            char message[512];

            if (harper_safe_get_lint_text(doc, lints[i], span_text, sizeof(span_text)) >= 0 &&
                harper_safe_get_lint_message(lints[i], message, sizeof(message)) >= 0) {

                int32_t suggestion_count = harper_safe_get_suggestion_count(lints[i]);
                printf("Lint %d: '%s' : %s (suggestions: %d)\n",
                       i, span_text, message, suggestion_count);

                for (int32_t j = 0; j < suggestion_count; j++) {
                    char suggestion_text[256];
                    if (harper_safe_get_suggestion_text(lints[i], j, suggestion_text, sizeof(suggestion_text)) >= 0) {
                        printf("  Suggestion %d: %s\n", j, suggestion_text);
                    }
                }
            }

            harper_safe_free_lint(lints[i]);
        }
    }

    harper_safe_free_lint_group(group);
    harper_safe_free_document(doc);

    // Safety demo: using or double-freeing a stale handle fails gracefully instead of crashing!
    printf("Double-free doc result: %s\n", harper_safe_free_document(doc) ? "true" : "false (safely ignored)");
    printf("Stale doc token count: %d\n", harper_safe_get_token_count(doc));

    return 0;
}
