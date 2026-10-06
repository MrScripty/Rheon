"""Static audit of already executed binaries. Never builds or executes a probe."""
from pathlib import Path
import hashlib
import gzip
import json
import re

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
OLD = ROOT / "evidence/forcing-increment-comparison"
PREFIX = "rheon::research_increment_comparison::scalar::"


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def disassembly(name):
    raw = gzip.decompress((HERE/(name+".gz")).read_bytes())
    require(sha(raw) == (HERE/(name+".sha256")).read_text().strip(), "lossless exact disassembly archive")
    return raw.decode()


def functions(raw, symbols):
    sizes = {}
    for line in symbols.splitlines():
        m = re.match(r"^([a-f0-9]+) ([a-f0-9]+) [tT] (.+)$", line)
        if m:
            sizes[int(m[1], 16)] = (int(m[2], 16), m[3])
    result = {}
    for part in re.split(r"(?m)(?=^[a-f0-9]+ <)", raw):
        m = re.match(r"^([a-f0-9]+) <(.+)>:", part)
        if not m:
            continue
        start = int(m[1], 16)
        require(start in sizes and sizes[start][1] == m[2], "ELF-sized address identity")
        end = start + sizes[start][0]
        code = {}
        for line in part.splitlines():
            fields = line.split("\t")
            if len(fields) >= 3 and re.match(r"^\s*[a-f0-9]+:$", fields[0]):
                address = int(fields[0].strip()[:-1], 16)
                if start <= address < end:
                    code[address] = fields[-1].strip()
        require(code and min(code) == start, "function entry captured")
        result[start] = dict(name=m[2], start=start, end=end, code=code)
    return result


def transfers(f):
    """Classify EVERY call and branch by numeric address, including tail jumps."""
    edges = []
    for address, asm in f["code"].items():
        m = re.match(r"^(call\S*|j\S*)\s+(.+)$", asm)
        if not m:
            continue
        direct = re.match(r"^([a-f0-9]+)\s", m[2])
        target = int(direct[1], 16) if direct else None
        internal = target is not None and f["start"] <= target < f["end"]
        if internal:
            require(target in f["code"], "internal transfer instruction captured")
        edges.append(dict(address=address, instruction=asm, opcode=m[1], target=target,
                          kind="call" if m[1].startswith("call") else "internal_branch" if internal else "outgoing_jump"))
    return edges


def fixed_frame(f):
    """Conservative fixed frame, recognizing bounded LLVM stack-probe loops."""
    code = f["code"]
    pushes = sum(8 for a in code.values() if a.startswith("push"))
    decrements = []
    for address, asm in code.items():
        m = re.match(r"^sub\s+\$0x([a-f0-9]+),%rsp$", asm)
        if m:
            decrements.append((address, int(m[1], 16)))
        if re.search(r",%rsp$", asm) and not asm.startswith(("sub", "add", "cmp")):
            require(False, "no unrecognized stack mutation")
    # Probe loop: mov rsp,r11; sub fixed,r11; sub 4096,rsp; store; cmp; jne.
    entries = list(code.items())
    loop = 0
    for i, (_, asm) in enumerate(entries):
        m = re.match(r"^sub\s+\$0x([a-f0-9]+),%r11$", asm)
        if not m:
            continue
        require(i > 0 and entries[i-1][1] == "mov    %rsp,%r11", "fixed probe base")
        block = entries[i+1:i+5]
        require(len(block) == 4 and block[0][1] == "sub    $0x1000,%rsp"
                and block[1][1] == "movq   $0x0,(%rsp)"
                and block[2][1] == "cmp    %r11,%rsp"
                and re.match(r"^jne\s+" + format(block[0][0], "x") + r"\s", block[3][1]), "recognized probe loop")
        total = int(m[1], 16)
        require(total % 4096 == 0, "whole fixed probe pages")
        loop += total - 4096
    negative = max([int(v, 16) for asm in code.values()
                    for v in re.findall(r"-0x([a-f0-9]+)\(%rsp\)", asm)] or [0])
    return dict(push_bytes=pushes, fixed_decrement_bytes=sum(v for _, v in decrements)+loop,
                fixed_probe_loop_bytes=loop, below_rsp_bytes=negative, return_address_bytes=8,
                frame_bound=pushes+sum(v for _, v in decrements)+loop+negative+8)


