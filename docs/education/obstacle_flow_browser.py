"""Exercise every recorded flow control and profile in real Chromium."""
def qualify(page,mobile,load,check_errors):
    load(page,'obstacle-flow-lab.html')
    presentation=page.evaluate("fetch('obstacle-flow-presentation.json').then(r=>r.json())")
    if not presentation['included']:
        if 'fields are absent' not in page.locator('main').inner_text() or page.locator('#flow-pressure-case').count():raise RuntimeError('Missing source-only flow notice')
        return {'included':False,'pressure_views':0,'shear_frames':0,'mobile_no_overflow':True,'browser_fluid_solves':0}
    page.wait_for_function('window.__obstacleFlow !== undefined')
    records=page.evaluate('window.__obstacleFlow.records');pressure_views=0;shear_frames=0
    number=lambda x:page.evaluate('(x)=>x.toPrecision(6)',x)
    def expect(id,index,value):
        actual=page.locator('#'+id+' dd').all_text_contents()
        if actual[index]!=value:raise RuntimeError('Flow readout mismatch '+str([id,index,actual[index],value]))
    for i,c in enumerate(records['pressure_cases']):
        page.select_option('#flow-pressure-case',str(i))
        expect('flow-pressure-metrics',0,f"{c['stamp'][0]}:{c['stamp'][1]}")
        expect('flow-pressure-metrics',3,number(c['residual'][1])+' m³/s²')
        for state in ['before','after']:
            page.select_option('#flow-state',state)
            for z in range(3):
                page.select_option('#flow-slice',str(z))
                if page.locator('#flow-pressure-view rect').count()!=9:raise RuntimeError('Missing pressure cell field')
                if page.locator('#flow-pressure-view').get_attribute('data-state')!=state or page.locator('#flow-pressure-view').get_attribute('data-slice')!=str(z):raise RuntimeError('Stale pressure view')
                pressure_views+=1
        check_errors()
    for i,c in enumerate(records['shear_cases']):
        page.select_option('#flow-shear-case',str(i))
        for f in c['frames']:
            page.locator('#flow-step').fill(str(f['step']));page.locator('#flow-step').dispatch_event('input')
            if page.locator('#flow-shear-view circle').count()!=len(f['velocity']):raise RuntimeError('Missing native shear values')
            expect('flow-shear-metrics',4,f"{f['step']} / {c['dt']} s")
            if f['step']:expect('flow-shear-metrics',5,number(f['energy'][1])+' J')
            actual=page.locator('#flow-shear-view text').all_text_contents()
            if actual[:len(f['velocity'])]!=[number(v) for v in f['velocity']]:raise RuntimeError('Shear field display differs from native data')
            shear_frames+=1
        check_errors()
    page.select_option('#flow-pressure-case','0');page.select_option('#flow-state','before');page.select_option('#flow-shear-case','1')
    page.screenshot(path='/tmp/rheon-obstacle-flow-lab.jpg',full_page=True,type='jpeg',quality=85)
    load(mobile,'obstacle-flow-lab.html');mobile.wait_for_function('window.__obstacleFlow !== undefined')
    mobile.select_option('#flow-shear-case','5');mobile.locator('#flow-step').fill('3');mobile.locator('#flow-step').dispatch_event('input')
    if not mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'):raise RuntimeError('Flow lab mobile overflow')
    mobile.screenshot(path='/tmp/rheon-obstacle-flow-mobile.jpg',full_page=True,type='jpeg',quality=85)
    check_errors()
    return {'included':True,'pressure_views':pressure_views,'shear_frames':shear_frames,'mobile_no_overflow':True,'browser_fluid_solves':0}
