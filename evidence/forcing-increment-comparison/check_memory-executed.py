"""Bound additional live workspace using actual Rust layout and linked x86 code.

The ELF .stack_sizes section was unavailable on this stable compiler/link path;
this reader instead conservatively counts every fixed stack decrement/push,
128 bytes of red-zone allowance per function and eight return-address bytes.
The mathematical kernel is acyclic. Panic bounds paths are unreachable for the
private normalized scalar/fixed loop invariants; no successful math path uses
allocation, recursion, dynamic stack sizing or an unbounded external call.
"""
from pathlib import Path
import hashlib,json,re,subprocess,sys
P=Path(__file__).resolve().parent
PREFIX='rheon::research_increment_comparison::scalar::'
def require(ok,message):
    if not ok:raise ValueError(message)
def instructions(body):
    out=[]
    for line in body.splitlines():
        pieces=line.split('\t')
        if len(pieces)>=3 and re.match(r'^\s*[a-f0-9]+:$',pieces[0]):
            asm=pieces[-1].strip()
            if asm:out.append(asm)
    return out
def run():
    binary=Path('/workspace/.rheon-tools/increment-comparison-target/release/deps/rheon-6fcb0e06fe041f73')
    require(binary.is_file(),'exact linked preflight binary exists')
    source=json.loads((P/'prototype-policy.json').read_text())
    for name,sha in source['source_sha256'].items():require(hashlib.sha256(Path(name).read_bytes()).hexdigest()==sha,'frozen prototype source before E1')
    scalar=json.loads((P/'scalar-oracle-linked.json').read_text())
    require(scalar['status']=='PASS_SCALAR_ORACLE_AND_LAYOUT_ONLY','actual independent scalar preflight')
    raw=(P/'native-disassembly.txt').read_text()
    funcs={}
    for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw):
        first=part.splitlines()[0] if part.splitlines() else ''
        match=re.match(r'^([a-f0-9]+) <(.+)>:$',first)
        if match:funcs[match[2]]=dict(address=match[1],body=part)
    names=[PREFIX+'ChartWorkspace::solve',PREFIX+'ChartWorkspace::factor']+[PREFIX+'Scalar::'+x for x in ('add','mul','div','to_f64','from_f64')]
    records={}
    for name in names:
        require(name in funcs,'all compiled kernel functions observed')
        item=funcs[name];code=instructions(item['body']);pushes=0;decrements=0;calls=[]
        for asm in code:
            if asm.startswith('push'):pushes+=8
            if '%rsp' in asm and re.match(r'^sub\s',asm):
                m=re.match(r'^sub\s+\$0x([0-9a-f]+),%rsp$',asm)
                require(m is not None,'fixed stack decrement only')
                decrements+=int(m[1],16)
            require(not ('%rsp' in asm and re.match(r'^(and|mov|lea|enter|alloca)\s.*,%rsp',asm)),'no dynamic stack sizing')
            if asm.startswith('call'):
                if '<' in asm:
                    target=asm.split('<',1)[1].rsplit('>',1)[0]
                    if target in names:calls.append(target)
                    elif target.startswith('memset@'):calls.append('memset')
                    elif '_DYNAMIC' in target:
                        # Actual relocation resolves this sole bounds-check GOT
                        # slot; source-fixed normalized mantissas/indices exclude it.
                        require('# 1ce690 ' in asm,'only the bound panic slot allowed')
                    else:require(False,'unexpected additional kernel callee')
                else:require(False,'unresolved additional kernel callee')
        records[name]=dict(address=item['address'],push_bytes=pushes,fixed_stack_decrement_bytes=decrements,
                           red_zone_allowance_bytes=128,return_address_bytes=8,frame_bound=pushes+decrements+136,
                           callees=sorted(set(calls)))
    symbols=(P/'native-symbols.txt').read_text()
    require('00000000001b46ad T core::panicking::panic_bounds_check' in symbols,'panic slot target identity')
    reloc=subprocess.check_output(['readelf','-rW',str(binary)]).decode()
    require(re.search(r'^00000000001ce690 .*R_X86_64_RELATIVE\s+1b46ad$',reloc,re.M),'bounds GOT relocation')
    # Follow only instructions reachable from the actual default IFUNC entry.
    # Adjacent fortified wrapper entries are not part of this function.
    code={}
    for line in (P/'memset-leaf-disassembly.txt').read_text().splitlines():
        pieces=line.split('\t')
        if len(pieces)>=3 and re.match(r'^\s*[a-f0-9]+:$',pieces[0]):
            code[int(pieces[0].strip()[:-1],16)]=pieces[-1].strip()
    addresses=sorted(code);following=dict(zip(addresses,addresses[1:]))
    entry=json.loads((P/'memset-binding.json').read_text())['resolved_file_address']
    pending=[entry];visited=set();branch_addresses=[]
    while pending:
        address=pending.pop()
        if address in visited:continue
        require(address in code,'memset reachable instruction is captured')
        visited.add(address);asm=code[address]
        require(not asm.startswith(('push','call','enter')) and '%rsp' not in asm,'reachable libc memset is a leaf without stack use')
        if asm.startswith('ret'):continue
        unconditional=False
        if re.match(r'^j\S*\s',asm):
            match=re.match(r'^j\S*\s+([a-f0-9]+)\s',asm)
            require(match is not None,'fixed memset branch target')
            target=int(match[1],16);require(target in code,'memset reachable branch span closed')
            branch_addresses.append(target);pending.append(target)
            unconditional=asm.startswith('jmp')
        if not unconditional:
            require(address in following,'memset reachable fallthrough captured')
            pending.append(following[address])
    require(len(visited)>50,'full resolved memset branch graph checked')
    def peak(name,path=()):
        if name=='memset':return 136
        require(name not in path,'no recursive additional kernel')
        row=records[name]
        return row['frame_bound']+max([peak(c,path+(name,)) for c in row['callees']] or [0])
    maximum=peak(PREFIX+'ChartWorkspace::solve')
    require(maximum<=2048,'compiled simultaneous additional kernel within frozen ceiling')
    layout=scalar['layout'];extra=layout['workspace_bytes']+layout['borrow_descriptor_bytes']+maximum
    require(extra<=layout['additional_bytes_with_ceiling']<=65536,'compiled additional workspace bound')
    # The fixed accepted extraction resides inside ChartWorkspace. Old/current
    # points, baseline Work/geometry and matrices retain their existing budgets.
    return dict(status='PASS_SCALAR_AND_SIMULTANEOUS_WORKSPACE_PREFLIGHT',policy_id=source['policy_id'],
        prototype='5270e91c1ee1e14b936d66d750ce8a631c078cfb',prototype_policy_sha256=hashlib.sha256((P/'prototype-policy.json').read_bytes()).hexdigest(),
        binary=str(binary),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),compiled_kernel_stack_bound=maximum,
        workspace_bytes=layout['workspace_bytes'],borrow_descriptor_bytes=layout['borrow_descriptor_bytes'],
        compiled_additional_live_bound=extra,reserved_additional_bytes=layout['additional_bytes_with_ceiling'],
        baseline_forced_nominal_bytes=layout['baseline_forced_nominal_bytes'],combined_reservation_bytes=scalar['combined_baseline_and_additional_reservation_bytes'],
        kernel_functions=records,memset_branch_targets=sorted(set(branch_addresses)),memset_reachable_instructions=len(visited),
        libc_sha256=hashlib.sha256(Path(json.loads((P/'memset-binding.json').read_text())['library']).read_bytes()).hexdigest(),
        proof_scope='this compiled x86_64 binary and default libc CPU variant; fixed successful arithmetic/defined Result failure paths. Panic on impossible private invariants is not a bounded return. Logging/oracle are host evidence, outside numerical workspace.',
        e1_permitted=True,owner_advances=0,e1_already_executed=False)
if __name__=='__main__':
    try:print(json.dumps(run(),indent=2,sort_keys=True))
    except Exception as exc:
        print(json.dumps(dict(status='MEMORY_PREFLIGHT_BLOCKED',error=str(exc),e1_permitted=False)))
        raise
