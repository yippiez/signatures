/*
 * comments_strings.c - Fake declarations inside comments and string/char
 * literals that must be ignored by the signatures CLI.
 *
 * struct FakeInBlockComment { int x; };
 * void fake_in_block_comment(void);
 * #define FAKE_MACRO_IN_COMMENT 999
 * enum FakeEnum { A, B };
 */

#include <stdio.h>
#include <string.h>

// int fake_line_comment_proto(int a, int b);
// struct FakeLineStruct { char c; };
// typedef int FakeLineTypedef;

#define GREETING "struct NotAStruct { int x; };"
#define DECL_STRING "void not_a_function(int fake);"
#define CHAR_LITERAL_BRACE '{'

static const char *SQL = "SELECT id FROM users WHERE active = 1;";
static const int VERSION = 3;

/* Real declarations start here */

struct Config {
    int debug;
    int verbose;
    char logfile[256];
};

typedef struct Config Config;

void print_fake_declarations(void) {
    /* enum Hidden { X = 0 }; should be ignored */
    const char *msg = "typedef int NotReal;";
    printf("%s\n", msg);
    // void also_fake(void);
    puts("struct AlsoFake { double d; };");
}

int parse_config(const char *filename, struct Config *cfg) {
    /* int nested_fake(void) { return 0; } */
    if (!filename || !cfg) return -1;
    cfg->debug = 0;
    cfg->verbose = 0;
    return 0;
}

static int validate(const struct Config *cfg) {
    (void)cfg;
    return 1;
}

/* Another block comment with fake stuff:
 * static const int FAKE_CONST = 42;
 * union FakeUnion { int a; float b; };
 */
void dump_config(const struct Config *cfg) {
    const char *fmt = "int not_printed(void);";
    (void)fmt;
    printf("debug=%d verbose=%d\n", cfg->debug, cfg->verbose);
}
