"""Attempt the available browser with its sandbox enabled; never bypass it."""
import hashlib
import json
from pathlib import Path
import sys
from playwright.sync_api import sync_playwright

target = Path(sys.argv[1]).resolve()
with sync_playwright() as p:
    try:
        browser = p.chromium.launch(executable_path='/usr/bin/chromium', headless=True, chromium_sandbox=True)
    except Exception as error:
        print(json.dumps(dict(status='BLOCKED_BROWSER_SANDBOX', reason=str(error), sandbox_bypass=False, html_sha256=hashlib.sha256(target.read_bytes()).hexdigest()), indent=2))
        sys.exit(0)
    page = browser.new_page()
    errors = []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.goto(target.as_uri())
    if errors or page.locator('#h').input_value() != '0.003125':
        raise ValueError('browser execution/default selection failed: ' + str(errors))
    print(json.dumps(dict(status='PASS', scope='Browser initial render and default controls', info=page.locator('#info').inner_text(), html_sha256=hashlib.sha256(target.read_bytes()).hexdigest()), indent=2))
    browser.close()
