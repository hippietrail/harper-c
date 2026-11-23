// florb.c - C program using the raw pointer & zero-copy slice Rust FFI library

#include <stdio.h>
#include <stdlib.h>
#include "harper_raw.h"

static int lint_cmp(const void* a, const void* b) {
    const Lint* lint_a = *(const Lint* const*)a;
    const Lint* lint_b = *(const Lint* const*)b;

    int32_t start_a = 0, start_b = 0, scratch = 0;
    harper_raw_get_lint_range(lint_a, &start_a, &scratch);
    harper_raw_get_lint_range(lint_b, &start_b, &scratch);

    return (start_a > start_b) - (start_a < start_b);
}

int main(int argc, char* argv[]) {
    HarperSlice version = harper_raw_get_version();
    if (version.ptr != NULL) {
        printf("Using Harper Core version: %.*s\n", (int)version.len, version.ptr);
    }

    const char* text = argc > 1 ? argv[1] : "Helloo ,Wrld!";

    Document* doc = harper_raw_create_document(text);
    if (doc == NULL) {
        printf("Failed to create document\n");
        return 1;
    }

    LintGroup* lint_group = harper_raw_create_lint_group();
    if (lint_group == NULL) {
        printf("Failed to create lint group\n");
        harper_raw_free_document(doc);
        return 1;
    }

    int32_t lint_count = 0;
    Lint** lints = harper_raw_get_lints(doc, lint_group, &lint_count);

    if (lints != NULL) {
        qsort(lints, (size_t)lint_count, sizeof(Lint*), lint_cmp);

        printf("%d lints:\n", lint_count);
        for (int32_t i = 0; i < lint_count; i++) {
            HarperSlice message = harper_raw_get_lint_message(lints[i]);
            char* span_text = harper_raw_get_lint_text(doc, lints[i]);

            if (message.ptr != NULL && span_text != NULL) {
                int32_t suggestion_count = harper_raw_get_suggestion_count(lints[i]);
                printf("Lint %d: '%s' : %.*s (suggestions: %d)\n",
                       i, span_text, (int)message.len, message.ptr, suggestion_count);

                for (int32_t j = 0; j < suggestion_count; j++) {
                    char* suggestion_text = harper_raw_get_suggestion_text(lints[i], j);
                    if (suggestion_text != NULL) {
                        printf("  Suggestion %d: %s\n", j, suggestion_text);
                        harper_raw_free_string(suggestion_text);
                    }
                }
            }

            harper_raw_free_string(span_text);
        }

        // After this call, all `message` HarperSlices point to freed memory!
        harper_raw_free_lints(lints, lint_count);
    }

    harper_raw_free_lint_group(lint_group);
    harper_raw_free_document(doc);

    return 0;
}
