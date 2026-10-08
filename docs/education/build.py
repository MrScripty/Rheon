"""Build an offline-capable GitHub Pages book. Requires Pandoc and npm ci.
Generated _site is disposable. Historical manuscript/artifacts are untouched.
"""
from pathlib import Path
import argparse, hashlib, html, json, os, re, shutil, subprocess, zipfile
from urllib.parse import urlsplit
from native_sequence import GUIDES, metadata, validate_labs, publish, validate_proof_qualification
from static_obstacle import validate_packet, publish as publish_obstacle
from obstacle_flow import validate_packet as validate_flow_packet, publish as publish_flow
from aligned_strain_packet import validate_packet as validate_strain_packet, publish as publish_strain
from aligned_strain_html import render_lab
from pdf_freshness import input_hashes
from markdown_bundle import write_bundle
from sphere_contact_lab import validate_packet as validate_contact_packet, publish as publish_contact
ROOT=Path(__file__).resolve().parents[2]; HERE=Path(__file__).resolve().parent
BOOK=ROOT/'docs/research-book'; OUT=HERE/'_site'
ASSETS=HERE/'node_modules'; NATIVE_LABS=None; OBSTACLE_RECORDS=None; FLOW_RECORDS=None; STRAIN_RECORDS=None; CONTACT_RECORDS=None; CURRENT_PROOF=None
FIGURES={'03':'staggered-grid.svg','04':'pressure-residual.svg','05':'multigrid-mechanism.svg','06':'interpolation-mass.svg','07':'transport-refinement.svg','09':'curvature-refinement.svg','10':'box-diffusion.svg','13':'memory-scaling.svg','15':'rounding-gap.svg','18':'cycle-circulation.svg','19':'expansion/projection.png','20':'expansion/mesh-hit.png','21':'expansion/density-viscosity.png','22':'expansion/density-viscosity.png','23':'expansion/slip-wetting.png','24':'expansion/slip-wetting.png'}

def command(args,**kwargs):
    result=subprocess.run(args,text=True,capture_output=True,**kwargs)
    if result.returncode:raise RuntimeError(result.stderr)
    return result.stdout

def validate_output(output,repo):
    output,repo=Path(output).resolve(),Path(repo).resolve()
    if output.is_relative_to(repo) or repo.is_relative_to(output):
        raise ValueError('Generated edition must be outside this checkout and its ancestors')
    if output.exists():raise ValueError('Output directory must be fresh; choose a new edition path.')
    probe=output.parent
    while not probe.exists():probe=probe.parent
    if subprocess.run(['git','-C',str(probe),'rev-parse','--is-inside-work-tree'],capture_output=True).returncode==0:
        raise ValueError('Generated edition must be outside every Git checkout')

