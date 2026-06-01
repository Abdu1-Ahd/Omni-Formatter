#include <stddef.h>
#include <stdint.h>

void _wasm_c_stubs_init(void) {}

void* stderr = NULL;

int fprintf(void* stream, const char* format, ...) {
    return 0;
}

int snprintf(char* s, size_t n, const char* format, ...) {
    if (n > 0) s[0] = '\0';
    return 0;
}

int fclose(void* stream) { return 0; }
void* fdopen(int fd, const char* mode) { return NULL; }

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
void abort(void) { rs_abort(); }

int iswalpha(int c) { return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'); }

int fputs(const char* s, void* stream) { return 0; }
int fputc(int c, void* stream) { return 0; }
size_t fwrite(const void* ptr, size_t size, size_t nmemb, void* stream) { return nmemb; }

int _CLOCK_MONOTONIC = 1;
int clock_gettime(int clk_id, void* tp) { return 0; }

