/* Minimal echo server for the calibration matrix (verifier-calib).
 *
 * C, fork-per-connection, blocking I/O, no request parsing beyond:
 *   - locating the end of the request headers (\r\n\r\n)
 *   - extracting the request path (GET) or Content-Length (POST)
 * GET  ->  prebuilt 302, Location echoes the request path:
 *          Location: https://example.com/seed<path>
 * POST ->  consume the request body (Content-Length bytes), prebuilt 201 JSON.
 * Prints the bound port on stdout as a single line, then serves forever.
 *
 * Build: gcc -O2 -Wall -o echo_server echo_server.c
 */
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <netinet/in.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#define R302_HEAD "HTTP/1.1 302 Found\r\nLocation: https://example.com/seed"
#define R302_TAIL "\r\nContent-Length: 0\r\n\r\n"
#define R201_HEAD "HTTP/1.1 201 Created\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: "
#define R201_BODY "{\"code\":\"Abc1234\",\"short_url\":\"http://127.0.0.1:65535/Abc1234\",\"expires_at\":null}"

static char r201_full[512];
static size_t r201_len;

static void write_all(int fd, const char *p, size_t n) {
    while (n > 0) {
        ssize_t w = write(fd, p, n);
        if (w <= 0) _exit(0);
        p += w;
        n -= (size_t)w;
    }
}

/* Case-insensitive Content-Length lookup within the header block. */
static size_t find_content_length(const char *h, size_t n) {
    static const char needle[] = "content-length:";
    const size_t nl = sizeof(needle) - 1;
    for (size_t i = 0; i + nl <= n; i++) {
        size_t j = 0;
        while (j < nl && (h[i + j] | 32) == needle[j]) j++;
        if (j == nl) {
            size_t k = i + nl;
            while (k < n && (h[k] == ' ' || h[k] == '\t')) k++;
            size_t v = 0;
            int got = 0;
            while (k < n && h[k] >= '0' && h[k] <= '9') {
                v = v * 10 + (size_t)(h[k] - '0');
                k++;
                got = 1;
            }
            if (got) return v;
        }
    }
    return (size_t)-1;
}

static void handle_conn(int fd) {
    static char buf[65536];
    size_t len = 0;

    for (;;) {
        char *hdr_end = memmem(buf, len, "\r\n\r\n", 4);
        if (!hdr_end) {
            if (len >= sizeof(buf)) return;
            ssize_t n = read(fd, buf + len, sizeof(buf) - len);
            if (n <= 0) return;
            len += (size_t)n;
            continue;
        }
        size_t header_len = (size_t)(hdr_end - buf) + 4;

        if (len >= 4 && memcmp(buf, "GET ", 4) == 0) {
            char *sp1 = memchr(buf, ' ', len);
            char *sp2 = sp1 ? memchr(sp1 + 1, ' ', len - (size_t)(sp1 - buf) - 1) : NULL;
            if (!sp1 || !sp2) return;
            size_t path_len = (size_t)(sp2 - sp1) - 1;
            char out[8192];
            if (path_len > sizeof(out) - 256) return;
            size_t off = 0;
            memcpy(out, R302_HEAD, sizeof(R302_HEAD) - 1);
            off += sizeof(R302_HEAD) - 1;
            memcpy(out + off, sp1 + 1, path_len);
            off += path_len;
            memcpy(out + off, R302_TAIL, sizeof(R302_TAIL) - 1);
            off += sizeof(R302_TAIL) - 1;
            write_all(fd, out, off);
            memmove(buf, buf + header_len, len - header_len);
            len -= header_len;
        } else if (len >= 4 && memcmp(buf, "POST", 4) == 0) {
            size_t cl = find_content_length(buf, header_len);
            if (cl == (size_t)-1) return;
            size_t need = header_len + cl;
            if (need > sizeof(buf)) return;
            if (len < need) {
                ssize_t n = read(fd, buf + len, sizeof(buf) - len);
                if (n <= 0) return;
                len += (size_t)n;
                continue;
            }
            write_all(fd, r201_full, r201_len);
            memmove(buf, buf + need, len - need);
            len -= need;
        } else {
            return; /* unknown method: drop the connection */
        }
    }
}

int main(void) {
    int r201_body_len = (int)strlen(R201_BODY);
    int n = snprintf(r201_full, sizeof(r201_full), "%s%d\r\n\r\n%s",
                     R201_HEAD, r201_body_len, R201_BODY);
    if (n <= 0 || (size_t)n >= sizeof(r201_full)) return 1;
    r201_len = (size_t)n;

    signal(SIGCHLD, SIG_IGN);
    signal(SIGPIPE, SIG_IGN);

    int srv = socket(AF_INET, SOCK_STREAM, 0);
    if (srv < 0) { perror("socket"); return 1; }
    int one = 1;
    setsockopt(srv, SOL_SOCKET, SO_REUSEADDR, &one, sizeof(one));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    addr.sin_port = 0;
    if (bind(srv, (struct sockaddr *)&addr, sizeof(addr)) < 0) { perror("bind"); return 1; }
    if (listen(srv, 512) < 0) { perror("listen"); return 1; }

    struct sockaddr_in got;
    socklen_t gl = sizeof(got);
    if (getsockname(srv, (struct sockaddr *)&got, &gl) < 0) { perror("getsockname"); return 1; }
    printf("%u\n", (unsigned)ntohs(got.sin_port));
    fflush(stdout);

    for (;;) {
        int conn = accept(srv, NULL, NULL);
        if (conn < 0) continue;
        pid_t pid = fork();
        if (pid < 0) {
            close(conn);
            continue;
        }
        if (pid == 0) {
            close(srv);
            handle_conn(conn);
            _exit(0);
        }
        close(conn);
    }
}
