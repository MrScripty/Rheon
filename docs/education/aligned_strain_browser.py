"""Real Chromium controls checked against the independent Fraction operator.

The browser evaluates fixed native rows. This verifier independently rederives
those rows from the pinned specimen, checks every strain and force component,
and checks the displayed metrics after actual control events.
"""
from fractions import Fraction as F
from pathlib import Path
import json
import math
import re
import subprocess

from aligned_strain_packet import ROOT, browser_expectation, validate_packet, require


VECTOR_KEYS = ('field','strains','action','forces')
METRIC_KEYS = ('normalD','shearD','D','work','identity','B','energy')


def close(actual,expected,label,scale=None,tolerance=F(1,10**10)):
    require(type(actual) in (int,float) and math.isfinite(actual),'browser finite '+label)
    expected = F(expected)
    magnitude = abs(expected) if scale is None else max(abs(expected),F(scale))
    require(abs(F(actual)-expected)<=tolerance*magnitude,'browser independent mismatch '+label)


def check_snapshot(packet,snapshot):
    controls = snapshot.get('controls',{})
    expected = browser_expectation(packet,controls.get('preset'),controls.get('amplitude'),
                                    controls.get('secondary'),controls.get('mu'),controls.get('cornerSelection'))
    for key in VECTOR_KEYS:
        actual = snapshot.get(key)
        require(type(actual) is list and len(actual)==len(expected[key]),'browser vector shape '+key)
        # A same-unit vector magnitude covers cancellation to exact zero.
        # These are nearest-rounded fixture comparisons, not IEEE enclosures.
        scale = max(map(abs,expected[key]),default=F(0))
        for i,(a,b) in enumerate(zip(actual,expected[key])):
            close(a,b,key+'['+str(i)+']',scale=scale)
    for key in METRIC_KEYS:
        scale = expected['D']+abs(expected['work']) if key=='identity' else None
        close(snapshot.get(key),expected[key],key,scale=scale)
    selected = snapshot.get('selectedRow')
    require(type(selected) is int and 0<=selected<len(packet['rows']),'browser selected native row')
    require(snapshot.get('selectedRowData')==packet['rows'][selected],'browser selected row metadata')
    require(expected['D']>=0 and snapshot['D']>=0 and snapshot['normalD']>=0 and snapshot['shearD']>=0,
            'browser nonnegative loss')
    require(snapshot['work']<=0,'browser stationary force work sign')
    return expected


def displayed_number(text,label):
    match=re.match(r'^\s*([+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?)',text,re.I)
    require(match is not None,'browser displayed number '+label)
    return float(match[1])


def inspect(page,packet):
    snapshot=page.evaluate('window.alignedStrainLab.snapshot()')
    expected=check_snapshot(packet,snapshot)
    for key in METRIC_KEYS:
        locator=page.locator('#strain-'+key)
        require(locator.count()==1,'browser displayed metric '+key)
        close(float(locator.get_attribute('data-value')),expected[key],'display data '+key,
              scale=expected['D']+abs(expected['work']) if key=='identity' else None)
        visible=displayed_number(locator.inner_text(),key)
        # Five significant figure rendering is separately checked against the
        # exact result. Identity diagnostics can visually round harmless noise.
        scale=expected['D']+abs(expected['work']) if key=='identity' else None
        close(visible,expected[key],'visible metric '+key,scale=scale,tolerance=F(1,1000))
    row_id=snapshot['selectedRow'];row=packet['rows'][row_id]
    panel=page.locator('#strain-inspector')
    require(panel.get_attribute('data-row-id')==str(row_id),'browser row inspector identity')
    scale=max(map(abs,expected['strains']),default=F(0))
    close(float(panel.get_attribute('data-strain')),expected['strains'][row_id],'displayed row strain',scale=scale)
    close(float(page.locator('#strain-row-strain').get_attribute('data-value')),
          expected['strains'][row_id],'visible row strain data',scale=scale)
    cells=page.locator('#strain-row-terms tr')
    require(cells.count()==max(1,len(row['terms'])),'browser row support count')
    if not row['terms']:
        require('Zero coefficients' in cells.inner_text(),'browser retained zero row explanation')
        require(snapshot['strains'][row_id]==0,'browser zero row evaluates zero')
    else:
        for i,term in enumerate(row['terms']):
            require(cells.nth(i).get_attribute('data-active')==str(term['active']),'browser active support identity')
            values=cells.nth(i).locator('td').all_text_contents()
            close(displayed_number(values[1],'coefficient'),F(term['coefficient']),'displayed coefficient',tolerance=F(1,10000))
            close(displayed_number(values[2],'velocity'),expected['field'][term['active']],
                  'displayed support velocity',tolerance=F(1,10000))
    require(page.locator('#strain-view').get_attribute('data-selected-row')==str(row_id),
            'browser scene row selection')
    require(page.locator('#strain-view .strain-face').count()>0,'browser active face markers')
    return snapshot,expected


def evidence_path(directory,name,repo=ROOT):
    directory=Path(directory).resolve();repo=Path(repo).resolve()
    require(not directory.is_relative_to(repo) and not repo.is_relative_to(directory),
            'browser pixels must be outside Git checkout')
    existing=directory
    while not existing.exists():existing=existing.parent
    require(subprocess.run(['git','-C',str(existing),'rev-parse','--is-inside-work-tree'],
                           capture_output=True).returncode != 0,'browser evidence inside Git')
    directory.mkdir(parents=True,exist_ok=True)
    path=directory/name
    require(not path.exists(),'browser pixel evidence must be fresh')
    return path