def build():
    verify_reference(BOOK/'expansion')
    sequence=metadata(ROOT)
    validate_proof_qualification(sequence,ROOT,CURRENT_PROOF)
    validate_labs(sequence,NATIVE_LABS)
    validate_packet(ROOT,OBSTACLE_RECORDS)
    validate_flow_packet(ROOT,FLOW_RECORDS)
    validate_strain_packet(ROOT,STRAIN_RECORDS)
    validate_contact_packet(ROOT,CONTACT_RECORDS)
    pdf_inputs=input_hashes(ROOT)
    edition_commit=command(['git','rev-parse','HEAD'],cwd=ROOT).strip()
    source_base=sequence['current_reconstruction_proof']['source_head']
    if OUT.exists():raise ValueError('Output directory must be fresh; choose a new edition path.')
    OUT.mkdir(parents=True); (OUT/'chapters').mkdir(); (OUT/'downloads').mkdir(); (OUT/'proofs').mkdir(); (OUT/'implementation').mkdir()
    for name in ['style.css','labs.js']:shutil.copy2(HERE/name,OUT/name)
    vendor=OUT/'vendor'; vendor.mkdir()
    for name in ['three.module.js','three.core.js']:shutil.copy2(ASSETS/'three/build'/name,vendor/name)
    shutil.copy2(ASSETS/'three/examples/jsm/controls/OrbitControls.js',vendor/'OrbitControls.js')
    shutil.copy2(ASSETS/'three/LICENSE',vendor/'THREE-LICENSE.txt')
    shutil.copytree(ASSETS/'katex/dist',vendor/'katex',ignore=shutil.ignore_patterns('contrib'))
    shutil.copy2(ASSETS/'katex/LICENSE',vendor/'KATEX-LICENSE.txt')
    shutil.copytree(BOOK/'figures',OUT/'figures',ignore=shutil.ignore_patterns('*.png'))
    shutil.copytree(BOOK/'expansion/figures',OUT/'figures/expansion')
    if CONTACT_RECORDS is not None:
        (OUT/'figures/native').mkdir()
        for image in ['finite-static-sphere-contact.jpg','spherical-rigid-motion-example.jpg','mesh-traction-example.jpg']:
            shutil.copy2(CONTACT_RECORDS/image,OUT/'figures/native'/image)
    qualification=BOOK/'expansion/bounded-proof-qualification.json'
    if not qualification.exists():qualification=BOOK/'expansion/proof-qualification.json'
    shutil.copy2(qualification,OUT/'proof-qualification.json')
    if CURRENT_PROOF is not None:
        shutil.copytree(CURRENT_PROOF,OUT/'current-proof-qualification',ignore=shutil.ignore_patterns('proof-work','cache'))
    for name in ['reference-data.json','reference.py','reference-qualification.json','sources.json']:
        shutil.copy2(BOOK/'expansion'/name,OUT/name)
    chapter_files=sorted((BOOK/'chapters').glob('*.md'))+sorted((BOOK/'appendices').glob('*.md'))
    selected_guides=[BOOK/'implementation'/f'{slug}.md' for slug in GUIDES]
    files=chapter_files+selected_guides+[p for p in sorted((BOOK/'implementation').glob('*.md')) if p not in selected_guides]
    page_paths={p.resolve():('implementation/' if p.parent.name=='implementation' else 'chapters/')+p.stem+'.html' for p in files}
    manifest=[]; pages=[]; markdown=[]; linked_sources={}
    front='# Rheon — Discrete Fluid Simulation\n\nPuma · PR36 local review edition · 2026-10-08\n\nPinned research source: d31cf735645579357f9f58dcc55958e23f77af59. Original research, finite contracts and local mechanism references. See Appendix F and the learning path for assumptions, evidence and primary sources. One frictionless sphere impact then stop; no repeated/resting contact or fluid coupling.\n\n'
    for path in files:
        text=path.read_text(); title=text.splitlines()[0].removeprefix('# '); slug=path.stem
        fig=FIGURES.get(slug[:2])
        if path in chapter_files and fig:text+=f'\n\n![Original reference illustration for {title}](figures/{fig})\n'
        if CONTACT_RECORDS is not None and slug in ('static-sphere-contact','spherical-rigid-motion','triangle-mesh-traction'):
            image={'static-sphere-contact':'finite-static-sphere-contact.jpg','spherical-rigid-motion':'spherical-rigid-motion-example.jpg','triangle-mesh-traction':'mesh-traction-example.jpg'}[slug]
            caption={'static-sphere-contact':'Actual native face, edge, vertex and oblique-edge events. Blue: stored start; gold: stored post-impact end; red: contact normal. Wire sphere: declared collider; solid mesh: retained render/traction geometry. One frictionless impact with e=0.5, then stop.', 'spherical-rigid-motion':'Actual retained native mesh at 0, 0.25, 0.5 and 1 second under sampled surface traction, with declared spherical inertia. The orange material facets receive the load; red marks the COM path and velocity. No contact or fluid coupling.', 'triangle-mesh-traction':'Actual native consistent corner forces on the unit-area triangle: 0.25, 0.5 and 0.25 N in +z. Resultant force is (0,0,1) N and torque about the origin is (0.25,-1,0) N m. Static load reduction; no body or fluid step.'}[slug]
            text+=f'\n\n![{caption}](figures/native/{image})\n'
        markdown.append(text)
        body=command(['pandoc','-f','markdown+tex_math_single_backslash','-t','html5','--katex'],input=text)
        folder='implementation' if path.parent.name=='implementation' else 'chapters'
        def link(match):
            attribute=match[1]; target=html.unescape(match[2]); base,_,fragment=target.partition('#')
            if '://' in base or base.startswith('mailto:'):return match[0]
            source=(path.parent/base).resolve()
            if source in page_paths:
                destination=OUT/page_paths[source]
            elif source==HERE/'native-labs.html':
                destination=OUT/'native-labs.html'
            elif source==HERE/'aligned-strain-lab.html':
                destination=OUT/'aligned-strain-lab.html'
            elif source==HERE/'obstacle-flow-lab.html':
                destination=OUT/'obstacle-flow-lab.html'
            elif source==HERE/'obstacle-lab.html':
                destination=OUT/'obstacle-lab.html'
            elif source.suffix=='.lean' and source.parent==ROOT/'proofs/Rheon':
                destination=OUT/'proofs'/source.name
            elif source.is_relative_to(ROOT) and source.is_file() and source.relative_to(ROOT).parts[0] in ('docs','evidence','proofs','src','examples','tests','experiments','tools'):
                relative=source.relative_to(ROOT)
                destination=OUT/'source-files'/relative
                destination.parent.mkdir(parents=True,exist_ok=True)
                shutil.copy2(source,destination)
                linked_sources[str(relative)]=hashlib.sha256(source.read_bytes()).hexdigest()
            else:return match[0]
            result=os.path.relpath(destination,OUT/folder)
            return attribute+'="'+html.escape(result+('#'+fragment if fragment else ''),quote=True)+'"'
        body=re.sub(r'(href|src)="([^"]+)"',link,body)
        pages.append({'slug':slug,'title':title,'html':body,'folder':folder})
        manifest.append({'folder':folder,'slug':slug,'title':title,'source':str(path.relative_to(ROOT)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    payload=json.dumps(pages)
    node=r'''const katex=require('katex');let raw='';process.stdin.on('data',x=>raw+=x);process.stdin.on('end',()=>{const pages=JSON.parse(raw);let count=0;for(const p of pages)p.html=p.html.replace(/<span\s+class="math (inline|display)">([\s\S]*?)<\/span>/g,(_,kind,tex)=>{count++;tex=tex.replace(/&amp;/g,'&').replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/&#(\d+);/g,(_,n)=>String.fromCodePoint(+n));return katex.renderToString(tex,{displayMode:kind==='display',throwOnError:true,strict:'error',output:'htmlAndMathml'});});process.stdout.write(JSON.stringify({pages,count}));});'''
    rendered=json.loads(command(['node','-e',node],input=payload,cwd=ASSETS.parent)); pages=rendered['pages']
    def nav(prefix):return '<nav aria-label="Book chapters"><a href="'+prefix+'index.html">Overview</a><a href="'+prefix+'labs.html">3D laboratories</a><a href="'+prefix+'proofs.html">Proofs & evidence</a><a href="'+prefix+'native-labs.html">Native wall/force progression</a><a href="'+prefix+'obstacle-lab.html">Static obstacle geometry</a><a href="'+prefix+'obstacle-flow-lab.html">Obstacle pressure & shear</a><a href="'+prefix+'aligned-strain-lab.html">Interactive finite strain</a><a href="'+prefix+'sphere-contact-lab.html">Recorded sphere contact</a><input id="search" type="search" aria-label="Filter chapters" placeholder="Find a chapter…">'+''.join(f'<a class="chapter-link" href="{prefix}chapters/{p["slug"]}.html">{html.escape(p["title"])}</a>' for p in pages if p['folder']=='chapters')+'<h2>Learning path</h2>'+''.join(f'<a class="chapter-link" href="{prefix}implementation/{p["slug"]}.html">{html.escape(p["title"])}</a>' for p in pages if p['folder']=='implementation' and p['slug'] in GUIDES)+'<details><summary>Further implementation guides</summary>'+''.join(f'<a class="chapter-link" href="{prefix}implementation/{p["slug"]}.html">{html.escape(p["title"])}</a>' for p in pages if p['folder']=='implementation' and p['slug'] not in GUIDES)+'</details></nav>'
    def shell(title,body,prefix='',extra=''):
        return f'''<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{html.escape(title)} · Rheon</title><noscript><style>#menu,#search{{display:none!important}}@media(max-width:760px){{aside{{display:block!important;position:static!important;width:100%;max-height:320px}}}}</style></noscript><link rel="stylesheet" href="{prefix}style.css"><link rel="stylesheet" href="{prefix}vendor/katex/katex.min.css">{extra}</head><body><a class="skip" href="#main">Skip to content</a><header><a class="brand" href="{prefix}index.html">Rheon<span>Discrete Fluid Simulation</span></a><button id="menu" aria-expanded="false" aria-controls="navigation">Chapters</button><div class="header-links"><a href="{prefix}labs.html">Explore in 3D</a><a href="{prefix}downloads/Rheon-expanded-book.pdf">PDF</a><a href="{prefix}print.html">Print</a></div></header><aside id="navigation">{nav(prefix)}</aside><main id="main">{body}</main><footer>Puma · Research & teaching edition · Exact contracts and local references; see the evidence map.</footer><script>const b=document.querySelector('#menu');b.onclick=()=>{{const v=b.getAttribute('aria-expanded')!=='true';b.setAttribute('aria-expanded',v);document.querySelector('aside').classList.toggle('open',v)}};document.querySelector('#search').oninput=e=>document.querySelectorAll('.chapter-link').forEach(a=>a.hidden=!a.textContent.toLowerCase().includes(e.target.value.toLowerCase()));</script></body></html>'''
    for i,p in enumerate(pages):
        body=p['html'].replace('src="figures/','src="../figures/')
        body+='<div class="pager">'+(f'<a href="../{pages[i-1]['folder']}/{pages[i-1]['slug']}.html">← {html.escape(pages[i-1]["title"])}</a>' if i else '')+(f'<a href="../{pages[i+1]['folder']}/{pages[i+1]['slug']}.html">{html.escape(pages[i+1]["title"])} →</a>' if i+1<len(pages) else '')+'</div>'
        (OUT/p['folder']/f'{p["slug"]}.html').write_text(shell(p['title'],body,'../'))
    home='''<p class="eyebrow">Puma / RESEARCH + TEACHING EDITION</p><h1 class="hero-title">From discrete flow<br>to liquid contact.</h1><p class="lede">A fluid research book with exact mathematical contracts, six stored-reference 3D laboratories and a connected native wall/force progression.</p><div class="hero-actions"><a class="button" href="chapters/01-purpose-and-model.html">Read the book</a><a class="button secondary" href="labs.html">Explore the laboratories</a></div><figure class="hero-figure"><img src="figures/expansion/slip-wetting.png" alt="Original constant-volume cap profiles and Navier-slip Couette curves"><figcaption>Same liquid volume, different contact angles. Wall slip controls a different mechanism.</figcaption></figure><div class="cards"><article><span>01—18 / FOUNDATIONS</span><h2>Understand the structure</h2><p>MAC geometry, projection, transport, interface representation, precision and implementation contracts.</p><a href="chapters/03-staggered-grids.html">Start with the grid →</a></article><article><span>19—25 / EXPANSION</span><h2>Separate the physics</h2><p>Moving meshes, force work, liquid density, tensor viscosity, wetting energy, slip and capillarity.</p><a href="chapters/19-forces-and-moving-boundaries.html">Read the new chapters →</a></article><article><span>APPENDIX F / EVIDENCE</span><h2>Inspect each claim</h2><p>Inspect historical and current source-bound theorems, including planar clipping and derived finite-strain work, beside numerical references and remaining research gaps.</p><a href="proofs.html">Open proofs and evidence →</a></article></div><h2>Keep a reading copy</h2><p><a href="downloads/Rheon-expanded-book.pdf">Illustrated PDF</a> · <a href="downloads/Rheon-expanded-markdown.zip">Markdown + figures (ZIP)</a> · <a href="downloads/Rheon-expanded-book.md">Markdown text</a> · <a href="reference-data.json">Shared reference data</a></p>'''
    home+='<h2>From viscosity to one sphere impact</h2><p><a href="sphere-contact-lab.html">Inspect actual contact snapshots</a> · <a href="implementation/release-learning-path.html">Follow the complete learning path</a>: strain, an experimental viscosity/pressure step, mesh traction, impulses, spherical motion and finite-triangle contact. One frictionless impact then stop; no repeated/resting contact or fluid coupling.</p>'
    home+='<h2>From finite wall friction to forced no-slip</h2><p><a href="native-labs.html">Follow the three recorded native labs</a> · <a href="implementation/requirements-roadmap.html">Inspect remaining requirements and the next geometry contract</a></p>'
    home+='<h2>Pressure and reduced wall shear</h2><p><a href="obstacle-flow-lab.html">Inspect native field responses</a> · <a href="implementation/static-obstacle-flow.html">Bounded operator contract</a></p>'
    home+='<h2>One static obstacle geometry</h2><p><a href="obstacle-lab.html">Inspect native geometry controls</a> · <a href="implementation/static-obstacle-geometry.html">Shared volumes, openings, connectivity and collision source</a></p>'
    home+='<h2>From strain rows to resisting force</h2><p><a href="aligned-strain-lab.html">Compute strain and dissipation interactively</a> · <a href="implementation/aligned-strain-laboratory.html">Read the specimen and exact algebra</a>. This bounded laboratory recomputes finite rows for selected face velocities; it advances no fluid.</p>'
    (OUT/'index.html').write_text(shell('Overview',home))
    (OUT/'sphere-contact-lab.html').write_text(shell('Recorded sphere contact',publish_contact(ROOT,OUT,CONTACT_RECORDS)))
    (OUT/'native-labs.html').write_text(shell('Native wall and force progression',publish(sequence,OUT,NATIVE_LABS)))
    obstacle_body=publish_obstacle(ROOT,OUT,OBSTACLE_RECORDS)
    obstacle_extra='<script type="importmap">{"imports":{"three":"./vendor/three.module.js","three/addons/controls/OrbitControls.js":"./vendor/OrbitControls.js"}}</script><script type="module" src="obstacle.js"></script>' if OBSTACLE_RECORDS is not None else ''
    (OUT/'obstacle-lab.html').write_text(shell('Shared static obstacle geometry',obstacle_body,extra=obstacle_extra))
    flow_body=publish_flow(ROOT,OUT,FLOW_RECORDS)
    (OUT/'obstacle-flow-lab.html').write_text(shell('Obstacle pressure and reduced shear',flow_body,extra='<script type="module" src="obstacle_flow.js"></script>' if FLOW_RECORDS is not None else ''))
    strain_data=publish_strain(ROOT,OUT,STRAIN_RECORDS)
    for name in ['aligned_strain.css','aligned_strain.js']:shutil.copy2(HERE/name,OUT/name)
    (OUT/'aligned-strain-lab.html').write_text(shell('Interactive finite aligned strain',render_lab(strain_data),extra='<link rel="stylesheet" href="aligned_strain.css"><script type="module" src="aligned_strain.js"></script>' if strain_data is not None else ''))
    labs='''<p class="eyebrow">SIX PROGRESSIVE LABORATORIES</p><h1>Explore the mechanism.</h1><p class="lede">Rotate the scene, choose a reference state, and inspect the numbers behind it. Each lab uses the same deterministic data as the book figures.</p><p role="note"><strong>Stored-reference exploration.</strong> These controls select recorded analytical or dense-solve reference states. The projection slider recomputes a displayed algebraic blend of two stored fields. No control advances a live fluid or contact-line simulation.</p><p><a href="native-labs.html">Continue to the three native wall/force labs →</a></p><p><a href="aligned-strain-lab.html">Compute finite strain, viscous force and dissipation →</a></p><label for="lab">Laboratory</label><select id="lab"><option value="projection">1 · MAC pressure projection</option><option value="collision">2 · Triangle mesh and earliest collision</option><option value="hydrostatic">3 · Layered density and hydrostatic pressure</option><option value="viscous">4 · Implicit viscous shear decay</option><option value="slip">5 · Navier slip and wall traction</option><option value="cap">6 · Constant-volume wetting and capillarity</option></select><section class="laboratory"><div id="scene" tabindex="0" aria-label="Interactive three-dimensional reference scene. Drag to rotate; scroll to zoom."></div><div class="lab-panel"><h2 id="lab-title"></h2><p id="lab-description"></p><div id="controls"></div><dl id="metrics" aria-live="polite"></dl><p id="lab-limit"></p><button id="reset-view">Reset camera</button><a id="chapter-target" href="chapters/04-pressure-projection.html">Read the chapter →</a></div></section><noscript><p>The interactive controls require JavaScript. Read the chapter links in the navigation and download the PDF or Markdown for equations, figures and limitations.</p></noscript><p id="render-status" role="status">Loading local 3D assets…</p><p>Reference implementation: <a href="reference.py">original Python source</a> · <a href="reference-data.json">complete data</a> · <a href="reference-qualification.json">numerical receipt</a>. Analytic caps and shear modes are not a general liquid simulation.</p>'''
    extra='<script type="importmap">{"imports":{"three":"./vendor/three.module.js","three/addons/controls/OrbitControls.js":"./vendor/OrbitControls.js"}}</script><script type="module" src="labs.js"></script>'
    (OUT/'labs.html').write_text(shell('3D laboratories',labs,extra=extra))
    receipt=json.loads(qualification.read_text()) if qualification.exists() else {'status':'pending','reason':'Pinned project build and axiom audit must complete.'}
    proofs='<p class="eyebrow">CHECK THE HYPOTHESES</p><h1>Proofs & evidence</h1><p>Finite exact algebra, local numerical references and production implementation are distinct evidence levels. The proof inventory does not verify mesh assembly, a liquid solver or floating-point behavior.</p><h2>Historical expansion qualification receipt</h2><pre>'+html.escape(json.dumps(receipt,indent=2))+'</pre><p><a href="chapters/F-expansion-contracts-and-sources.html">Full contract map and primary sources →</a></p>'
    proofs+=f'<h2>Current pinned source and separate historical qualification</h2><p>The pinned reconstruction has {sequence["public_theorems"]} public theorems, {sequence["explicit_expected_declarations"]} explicitly expected declarations and {sequence["audited_declarations"]} actually audited declarations. Its fresh local pinned normal lake source build, allowed-axiom audit and rejection/source-policy probes passed. Current proof bytes match the reviewed inventory. The earlier PR24 57/77 qualification and recorded native lab identities remain historical. This book build checks source bindings; it does not rerun Lean or prove Rust assembly, IEEE arithmetic, approximate pressure solves or global convergence.</p><p><a href="{sequence["lean_ci"]}">Historical reconstruction Lean CI (not PR36)</a> · <a href="native-sequence.json">Pinned source, local receipt and historical bindings</a></p>'
    if CURRENT_PROOF is not None:
        proofs+='<p><a href="current-proof-qualification/qualification.json">Fresh PR36 local proof receipt</a> · <a href="current-proof-qualification/lean-audit.log">Actual transitive axiom audit</a>. No hosted PR36 CI was requested.</p>'
    proofs+=f'<p><strong>Historical expansion: {receipt.get("public_theorems", "Pending")} public theorems · {receipt.get("audited_declarations", "Pending")} audited declarations.</strong> Definitions and generated proof helpers are counted separately from public theorems.</p>'
    proofs+='''<div class="cards"><article><h2>Planar first contact</h2><p>Five theorems derive a strict crossing, first hit and permitted clipped segment for an infinite stationary plane. Finite-facet containment and earliest mesh queries remain outside the proof.</p><a href="chapters/20-collision-mesh-pipeline.html">Read the collision contract</a></article><article><h2>Derived viscous work</h2><p>Four theorems derive dissipation and energy nonincrease from fixed finite strain and exact backward-Euler equations. Stencil assembly, forcing and approximate solves remain outside the proof.</p><a href="chapters/22-viscosity-and-stress.html">Read the viscosity bridge</a></article></div>'''
    for path in sorted((ROOT/'proofs/Rheon').glob('*.lean')):
        shutil.copy2(path,OUT/'proofs'/path.name)
        proofs+=f'<details><summary>{path.name} — source and hypotheses</summary><p><a href="proofs/{path.name}">Download Lean source</a></p><pre><code>{html.escape(path.read_text())}</code></pre></details>'
    (OUT/'proofs.html').write_text(shell('Proofs and evidence',proofs))
    reading=[p for p in pages if p['folder']=='chapters']+[next(p for p in pages if p['folder']=='implementation' and p['slug']==slug) for slug in GUIDES]
    text_by_source={path.resolve():text for path,text in zip(files,markdown)}
    reading_markdown=[text_by_source[(ROOT/next(m['source'] for m in manifest if m['folder']==p['folder'] and m['slug']==p['slug'])).resolve()] for p in reading]
    reading_paths=[ROOT/next(m['source'] for m in manifest if m['folder']==p['folder'] and m['slug']==p['slug']) for p in reading]
    write_bundle(ROOT,OUT,front,list(zip(reading_paths,reading_markdown)),files,command)
    # A self-contained print edition: all math is already rendered locally.
    toc='<h1>Contents</h1><ol>'+''.join(f'<li><a href="#{p["slug"]}">{html.escape(p["title"])}</a></li>' for p in reading)+'</ol>'
    cover='<section class="cover"><p>PUMA / EXPANDED RESEARCH EDITION</p><h1>Rheon</h1><h2>Discrete Fluid Simulation</h2><p>Solids, liquids and contact</p><img src="figures/expansion/slip-wetting.png" alt="Original reference profiles"><p>Mathematics · Checked contracts · Numerical references · 3D teaching laboratories</p><p>8 October 2026 · Local review candidate</p></section>'
    def print_body(p):
        def root_link(match):
            value=match[2]
            if '://' in value or value.startswith(('#','mailto:','data:')):return match[0]
            if match[1]=='src' and value.startswith('figures/'):return match[0]
            url=urlsplit(html.unescape(value))
            absolute=os.path.normpath(os.path.join(p['folder'],url.path))
            if match[1]=='href':
                present=next((item for item in reading if absolute==item['folder']+'/'+item['slug']+'.html'),None)
                if present:return 'href="#'+present['slug']+'"'
                original=next((source for source,dest in page_paths.items() if dest==absolute),None)
                if original is not None:relative=str(original.relative_to(ROOT))
                elif absolute.startswith('proofs/'):relative='proofs/Rheon/'+Path(absolute).name
                elif absolute.startswith('source-files/'):relative=absolute.removeprefix('source-files/')
                elif absolute=='native-labs.html':relative='docs/education/README.md'
                elif absolute=='aligned-strain-lab.html':relative='docs/education/README.md'
                elif absolute=='obstacle-flow-lab.html':relative='docs/education/README.md'
                elif absolute=='obstacle-lab.html':relative='docs/education/README.md'
                else:raise ValueError('Unmapped portable PDF link: '+absolute)
                absolute='https://github.com/MrScripty/Rheon/blob/'+source_base+'/'+relative+('#'+url.fragment if url.fragment else '')
            return match[1]+'="'+absolute+'"'
        return re.sub(r'(href|src)="([^"]+)"',root_link,p['html'])
    contents=cover+'<section class="toc">'+toc+'</section>'+''.join(f'<section class="book-chapter" id="{p["slug"]}">{print_body(p)}</section>' for p in reading)
    (OUT/'print.html').write_text(f'<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Rheon — Expanded research edition</title><link rel="stylesheet" href="style.css"><link rel="stylesheet" href="vendor/katex/katex.min.css"></head><body class="print-book">{contents}</body></html>')
    if OUT==HERE/'_site' and (HERE/'downloads/Rheon-expanded-book.pdf').exists():shutil.copy2(HERE/'downloads/Rheon-expanded-book.pdf',OUT/'downloads/Rheon-expanded-book.pdf')
    build_receipt={'schema':'rheon-education-build-v1','chapters':len(chapter_files),'implementation_guides':len(files)-len(chapter_files),'native_bundles_included':NATIVE_LABS is not None,'obstacle_records_included':OBSTACLE_RECORDS is not None,'obstacle_flow_records_included':FLOW_RECORDS is not None,'sphere_contact_packet_included':CONTACT_RECORDS is not None,'sphere_contact_packet_sha256':hashlib.sha256((CONTACT_RECORDS/'packet-provenance.json').read_bytes()).hexdigest() if CONTACT_RECORDS is not None else None,'reading_order':[p['slug'] for p in reading],'aligned_strain_packet_included':STRAIN_RECORDS is not None,'proof_inventory_sha256':sequence['proof_inventory_sha256'],'rendered_math_expressions':rendered['count'],'sources':manifest,'linked_source_files':linked_sources,'reference_data_sha256':hashlib.sha256((OUT/'reference-data.json').read_bytes()).hexdigest(),'pandoc':command(['pandoc','--version']).splitlines()[0],'source_base':source_base,'edition_commit':edition_commit,'historical_proof_status':receipt.get('status'),'current_proof_status':'reviewed-current-source-inventory-matched'}
    if input_hashes(ROOT)!=pdf_inputs:raise RuntimeError('PDF inputs changed during HTML build; rebuild.')
    build_receipt['pdf_inputs']=pdf_inputs
    (OUT/'build-receipt.json').write_text(json.dumps(build_receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in build_receipt.items() if k not in ['sources','pdf_inputs']},indent=2))
def verify_reference(reference):
    qualification=json.loads((reference/'reference-qualification.json').read_text())
    for name,key in [('reference.py','source_sha256'),('reference-data.json','data_sha256')]:
        digest=hashlib.sha256((reference/name).read_bytes()).hexdigest()
        if digest!=qualification.get(key):
            raise ValueError(f'{name} does not match reference qualification; regenerate and qualify before publication.')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir',required=True,type=Path)
    parser.add_argument('--asset-dir',type=Path,default=ASSETS)
    parser.add_argument('--native-labs-dir',type=Path)
    parser.add_argument('--obstacle-records-dir',type=Path)
    parser.add_argument('--obstacle-flow-records-dir',type=Path)
    parser.add_argument('--aligned-strain-records-dir',type=Path)
    parser.add_argument('--sphere-contact-records-dir',type=Path)
    parser.add_argument('--current-proof-qualification-dir',type=Path)
    args=parser.parse_args(); OUT=args.output_dir.resolve(); ASSETS=args.asset_dir.resolve(); NATIVE_LABS=args.native_labs_dir; OBSTACLE_RECORDS=args.obstacle_records_dir; FLOW_RECORDS=args.obstacle_flow_records_dir; STRAIN_RECORDS=args.aligned_strain_records_dir; CONTACT_RECORDS=args.sphere_contact_records_dir; CURRENT_PROOF=args.current_proof_qualification_dir
    validate_output(OUT,ROOT)
    build()
