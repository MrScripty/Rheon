"""Exercise the built site without writes; render/requalify only with --render-pdf."""
from pathlib import Path
from contextlib import contextmanager
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from threading import Thread
import argparse
import hashlib
import json
import shutil
from playwright.sync_api import sync_playwright
from pdf_freshness import input_hashes, verify_pdf, write_receipt
from browser_qualification import LABS, book_sources, source_hashes, verify_browser_qualification
from native_browser import qualify as qualify_native
from native_sequence import metadata
from obstacle_browser import qualify as qualify_obstacle
from obstacle_flow_browser import qualify as qualify_obstacle_flow
from aligned_strain_browser import qualify as qualify_aligned_strain
from sphere_contact_lab import validate_packet as validate_contact_packet

HERE = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


@contextmanager
def serve_site(directory):
    class QuietHandler(SimpleHTTPRequestHandler):
        def log_message(self, *args):
            pass
    server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(directory)))
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f'http://127.0.0.1:{server.server_port}'
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def check_planar_statement(page):
    paragraph = page.locator('p').filter(has=page.locator('code', has_text='wall_hit_on_surface'))
    expected = ['g(x(t))=(1-t)g(a)+tg(b)', 't_*', 'g(x(t_*))=0',
                r'0\le t<t_*', 'x(t_*)', r'0\le s\le1']
    actual = paragraph.locator('annotation[encoding="application/x-tex"]').all_text_contents()
    require(paragraph.count() == 1 and actual == expected and not paragraph.locator('em').count(),
            f'Rendered planar statement changed: {actual!r}')


