#!/usr/bin/env python3
"""The one HTTP client behind check-quotes.py, check-sources.py and archive-sources.py.

Every request carries the project's User-Agent, waits its turn at a Pacer and times out. One that
ends in a network error, a 429 or a 5xx is tried once more after a wait. Each script passes its
own pace, timeout and retry wait. The User-Agent names the project and, when CROSSREF_MAILTO is
set and not blank, a contact address, which is what puts a Crossref request in its polite pool.

Usage:
    python3 scripts/fetching.py     # self-check against a server on loopback

Stdlib only, so it runs anywhere the repo is checked out
"""

from __future__ import annotations

import hashlib
import http.client
import http.server
import os
import sys
import threading
import time
import urllib.error
import urllib.request
from collections.abc import Callable, Mapping
from typing import NamedTuple

REPO_URL = "https://github.com/peterklingelhofer/exhale"
CHUNK = 64 * 1024


def user_agent(mailto: str) -> str:
    """exhale-citations (<repo>), with a mailto after the repo address when there is one"""
    mailto = mailto.strip()
    return f"exhale-citations ({REPO_URL}{f'; mailto:{mailto}' if mailto else ''})"


# An unset repository variable reaches the workflow as an empty string
USER_AGENT = user_agent(mailto=os.environ.get("CROSSREF_MAILTO", ""))


class Reply(NamedTuple):
    status: int | None              # None when no answer came at all
    url: str | None                 # where the redirects ended
    headers: Mapping[str, str]      # empty when no answer came
    body: object                    # what read made of the body of a 200, else None
    error: str                      # "HTTP 503", "timed out" or the network error, empty after a 2xx


class Pacer:
    """Hands out request start times at least gap seconds apart, across threads, and counts them"""

    def __init__(self, gap: float) -> None:
        self.gap = gap
        self.lock = threading.Lock()
        self.next_start = 0.0
        self.requests = 0

    def wait(self) -> None:
        with self.lock:
            start = max(time.monotonic(), self.next_start)
            self.next_start = start + self.gap
            self.requests += 1
        time.sleep(max(0.0, start - time.monotonic()))


def read_text(response: http.client.HTTPResponse, deadline: float) -> str:
    """The whole body as text. A body shorter than its Content-Length is an error. Every reader takes
    the deadline and only read_sha256 keeps it"""
    return response.read().decode("utf-8", errors="replace")


def read_sha256(response: http.client.HTTPResponse, deadline: float) -> str:
    """The SHA-256 of the body, streamed so a large PDF is never held in memory"""
    digest = hashlib.sha256()
    while piece := response.read(CHUNK):
        digest.update(piece)
        if time.monotonic() > deadline:
            raise TimeoutError("the body took longer than the timeout")
    return digest.hexdigest()


def attempt(url: str, timeout: float, read: Callable[..., object] | None) -> Reply:
    """One GET with redirects followed. Any HTTP status is a reply, and a refused connection, a
    timeout or a body that fails to read is no answer. read turns the response of a 200 into the
    reply's body before the timeout runs out, and None leaves the body unread, which a large PDF can be"""
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    deadline = time.monotonic() + timeout
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = read(response=response, deadline=deadline) if read and response.status == 200 else None
            return Reply(status=response.status, url=response.geturl(), headers=response.headers, body=body, error="")
    except urllib.error.HTTPError as exc:
        with exc:
            return Reply(status=exc.code, url=exc.geturl(), headers=exc.headers, body=None, error=f"HTTP {exc.code}")
    except (OSError, http.client.HTTPException, ValueError) as exc:
        reason = getattr(exc, "reason", exc)
        error = "timed out" if isinstance(reason, TimeoutError) else str(reason) or type(exc).__name__
        return Reply(status=None, url=None, headers={}, body=None, error=error)


def get(
    url: str,
    pacers: tuple[Pacer, ...],
    timeout: float,
    retry_wait: float,
    read: Callable[..., object] | None = None,
) -> Reply:
    """A GET that waits at each pacer, then once more after retry_wait seconds on a network error,
    a 429 or a 5xx. A retry that gets no answer leaves the first reply standing"""

    def once() -> Reply:
        for pacer in pacers:
            pacer.wait()
        return attempt(url=url, timeout=timeout, read=read)

    first = once()
    if first.status is not None and first.status != 429 and first.status < 500:
        return first
    print(f"  {first.error}, retrying in {retry_wait:g} s: {url}", file=sys.stderr)
    time.sleep(retry_wait)
    second = once()
    return first if second.status is None and first.status is not None else second


def self_check() -> None:
    seen: list[str] = []
    body = b"caf\xc3\xa9"

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            seen.append(self.path)
            if self.path == "/flaky" and seen.count("/flaky") == 1:
                self.send_response(503)
            elif self.path == "/gone":
                self.send_response(404)
            elif self.path == "/moved":
                self.send_response(302)
                self.send_header("Location", "/ok")
            else:
                self.send_response(200)
                self.send_header("X-Agent", self.headers["User-Agent"])
            # /cut promises more bytes than it sends
            self.send_header("Content-Length", str(len(body) + (50 if self.path == "/cut" else 0)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *args: object) -> None:
            pass

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_port}"
    pacer = Pacer(gap=0.05)

    def fetch(path: str, read: Callable[..., object] | None = read_text) -> Reply:
        return get(url=base + path, pacers=(pacer,), timeout=5, retry_wait=0.05, read=read)

    try:
        ok = fetch(path="/ok")
        assert (ok.status, ok.body, ok.error, ok.headers["X-Agent"]) == (200, "caf\u00e9", "", USER_AGENT)
        assert fetch(path="/ok", read=None).body is None
        assert fetch(path="/ok", read=read_sha256).body == hashlib.sha256(body).hexdigest()
        assert fetch(path="/moved").url == base + "/ok"
        assert fetch(path="/cut").status is None
        seen.clear()
        gone = fetch(path="/gone")
        assert (gone.status, gone.error, gone.body, seen) == (404, "HTTP 404", None, ["/gone"])
        seen.clear()
        assert fetch(path="/flaky").status == 200 and seen == ["/flaky", "/flaky"]
    finally:
        server.shutdown()
        server.server_close()
    # nothing listens on the port now: no answer, tried twice, and the pacer counted both starts
    before = pacer.requests
    refused = fetch(path="/ok")
    assert (refused.status, refused.url) == (None, None) and refused.error and pacer.requests == before + 2
    assert user_agent(mailto="") == user_agent(mailto="  ") == f"exhale-citations ({REPO_URL})"
    assert user_agent(mailto=" a@b.org ") == f"exhale-citations ({REPO_URL}; mailto:a@b.org)"
    start, quick = time.monotonic(), Pacer(gap=0.2)
    quick.wait()
    quick.wait()
    assert time.monotonic() - start >= 0.19
    print("fetching self-check passed")


if __name__ == "__main__":
    self_check()
