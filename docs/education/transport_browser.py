"""Independent sampling/transport oracles and bounded teaching-browser checks.

The Python formulas below independently evaluate the affine stencil and the
companion's stated periodic forward/backward correction. They do not invoke a
native pressure solver, establish general conservation, or advance a fluid.
"""
from fractions import Fraction as F
from itertools import product
from pathlib import Path
import hashlib
import math

from teaching_browser import (close, display, evidence_path, metrics, no_overflow,
                              require, same_camera, scroll_to, settle, snapshot)

ROOT = Path(__file__).resolve().parents[2]
OFFSETS = {'cell': (F(1,2),)*3, 'x': (F(0),F(1,2),F(1,2)),
           'y': (F(1,2),F(0),F(1,2)), 'z': (F(1,2),F(1,2),F(0))}
FIXTURES = {'symmetric': [F(0),F(0),F(1),F(1),F(0),F(0)],
            'asymmetric': [F(0),F(1,5),F(1),F(2,5),F(0),F(0)]}


def sampling_expectation(component='x', wrong=False, position=(.67,.84,1.12)):
    position = tuple(F(str(value)) for value in position)
    spacing = (F(1,5),F(3,10),F(2,5))
    origin = (F(-1,10),F(1,5),F(-3,10))
    coefficient = (F(7,10),F(-11,10),F(2,5))
    def affine(point):
        return F(6,5) + sum(a*b for a,b in zip(coefficient,point))
    stored = OFFSETS[component]
    sampled = OFFSETS['cell'] if wrong else stored
    local = [(p-o)/h-offset for p,o,h,offset in zip(position,origin,spacing,sampled)]
    lower = [math.floor(value) for value in local]
    fraction = [value-index for value,index in zip(local,lower)]
    donors = []
    for corner in product((0,1),repeat=3):
        weight = math.prod(t if bit else 1-t for t,bit in zip(fraction,corner))
        point = [o+h*(index+bit+offset) for o,h,index,bit,offset in zip(origin,spacing,lower,corner,stored)]
        donors.append({'position':point,'weight':weight,'value':affine(point)})
    value = sum(donor['weight']*donor['value'] for donor in donors)
    exact = affine(position)
    return {'position':position,'donors':donors,'value':value,'exact':exact,'error':value-exact,
            'weightSum':sum(donor['weight'] for donor in donors),
            'minimum':min(donor['value'] for donor in donors),
            'maximum':max(donor['value'] for donor in donors),'component':component,'wrong':wrong}


def linear_step(values, courant):
    """Periodic piecewise-linear departure interpolation, with exact Fraction support."""
    result = []
    for index in range(len(values)):
        departure = index-courant
        lower = math.floor(departure)
        fraction = departure-lower
        result.append((1-fraction)*values[lower%len(values)]+fraction*values[(lower+1)%len(values)])
    return result


def corrected_step(values, courant):
    forward = linear_step(values,courant)
    backward = linear_step(forward,-courant)
    candidate = [new+(old-back)/2 for old,new,back in zip(values,forward,backward)]
    fallback = []
    for index,value in enumerate(candidate):
        lower = math.floor(index-courant)
        donors = [values[lower%len(values)],values[(lower+1)%len(values)]]
        fallback.append(not min(donors)<=value<=max(donors))
    limited = [baseline if revert else proposal for baseline,proposal,revert in zip(forward,candidate,fallback)]
    return {'forward':forward,'candidate':candidate,'limited':limited,'fallback':fallback}


def transport_expectation(profile='sine', n=64, courant=.5, steps=1):
    if profile in FIXTURES:
        initial = list(FIXTURES[profile])
    elif profile == 'sine':
        initial = [math.sin(2*math.pi*index/n) for index in range(n)]
    elif profile == 'top-hat':
        # The companion's n=64 fixture is exactly q[20:36]=1.
        initial = [float(5/16 <= index/n < 9/16) for index in range(n)]
    else:
        raise ValueError('Unknown independent transport profile: '+profile)
    n = len(initial)
    c = F(str(courant)) if profile in FIXTURES else float(courant)
    linear,raw,limited = initial[:],initial[:],initial[:]
    fallback_count = 0
    last_fallback = [False]*n
    for _ in range(steps):
        linear = linear_step(linear,c)
        raw = corrected_step(raw,c)['candidate']
        result = corrected_step(limited,c)
        limited,last_fallback = result['limited'],result['fallback']
        fallback_count += sum(last_fallback)
    exact = ([math.sin(2*math.pi*(index/n-courant*steps/n)) for index in range(n)]
             if profile=='sine' else linear_step(initial,c*steps))
    def diagnostics(values):
        return {'min':min(values),'max':max(values),'sum':sum(values),
                'amountChange':sum(values)-sum(initial),
                'rms':math.sqrt(sum(float(value-reference)**2 for value,reference in zip(values,exact))/n)}
    return {'initial':initial,'linear':linear,'raw':raw,'limited':limited,'exact':exact,
            'linearMetrics':diagnostics(linear),'rawMetrics':diagnostics(raw),'limitedMetrics':diagnostics(limited),
            'fallbackCount':fallback_count,'lastFallback':sum(last_fallback)}


