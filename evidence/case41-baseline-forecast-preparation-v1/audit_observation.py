"""Fresh-ELF fixed-frame/known-boundary observations; explicitly not a memory proof."""
from pathlib import Path
import gzip,hashlib,importlib.util,json,re,subprocess
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
    if not ok:raise ValueError(msg)
def run():
    b=json.loads((P/'compile-binding.json').read_text());require(hashlib.sha256(Path(b['binary']).read_bytes()).hexdigest()==b['binary_sha256'],'exact new ELF')
    raw=subprocess.check_output(['objdump','-d','-C',b['binary']]);require(raw==gzip.decompress((P/'native-disassembly.txt.gz').read_bytes()),'lossless new ELF instructions')
    spec=importlib.util.spec_from_file_location('fresh_frame_reader',ROOT/'evidence/forcing-increment-memory-audit/audit.py');h=importlib.util.module_from_spec(spec);spec.loader.exec_module(h)
    selected=''.join(part for part in re.split(r'(?m)(?=^[a-f0-9]+ <)',raw.decode()) if part.splitlines() and 'rheon::' in part.splitlines()[0])
    fs=h.functions(selected,(P/'native-symbols.txt').read_text());symbols=(P/'native-symbols.txt').read_text();relocs=(P/'native-relocations.txt').read_text()
    aliases={}
    for line in symbols.splitlines():
        m=re.match(r'^([a-f0-9]+) [a-f0-9]+ [tT] (.+)$',line)
        if m:aliases.setdefault(int(m[1],16),[]).append(m[2])
    got={}
    for line in relocs.splitlines():
        m=re.match(r'^([a-f0-9]+) .*R_X86_64_RELATIVE\s+([a-f0-9]+)$',line)
        if m:got[int(m[1],16)]={'target':int(m[2],16),'names':aliases.get(int(m[2],16),[])}
        m=re.match(r'^([a-f0-9]+) .*R_X86_64_(?:GLOB_DAT|JUMP_SLOT)\s+\S+\s+(\S+)',line)
        if m:got[int(m[1],16)]={'target':None,'names':[m[2]]}
    suffixes=['candidate::case41_forecast_capture','candidate::Work::forecast_serialize','candidate::Work::case41_paired_equation','candidate::Work::equation','candidate::Work::point','case41_baseline_and_frozen_forecast']
    frames={}
    for suffix in suffixes:
        matches=[(addr,f) for addr,f in fs.items() if f['name'].endswith('::'+suffix)]
        if not matches and suffix=='candidate::Work::equation':
            frames[suffix]={'standalone_symbol':False,'folded_numerical_body_in':'candidate::Work::case41_paired_equation','shrink_credit':False}
            continue
        require(len(matches)==1,'one fresh function '+suffix)
        addr,f=matches[0];frames[suffix]=dict(address=addr,name=f['name'],**h.fixed_frame(f))
    root=frames['candidate::Work::case41_paired_equation']['address'];pending=[root];seen=set();transfers=[]
    while pending:
        addr=pending.pop()
        if addr in seen:continue
        seen.add(addr)
        for edge in h.transfers(fs[addr]):
            if edge['kind']=='internal_branch':continue
            target=edge['target'];names=[]
            if target is None:
                m=re.search(r'# ([a-f0-9]+) ',edge['instruction']);binding=got.get(int(m[1],16)) if m else None
                if binding:target=binding['target'];names=binding['names']
            if target in fs:pending.append(target);names=names+[fs[target]['name']]
            transfers.append(dict(caller=fs[addr]['name'],instruction=edge['instruction'],resolved_names=names,known_crate=target in fs))
    require(not any('forecast_serialize' in fs[a]['name'] for a in seen),'serializer outside known numerical crate graph')
    require(not any('std::io::stdio::_print' in ' '.join(e['resolved_names']) for e in transfers),'no resolved stdout edge in numerical graph')
    unresolved=[e for e in transfers if not e['known_crate']]
    return dict(status='INCOMPLETE_STATIC_RESOURCE_OBSERVATIONS',binary_sha256=b['binary_sha256'],fixed_frames=frames,known_reached_crate_functions=len(seen),known_serializer_outside_boundary=True,known_stdout_outside_boundary=True,external_or_unresolved_transfers=unresolved,external_or_unresolved_count=len(unresolved),complete_memory_bound=None,allocator_library_IO_bound=None,unknown_costs_zeroed=False,historical_memory_qualified=False,native_equations=0,scope='Only fixed frames and resolved crate/GOT transfers. Register-indirect callbacks, libraries, recursive/sort routes and allocation internal costs remain unqualified; operational child limits are a separate protocol.')
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