def exercise_browser(base_url, render_pdf=False, artifact_dir=None):
    errors, failed = [], []
    def track(page):
        page.on('pageerror', lambda e: errors.append(str(e)))
        page.on('response', lambda r: failed.append(f'{r.status} {r.url}') if r.status >= 400 else None)
        page.on('requestfailed', lambda r: failed.append(f'{r.failure} {r.url}'))
    def check_errors():
        require(not errors, f'Browser page errors: {errors}')
        require(not failed, f'Browser network failures: {failed}')
    def load(page, path):
        page.goto(base_url.rstrip('/') + '/' + path)
        page.wait_for_load_state('networkidle')
        check_errors()

    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=shutil.which('chromium'), headless=True,
                                    args=['--no-sandbox', '--enable-unsafe-swiftshader'])
        page = browser.new_page(viewport={'width': 1440, 'height': 1050}, device_scale_factor=1)
        track(page)
        load(page, 'index.html')
        page.screenshot(path='/tmp/rheon-education-home.jpg', type="jpeg", quality=85, full_page=True)
        load(page, 'labs.html')
        page.wait_for_selector('#metrics dd')
        page.wait_for_timeout(500)
        require(page.locator('canvas').count() == 1, 'WebGL did not initialize')
        checks = []
        for lab in LABS:
            page.select_option('#lab', lab)
            page.wait_for_timeout(80)
            before = page.locator('#metrics').inner_text()
            controls = page.locator('#controls select')
            if controls.count():
                controls.last.select_option(index=controls.last.locator('option').count()-1)
            ranges = page.locator('#controls input[type=range]')
            if ranges.count():
                ranges.first.fill(ranges.first.get_attribute('max'))
                ranges.first.dispatch_event('input')
            after = page.locator('#metrics').inner_text()
            require(before != after, f'Controls did not change metrics: {lab}')
            check_errors()
            page.screenshot(path=f'/tmp/rheon-education-{lab}.jpg', type="jpeg", quality=85, full_page=True)
            checks.append({'lab': lab, 'control_changes_metrics': True, 'readout': after})
        page.select_option('#lab', 'cap')
        page.locator('input[type=range]').fill('12')
        page.locator('input[type=range]').dispatch_event('input')
        page.screenshot(path='/tmp/rheon-education-cap90.jpg', type="jpeg", quality=85, full_page=True)
        page.mouse.move(500, 500)
        page.mouse.down()
        page.mouse.move(610, 540, steps=5)
        page.mouse.up()
        page.locator('#reset-view').click()
        load(page, 'chapters/23-wetting-and-adhesion.html')
        require(page.locator('.katex-error').count() == 0, 'KaTeX rendering errors')
        require(page.locator('.katex').count() > 5, 'Missing rendered chapter mathematics')
        page.screenshot(path='/tmp/rheon-education-chapter.jpg', type="jpeg", quality=85, full_page=True)
        load(page, 'chapters/20-collision-mesh-pipeline.html')
        check_planar_statement(page)
        page.screenshot(path='/tmp/rheon-education-planar-statement.jpg', type="jpeg", quality=85, full_page=True)
        mobile = browser.new_page(viewport={'width': 390, 'height': 844}, is_mobile=True, has_touch=True)
        track(mobile)
        load(mobile, 'index.html')
        mobile.locator('#menu').click()
        require(mobile.locator('#navigation').is_visible(), 'Mobile navigation did not open')
        mobile.locator('#search').fill('wetting')
        require(mobile.locator('.chapter-link:visible').count() == 1, 'Mobile chapter search failed')
        mobile.screenshot(path='/tmp/rheon-education-mobile.jpg', type="jpeg", quality=85, full_page=True)
        load(mobile, 'labs.html#cap')
        mobile.wait_for_selector('#metrics dd')
        require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'),
                'Mobile laboratory has horizontal overflow')
        native = qualify_native(page, mobile, load, check_errors, metadata(HERE.parents[1]), base_url)
        obstacle = qualify_obstacle(page, mobile, load, check_errors)
        obstacle_flow = qualify_obstacle_flow(page, mobile, load, check_errors)
        packet_dir = Path(artifact_dir)/'aligned-strain-packet' if artifact_dir is not None else HERE/'_site/aligned-strain-packet'
        strain = qualify_aligned_strain(page, mobile, load, check_errors, packet_dir if packet_dir.exists() else None, artifact_dir=Path(artifact_dir)/'strain-browser-evidence' if render_pdf and artifact_dir is not None else None)
        contact_cases = 0
        contact_packet = Path(artifact_dir)/'sphere-contact-packet' if artifact_dir is not None else HERE/'_site/sphere-contact-packet'
        load(page, 'sphere-contact-lab.html')
        if contact_packet.exists():
            cases = validate_contact_packet(HERE.parents[1], contact_packet)
            for i, (name, raw) in enumerate(cases):
                page.select_option('#contact-case', str(i))
                shown = page.locator('.contact-case:visible')
                require(shown.count() == 1 and shown.locator('h2').inner_text() == name,
                        'Contact case selection differs from native packet')
                cells = shown.locator('td').all_text_contents()
                require(cells[1:7] == [json.dumps(v) for v in
                        [raw['before']['time_s'],raw['after']['time_s'],raw['before']['center'],
                         raw['after']['center'],raw['before']['velocity'],raw['after']['velocity']]],
                        'Contact readout differs from actual native snapshot')
                require(shown.locator('svg circle[data-role=collider]').count() == 2,
                        'Declared collider projections missing')
                contact_cases += 1
            load(mobile, 'sphere-contact-lab.html')
            require(mobile.evaluate('document.documentElement.scrollWidth <= innerWidth+1'),
                    'Mobile contact playback has horizontal overflow')
        else:
            require('packet is absent' in page.locator('main').inner_text(),
                    'Source-only contact page must state packet absence')
        nojs = browser.new_context(java_script_enabled=False, viewport={'width':390,'height':844})
        readable = nojs.new_page()
        track(readable)
        load(readable, 'implementation/static-sphere-contact.html')
        require(readable.locator('#navigation').is_visible() and readable.locator('main h1').is_visible(),
                'No-JavaScript mobile reading/navigation failed')
        require(readable.locator('.katex-error').count() == 0,
                'No-JavaScript mathematics failed')
        load(readable, 'sphere-contact-lab.html')
        if contact_packet.exists():
            require(readable.locator('.contact-case:visible').count() == contact_cases,
                    'No-JavaScript contact snapshots unavailable')
        nojs.close()
        load(page, 'print.html')
        page.evaluate('document.fonts.ready')
        check_planar_statement(page)
        reading_order = json.loads((Path(artifact_dir)/'build-receipt.json').read_text())['reading_order'] if artifact_dir is not None else None
        if reading_order is not None:
            require(page.locator('.book-chapter').evaluate_all('(ss)=>ss.map(s=>s.id)') == reading_order,
                    'Print reading order differs from release manifest')
        check_errors()
        if render_pdf:
            destination = Path(artifact_dir) if artifact_dir is not None else HERE
            (destination/'downloads').mkdir(exist_ok=True)
            page.pdf(path=str(destination/'downloads/Rheon-expanded-book.pdf'), print_background=True,
                     prefer_css_page_size=True, display_header_footer=True, header_template='<div></div>',
                     footer_template='<div style="font-size:9px;width:100%;text-align:center;color:#52676d">Rheon · Puma · <span class="pageNumber"></span> / <span class="totalPages"></span></div>')
        check_errors()
        receipt = {'schema': 'rheon-education-browser-v1', 'browser': browser.version,
                   'webgl': True, 'labs': checks, 'mobile_navigation_search': True,
                   'mobile_no_horizontal_overflow': True, 'katex_no_errors': True,
                   'page_errors': errors, 'http_failures': failed,
                   'planar_statement_hit_time_rendering': True}
        receipt['native_labs'] = native
        receipt['static_obstacle'] = obstacle
        receipt['obstacle_flow'] = obstacle_flow
        receipt['aligned_strain'] = strain
        receipt['sphere_contact'] = {'included':contact_packet.exists(),'native_cases_checked':contact_cases,
                                    'browser_physics_calls':0,'pose_interpolations':0}
        receipt['no_javascript_reading'] = True
        receipt['print_reading_order'] = reading_order
        browser.close()
        return receipt


