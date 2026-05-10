/**
 * Minimal smoke test that verifies silvestre.h compiles with a C compiler
 * and the API can be called at link time.
 *
 * Build (compile-only):
 *   cc -c -I../include smoke_test.c -o smoke_test.o
 */
#include "silvestre.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    /* Version string is non-null and non-empty. */
    const char *ver = silvestre_version();
    assert(ver != NULL);
    assert(strlen(ver) > 0);
    printf("silvestre version: %s\n", ver);

    /* Null-pointer guards return expected sentinel values. */
    assert(silvestre_image_width(NULL) == 0);
    assert(silvestre_image_height(NULL) == 0);
    assert(silvestre_image_pixels(NULL) == NULL);
    assert(silvestre_image_pixels_len(NULL) == 0);

    /* Loading from null path returns null and sets an error. */
    SilvestreImage *img = silvestre_image_load(NULL);
    assert(img == NULL);
    const char *err = silvestre_last_error();
    assert(err != NULL);
    printf("expected error: %s\n", err);

    /* Create a 1x1 red RGBA image from buffer. */
    uint8_t pixel[4] = {255, 0, 0, 255};
    img = silvestre_image_from_buffer(pixel, 4, 1, 1);
    assert(img != NULL);
    assert(silvestre_image_width(img) == 1);
    assert(silvestre_image_height(img) == 1);
    assert(silvestre_image_pixels_len(img) == 4);

    const uint8_t *px = silvestre_image_pixels(img);
    assert(px != NULL);
    assert(px[0] == 255 && px[1] == 0 && px[2] == 0 && px[3] == 255);

    /* Apply invert filter. */
    int32_t rc = silvestre_apply_filter(img, "invert", NULL);
    assert(rc == 0);
    px = silvestre_image_pixels(img);
    assert(px[0] == 0 && px[1] == 255 && px[2] == 255 && px[3] == 255);

    /* Free the image. */
    silvestre_image_free(img);

    /* Free null is a no-op. */
    silvestre_image_free(NULL);

    printf("All C smoke tests passed.\n");
    return 0;
}
