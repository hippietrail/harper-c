# Example Harper C Bindings

An example of a C interface for the Harper grammar checking library, built in Rust.

This project is intended to help illustrate how to:
- Make a C interface for a Rust library.
- Use the Harper library.

## Building and Running

1. Build the Rust library:
   ```bash
   cargo build --release
   ```

2. Compile your C code:
   ```bash
   cc your_program.c -L target/release -lharper_c -o your_program
   ```

3. From a Bash, Zsh, etc. shell:

   ```bash
   ./florb "Plz chek for a error ."

   Using Harper Core version: 0.67.0
   4 lints:
   Lint 0: 'a' : Incorrect indefinite article. (suggestions: 1)
     Suggestion 0: Replace with: "an"
   Lint 1: ' ' : Unnecessary space at the end of the sentence. (suggestions: 1)
     Suggestion 0: Remove error
   Lint 2: 'Plz' : Did you mean to spell `Plz` this way? (suggestions: 3)
     Suggestion 0: Replace with: "Pl"
     Suggestion 1: Replace with: "P's"
     Suggestion 2: Replace with: "Pb"
   Lint 3: 'chek' : Did you mean to spell `chek` this way? (suggestions: 3)
     Suggestion 0: Replace with: "chew"
     Suggestion 1: Replace with: "check"
     Suggestion 2: Replace with: "cheek"
   ```

## API Reference

### Document Management
- `Document* harper_create_document(const char* text)` - Create document from text
- `void harper_free_document(Document* doc)` - Free a document
- `char* harper_get_document_text(const Document* doc)` - Get document text (free with `free()`)
- `int32_t harper_get_token_count(const Document* doc)` - Get token count
- `char* harper_get_token_text(const Document* doc, int32_t index)` - Get token text (free with `free()`)

### Linting
- `LintGroup* harper_create_lint_group(void)` - Create a lint group
- `void harper_free_lint_group(LintGroup* group)` - Free a lint group
- `Lint** harper_get_lints(const Document* doc, LintGroup* group, int32_t* count)` - Get lints
- `void harper_free_lints(Lint** lints, int32_t count)` - Free lints array

### Lint Information
- `char* harper_get_lint_message(const Lint* lint)` - Get lint message (free with `free()`)
- `int32_t harper_get_lint_start(const Lint* lint)` - Get start position
- `int32_t harper_get_lint_end(const Lint* lint)` - Get end position
- `int32_t harper_get_suggestion_count(const Lint* lint)` - Get number of suggestions
- `char* harper_get_suggestion_text(const Lint* lint, int32_t index)` - Get suggestion text (free with `free()`)

### Version
- `char* harper_get_version(void)` - Get Harper Core version (free with `free()`)

## Example

```c
#include <stdio.h>
#include <stdlib.h>
#include "harper.h"

int main() {
    char* version = harper_get_version();
    printf("Using Harper Core version: %s\n", version);
    free(version);
    
    Document* doc = harper_create_document("Example text");
    // ... use the document ...
    harper_free_document(doc);
    return 0;
}
```

## Memory Management
- All functions that return `char*` must be freed with `free()`
- Check for `NULL` return values to detect errors
- Free all allocated resources to prevent memory leaks

## Example Code

The example (`src/florb.c`) demonstrates basic usage of the Harper library:

1. Creating a document from text
2. Getting the document's text content
3. Getting token information
4. Proper memory management

## API Reference

The C API is defined in [src/harper.h](cci:7://file:///Users/hippietrail/harper/harper-c/src/harper.h:0:0-0:0):













- `harper_create_document(text)`: Create a new document from text




- `harper_free_document(doc)`: Free a document



- `harper_get_document_text(doc)`: Get the full text of a document



- `harper_get_token_count(doc)`: Get the number of tokens in a document



- `harper_get_token_text(doc, index)`: Get the text of a specific token



- `harper_create_lint_group()`: Create a new lint group



- `harper_free_lint_group(lint_group)`: Free a lint group



- `harper_get_lints(doc, lint_group, count)`: Get all lints for a document



- `harper_free_lints(lints, count)`: Free an array of lints



- `harper_get_lint_message(lint)`: Get the message for a lint



- `harper_get_lint_start(lint)`: Get the start position of a lint



- `harper_get_lint_end(lint)`: Get the end position of a lint

## Memory Management
- Documents created with `harper_create_document` must be freed with `harper_free_document`
- Lint groups created with `harper_create_lint_group` must be freed with `harper_free_lint_group`
- Lints created with `harper_get_lints` must be freed with `harper_free_lints`
- Strings returned by `harper_get_document_text`, `harper_get_token_text`, and `harper_get_version` must be freed with `free()`

## Error Handling
- Functions that return pointers return `NULL` on error
- Functions that return integers return `-1` on error
- Always check return values for errors
