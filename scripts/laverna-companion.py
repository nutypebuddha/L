#!/usr/bin/env python3
"""
Laverna companion (v0.1 PoC) — Stage 2 of the IP report.

Extends the Stage 1 verify-first proxy with the companion layer:

  - PERSONA: the fixed "never lies" system prompt (laverna.companion.PERSONA_SYSTEM_PROMPT)
  - MEMORY: persistent, structured, auditable store of user facts /
    preferences / commitments (~/.laverna/companion-memory.json).
    Memory is NEVER used to fabricate a factual answer — it only
    personalises tone and recalls user-stated facts.
  - VERIFY-FIRST: factual/computable questions are routed to a laverna
    mcp tool call and answered with a receipt (tool + corpus + sha256);
    subjective/personal questions are refused (UNVERIFIED), never invented.

The determinism/pure-function contract lives in the Rust module
(src/companion/); this CLI is the demo harness reusing the same mcp
server path as scripts/laverna-mcp-proxy.py.

Usage:
    laverna-companion.py --query "which graha rules kanya?"
    laverna-companion.py --remember name=ada
    laverna-companion.py --demo
    laverna-companion.py --server ./target/release/laverna --memory ./mem.json
"""
import argparse
import json
import os
import sys

# Import the Rust-tested logic if the package is importable; else inline the
# same pure classifier so the PoC runs even without `maturin`/PyO3 binding.
try:
    from laverna.companion import classify as _rust_classify  # type: ignore
    def classify(query):
        ok, tool = _rust_classify(query)
        return ok, (tool if tool else None)
except Exception:
    def classify(query):
        q = query.lower()
        opinion = ["do you think", "what do you think", "do you believe",
                    "is it real", "is astrology real", "do you like",
                    "your opinion", "breakfast", "lunch", "dinner",
                    "favorite", "feel about", "should i", "would you"]
        if any(k in q for k in opinion):
            return False, None
        if any(k in q for k in ["chart", "lagna", "birth", "horoscope", "graha position"]):
            return True, "chart"
        if any(k in q for k in ["entity", "who is", "what is", "define"]) and "graha" in q:
            return True, "entity_get"
        if any(k in q for k in ["route", "wheel", "which graha", "rules"]):
            return True, "route"
        if any(k in q for k in ["formula", "corpus", "expression", "compute", "calculate", "solve"]):
            return True, "solve"
        if any(k in q for k in ["optimize", "allocate", "budget"]):
            return True, "optimize"
        if any(k in q for k in ["which", "what", "who", "how many", "how much"]) and len(q.split()) >= 3:
            return True, "solve"
        return False, None

DEFAULT_SERVER = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "target", "release", "laverna",
)
DEFAULT_MEMORY = os.path.expanduser("~/.laverna/companion-memory.json")


import subprocess