def vector(actual, expected, label, scale=None, tolerance=1e-11):
    require(isinstance(actual,list) and len(actual)==len(expected),'transport vector shape '+label)
    magnitude = max((abs(float(value)) for value in expected),default=0) if scale is None else scale
    for index,(value,reference) in enumerate(zip(actual,expected)):
        close(value,float(reference),label+'['+str(index)+']',scale=magnitude,tolerance=tolerance)


def check_sampler(actual, component='x', wrong=False, position=(.67,.84,1.12)):
    expected = sampling_expectation(component,wrong,position)
    require(actual.get('component')==component and actual.get('wrong') is wrong,'sampler controls')
    vector(actual.get('position'),expected['position'],'sampler query')
    donors = actual.get('donors')
    require(isinstance(donors,list) and len(donors)==8,'sampler eight donors')
    donors = sorted(donors,key=lambda donor:donor['position'])
    reference = sorted(expected['donors'],key=lambda donor:donor['position'])
    signal = max(abs(float(donor['value'])) for donor in reference)
    for index,(observed,truth) in enumerate(zip(donors,reference)):
        vector(observed.get('position'),truth['position'],f'sampler donor {index}')
        close(observed.get('weight'),float(truth['weight']),f'sampler weight {index}',scale=1)
        close(observed.get('value'),float(truth['value']),f'sampler value {index}',scale=signal)
        require(observed['weight']>=0,'sampler nonnegative weight')
    for key in ('value','exact','error','minimum','maximum'):
        close(actual.get(key),float(expected[key]),'sampler '+key,scale=signal)
    close(actual.get('weightSum'),1,'sampler partition of unity')
    require(actual['minimum']-signal*1e-11<=actual['value']<=actual['maximum']+signal*1e-11,'sampler convex bound')
    return expected


def check_transport(actual, profile='sine', n=64, courant=.5, steps=1):
    expected = transport_expectation(profile,n,courant,steps)
    signal = max(abs(float(value)) for key in ('initial','linear','raw','limited','exact') for value in expected[key])
    for key in ('initial','linear','raw','limited','exact'):
        vector(actual.get(key),expected[key],'transport '+key,scale=signal)
    count = len(expected['initial'])
    for group in ('linearMetrics','rawMetrics','limitedMetrics'):
        observed = actual.get(group,{})
        for key in ('min','max','sum','amountChange','rms'):
            scale = signal*count if key in ('sum','amountChange') else signal
            close(observed.get(key),float(expected[group][key]),group+' '+key,scale=scale)
    require(type(actual.get('fallbackCount')) is int and actual['fallbackCount']==expected['fallbackCount'],
            'transport cumulative fallback count')
    require(type(actual.get('lastFallback')) is int and actual['lastFallback']==expected['lastFallback'],
            'transport final fallback count')
    for value in actual['limited']:
        require(min(expected['initial'])-signal*1e-11<=value<=max(expected['initial'])+signal*1e-11,
                'limited transport global bound')
    return expected


def check_reference_lines(lines, reference):
    require(isinstance(lines,list) and len(lines)==3,'transport three rendered polylines')
    require([line.get('series') for line in lines]==['linear','raw','limited'],'transport rendered series order')
    for line,z in zip(lines,(-.45,0,.45)):
        values = reference[line['series']]
        points = [[-.8+1.6*index/len(values),.45*float(value),z] for index,value in enumerate(values)]
        points.append([.8,.45*float(values[0]),z])
        require(len(line.get('points',[]))==len(points),'transport periodic seam vertex')
        for index,(observed,expected) in enumerate(zip(line['points'],points)):
            vector(observed,expected,f"transport rendered {line['series']} point {index}",scale=1,tolerance=1e-7)


