/*
 * shapes -- a small C library for module 14 (Unsafe & FFI).
 *
 * Every exported name goes through PFX(), which prepends SHAPES_PREFIX
 * (set by build.rs). The exercise crate builds it with an empty prefix, so
 * the names are exactly as written here; the solution crate uses "sol_" so
 * both copies can be linked into the same test binary without clashing.
 */
#ifndef SHAPES_H
#define SHAPES_H

#include <stddef.h>
#include <stdint.h>

#ifndef SHAPES_PREFIX
#define SHAPES_PREFIX
#endif
#define SHAPES_CAT2(a, b) a##b
#define SHAPES_CAT(a, b) SHAPES_CAT2(a, b)
#define PFX(name) SHAPES_CAT(SHAPES_PREFIX, name)

typedef struct {
    double x;
    double y;
} Point;

double PFX(shapes_distance)(Point a, Point b);

/* Shoelace formula. 0 for fewer than 3 points or a NULL pointer. */
double PFX(shapes_polygon_area)(const Point *points, size_t len);

/* An opaque handle: Rust never sees the fields. */
typedef struct Counter Counter;

Counter *PFX(counter_new)(const char *name); /* NULL if out of memory */
void PFX(counter_add)(Counter *c, int64_t n);
int64_t PFX(counter_get)(const Counter *c);
/* Copies the name into buf (always NUL-terminated if buf_len > 0, truncated
   if needed). Returns the full length of the name, like snprintf. */
size_t PFX(counter_name)(const Counter *c, char *buf, size_t buf_len);
void PFX(counter_free)(Counter *c);

/* Calls cb once per point, passing user_data through untouched. */
void PFX(shapes_for_each_point)(const Point *points, size_t len,
                                void (*cb)(Point p, void *user_data), void *user_data);

/* Implemented in RUST (Exercise 4) and called from C. */
uint32_t PFX(rust_checksum)(const uint8_t *data, size_t len);

/* Calls rust_checksum twice and adds the results (wrapping). */
uint32_t PFX(shapes_checksum_twice)(const uint8_t *data, size_t len);

#endif