def verify(render_pdf=False, base_url=None, artifact_dir=None):
    repo = HERE.parents[1]
    inputs = input_hashes(repo)
    sources, books = source_hashes(HERE), book_sources(repo)
    artifact = Path(artifact_dir) if artifact_dir is not None else HERE
    site = Path(artifact_dir) if artifact_dir is not None else HERE/'_site'
    retained = [artifact/'downloads/Rheon-expanded-book.pdf', artifact/'pdf-inputs.json',
                artifact/'browser-qualification.json']
    if not render_pdf:
        verify_pdf(repo, artifact_dir=artifact_dir)
        verify_browser_qualification(repo, artifact_dir=artifact_dir)
        before = [p.read_bytes() for p in retained]
    if base_url is None:
        with serve_site(site) as url:
            receipt = exercise_browser(url, render_pdf, artifact_dir) if artifact_dir is not None else exercise_browser(url, render_pdf)
    else:
        receipt = exercise_browser(base_url, render_pdf, artifact_dir) if artifact_dir is not None else exercise_browser(base_url, render_pdf)
    require(source_hashes(HERE) == sources and book_sources(repo) == books,
            'Browser source/book inputs changed during qualification')
    if render_pdf:
        write_receipt(repo, inputs, artifact_dir=artifact_dir) if artifact_dir is not None else write_receipt(repo, inputs)
        receipt['reviewed_sources'] = sources
        receipt['book_sources'] = books
        receipt['pdf_sha256'] = hashlib.sha256(retained[0].read_bytes()).hexdigest()
        retained[2].write_text(json.dumps(receipt, indent=2)+'\n')
    else:
        require([p.read_bytes() for p in retained] == before,
                'Publication verification changed the retained PDF or qualification receipts')
        verify_pdf(repo, artifact_dir=artifact_dir)
        verify_browser_qualification(repo, artifact_dir=artifact_dir)
    print(json.dumps({'mode': 'render-pdf' if render_pdf else 'check',
                      'browser': receipt['browser'], 'labs': [x['lab'] for x in receipt['labs']],
                      'page_errors': receipt['page_errors'], 'http_failures': receipt['http_failures']}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--check', action='store_true', help='Read-only publication verification (default)')
    mode.add_argument('--render-pdf', action='store_true', help='Render and requalify; inspect the new PDF before publication')
    parser.add_argument('--url', help='Existing preview URL; otherwise serve the built site on a private local port')
    parser.add_argument('--output-dir', type=Path, help='Edition and receipts outside Git')
    args = parser.parse_args()
    verify(render_pdf=args.render_pdf, base_url=args.url, artifact_dir=args.output_dir)