def displayed(text, expected, label):
    """Compare seven-significant-figure GUI numbers with the inspected arrays."""
    close(float(text),float(expected),'visible '+label,tolerance=5.1e-7)


def sampler_readout(page, actual):
    values = metrics(page)
    for key,field in [('Weight sum','weightSum'),('Interpolated value','value'),
                      ('Direct affine value','exact'),('Signed error','error')]:
        displayed(values[key],actual[field],key)
    for value,expected in zip(values['Sample position'].split(', '),actual['position']):
        close(float(value),expected,'visible sample position',scale=1,tolerance=.00051)
    bounds = values['Donor range'].split(' … ')
    require(len(bounds)==2,'sampler visible donor bounds')
    displayed(bounds[0],actual['minimum'],'donor minimum')
    displayed(bounds[1],actual['maximum'],'donor maximum')
    require(values['Within donor range']=='true','sampler visible convex bound')
    rows = page.locator('.donor-table tr')
    require(rows.count()==9,'sampler visible eight-donor table')
    for index,donor in enumerate(actual['donors']):
        cells = rows.nth(index+1).locator('td').all_text_contents()
        require(len(cells)==3,'sampler donor table columns')
        for value,expected in zip(cells[0].split(', '),donor['position']):
            close(float(value),expected,'visible donor coordinate',scale=1,tolerance=.0051)
        displayed(cells[1],donor['weight'],'donor weight')
        displayed(cells[2],donor['value'],'donor value')


def transport_readout(page, actual):
    values = metrics(page)
    require(values['Grid / steps']==f"{len(actual['initial'])} samples / {actual['steps']} steps",'visible grid and steps')
    courant,travel = values['Courant / travel'].removesuffix(' periods').split(' / ')
    displayed(courant,actual['courant'],'Courant')
    displayed(travel,actual['courant']*actual['steps']/len(actual['initial']),'travel')
    # JavaScript reduces from the left; explicit addition preserves that order
    # when checking a displayed near-zero signed total from actual sample data.
    total = 0.0
    for value in actual['initial']:
        total += value
    displayed(values['Initial sum'],total,'initial sum')
    for series,label in [('linear','Linear · teal'),('raw','Raw · orange'),('limited','Limited · blue')]:
        result = actual[series+'Metrics']
        low,high = values[label+' range'].split(' … ')
        displayed(low,result['min'],label+' minimum')
        displayed(high,result['max'],label+' maximum')
        displayed(values[label+' sum Δ'],result['amountChange'],label+' sum change')
        displayed(values[label+' RMS'],result['rms'],label+' RMS')
    require(values['Fallback cells · last / total']==f"{actual['lastFallback']} / {actual['fallbackCount']}",
            'visible last/cumulative fallback count')
    expected = 'Analytic sine translation' if actual['profile']=='sine' else 'Translated initial piecewise-linear interpolant'
    require(values['RMS reference']==expected,'visible RMS comparison definition')


def control_values(page):
    return page.locator('#controls select,#controls input').evaluate_all(
        'nodes=>nodes.map(node=>node.value === "fixed" ? "fixed" : Number(node.value))')


def inspect_example(page):
    from teaching_browser import arrows
    state = snapshot(page)
    require(state['webgl'],'transport teaching WebGL')
    actual = state.get('reference')
    require(isinstance(actual,dict),'read-only recomputed reference snapshot')
    controls = control_values(page)
    if state['lab']=='sampler':
        component,wrong,displacement = controls
        expected = check_sampler(actual,['cell','x','y','z'][component],bool(wrong),[.67+(displacement-50)*.004,.84,1.12])
        sampler_readout(page,actual)
        arrows(state['arrows'],[([(float(p)-origin)*2.5 for p,origin in zip(donor['position'],(.67,.84,1.12))],
                                 [0,.7*float(donor['weight']),0]) for donor in expected['donors']], 'sampler weight')
    else:
        require(state['lab']=='transport','bounded transport lab selection')
        profile,size,courant,steps = controls
        require((size=='fixed') == (profile>=2),'fixture exposes its actual fixed sample count')
        require(page.get_by_label('Sample count',exact=True).is_disabled() == (profile>=2),'fixture sample count control state')
        expected = check_transport(actual,['sine','top-hat','symmetric','asymmetric'][profile],
                                   64 if size=='fixed' else [32,64,128,256][size],courant/100,steps)
        transport_readout(page,actual)
        check_reference_lines(state.get('referenceLines'),expected)
    require(page.locator('#lab-expected').text_content().strip(),'transport expected-result explanation')
    require(page.locator('#lab-limit').text_content().strip(),'transport limitations explanation')
    require(page.locator('#transport-sources').get_attribute('hidden') is None,'recomputed source links are available')
    return state


