#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

void _wasm_c_stubs_init(void) {}

FILE *const stderr = NULL;

int fprintf(FILE* stream, const char* format, ...) {
    return 0;
}

int sprintf(char* s, const char* format, ...) {
    if (s) s[0] = '\0';
    return 0;
}

int snprintf(char* s, size_t n, const char* format, ...) {
    if (n > 0) s[0] = '\0';
    return 0;
}

int fclose(FILE* stream) { return 0; }
FILE* fdopen(int fd, const char* mode) { return NULL; }

int strncmp(const char* s1, const char* s2, size_t n) {
    while (n--) {
        if (*s1 != *s2) return *s1 - *s2;
        if (*s1 == 0) break;
        s1++; s2++;
    }
    return 0;
}

char* strncpy(char* dest, const char* src, size_t n) {
    size_t i;
    for (i = 0; i < n && src[i] != '\0'; i++) dest[i] = src[i];
    for ( ; i < n; i++) dest[i] = '\0';
    return dest;
}

extern void rs_abort(void);
void abort(void) { rs_abort(); while(1) {} }

int fputs(const char* s, FILE* stream) { return 0; }
int fputc(int c, FILE* stream) { return 0; }
size_t fwrite(const void* ptr, size_t size, size_t nmemb, FILE* stream) { return nmemb; }

int _CLOCK_MONOTONIC = 1;
int clock_gettime(int clk_id, void* tp) { return 0; }

