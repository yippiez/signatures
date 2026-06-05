#include <stdio.h>

#define MAX 100
static const int LIMIT = 5;

struct Point {
    int x;
    int y;
};

int add(int a, int b) {
    return a + b;
}

void process(struct Point *p) {
    /* void not_real(void) inside a comment */
}