def set_range(page, label, value):
    node = page.get_by_label(label,exact=True)
    node.fill(str(value))
    node.dispatch_event('input')
    settle(page)


def sampler_controls(page, component=1, wrong=0, displacement=50):
    page.select_option('#lab','sampler')
    page.get_by_label('Stored lattice',exact=True).select_option(str(component))
    page.get_by_label('Sampling offset',exact=True).select_option(str(wrong))
    set_range(page,'Sample x displacement',displacement)


def transport_controls(page, profile='sine', n=64, courant=.5, steps=1, period=False):
    page.select_option('#lab','transport')
    page.get_by_label('Sample count',exact=True).select_option(str([32,64,128,256].index(n)))
    page.get_by_label('Scalar profile',exact=True).select_option(str(['sine','top-hat','symmetric','asymmetric'].index(profile)))
    if period:
        before = snapshot(page)
        page.locator('#one-period').click()
        settle(page)
        same_camera(before,snapshot(page),'one-period button camera')
    else:
        set_range(page,'Courant × 100',round(courant*100))
        set_range(page,'Transport steps',steps)


def qualify(page, mobile, load, check_errors, artifact_dir=None):
    from teaching import CHAPTER_STOPS
    source_paths = {'transport_reference.js': ROOT/'docs/education/transport_reference.js',
                    'transport_labs.js': ROOT/'docs/education/transport_labs.js',
                    'transport-companion.py': ROOT/'docs/research-book/companion/experiments.py',
                    'sampling-companion.py': ROOT/'docs/research-book/companion/depth_experiments.py'}
    screenshots = []
    def picture(name,target=page):
        if artifact_dir is not None:
            destination = evidence_path(artifact_dir,name)
            settle(target)
            target.screenshot(path=str(destination),type='jpeg',quality=85,full_page=False)
            screenshots.append(str(destination))
    def checked(target=page):
        settle(target)
        result = inspect_example(target)
        check_errors()
        return result

    load(page,'transport-labs.html')
    page.wait_for_selector('#metrics dd')
    source_hashes = {}
    for name,path in source_paths.items():
        published = page.evaluate('async name=>(await fetch(name)).text()',name)
        require(published==path.read_text(),'published recomputed source bytes '+name)
        source_hashes[name] = hashlib.sha256(path.read_bytes()).hexdigest()
    sampler_states = 0
    for component in range(4):
        for wrong in (0,1):
            sampler_controls(page,component,wrong)
            checked()
            sampler_states += 1
    for wrong in (0,1):
        for displacement in (0,100):
            sampler_controls(page,1,wrong,displacement)
            checked()
            sampler_states += 1
    sampler_controls(page,1,1)
    page.locator('.donor-table summary').click()
    require(page.locator('.donor-table table').is_visible(),'donor disclosure exposes all weights')
    picture('transport-desktop-sampler.jpg')

    transport_cases = [dict(profile='symmetric',n=64,courant=.5,steps=1),
                       dict(profile='asymmetric',n=64,courant=.25,steps=1)]
    transport_cases += [dict(profile='sine',n=n,courant=.5,steps=2*n,period=True) for n in (32,64,128,256)]
    transport_cases += [dict(profile='top-hat',n=64,courant=.5,steps=128),
                       dict(profile='sine',n=32,courant=.5,steps=0),
                       dict(profile='sine',n=32,courant=0,steps=1),
                       dict(profile='sine',n=32,courant=1,steps=1)]
    for case in transport_cases:
        transport_controls(page,**case)
        state = checked()
        if case['profile']=='asymmetric':
            picture('transport-desktop-asymmetric.jpg')
        if case.get('period') and case['n']==64:
            picture('transport-desktop-refinement.jpg')
    require(len(transport_cases)==10 and sampler_states==12,'bounded numerical coverage')

    load(page,'transport-labs.html')
    page.wait_for_selector('#metrics dd')
    page.locator('#scene').press('ArrowRight')
    settle(page)
    camera = snapshot(page)
    stop_presets = [('sampler','sampler',[1,0,50]),('sampler-compare','sampler',[1,1,50]),
                    ('transport','transport',[2,'fixed',50,1]),('transport-compare','transport',[3,'fixed',25,1])]
    for anchor,lab,controls in stop_presets:
        scroll_to(page,'#'+anchor)
        observed = checked()
        require(observed['lab']==lab,'transport tour scroll lab '+anchor)
        require(control_values(page)==controls,
                'transport tour scroll preset '+anchor)
        require(page.locator('#'+anchor).evaluate('node=>node.classList.contains("current-lesson")'),'transport active tour stop '+anchor)
        same_camera(camera,observed,'transport tour scroll camera')
        require(abs(page.locator('.example-workspace').bounding_box()['y']-84)<=2,'transport desktop sticky example')
        no_overflow(page,'transport desktop tour')
    page.locator('.example-workspace').evaluate('node=>node.scrollTop=0')
    picture('transport-desktop-reading.jpg')
    sampler_controls(page,1,1)
    require(page.locator('#follow-reading').get_attribute('aria-pressed')=='false','transport manual controls pause follow')
    scroll_to(page,'#transport')
    require(snapshot(page)['lab']=='sampler','transport scroll overwrote manual comparison')
    same_camera(camera,snapshot(page),'transport manual controls preserve camera')
    page.locator('#follow-reading').click()
    settle(page)
    resumed = checked()
    require(resumed['lab']=='transport' and resumed['reference']['profile']=='symmetric','transport resume active reading preset')
    page.locator('#one-period').click()
    one_period = checked()
    require(one_period['reference']['profile']=='sine' and one_period['reference']['n']==64 and
            one_period['reference']['steps']==128 and one_period['reference']['courant']==.5,'one-period button selects full-period sine')
    require(page.locator('#follow-reading').get_attribute('aria-pressed')=='false','one-period button pauses follow')
    same_camera(camera,one_period,'one-period preserves manual camera')

    chapter_stops = 0
    direct_links = [('transport-labs.html#'+anchor,lab,controls) for anchor,lab,controls in stop_presets]
    preset_controls = {'correct':[1,0,50],'wrong':[1,1,50],'symmetric':[2,'fixed',50,1],
                       'asymmetric':[3,'fixed',25,1],'smooth':[0,1,50,128]}
    for prefix in ('03','06','07'):
        source = list((ROOT/'docs/research-book/chapters').glob(prefix+'-*.md'))
        require(len(source)==1,'transport chapter source identity')
        load(page,'chapters/'+source[0].stem+'.html')
        page.wait_for_selector('#metrics dd')
        chapter_camera = snapshot(page)
        for anchor,lab,preset in CHAPTER_STOPS[prefix]:
            if lab not in ('sampler','transport'):
                continue
            scroll_to(page,'#'+anchor)
            observed = checked()
            require(observed['lab']==lab,'transport chapter scroll lab '+anchor)
            require(control_values(page)==preset_controls[preset],
                    'transport chapter scroll preset '+anchor)
            direct_links.append(('chapters/'+source[0].stem+'.html#'+anchor,lab,preset_controls[preset]))
            same_camera(chapter_camera,observed,'transport chapter camera')
            no_overflow(page,'transport chapter '+prefix)
            chapter_stops += 1
    require(chapter_stops==7,'all seven new chapter stops covered')
    for destination,lab,controls in direct_links:
        load(page,destination)
        page.wait_for_selector('#metrics dd')
        state = checked()
        require(state['lab']==lab and control_values(page)==controls,
                'transport direct link selects intended preset '+destination)
        no_overflow(page,'transport direct link '+destination)
    require(len(direct_links)==11,'all eleven transport direct links covered')

    mobile_checks = []
    for width in (390,320):
        mobile.set_viewport_size({'width':width,'height':844})
        load(mobile,'transport-labs.html')
        mobile.wait_for_selector('#metrics dd',state='attached')
        require(mobile.locator('body').get_attribute('data-reading-mode')=='text' and
                mobile.locator('.reading-text').is_visible() and not mobile.locator('.example-workspace').is_visible(),
                'transport phone text default')
        require(mobile.locator('.reading-text').evaluate('node=>parseFloat(getComputedStyle(node).fontSize)')>=16 and
                mobile.locator('.reading-text').bounding_box()['width']>=width-48,'transport phone readable text')
        no_overflow(mobile,'transport phone text')
        scroll_to(mobile,'#sampler-compare')
        before = snapshot(mobile)
        position = mobile.evaluate('scrollY')
        mobile.locator('button[data-reading-mode=example]').click()
        settle(mobile)
        require(snapshot(mobile)['reference']==before['reference'],'transport phone mode preserves selected state')
        mobile.locator('button[data-reading-mode=text]').click()
        settle(mobile)
        require(abs(mobile.evaluate('scrollY')-position)<=2,'transport phone mode preserves reading position')
        mobile.locator('button[data-reading-mode=split]').click()
        settle(mobile)
        require(mobile.locator('.reading-text').is_visible() and mobile.locator('.example-workspace').is_visible(),
                'transport phone split visibility')
        for anchor in ('sampler-compare','transport-compare'):
            scroll_to(mobile,'#'+anchor)
            state = checked(mobile)
            require(state['lab']==('sampler' if anchor.startswith('sampler') else 'transport'),
                    'transport phone split selected lab')
            if state['lab']=='sampler':
                require(state['reference']['wrong'] is True,'transport phone split wrong-offset preset')
            else:
                require(state['reference']['profile']=='asymmetric' and state['reference']['courant']==.25,
                        'transport phone split nonconservation preset')
            same_camera(before,state,'transport phone split camera')
        no_overflow(mobile,'transport phone split')
        picture(f'transport-phone-{width}-split.jpg',mobile)
        mobile.locator('button[data-reading-mode=example]').click()
        settle(mobile)
        require(not mobile.locator('.reading-text').is_visible() and mobile.locator('.example-workspace').is_visible(),
                'transport phone example visibility')
        transport_controls(mobile,'asymmetric',64,.25,1)
        checked(mobile)
        require(mobile.locator('#follow-reading').get_attribute('aria-pressed')=='false','transport phone manual pause')
        no_overflow(mobile,'transport phone example')
        picture(f'transport-phone-{width}-example.jpg',mobile)
        mobile.locator('button[data-reading-mode=text]').click()
        require(mobile.locator('.reading-text').is_visible(),'transport phone returns to text')
        mobile_links = [('transport-labs.html#transport-compare','transport',{'profile':'asymmetric','courant':.25})]
        if width==320:
            mobile_links.append(('chapters/03-staggered-grids.html#layout-and-indexing','sampler',{'component':'x','wrong':False}))
        for destination,lab,expected_fields in mobile_links:
            load(mobile,destination)
            mobile.wait_for_selector('#metrics dd',state='attached')
            settle(mobile)
            state = snapshot(mobile)
            require(state['lab']==lab and all(state['reference'].get(key)==value for key,value in expected_fields.items()),
                    'transport phone direct link selects intended preset '+destination)
            mobile.locator('button[data-reading-mode=example]').click()
            settle(mobile)
            require(snapshot(mobile)['reference']==state['reference'],'phone direct-linked preset survives Example mode '+destination)
            same_camera(state,snapshot(mobile),'phone direct-linked Example camera')
        mobile_checks.append({'width':width,'modes':['text','split','example'],'readable_text':True,
                              'no_horizontal_overflow':True,'split_scroll_presets':True,
                              'control_comparison':True,'mode_preserves_lesson_and_scroll':True,
                              'direct_link_selects_preset':True,'direct_link_cases':len(mobile_links)})
        check_errors()
    mobile.set_viewport_size({'width':390,'height':844})
    require(all(hashlib.sha256(path.read_bytes()).hexdigest()==source_hashes[name] for name,path in source_paths.items()),
            'recomputed source inputs changed during browser qualification')
    check_errors()
    return {'schema':'rheon-transport-browser-v1','source_sha256':source_hashes,
            'numerical':{'sampler_states':sampler_states,'transport_states':len(transport_cases),
                         'sampler_donors_checked':sampler_states*8,'transport_curves_checked':len(transport_cases)*3,
                         'sine_refinements':[32,64,128,256],'affine_wrong_offset_bias':-.07,
                         'symmetric':{'raw_min':-.125,'raw_max':1.125,'fallback':2,'limited_sum':2},
                         'asymmetric':{'fallback':1,'initial_sum':1.6,'limited_sum':1.61875,'sum_change':.01875},
                         'fixed_plot_scale':True},
            'desktop_scroll_presets':4,'chapter_scroll_presets':chapter_stops,'direct_links':len(direct_links),
            'camera_preserved_during_scroll_and_controls':True,'manual_comparison_pauses_follow':True,
            'follow_resumes_current_section':True,'one_period_button':True,'mobile':mobile_checks,
            'fluid_solves':0,'fluid_advances':0,'screenshots':screenshots}
