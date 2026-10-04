"""Build an offline-capable GitHub Pages book. Requires Pandoc and npm ci.
Generated _site is disposable. Historical manuscript/artifacts are untouched.
"""
from pathlib import Path
import hashlib, html, json, re, shutil, subprocess, zipfile
from pdf_freshness import input_hashes
ROOT=Path(__file__).resolve().parents[2]; HERE=Path(__file__).resolve().parent
BOOK=ROOT/'docs/research-book'; OUT=HERE/'_site'
FIGURES={'03':'staggered-grid.svg','04':'pressure-residual.svg','05':'multigrid-mechanism.svg','06':'interpolation-mass.svg','07':'transport-refinement.svg','09':'curvature-refinement.svg','10':'box-diffusion.svg','13':'memory-scaling.svg','15':'rounding-gap.svg','18':'cycle-circulation.svg','19':'expansion/projection.png','20':'expansion/mesh-hit.png','21':'expansion/density-viscosity.png','22':'expansion/density-viscosity.png','23':'expansion/slip-wetting.png','24':'expansion/slip-wetting.png'}

def command(args,**kwargs):
    result=subprocess.run(args,text=True,capture_output=True,**kwargs)
    if result.returncode:raise RuntimeError(result.stderr)
    return result.stdout

def build():
    pdf_inputs=input_hashes(ROOT)
    if OUT.exists():shutil.rmtree(OUT)
    OUT.mkdir(); (OUT/'chapters').mkdir(); (OUT/'downloads').mkdir(); (OUT/'proofs').mkdir()
    for name in ['style.css','labs.js']:shutil.copy2(HERE/name,OUT/name)
    vendor=OUT/'vendor'; vendor.mkdir()
    for name in ['three.module.js','three.core.js']:shutil.copy2(HERE/'node_modules/three/build'/name,vendor/name)
    shutil.copy2(HERE/'node_modules/three/examples/jsm/controls/OrbitControls.js',vendor/'OrbitControls.js')
    shutil.copy2(HERE/'node_modules/three/LICENSE',vendor/'THREE-LICENSE.txt')
    shutil.copytree(HERE/'node_modules/katex/dist',vendor/'katex',ignore=shutil.ignore_patterns('contrib'))
    shutil.copy2(HERE/'node_modules/katex/LICENSE',vendor/'KATEX-LICENSE.txt')
    shutil.copytree(BOOK/'figures',OUT/'figures',ignore=shutil.ignore_patterns('*.png'))
    shutil.copytree(BOOK/'expansion/figures',OUT/'figures/expansion')
    qualification=BOOK/'expansion/bounded-proof-qualification.json'
    if not qualification.exists():qualification=BOOK/'expansion/proof-qualification.json'
    shutil.copy2(qualification,OUT/'proof-qualification.json')
    for name in ['reference-data.json','reference.py','reference-qualification.json','sources.json']:
        shutil.copy2(BOOK/'expansion'/name,OUT/name)
    files=sorted((BOOK/'chapters').glob('*.md'))+sorted((BOOK/'appendices').glob('*.md'))
    manifest=[]; pages=[]; markdown=[]
    front='# Rheon — Discrete Fluid Simulation\n\nPuma · Expanded solids and liquids teaching edition · 2026-10-04\n\nOriginal research, finite contracts and local mechanism references. See Appendix F for assumptions, evidence and primary sources.\n\n'
    for path in files:
        text=path.read_text(); title=text.splitlines()[0].removeprefix('# '); slug=path.stem
        fig=FIGURES.get(slug[:2])
        if fig:text+=f'\n\n![Original reference illustration for {title}](figures/{fig})\n'
        markdown.append(text)
        body=command(['pandoc','-f','markdown+tex_math_single_backslash','-t','html5','--katex'],input=text)
        pages.append({'slug':slug,'title':title,'html':body})
        manifest.append({'slug':slug,'title':title,'source':str(path.relative_to(ROOT)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    payload=json.dumps(pages)
    node=r'''const katex=require('katex');let raw='';process.stdin.on('data',x=>raw+=x);process.stdin.on('end',()=>{const pages=JSON.parse(raw);let count=0;for(const p of pages)p.html=p.html.replace(/<span\s+class="math (inline|display)">([\s\S]*?)<\/span>/g,(_,kind,tex)=>{count++;tex=tex.replace(/&amp;/g,'&').replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/&#(\d+);/g,(_,n)=>String.fromCodePoint(+n));return katex.renderToString(tex,{displayMode:kind==='display',throwOnError:true,strict:'error',output:'htmlAndMathml'});});process.stdout.write(JSON.stringify({pages,count}));});'''
    rendered=json.loads(command(['node','-e',node],input=payload,cwd=HERE)); pages=rendered['pages']
    def nav(prefix):return '<nav aria-label="Book chapters"><a href="'+prefix+'index.html">Overview</a><a href="'+prefix+'labs.html">3D laboratories</a><a href="'+prefix+'proofs.html">Proofs & evidence</a><input id="search" type="search" aria-label="Filter chapters" placeholder="Find a chapter…">'+''.join(f'<a class="chapter-link" href="{prefix}chapters/{p["slug"]}.html">{html.escape(p["title"])}</a>' for p in pages)+'</nav>'
    def shell(title,body,prefix='',extra=''):
        return f'''<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{html.escape(title)} · Rheon</title><link rel="stylesheet" href="{prefix}style.css"><link rel="stylesheet" href="{prefix}vendor/katex/katex.min.css">{extra}</head><body><a class="skip" href="#main">Skip to content</a><header><a class="brand" href="{prefix}index.html">Rheon<span>Discrete Fluid Simulation</span></a><button id="menu" aria-expanded="false" aria-controls="navigation">Chapters</button><div class="header-links"><a href="{prefix}labs.html">Explore in 3D</a><a href="{prefix}downloads/Rheon-expanded-book.pdf">PDF</a></div></header><aside id="navigation">{nav(prefix)}</aside><main id="main">{body}</main><footer>Puma · Research & teaching edition · Exact contracts and local references; see the evidence map.</footer><script>const b=document.querySelector('#menu');b.onclick=()=>{{const v=b.getAttribute('aria-expanded')!=='true';b.setAttribute('aria-expanded',v);document.querySelector('aside').classList.toggle('open',v)}};document.querySelector('#search').oninput=e=>document.querySelectorAll('.chapter-link').forEach(a=>a.hidden=!a.textContent.toLowerCase().includes(e.target.value.toLowerCase()));</script></body></html>'''
    for i,p in enumerate(pages):
        body=p['html'].replace('src="figures/','src="../figures/')
        body+='<div class="pager">'+(f'<a href="{pages[i-1]["slug"]}.html">← {html.escape(pages[i-1]["title"])}</a>' if i else '')+(f'<a href="{pages[i+1]["slug"]}.html">{html.escape(pages[i+1]["title"])} →</a>' if i+1<len(pages) else '')+'</div>'
        (OUT/'chapters'/f'{p["slug"]}.html').write_text(shell(p['title'],body,'../'))
    home='''<p class="eyebrow">Puma / RESEARCH + TEACHING EDITION</p><h1 class="hero-title">From discrete flow<br>to liquid contact.</h1><p class="lede">A complete fluid research book with exact mathematical contracts, original numerical references, and six interactive 3D laboratories.</p><div class="hero-actions"><a class="button" href="chapters/01-purpose-and-model.html">Read the book</a><a class="button secondary" href="labs.html">Explore the laboratories</a></div><figure class="hero-figure"><img src="figures/expansion/slip-wetting.png" alt="Original constant-volume cap profiles and Navier-slip Couette curves"><figcaption>Same liquid volume, different contact angles. Wall slip controls a different mechanism.</figcaption></figure><div class="cards"><article><span>01—18 / FOUNDATIONS</span><h2>Understand the structure</h2><p>MAC geometry, projection, transport, interface representation, precision and implementation contracts.</p><a href="chapters/03-staggered-grids.html">Start with the grid →</a></article><article><span>19—25 / EXPANSION</span><h2>Separate the physics</h2><p>Moving meshes, force work, liquid density, tensor viscosity, wetting energy, slip and capillarity.</p><a href="chapters/19-forces-and-moving-boundaries.html">Read the new chapters →</a></article><article><span>APPENDIX F / EVIDENCE</span><h2>Inspect each claim</h2><p>Inspect 42 public theorems, including planar clipping and derived finite-strain work, beside numerical references and remaining research gaps.</p><a href="proofs.html">Open proofs and evidence →</a></article></div><h2>Keep a reading copy</h2><p><a href="downloads/Rheon-expanded-book.pdf">Illustrated PDF</a> · <a href="downloads/Rheon-expanded-markdown.zip">Markdown + figures (ZIP)</a> · <a href="downloads/Rheon-expanded-book.md">Markdown text</a> · <a href="reference-data.json">Shared reference data</a></p>'''
    (OUT/'index.html').write_text(shell('Overview',home))
    labs='''<p class="eyebrow">SIX PROGRESSIVE LABORATORIES</p><h1>Explore the mechanism.</h1><p class="lede">Rotate the scene, choose a reference state, and inspect the numbers behind it. Each lab uses the same deterministic data as the book figures.</p><label for="lab">Laboratory</label><select id="lab"><option value="projection">1 · MAC pressure projection</option><option value="collision">2 · Triangle mesh and earliest collision</option><option value="hydrostatic">3 · Layered density and hydrostatic pressure</option><option value="viscous">4 · Implicit viscous shear decay</option><option value="slip">5 · Navier slip and wall traction</option><option value="cap">6 · Constant-volume wetting and capillarity</option></select><section class="laboratory"><div id="scene" tabindex="0" aria-label="Interactive three-dimensional reference scene. Drag to rotate; scroll to zoom."></div><div class="lab-panel"><h2 id="lab-title"></h2><p id="lab-description"></p><div id="controls"></div><dl id="metrics" aria-live="polite"></dl><p id="lab-limit"></p><button id="reset-view">Reset camera</button><a id="chapter-target" href="chapters/04-pressure-projection.html">Read the chapter →</a></div></section><p id="render-status" role="status">Loading local 3D assets…</p><p>Reference implementation: <a href="reference.py">original Python source</a> · <a href="reference-data.json">complete data</a> · <a href="reference-qualification.json">numerical receipt</a>. Analytic caps and shear modes are not a general liquid simulation.</p>'''
    extra='<script type="importmap">{"imports":{"three":"./vendor/three.module.js","three/addons/controls/OrbitControls.js":"./vendor/OrbitControls.js"}}</script><script type="module" src="labs.js"></script>'
    (OUT/'labs.html').write_text(shell('3D laboratories',labs,extra=extra))
    receipt=json.loads(qualification.read_text()) if qualification.exists() else {'status':'pending','reason':'Pinned project build and axiom audit must complete.'}
    proofs='<p class="eyebrow">CHECK THE HYPOTHESES</p><h1>Proofs & evidence</h1><p>Finite exact algebra, local numerical references and production implementation are distinct evidence levels. The proof inventory does not verify mesh assembly, a liquid solver or floating-point behavior.</p><h2>Current qualification receipt</h2><pre>'+html.escape(json.dumps(receipt,indent=2))+'</pre><p><a href="chapters/F-expansion-contracts-and-sources.html">Full contract map and primary sources →</a></p>'
    proofs+=f'<p><strong>{receipt.get("public_theorems", "Pending")} public theorems · {receipt.get("audited_declarations", "Pending")} audited declarations.</strong> Definitions and generated proof helpers are counted separately from public theorems.</p>'
    proofs+='''<div class="cards"><article><h2>Planar first contact</h2><p>Five theorems derive a strict crossing, first hit and permitted clipped segment for an infinite stationary plane. Finite-facet containment and earliest mesh queries remain outside the proof.</p><a href="chapters/20-collision-mesh-pipeline.html">Read the collision contract</a></article><article><h2>Derived viscous work</h2><p>Four theorems derive dissipation and energy nonincrease from fixed finite strain and exact backward-Euler equations. Stencil assembly, forcing and approximate solves remain outside the proof.</p><a href="chapters/22-viscosity-and-stress.html">Read the viscosity bridge</a></article></div>'''
    for path in sorted((ROOT/'proofs/Rheon').glob('*.lean')):
        shutil.copy2(path,OUT/'proofs'/path.name)
        proofs+=f'<details><summary>{path.name} — source and hypotheses</summary><p><a href="proofs/{path.name}">Download Lean source</a></p><pre><code>{html.escape(path.read_text())}</code></pre></details>'
    (OUT/'proofs.html').write_text(shell('Proofs and evidence',proofs))
    (OUT/'downloads/Rheon-expanded-book.md').write_text(front+'\n\n'.join(markdown))
    with zipfile.ZipFile(OUT/'downloads/Rheon-expanded-markdown.zip','w',zipfile.ZIP_DEFLATED) as bundle:
        bundle.write(OUT/'downloads/Rheon-expanded-book.md','Rheon-expanded-book.md')
        for path in sorted((OUT/'figures').rglob('*')):
            if path.is_file():bundle.write(path,str(path.relative_to(OUT)))
    # A self-contained print edition: all math is already rendered locally.
    toc='<h1>Contents</h1><ol>'+''.join(f'<li><a href="#{p["slug"]}">{html.escape(p["title"])}</a></li>' for p in pages)+'</ol>'
    cover='<section class="cover"><p>PUMA / EXPANDED RESEARCH EDITION</p><h1>Rheon</h1><h2>Discrete Fluid Simulation</h2><p>Solids, liquids and contact</p><img src="figures/expansion/slip-wetting.png" alt="Original reference profiles"><p>Mathematics · Checked contracts · Numerical references · 3D teaching laboratories</p><p>4 October 2026</p></section>'
    contents=cover+'<section class="toc">'+toc+'</section>'+''.join(f'<section class="book-chapter" id="{p["slug"]}">{p["html"]}</section>' for p in pages)
    (OUT/'print.html').write_text(f'<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Rheon — Expanded research edition</title><link rel="stylesheet" href="style.css"><link rel="stylesheet" href="vendor/katex/katex.min.css"></head><body class="print-book">{contents}</body></html>')
    if (HERE/'downloads/Rheon-expanded-book.pdf').exists():shutil.copy2(HERE/'downloads/Rheon-expanded-book.pdf',OUT/'downloads/Rheon-expanded-book.pdf')
    build_receipt={'schema':'rheon-education-build-v1','chapters':len(pages),'rendered_math_expressions':rendered['count'],'sources':manifest,'reference_data_sha256':hashlib.sha256((OUT/'reference-data.json').read_bytes()).hexdigest(),'pandoc':command(['pandoc','--version']).splitlines()[0],'source_base':command(['git','rev-parse','HEAD'],cwd=ROOT).strip(),'proof_status':receipt.get('status')}
    if input_hashes(ROOT)!=pdf_inputs:raise RuntimeError('PDF inputs changed during HTML build; rebuild.')
    build_receipt['pdf_inputs']=pdf_inputs
    (OUT/'build-receipt.json').write_text(json.dumps(build_receipt,indent=2)+'\n');print(json.dumps({k:v for k,v in build_receipt.items() if k not in ['sources','pdf_inputs']},indent=2))
if __name__=='__main__':build()
