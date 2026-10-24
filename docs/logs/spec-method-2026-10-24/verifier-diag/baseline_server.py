#!/usr/bin/env python3
"""Minimal 302 echo server for the client-side baseline (Task D).

Raw socket server: one thread per connection, no request parsing beyond
finding the end of the request headers, fixed precomputed 302 response.
This is the cheapest possible HTTP/1.1 keep-alive 302 responder in Python
stdlib, used to measure the load client's own overhead floor.
"""
import socket
import sys
import threading

RESP = (b"HTTP/1.1 302 Found\r\n"
        b"Location: https://example.com/seed/seed0001\r\n"
        b"Content-Length: 0\r\n"
        b"\r\n")


def handle(conn: socket.socket) -> None:
    buf = b""
    try:
        while True:
            while b"\r\n\r\n" not in buf:
                chunk = conn.recv(65536)
                if not chunk:
                    return
                buf += chunk
            _head, _, buf = buf.partition(b"\r\n\r\n")
            conn.sendall(RESP)
    except OSError:
        pass
    finally:
        conn.close()


def main() -> None:
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", 0))
    srv.listen(256)
    print(srv.getsockname()[1], flush=True)
    while True:
        conn, _ = srv.accept()
        threading.Thread(target=handle, args=(conn,), daemon=True).start()


if __name__ == "__main__":
    main()
