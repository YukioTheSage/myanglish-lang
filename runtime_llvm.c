#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *key;
    int64_t value;
} mlang_map_entry_i8ptr_i64;

typedef struct {
    mlang_map_entry_i8ptr_i64 *entries;
    size_t len;
    size_t cap;
} mlang_map_i8ptr_i64;

static char *mlang_strdup(const char *s) {
    if (s == NULL) {
        return NULL;
    }
    size_t n = strlen(s);
    char *p = (char *)malloc(n + 1);
    if (p == NULL) {
        return NULL;
    }
    memcpy(p, s, n + 1);
    return p;
}

void *mlang_alloc(int64_t size) {
    if (size <= 0) {
        size = 1;
    }
    return calloc(1, (size_t)size);
}

void *mlang_realloc(void *ptr, int64_t size) {
    if (size <= 0) {
        size = 1;
    }
    return realloc(ptr, (size_t)size);
}

void mlang_print_array_i64(int64_t *data, int64_t len) {
    printf("[");
    for (int64_t i = 0; i < len; i++) {
        if (i > 0) {
            printf(", ");
        }
        printf("%lld", (long long)data[i]);
    }
    printf("]\n");
}

void mlang_print_array_i8ptr(char **data, int64_t len) {
    printf("[");
    for (int64_t i = 0; i < len; i++) {
        if (i > 0) {
            printf(", ");
        }
        if (data[i] == NULL) {
            printf("nil");
        } else {
            printf("\"%s\"", data[i]);
        }
    }
    printf("]\n");
}

void *mlang_map_new_i8ptr_i64(void) {
    mlang_map_i8ptr_i64 *m = (mlang_map_i8ptr_i64 *)calloc(1, sizeof(mlang_map_i8ptr_i64));
    return (void *)m;
}

static void mlang_map_grow_i8ptr_i64(mlang_map_i8ptr_i64 *m) {
    if (m->len < m->cap) {
        return;
    }
    size_t next_cap = (m->cap == 0) ? 8 : (m->cap * 2);
    mlang_map_entry_i8ptr_i64 *next =
        (mlang_map_entry_i8ptr_i64 *)realloc(m->entries, next_cap * sizeof(mlang_map_entry_i8ptr_i64));
    if (next == NULL) {
        return;
    }
    m->entries = next;
    m->cap = next_cap;
}

void mlang_map_set_i8ptr_i64(void *raw, char *key, int64_t value) {
    mlang_map_i8ptr_i64 *m = (mlang_map_i8ptr_i64 *)raw;
    if (m == NULL || key == NULL) {
        return;
    }

    for (size_t i = 0; i < m->len; i++) {
        if (m->entries[i].key != NULL && strcmp(m->entries[i].key, key) == 0) {
            m->entries[i].value = value;
            return;
        }
    }

    mlang_map_grow_i8ptr_i64(m);
    if (m->len >= m->cap) {
        return;
    }

    m->entries[m->len].key = mlang_strdup(key);
    m->entries[m->len].value = value;
    m->len++;
}

int64_t mlang_map_get_i8ptr_i64(void *raw, char *key) {
    mlang_map_i8ptr_i64 *m = (mlang_map_i8ptr_i64 *)raw;
    if (m == NULL || key == NULL) {
        return 0;
    }

    for (size_t i = 0; i < m->len; i++) {
        if (m->entries[i].key != NULL && strcmp(m->entries[i].key, key) == 0) {
            return m->entries[i].value;
        }
    }

    return 0;
}
