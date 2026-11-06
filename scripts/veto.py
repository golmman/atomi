#!/usr/bin/env python3
"""make veto: our validator + `provari check` on the veto fixtures (D16).

Stdlib only. A missing oracle makes the gate red (INCONCLUSIVE), never
skipped. Classes per working agreement 4.
"""
import json
import os
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(ROOT, "target", "veto")
PV_DIR = os.path.join(OUT_DIR, "provari")
VETO_TSV = os.path.join(ROOT, "tests", "fixtures", "veto.tsv")

# Documented rules differences that make a checker-reject a rules-divergence.
DOCUMENTED_DIVERGENCES = {}

failures = []


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True)


def jline(out):
    return json.loads(out.stdout.strip().splitlines()[-1])


def main():
    atomi = os.path.join(ROOT, "target", "release", "atomi")
    provari = "/tmp/atomi-oracle/provari/target/release/provari"
    argv = sys.argv[1:]
    for i, a in enumerate(argv):
        if a == "--atomi":
            atomi = argv[i + 1]
        elif a == "--provari":
            provari = argv[i + 1]
    if not os.path.exists(provari):
        print("INCONCLUSIVE: oracle not built (run make oracle)")
        sys.exit(1)
    os.makedirs(PV_DIR, exist_ok=True)

    rows = []
    for line in open(VETO_TSV):
        if not line.strip() or line.startswith("#") or line.startswith("name\t"):
            continue
        f = line.rstrip("\n").split("\t")
        rows.append((f[0], f[1], f[2]))

    def atomi_validate(path):
        return jline(run([atomi, "validate", path]))

    def provari_check(path):
        try:
            return jline(run([provari, "check", path]))
        except Exception:
            return {"valid": None}

    def atomi_moves(fen):
        return json.loads(run([atomi, "moves", fen]).stdout)["moves"]

    proofs = {}
    table = []
    for name, fen, role in rows:
        path = os.path.join(OUT_DIR, f"{name}.jsonl")
        pj = jline(run([atomi, "prove", fen, "--memory", "192", "--seconds",
                        "600", "--proof-out", path]))
        if not pj["found"]:
            cls = "not-proved"
        elif pj["proof"] != "written":
            cls = "export-failure"
        else:
            v = atomi_validate(path)
            c = provari_check(path)
            proofs[name] = (path, fen, v, c)
            if not v["valid"]:
                cls = "validator-reject"
            elif c.get("valid") is None:
                cls = "unchecked"
            elif not c.get("valid"):
                cls = DOCUMENTED_DIVERGENCES.get(name, "checker-reject")
            elif v["positions"] != c.get("won_positions"):
                cls = "disagreement"
            else:
                cls = "accepted"
            if cls != "accepted":
                failures.append(f"{name}: {cls} ours={v.get('valid')}/"
                                f"{v.get('positions')} checker={c.get('valid')}/"
                                f"{c.get('won_positions')}")
            table.append((name, cls, v.get("positions"), c.get("won_positions"),
                          v.get("length"), v.get("header_moves"), c.get("valid")))
            continue
        table.append((name, cls, None, None, None, None, None))
        failures.append(f"{name}: {cls}")

    # Negative controls on the control row.
    ctl = []
    ctrl = next((n for n, _, r in rows if r == "control"), None)
    if ctrl and ctrl in proofs:
        path, fen, v, c = proofs[ctrl]
        lines = open(path).read().splitlines()
        header = json.loads(lines[0])
        root_key = " ".join(header["root"].split(" ")[:4]) + " " + header["root"].split(" ")[4]
        idx = next(i for i, l in enumerate(lines[1:], 1)
                   if json.loads(l).get("fen") and
                   " ".join(json.loads(l)["fen"].split(" ")[:5]) == root_key)
        root_line = json.loads(lines[idx])
        rest = lines[:idx] + lines[idx + 1:]

        def verdicts(text, tag):
            tmp = os.path.join(OUT_DIR, f"ctl-{tag}.jsonl")
            open(tmp, "w").write(text + "\n")
            return atomi_validate(tmp), provari_check(tmp)

        # control: drop the root's line
        v2, c2 = verdicts("\n".join(rest), f"{ctrl}-drop")
        ok = v2["valid"] is False and c2.get("valid") is False
        ctl.append(f"drop-root: ours={v2['valid']} checker={c2.get('valid')} {'OK' if ok else 'FAIL'}")
        if not ok:
            failures.append(f"control drop-root: {v2} {c2}")

        # control: illegal root move (commoner one-step onto own square, else
        # any own piece onto an own square)
        placement, stm = fen.split(" ")[0], fen.split(" ")[1]
        sq = {}
        rk = 7
        for row in placement.split("/"):
            fl = 0
            for ch in row:
                if ch.isdigit():
                    fl += int(ch)
                else:
                    sq[fl + rk * 8] = ch
                    fl += 1
            rk -= 1
        white = stm == "w"
        king = next(s for s, p in sq.items() if p == ("K" if white else "k"))
        cand = None
        kx, ky = king % 8, king // 8
        for dx, dy in [(a, b) for a in (-1, 0, 1) for b in (-1, 0, 1) if (a, b) != (0, 0)]:
            t = (kx + dx) + (ky + dy) * 8
            if t in sq and sq[t].isupper() == white:
                cand = "abcdefgh"[kx] + str(ky + 1) + "abcdefgh"[t % 8] + str(t // 8 + 1)
                break
        if cand is None:
            own = [s for s, p in sq.items() if p.isupper() == white]
            cand = ("abcdefgh"[own[0] % 8] + str(own[0] // 8 + 1)
                    + "abcdefgh"[own[1] % 8] + str(own[1] // 8 + 1))
        bad = dict(root_line, move=cand)
        v2, c2 = verdicts("\n".join(lines[:idx] + [json.dumps(bad, separators=(",", ":"))] + lines[idx + 1:]),
                          f"{ctrl}-illegal")
        ok = (v2["valid"] is False and "illegal" in (v2.get("error") or "")
              and c2.get("valid") is False)
        ctl.append(f"illegal-root-move ({cand}): ours={v2['valid']}({v2.get('error')}) "
                   f"checker={c2.get('valid')} {'OK' if ok else 'FAIL'}")
        if not ok:
            failures.append(f"control illegal: {v2} {c2}")

        # control: substitute every other legal root move (sorted UCI)
        cands = sorted(m for m in atomi_moves(fen) if m != root_line["move"])
        decided = None
        for cand in cands:
            bad = dict(root_line, move=cand)
            v2, c2 = verdicts("\n".join(lines[:idx] + [json.dumps(bad, separators=(",", ":"))] + lines[idx + 1:]),
                              f"{ctrl}-alt-{cand}")
            ov, cv = v2["valid"], c2.get("valid")
            if ov and cv:
                ctl.append(f"alt-move {cand}: agreement (both accept)")
                continue
            decided = (cand, ov, cv)
            break
        if decided is None:
            ctl.append("alt-move: no rejecting alternative -> FAIL")
            failures.append("control alt-move: no rejecting alternative")
        else:
            cand, ov, cv = decided
            ok = ov is False and cv is False
            ctl.append(f"alt-move {cand}: ours={ov} checker={cv} {'OK' if ok else 'FAIL'}")
            if not ok:
                failures.append(f"control alt-move {cand}: ours={ov} checker={cv}")

    # 50-move control: shift every counter +1 in the rule50 row's proof.
    r50 = next((n for n, _, r in rows if r == "rule50"), None)
    if r50 and r50 in proofs:
        path, fen, v, c = proofs[r50]
        lines = open(path).read().splitlines()
        out = []
        for l in lines:
            o = json.loads(l)
            for fld in ("root", "fen"):
                if fld in o:
                    p = o[fld].split(" ")
                    p[4] = str(int(p[4]) + 1)
                    o[fld] = " ".join(p)
            out.append(json.dumps(o, separators=(",", ":")))
        tmp = os.path.join(OUT_DIR, "ctl-rule50.jsonl")
        open(tmp, "w").write("\n".join(out) + "\n")
        v2, c2 = atomi_validate(tmp), provari_check(tmp)
        ok = v2["valid"] is False and c2.get("valid") is False
        ctl.append(f"rule50+1: ours={v2['valid']} checker={c2.get('valid')} {'OK' if ok else 'FAIL'}")
        if not ok:
            failures.append(f"control rule50+1: {v2} {c2}")

    # Reverse control: provari prove -> our validator vs checker.
    rev = {}
    for name, fen, role in rows:
        out = os.path.join(PV_DIR, f"{name}.jsonl")
        pj = jline(run([provari, "prove", fen, "--guide", "hand", "--proofs", out, "--seconds", "60"]))
        if not pj.get("found"):
            rev[name] = "provari-no-win"
            failures.append(f"reverse {name}: provari found no win")
            continue
        v = atomi_validate(out)
        c = provari_check(out)
        ok = v["valid"] == c.get("valid") and (
            not v["valid"] or v["positions"] == c.get("won_positions"))
        rev[name] = "agree" if ok else "DISAGREE"
        if not ok:
            failures.append(f"reverse {name}: {v} vs {c}")

    print(f"{'row':<16}{'class':<18}{'pos/won':<10}{'len':<6}{'hdr':<6}{'chk':<7}{'rev'}")
    for name, cls, pos, won, length, hdr, chk in table:
        print(f"{name:<16}{cls:<18}{str(pos) + '/' + str(won):<10}"
              f"{str(length):<6}{str(hdr):<6}{str(chk):<7}{rev.get(name, '-')}")
    print("\ncontrols:")
    for r in ctl:
        print(" ", r)
    if failures:
        print("\nVETO: FAIL")
        for f in failures:
            print("  -", f)
        sys.exit(1)
    print("\nVETO: PASS")


if __name__ == "__main__":
    main()
