"""Check each recorded native geometry readout in real Chromium, without writes."""
import json

def qualify(page, mobile, load, check_errors):
    load(page,'obstacle-lab.html')
    presentation=page.evaluate("fetch('obstacle-presentation.json').then(r=>r.json())")
    if not presentation['included']:
        if 'records are absent' not in page.locator('main').inner_text() or page.locator('#obstacle-case').count():
            raise RuntimeError('Missing source-only geometry notice')
        return {'included':False,'cases':0,'cells':0,'flux_pairs':0,'mobile_no_overflow':True,'fluid_advances':0}
    records=page.evaluate("fetch('obstacle-records.json').then(r=>r.json())")
    page.wait_for_selector('#obstacle-metrics dd')
    if page.locator('#obstacle-scene canvas').count()!=1:
        raise RuntimeError('Static geometry WebGL did not initialize')
    cells=0;pairs=0
    def expect(values):
        actual=page.locator('#obstacle-metrics dd').all_text_contents()
        for i,value in values.items():
            if actual[i]!=value:
                raise RuntimeError('Static geometry recorded readout mismatch: '+json.dumps([i,actual[i],value]))
    def number(value):
        return page.evaluate('(x)=>x.toPrecision(7)',value)
    for c in records['cases']:
        page.select_option('#obstacle-case',c['id'])
        expect({3:str(c['components']),10:' / '.join(map(str,c['stamp']))})
        for i,volume in enumerate(c['volumes']):
            page.select_option('#obstacle-cell',str(i))
            p=[i%c['counts'][0],i//c['counts'][0]%c['counts'][1],i//(c['counts'][0]*c['counts'][1])]
            full=1.0
            for d in range(3):full*=c['origin'][d]+(p[d]+1)*c['spacing'][d]-(c['origin'][d]+p[d]*c['spacing'][d])
            expect({0:', '.join(map(str,p)),1:number(volume)+' / '+number(full)+' m³',2:'dry' if c['labels'][i] is None else str(c['labels'][i])})
            cells+=1
        for d,flux in enumerate(c['flux_controls']):
            page.select_option('#obstacle-axis',str(d))
            probe=c['probes'][d]
            expect({5:'XYZ'[d]+' · '+number(flux['area'])+' m²',6:', '.join(number(x) for x in flux['outward'])+' m³/s',7:number(0)+' m³/s',8:number(probe['t']),9:', '.join(number(x) for x in probe['position'])+' m'})
            pairs+=1
        check_errors()
    page.locator('#obstacle-reset').click()
    if page.locator('#obstacle-refusals li').count()!=3:
        raise RuntimeError('Missing static topology refusal records')
    page.screenshot(path='/tmp/rheon-static-obstacle-lab.png',full_page=True)
    load(mobile,'obstacle-lab.html');mobile.wait_for_selector('#obstacle-metrics dd')
    mobile.select_option('#obstacle-case','separator-x');mobile.select_option('#obstacle-axis','2')
    if not mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'):
        raise RuntimeError('Static geometry mobile overflow')
    mobile.screenshot(path='/tmp/rheon-static-obstacle-mobile.png',full_page=True)
    check_errors()
    return {'included':True,'cases':len(records['cases']),'cells':cells,'flux_pairs':pairs,'mobile_no_overflow':True,'fluid_advances':0}