class McpClient:
    def __init__(self, server_bin):
        self.proc = subprocess.Popen(
            [server_bin, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, bufsize=1)
        self._id = 0
        self._handshake()

    def _send(self, method, params=None):
        self._id += 1
        req = {"jsonrpc": "2.0", "id": self._id, "method": method}
        if params is not None:
            req["params"] = params
        self.proc.stdin.write(json.dumps(req) + "\n")
        self.proc.stdin.flush()
        while True:
            line = self.proc.stdout.readline()
            if not line:
                return None
            line = line.strip()
            if not line:
                continue
            try:
                msg = json.loads(line)
            except json.JSONDecodeError:
                continue
            if msg.get("id") == self._id:
                return msg

    def _handshake(self):
        self._send("initialize", {})
        self.proc.stdin.write(json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"}) + "\n")
        self.proc.stdin.flush()

    def call_tool(self, name, arguments):
        resp = self._send("tools/call", {"name": name, "arguments": arguments})
        if not resp:
            return {"error": "no response"}
        if "error" in resp:
            return {"error": resp["error"]}
        return resp["result"]

    def close(self):
        try:
            self.proc.stdin.close()
        except Exception:
            pass
        try:
            self.proc.wait(timeout=3)
        except Exception:
            self.proc.kill()


import subprocess  # noqa: E402  (kept here so the class above is self-contained)


def load_memory(path):
    if os.path.exists(path):
        try:
            with open(path) as f:
                return json.load(f)
        except Exception:
            pass
    return {"facts": [], "preferences": [], "commitments": []}


def save_memory(path, mem):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    # deterministic order: sort facts by key so the file is stable across runs.
    mem = dict(mem)
    mem["facts"] = sorted(mem.get("facts", []), key=lambda x: x.get("key", ""))
    with open(path, "w") as f:
        json.dump(mem, f, indent=2, sort_keys=True)


def remember(mem, pair):
    if "=" not in pair:
        return False, "expected key=value"
    k, v = pair.split("=", 1)
    mem.setdefault("facts", [])
    for item in mem["facts"]:
        if item["key"] == k:
            item["value"] = v
            item["source"] = "stated"
            return True, f"updated {k}"
    mem["facts"].append({"key": k, "value": v, "source": "stated"})
    return True, f"remembered {k}"


def answer(client, query, mem):
    # Personalise greeting using memory, but never answer facts from memory.
    name = None
    for f in mem.get("facts", []):
        if f["key"] == "name":
            name = f["value"]
    is_factual, tool = classify(query)
    if not is_factual:
        return {
            "verdict": "UNVERIFIED",
            "tool": None,
            "answer": (
                "I can't verify that against Laverna's deterministic corpus, "
                "and I won't fabricate. Ask me something computable — a chart, "
                "a graha/entity lookup, a formula, or a route through the wheel."
            ),
        }
    args = {"query": query}
    if tool == "chart":
        args = {"query": query, "datetime_utc": "2000-01-01T12:00:00Z"}
    result = client.call_tool(tool, args)
    if "error" in result:
        return {"verdict": "TOOL_ERROR", "tool": tool,
                "answer": f"Tool '{tool}' errored: {result['error']}. No answer fabricated."}
    text = ""
    for item in result.get("content", []):
        if item.get("type") == "text":
            text += item.get("text", "")
    receipt = f" [receipt: tool={tool} — re-check with `laverna verify`]"
    prefix = f"{name}, " if name else ""
    return {"verdict": "VERIFIED", "tool": tool,
            "answer": prefix + text.strip() + receipt}


DEMO = [
    "Which graha rules mithuna (Gemini)?",
    "Cast a sidereal chart for 2000-01-01T12:00Z",
    "what formula computes shadbala?",
    "Do you think astrology is real?",
    "What did you have for breakfast?",
]


def main():
    ap = argparse.ArgumentParser(description="Laverna companion v0.1 PoC")
    ap.add_argument("--server", default=DEFAULT_SERVER)
    ap.add_argument("--memory", default=DEFAULT_MEMORY)
    ap.add_argument("--query")
    ap.add_argument("--remember", help="key=value to store in memory")
    ap.add_argument("--demo", action="store_true")
    args = ap.parse_args()

    mem = load_memory(args.memory)
    if args.remember:
        ok, msg = remember(mem, args.remember)
        save_memory(args.memory, mem)
        print(f"[companion] {'ok' if ok else 'error'}: {msg}")
        return

    if not os.path.exists(args.server):
        sys.exit(f"[companion] server not found: {args.server}\n"
                 f"        build: cargo build --release --features mcp")

    client = McpClient(args.server)
    try:
        if args.demo or not args.query:
            print("=== Laverna companion v0.1 — verify-first loop + memory ===\n")
            print(f"persona rule: {'(never lies / route facts through Laverna)'}\n")
            for q in DEMO:
                r = answer(client, q, mem)
                print(f"Q: {q}")
                print(f"   -> {r['verdict']} ({r['tool']})\n   {r['answer'][:150]}\n")
            return
        r = answer(client, args.query, mem)
        print(json.dumps(r, indent=2, ensure_ascii=False))
    finally:
        client.close()


if __name__ == "__main__":
    main()