def qualify(page,mobile,load,check_errors,packet_dir=None,artifact_dir=None,repo=ROOT):
    load(page,'aligned-strain-lab.html')
    if packet_dir is None:
        require(page.locator('#strain-preset').count()==0 and 'no qualified specimen' in page.locator('main').inner_text(),
                'browser source-only strain notice')
        return {'included':False,'dynamic_states':0,'independent_rational_comparisons':0,
                'fluid_solves':0,'fluid_advances':0,'mobile_no_overflow':True,'screenshots':[]}
    validate_packet(repo,packet_dir)
    packet=json.loads((Path(packet_dir)/'data.json').read_text())
    page.wait_for_function('window.alignedStrainLab !== undefined')
    require(page.evaluate('window.alignedStrainLab.packet')==packet,'browser qualified packet binding')
    require(page.locator('#strain-row option').count()==210,'browser retains all native rows')
    require(page.locator('#strain-corner option').count()==12,'browser reflected corner controls')
    states=0;screenshots=[]
    def check(target=page):
        nonlocal states
        snapshot,expected=inspect(target,packet);states+=1;check_errors()
        return snapshot,expected
    def picture(name,target=page):
        if artifact_dir is not None:
            path=evidence_path(artifact_dir,name,repo)
            target.screenshot(path=str(path),full_page=True,type='jpeg',quality=85)
            screenshots.append(str(path))
    check();picture('aligned-strain-desktop-corner.jpg')
    for preset in ('corner','normal','shear','rotation'):
        page.select_option('#strain-preset',preset)
        before,_=check()
        page.locator('#strain-amplitude').fill('0.05');page.locator('#strain-amplitude').dispatch_event('input')
        after,_=check()
        require(before['D']!=after['D'],'browser amplitude control is unresponsive')
        page.locator('#strain-amplitude').fill('-2');page.locator('#strain-amplitude').dispatch_event('input')
        check()
        page.locator('#strain-mu').fill('0');page.locator('#strain-mu').dispatch_event('input')
        zero,_=check();require(zero['D']==0 and all(f==0 for f in zero['forces']),'browser zero viscosity')
        page.locator('#strain-mu').fill('2');page.locator('#strain-mu').dispatch_event('input');check()
    # Tiny values exercise the numerical path directly in the actual browser;
    # range event cases above cover the slider's five-hundredths resolution.
    page.evaluate('window.alignedStrainLab.setControls({preset:"corner",amplitude:1e-6,secondary:-1e-6,mu:1})')
    check()
    page.evaluate('window.alignedStrainLab.setControls({preset:"corner",amplitude:1,secondary:1,mu:1})')
    for edge in packet['cornerEdges']:
        page.select_option('#strain-corner',edge['id'])
        snapshot,expected=check()
        require(snapshot['controls']['cornerSelection']==edge['id'],'browser corner control is unresponsive')
        require(page.locator('#strain-inspector').get_attribute('data-corner-id')==edge['id'],
                'browser reflected corner inspector')
        loss=sum(F(packet['rows'][i]['weight'])*expected['strains'][i]**2 for i in edge['rowIds'])
        sign=edge['solidSigns'][0]*edge['solidSigns'][1]
        require(loss==4+2*sign,'browser independent reflected corner block')
    page.select_option('#strain-corner',packet['defaultCornerSelection'])
    page.locator('#strain-secondary').fill('-1');page.locator('#strain-secondary').dispatch_event('input')
    check();picture('aligned-strain-desktop-cancellation.jpg')
    for row in packet['rows']:
        if not row['terms']:
            page.select_option('#strain-row',str(row['id']));check()
    page.select_option('#strain-preset','shear')
    page.locator('#strain-show-force').check();check()
    require(page.locator('#strain-view').text_content(),'browser force scene missing')
    picture('aligned-strain-desktop-force.jpg')
    for layer in range(3):
        page.select_option('#strain-slice',str(layer));check()
        require(page.locator('#strain-view').get_attribute('data-slice')==str(layer),'browser slice is unresponsive')
    load(mobile,'aligned-strain-lab.html');mobile.wait_for_function('window.alignedStrainLab !== undefined')
    require(mobile.evaluate('window.alignedStrainLab.packet')==packet,'mobile qualified packet binding')
    mobile.select_option('#strain-preset','corner')
    mobile.select_option('#strain-corner',packet['cornerEdges'][0]['id'])
    mobile.locator('#strain-amplitude').fill('2');mobile.locator('#strain-amplitude').dispatch_event('input')
    check(mobile)
    require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'),'aligned strain mobile overflow')
    picture('aligned-strain-mobile-reflection.jpg',mobile)
    check_errors()
    return {'included':True,'dynamic_states':states,'active_faces':48,'rows':210,'independent_rational_comparisons':states,
            'retained_zero_rows':6,'reflected_corners':12,'mobile_no_overflow':True,
            'browser_arithmetic':'nearest-rounded finite-row gather and transpose scatter',
            'independent_reference':'Fraction geometry/rows/field/force/work; no unit-sized tolerance floor',
            'fluid_solves':0,'fluid_advances':0,'screenshots':screenshots}
