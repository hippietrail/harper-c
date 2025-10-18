// florb.c - C program that uses the Rust library

#include <stdio.h> // Standard I/O library for printf
#include <stdlib.h>
#include "harper.h" // Include the header file for the Rust library

int lint_cmp(const void* a, const void* b) {
    const Lint* lint_a = *(const Lint**)a;
    const Lint* lint_b = *(const Lint**)b;
    
    int32_t start_a, start_b, scratch;

    harper_get_lint_range(lint_a, &start_a, &scratch);
    harper_get_lint_range(lint_b, &start_b, &scratch);

    return start_a - start_b;
}

int main(int argc, char* argv[]) {
    char *version = harper_get_version();
    if (version != NULL) {
        printf("Using Harper Core version: %s\n", version);
        free(version);
    }
    
    const char* text = argc > 1 ? argv[1] : "Helloo ,Wrld!";
    
    // Create a document
    Document* doc = harper_create_document(text);
    if (doc == NULL) {
        printf("Failed to create document\n");
        return 1;
    }
    
    // Create a lint group
    LintGroup* lint_group = harper_create_lint_group();
    if (lint_group == NULL) {
        printf("Failed to create lint group\n");
        harper_free_document(doc);
        return 1;
    }
    
    // Get and print lints
    int32_t lint_count;
    Lint** lints = harper_get_lints(doc, lint_group, &lint_count);

    if (lints != NULL) {

        // Lints are in a random order. Let's quicksort them by the `start` of each
        qsort(lints, lint_count, sizeof(Lint*), lint_cmp);

        printf("%d lints:\n", lint_count);
        for (int32_t i = 0; i < lint_count; i++) {
            char* message = harper_get_lint_message(lints[i]);
            if (message != NULL) {
                int32_t start, end;
                harper_get_lint_range(lints[i], &start, &end);
                const int32_t suggestion_count = harper_get_suggestion_count(lints[i]);
                printf("Lint %d: '%.*s' : %s (suggestions: %d)\n", i, end - start, text + start, message, suggestion_count);
                
                // Print each suggestion
                for (int32_t j = 0; j < suggestion_count; j++) {
                    char* suggestion_text = harper_get_suggestion_text(lints[i], j);
                    if (suggestion_text != NULL) {
                        printf("  Suggestion %d: %s\n", j, suggestion_text);
                        free(suggestion_text);
                    }
                }
                
                free(message);
            }
        }
        harper_free_lints(lints, lint_count);
    }
    
    // Clean up
    harper_free_lint_group(lint_group);
    harper_free_document(doc);
    
    return 0;
}