def kernel_cfg(f, resolve):
    """Track actual stack depth at each reachable instruction and transfer."""
    code = f["code"]
    addresses = sorted(code)
    following = dict(zip(addresses, addresses[1:]))
    pending = [(f["start"], 0)]
    depths = {}
    edges = []
    maximum = 8
    while pending:
        address, depth = pending.pop()
        if address in depths:
            require(depths[address] == depth, "stack-balanced joins and loops")
            continue
        require(address in code and depth >= 0, "captured bounded stack state")
        depths[address] = depth
        asm = code[address]
        negative = max([int(v, 16) for v in re.findall(r"-0x([a-f0-9]+)\(%rsp\)", asm)] or [0])
        # Positive indexed local references in mul are protected by the same
        # private [u32;8] bounds/normalization invariant as the frozen source.
        require(not re.search(r"-0x[a-f0-9]+\(%rsp,", asm), "no negative indexed stack reference")
        maximum = max(maximum, 8+depth+negative)
        if asm.startswith("push"):
            depth += 8
        elif asm.startswith("pop"):
            depth -= 8
        elif re.search(r",%rsp$", asm):
            m = re.match(r"^(sub|add)\s+\$0x([a-f0-9]+),%rsp$", asm)
            require(m is not None, "fixed kernel stack mutations only")
            depth += int(m[2], 16) * (1 if m[1] == "sub" else -1)
        maximum = max(maximum, 8+depth)
        if asm.startswith("ret"):
            require(depth == 0, "balanced kernel return")
            continue
        require(not asm.startswith(("int3", "ud2", "enter", "leave")), "no unqualified reachable trap/stack operation")
        transfer = re.match(r"^(call\S*|j\S*)\s+(.+)$", asm)
        if transfer:
            direct = re.match(r"^([a-f0-9]+)\s", transfer[2])
            target = int(direct[1], 16) if direct else None
            internal = target in code if target is not None else False
            if transfer[1].startswith("call") or not internal:
                callee, disposition = resolve(asm, target)
                edges.append(dict(address=address, instruction=asm, stack_depth=depth,
                                  kind="call" if transfer[1].startswith("call") else "outgoing_jump",
                                  callee=callee, disposition=disposition))
                if disposition == "excluded_private_invariant_panic":
                    continue
                if not transfer[1].startswith("call"):
                    # Conservative tail edge retains caller allocation and
                    # counts the caller return address too; no ignored edge.
                    if transfer[1].startswith("jmp"):
                        continue
            elif internal:
                pending.append((target, depth))
                if transfer[1].startswith("jmp"):
                    continue
        require(address in following, "reachable fallthrough captured")
        pending.append((following[address], depth))
    return dict(frame_bound=maximum, reachable_instructions=len(depths), edges=edges,
                instruction_stack_depths={format(a, "x"): d for a, d in sorted(depths.items())})


