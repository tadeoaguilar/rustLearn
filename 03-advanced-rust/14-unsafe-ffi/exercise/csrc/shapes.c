#include "shapes.h"

#include <stdlib.h>
#include <string.h>

/* __builtin_sqrt compiles to a CPU instruction with gcc and clang, so this
   file needs no libm on any platform. */
double PFX(shapes_distance)(Point a, Point b) {
    double dx = a.x - b.x;
    double dy = a.y - b.y;
    return __builtin_sqrt(dx * dx + dy * dy);
}

double PFX(shapes_polygon_area)(const Point *points, size_t len) {
    if (points == NULL || len < 3) {
        return 0.0;
    }
    double twice_area = 0.0;
    for (size_t i = 0; i < len; i++) {
        size_t j = (i + 1) % len;
        twice_area += points[i].x * points[j].y - points[j].x * points[i].y;
    }
    return (twice_area < 0 ? -twice_area : twice_area) / 2.0;
}

struct Counter {
    char *name;
    int64_t value;
};

Counter *PFX(counter_new)(const char *name) {
    if (name == NULL) {
        name = "";
    }
    Counter *c = malloc(sizeof *c);
    if (c == NULL) {
        return NULL;
    }
    size_t len = strlen(name);
    c->name = malloc(len + 1);
    if (c->name == NULL) {
        free(c);
        return NULL;
    }
    memcpy(c->name, name, len + 1);
    c->value = 0;
    return c;
}

void PFX(counter_add)(Counter *c, int64_t n) {
    if (c != NULL) {
        c->value += n;
    }
}

int64_t PFX(counter_get)(const Counter *c) {
    return c != NULL ? c->value : 0;
}

size_t PFX(counter_name)(const Counter *c, char *buf, size_t buf_len) {
    if (c == NULL) {
        return 0;
    }
    size_t len = strlen(c->name);
    if (buf != NULL && buf_len > 0) {
        size_t n = len < buf_len - 1 ? len : buf_len - 1;
        memcpy(buf, c->name, n);
        buf[n] = '\0';
    }
    return len;
}

void PFX(counter_free)(Counter *c) {
    if (c != NULL) {
        free(c->name);
        free(c);
    }
}

void PFX(shapes_for_each_point)(const Point *points, size_t len,
                                void (*cb)(Point p, void *user_data), void *user_data) {
    if (points == NULL || cb == NULL) {
        return;
    }
    for (size_t i = 0; i < len; i++) {
        cb(points[i], user_data);
    }
}

uint32_t PFX(shapes_checksum_twice)(const uint8_t *data, size_t len) {
    return PFX(rust_checksum)(data, len) + PFX(rust_checksum)(data, len);
}
