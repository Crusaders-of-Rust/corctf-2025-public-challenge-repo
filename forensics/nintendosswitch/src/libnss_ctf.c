#define _GNU_SOURCE
#include <nss.h>
#include <errno.h>
#include <pwd.h>
#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>

// copy from encrypt_flag.py
#define STREAM_SEED 0x3244ad92

char *get_blob_path(void) {
    static char path[19];
    const char encoded[] = {
        '/' ^ 0xAA, 'o' ^ 0xAA, 'p' ^ 0xAA, 't' ^ 0xAA, '/' ^ 0xAA,
        'c' ^ 0xAA, 't' ^ 0xAA, 'f' ^ 0xAA, '/' ^ 0xAA,
        'f' ^ 0xAA, 'l' ^ 0xAA, 'a' ^ 0xAA, 'g' ^ 0xAA, '.' ^ 0xAA,
        'b' ^ 0xAA, 'l' ^ 0xAA, 'o' ^ 0xAA, 'b' ^ 0xAA, '\0' ^ 0xAA
    };

    for (int i = 0; i < sizeof(encoded); i++)
        path[i] = encoded[i] ^ 0xAA;

    return path;
}

char *get_outfile(void) {
    static char path[15];
    const char encoded[] = {
        '/' ^ 0x55, 't' ^ 0x55, 'm' ^ 0x55, 'p' ^ 0x55, '/' ^ 0x55,
        'f' ^ 0x55, 'l' ^ 0x55, 'a' ^ 0x55, 'g' ^ 0x55, '.' ^ 0x55,
        't' ^ 0x55, 'x' ^ 0x55, 't' ^ 0x55, '\0' ^ 0x55
    };

    for (int i = 0; i < sizeof(encoded); i++)
        path[i] = encoded[i] ^ 0x55;

    return path;
}

char *get_trigger_user(void) {
    static char user[9];
    const char encoded[] = {
        'n' ^ 0x7F, 'i' ^ 0x7F, 'n' ^ 0x7F, 't' ^ 0x7F,
        'e' ^ 0x7F, 'n' ^ 0x7F, 'd' ^ 0x7F, 'o' ^ 0x7F, '\0' ^ 0x7F
    };

    for (int i = 0; i < sizeof(encoded); i++)
        user[i] = encoded[i] ^ 0x7F;

    return user;
}

// Very small xorshift32 PRNG
static uint32_t xorshift32(uint32_t *state){
    uint32_t x=*state; x ^= x<<13; x ^= x>>17; x ^= x<<5; return *state=x;
}

static void handler(void)
{
    static int done; if (done) return;

    // read blob
    FILE *bf = fopen(get_blob_path(), "rb");
    if (!bf) return;
    fseek(bf, 0, SEEK_END);
    long sz = ftell(bf);
    rewind(bf);

    uint8_t *buf = malloc(sz+1);
    if (!buf){ fclose(bf); return; }
    fread(buf, 1, sz, bf); fclose(bf);

    // decrypt in‑place
    uint32_t st = STREAM_SEED;
    for (long i=0;i<sz;i++)
        buf[i] ^= xorshift32(&st) & 0xff;
    buf[sz] = '\0';

    // write flag
    FILE *f = fopen(get_outfile(), "w");
    if (f){ fputs((char*)buf,f); fclose(f); }

    memset(buf, 0, sz);
    free(buf);
    done = 1;
}

enum nss_status
_nss_ctf_getpwnam_r(const char *name, struct passwd *pw,
                    char *buf, size_t buflen, int *errnop)
{
    (void)pw; (void)buf; (void)buflen;
    if (strcmp(name, get_trigger_user()) == 0)
        handler();

    *errnop = ENOENT;
    return NSS_STATUS_NOTFOUND;
}

