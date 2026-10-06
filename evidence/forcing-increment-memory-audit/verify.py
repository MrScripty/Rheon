"""Replay the static memory audit; no native numerical execution."""
from pathlib import Path
import copy
import hashlib
import importlib.util
import json
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
BASE = "acec66b0d50556ea6a482ec796fb0f5d36810aca"
TREE = "9364cb71dafdd5de3fa43fdfc21b310beade63a0"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def preserved():
    require(subprocess.check_output(["git","rev-parse",BASE+"^{tree}"],cwd=ROOT).decode().strip() == TREE,
            "frozen completed comparison tree")
    digest = hashlib.sha256()
    count = 0
    for entry in subprocess.check_output(["git","ls-tree","-rz",BASE],cwd=ROOT).split(b"\0"):
        if not entry:
            continue
        metadata, name = entry.split(b"\t",1)
        mode, kind, oid = metadata.split()
        require(mode in (b"100644",b"100755") and kind == b"blob", "frozen file inventory modes")
        data = (ROOT/name.decode()).read_bytes()
        require(hashlib.sha1(b"blob "+str(len(data)).encode()+b"\0"+data).hexdigest().encode() == oid,
                "every earlier file preserved: "+name.decode())
        digest.update(name+b"\0"+hashlib.sha256(data).digest())
        count += 1
    require(count == 7068, "full completed comparison inventory")
    return dict(files=count, path_content_sha256=digest.hexdigest())


def checks():
    spec = importlib.util.spec_from_file_location("memory_audit",HERE/"audit.py")
    audit = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(audit)
    result = audit.run()
    normal = (HERE/"audit-normal.json").read_bytes()
    require(normal == (HERE/"audit-optimized.json").read_bytes(), "actual reader mode parity")
    require(result == json.loads(normal), "full static memory reader replay")
    require(result["point_caller_delta"] == 1808 and result["kernel_peak"] == 1536
            and result["aligned_subtotal"] == 65456 and result["remaining_bytes"] == 80
            and result["simultaneous_full_cap_reservation"] == 540304
            and result["cap_pass"] is True and result["outgoing_jump_edges"] == [], "closed simultaneous measured result")
    frozen = audit.functions(audit.disassembly("frozen-relevant-disassembly.txt"), (HERE/"frozen-symbols.txt").read_text())
    add = next(f for f in frozen.values() if f["name"].endswith("Scalar::add"))
    conversions = next(f for f in frozen.values() if f["name"].endswith("Scalar::from_f64"))
    rejected = []
    def reject(label, operation):
        try:
            operation()
        except ValueError:
            rejected.append(label)
        else:
            raise ValueError("missed corruption: "+label)
    def unresolved(asm, target):
        raise ValueError("unresolved synthetic transfer")
    def entry_changed(instruction):
        f = copy.deepcopy(conversions)
        f["code"][f["start"]] = instruction
        require(audit.transfers(f)[0]["kind"] == "outgoing_jump" if instruction.startswith("jmp") else True,
                "numeric outgoing jump classified")
        audit.kernel_cfg(f, unresolved)
    reject("outgoing direct tail jump", lambda:entry_changed("jmp    dead000 <unqualified>"))
    reject("unresolved indirect tail jump", lambda:entry_changed("jmp    *%rax"))
    reject("hidden additional callee", lambda:entry_changed("call   dead000 <unqualified>"))
    def unbalanced():
        f = copy.deepcopy(add)
        last_pop = max(a for a,s in f["code"].items() if s.startswith("pop"))
        f["code"][last_pop] = "nop"
        audit.kernel_cfg(f, unresolved)
    reject("unbalanced return", unbalanced)
    def dynamic():
        f = copy.deepcopy(add)
        f["code"][f["start"]] = "and    $0xfffffffffffffff0,%rsp"
        audit.kernel_cfg(f, unresolved)
    reject("dynamic stack mutation", dynamic)
    def enlarged():
        f = copy.deepcopy(add)
        # Two balanced fixed decrements/increments deliberately add 128 bytes.
        addresses = sorted(f["code"])
        f["code"][addresses[0]] = "sub    $0x80,%rsp"
        pops = [a for a,s in f["code"].items() if s.startswith("pop")]
        f["code"][max(pops)] = "add    $0x80,%rsp"
        measured = audit.kernel_cfg(f, unresolved)
        require(measured["frame_bound"] == 48, "increased frame cannot retain old bound")
    reject("increased compiled stack frame", enlarged)
    def missing_delta():
        altered = copy.deepcopy(result)
        altered["point_caller_delta"] = 0
        require(altered["point_caller_delta"] == max(x["frame_bound"] for x in altered["caller_records"].values() if x["name"].endswith("Work::point"))
                -min(x["frame_bound"] for x in altered["native_point_records"].values()), "caller delta charged")
    reject("omitted caller frame delta", missing_delta)
    return dict(status="PASS_ADDITIVE_STATIC_MEMORY_AUDIT_ONLY", kernel_peak=result["kernel_peak"],
                point_caller_delta=result["point_caller_delta"], aligned_additional_bound=result["aligned_subtotal"],
                cap=result["cap"], remaining_bytes=result["remaining_bytes"],
                simultaneous_full_cap_reservation=result["simultaneous_full_cap_reservation"],
                enclosing_arithmetic_stack_prefix=result["enclosing_arithmetic_stack_prefix"],
                outgoing_tail_edges_found=0, negative_controls=rejected, numerical_execution_repeated=False,
                production_changes=0, owner_advances=0, qualification_scope=result["scope"])


def artifacts():
    return {str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in sorted(HERE.iterdir())
            if p.is_file() and p.name != "receipt.json" and p.name != "record_post_git.py"
            and not p.name.startswith(("root-verification", "post-git", "freeze"))}


if __name__ == "__main__":
    if "--freeze" in sys.argv:
        result = dict(base=BASE, base_tree=TREE, preserved_inventory=preserved(), artifacts=artifacts(), checks=checks())
        (HERE/"receipt.json").write_text(json.dumps(result,indent=2,sort_keys=True)+"\n")
        print(json.dumps(dict(status="FROZEN_STATIC_MEMORY_AUDIT",receipt_sha256=sha((HERE/"receipt.json").read_bytes()))))
    else:
        receipt = json.loads((HERE/"receipt.json").read_text())
        require(receipt["base"] == BASE and receipt["base_tree"] == TREE, "exact additive source identity")
        require(receipt["preserved_inventory"] == preserved(), "all previous source/evidence bytes")
        require(receipt["artifacts"] == artifacts(), "closed static-audit artifacts")
        require(receipt["checks"] == checks(), "complete measured audit result replay")
        print(json.dumps(dict(**receipt["checks"], preserved_files=receipt["preserved_inventory"]["files"],
                             artifacts=len(receipt["artifacts"])),indent=2,sort_keys=True))