def run():
    binding = json.loads((HERE/"binary-binding.json").read_text())
    for key in ("frozen", "native_reference"):
        require(sha(Path(binding[key+"_binary"]).read_bytes()) == binding[key+"_binary_sha256"], "bound existing binary")
    old_receipt = json.loads((OLD/"receipt.json").read_text())
    for path, digest in old_receipt["artifacts"].items():
        require(sha((ROOT/path).read_bytes()) == digest, "every frozen numerical/source artifact preserved")
    require(sha((OLD/"receipt.json").read_bytes()) == "e32b08711a94747b93c706ae04cf5d7eb11fa94b75aa81a86b1369c327ed75c0", "frozen result receipt")
    def source_function(text, name, next_name):
        return text.split("    fn "+name+"(", 1)[1].split("    fn "+next_name+"(", 1)[0]
    production = (ROOT/"src/coupled_discrete.rs").read_text()
    reference = (ROOT/binding["native_reference_source"]).read_text()
    prototype = (OLD/"native_staged.rs").read_text()
    source_records = {}
    for name, next_name in (("point","partition"), ("partition","equation")):
        native_body = source_function(production, name, next_name)
        require(source_function(reference, name, next_name) == native_body,
                "reference point/partition source unchanged by diagnostic overlay")
        changed_body = source_function(prototype, name, next_name)
        if name == "point":
            changed_body, replacements = re.subn(
                r"        let values = if let Some\(research\).*?\n        };",
                "        let values = linear(block, rhs)?;", changed_body, count=1, flags=re.S)
            require(replacements == 1, "one private chart hook")
        require(changed_body == native_body, "all non-hook point/partition source preserved")
        source_records[name] = sha(native_body.encode())
    frozen = functions(disassembly("frozen-relevant-disassembly.txt"), (HERE/"frozen-symbols.txt").read_text())
    native = functions(disassembly("native-reference-disassembly.txt"), (HERE/"native-reference-symbols.txt").read_text())
    kernel = {a:f for a,f in frozen.items() if f["name"].startswith(PREFIX) and not f["name"].endswith("::conformance")}
    require(len(kernel) == 7, "all kernel functions by unique address")
    relocs = (HERE/"frozen-relocations.txt").read_text()
    panic_match = re.search(r"^([a-f0-9]+) [a-f0-9]+ T core::panicking::panic_bounds_check$", (HERE/"frozen-symbols.txt").read_text(), re.M)
    require(panic_match is not None, "panic symbol address")
    panic = panic_match[1].lstrip("0")

    def resolve(asm, target):
        if target in kernel:
            return target, "bounded_kernel"
        slot = re.search(r"# ([a-f0-9]+) ", asm)
        if slot and "<memset@" in asm:
            return "memset", "bound_default_libc_leaf"
        if slot and re.search(r"^0*"+slot[1]+r" .*R_X86_64_RELATIVE\s+"+panic+r"$", relocs, re.M):
            return "panic_bounds_check", "excluded_private_invariant_panic"
        raise ValueError("unqualified kernel transfer: "+asm)

    records = {a:kernel_cfg(f, resolve) for a,f in kernel.items()}
    # Reuse the frozen default-libc leaf only after independently closing its
    # reachable call AND branch graph, including all outgoing/indirect jumps.
    leaf = {}
    for line in (OLD/"memset-leaf-disassembly.txt").read_text().splitlines():
        fields = line.split("\t")
        if len(fields) >= 3 and re.match(r"^\s*[a-f0-9]+:$", fields[0]):
            leaf[int(fields[0].strip()[:-1], 16)] = fields[-1].strip()
    libc = json.loads((OLD/"memset-binding.json").read_text())
    require(sha(Path(libc["library"]).read_bytes()) == json.loads((OLD/"memory-preflight-staged.json").read_text())["libc_sha256"], "same frozen libc")
    leaf_record = kernel_cfg(dict(start=libc["resolved_file_address"], code=leaf), lambda asm,t: (_ for _ in ()).throw(ValueError("outgoing libc transfer: "+asm)))
    require(not leaf_record["edges"] and leaf_record["frame_bound"] == 8, "closed no-stack default memset leaf")

    def peak(address, path=()):
        if address == "memset":
            return 8, ["default libc memset"]
        require(address not in path, "no recursive additional kernel")
        record = records[address]
        best, names = record["frame_bound"], [kernel[address]["name"]]
        for edge in record["edges"]:
            if edge["disposition"] == "excluded_private_invariant_panic":
                continue
            child, child_names = peak(edge["callee"], path+(address,))
            value = 8+edge["stack_depth"]+child
            if value > best:
                best, names = value, [kernel[address]["name"], *child_names]
        return best, names

    solve = next(a for a,f in kernel.items() if f["name"].endswith("ChartWorkspace::solve"))
    kernel_peak, path = peak(solve)
    def rows(fs, suffix):
        return {a:dict(name=f["name"], **fixed_frame(f), transfers=transfers(f))
                for a,f in fs.items() if f["name"].endswith(suffix)}
    points, native_points = rows(frozen,"Work::point"), rows(native,"Work::point")
    partitions, native_partitions = rows(frozen,"Work::partition"), rows(native,"Work::partition")
    require(len(points) == len(native_points) == 4 and len(partitions) == 2 and len(native_partitions) == 3,
            "all compiled point/partition variants")
    point_delta = max(r["frame_bound"] for r in points.values())-min(r["frame_bound"] for r in native_points.values())
    partition_delta = max(0, max(r["frame_bound"] for r in partitions.values())-min(r["frame_bound"] for r in native_partitions.values()))
    caller_records = {**points, **partitions, **rows(frozen,"research_pinned_comparison")}
    all_transfers = {format(a,"x"):transfers(f) for a,f in frozen.items() if not f["name"].endswith("::conformance")}
    outgoing = [e for es in all_transfers.values() for e in es if e["kind"] == "outgoing_jump"]
    # For callers, a newly found outgoing jump invalidates this narrow result;
    # it must be resolved and charged, never silently treated as local flow.
    require(not outgoing, "all actual caller/kernel tail-jump edges closed")
    layout = json.loads((OLD/"scalar-oracle-staged.json").read_text())["layout"]
    extra = layout["workspace_bytes"]+layout["borrow_descriptor_bytes"]+point_delta+partition_delta+kernel_peak
    aligned = (extra+15)//16*16
    root = next(a for a,f in frozen.items() if f["name"].endswith("research_pinned_comparison"))
    caller_addresses = set(caller_records)
    caller_edges = {a:[e["target"] for e in transfers(frozen[a])
                       if e["kind"] == "call" and e["target"] in caller_addresses]
                    for a in caller_addresses}
    require(set(points).issubset(set(caller_edges[root])), "all point variants reachable from enclosing comparison")
    for a in points:
        require(sum(e["kind"] == "call" and e["target"] == solve for e in transfers(frozen[a])) == 1,
                "exactly one chart call per point variant")
    def prefix_peak(address, path=()):
        require(address not in path, "acyclic enclosing point callers")
        children = [prefix_peak(c, path+(address,)) for c in caller_edges[address]]
        if address in points:
            children.append((kernel_peak, ["chart kernel"]))
        child, child_path = max(children or [(0,[])], key=lambda v:v[0])
        return caller_records[address]["frame_bound"]+child, [format(address,"x"), *child_path]
    prefix, prefix_path = prefix_peak(root)
    root_code = frozen[root]["code"]
    require(root_code[0xa1f73] == "lea    0x39eb0(%rsp),%rax"
            and root_code[0xa1f7b] == "mov    %rax,0x1048(%rsp)", "one archived workspace borrowed by Work")
    require(root_code[0x97ebd] == "mov    $0xf1c0,%edx", "direct fixed scalar-array initialization")
    require(0x39eb0+layout["workspace_bytes"] <= caller_records[root]["fixed_decrement_bytes"], "workspace wholly inside enclosing fixed frame")
    baseline_prefix = prefix-extra
    require(baseline_prefix <= layout["baseline_step_stack_bytes"], "enclosing arithmetic prefix fits unchanged baseline stack reservation")
    return dict(status="PASS_FROZEN_KERNEL_AND_MEASURED_CALLER_BOUND", frozen_binary_sha256=binding["frozen_binary_sha256"],
                kernel_peak=kernel_peak, kernel_peak_path=path, kernel_records={format(a,"x"):dict(name=kernel[a]["name"],**r) for a,r in records.items()},
                libc_leaf_record=leaf_record, all_transfer_edges=all_transfers, outgoing_jump_edges=outgoing,
                caller_records={format(a,"x"):r for a,r in caller_records.items()}, native_point_records={format(a,"x"):r for a,r in native_points.items()},
                native_partition_records={format(a,"x"):r for a,r in native_partitions.items()},
                point_caller_delta=point_delta, partition_positive_delta=partition_delta, workspace_bytes=layout["workspace_bytes"],
                non_hook_source_sha256=source_records, caller_edges={format(a,"x"):[format(c,"x") for c in cs] for a,cs in caller_edges.items()},
                enclosing_arithmetic_stack_prefix=prefix, enclosing_arithmetic_prefix_path=prefix_path,
                baseline_arithmetic_prefix=baseline_prefix, baseline_prefix_slack=layout["baseline_step_stack_bytes"]-baseline_prefix,
                workspace_enclosing_frame_offset=0x39eb0, workspace_enclosing_end=0x39eb0+layout["workspace_bytes"],
                borrow_descriptor_bytes=layout["borrow_descriptor_bytes"], subtotal=extra, aligned_subtotal=aligned,
                cap=65536, cap_pass=aligned<=65536, remaining_bytes=65536-aligned,
                unchanged_baseline_reservation=layout["baseline_forced_nominal_bytes"], baseline_step_stack_reservation=layout["baseline_step_stack_bytes"],
                old_bound_plus_point_delta=64024+point_delta, old_ceiling_reservation_plus_point_delta=64152+point_delta,
                simultaneous_full_cap_reservation=layout["baseline_forced_nominal_bytes"]+65536,
                no_partition_shrink_credit=True, no_replaced_native_linear_frame_credit=True,
                numerical_execution_repeated=False, owner_advances=0,
                scope="Fixed successful/defined Result paths in this already-frozen x86_64 kernel, measured Work::point/partition frames, existing native baseline reservation. Host evidence output/unwind and public controller/third/lifecycle qualification remain outside this narrow audit; no retrospective pre-execution certificate.")


if __name__ == "__main__":
    print(json.dumps(run(), indent=2, sort_keys=True))
