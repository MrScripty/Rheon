#!/usr/bin/env python3
"""Capture the real native material UI through an existing private X11 display.

This does not build, instrument, or replace the application. XTest operates the
ordinary controls; ImageMagick reads the actual application window. OCR records
displayed precision, not a full numerical-state export. Output stays outside Git.
"""
from __future__ import annotations

import argparse
import csv
import ctypes as C
import ctypes.util
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import time

from PIL import Image, ImageChops, ImageStat


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class Input:
    def __init__(self, display):
        self.x = C.CDLL(ctypes.util.find_library('X11'))
        self.t = C.CDLL(ctypes.util.find_library('Xtst'))
        self.x.XOpenDisplay.argtypes = [C.c_char_p]
        self.x.XOpenDisplay.restype = C.c_void_p
        self.d = self.x.XOpenDisplay(display.encode())
        if not self.d:
            raise RuntimeError('Cannot open the authenticated X11 display')
        signatures = {
            'XFlush': ([C.c_void_p], C.c_int),
            'XCloseDisplay': ([C.c_void_p], C.c_int),
            'XSetInputFocus': ([C.c_void_p, C.c_ulong, C.c_int, C.c_ulong], C.c_int),
            'XStringToKeysym': ([C.c_char_p], C.c_ulong),
            'XKeysymToKeycode': ([C.c_void_p, C.c_ulong], C.c_uint),
            'XInternAtom': ([C.c_void_p, C.c_char_p, C.c_int], C.c_ulong),
            'XSendEvent': ([C.c_void_p, C.c_ulong, C.c_int, C.c_long, C.c_void_p], C.c_int),
        }
        for name, (args, result) in signatures.items():
            fn = getattr(self.x, name)
            fn.argtypes, fn.restype = args, result
        self.t.XTestFakeMotionEvent.argtypes = [C.c_void_p, C.c_int, C.c_int, C.c_int, C.c_ulong]
        self.t.XTestFakeButtonEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
        self.t.XTestFakeKeyEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]

    def flush(self):
        self.x.XFlush(self.d)

    def move(self, x, y):
        self.t.XTestFakeMotionEvent(self.d, -1, round(x), round(y), 0)
        self.flush()

    def button(self, down):
        self.t.XTestFakeButtonEvent(self.d, 1, int(down), 0)
        self.flush()

    def key(self, name, down):
        code = self.x.XKeysymToKeycode(self.d, self.x.XStringToKeysym(name.encode()))
        if not code:
            raise RuntimeError('Missing X11 key: ' + name)
        self.t.XTestFakeKeyEvent(self.d, code, int(down), 0)
        self.flush()

    def click(self, x, y):
        self.move(x, y)
        self.button(True)
        time.sleep(.06)
        self.button(False)
        time.sleep(.2)

    def drag(self, start, end, modifier=None):
        self.move(*start)
        if modifier:
            self.key(modifier, True)
        self.button(True)
        for i in range(1, 17):
            self.move(*(a + (b - a) * i / 16 for a, b in zip(start, end)))
            time.sleep(.025)
        self.button(False)
        if modifier:
            self.key(modifier, False)
        time.sleep(.25)

    def close_window(self, window):
        class Data(C.Union):
            _fields_ = [('b', C.c_char * 20), ('s', C.c_short * 10), ('l', C.c_long * 5)]

        class Client(C.Structure):
            _fields_ = [('type', C.c_int), ('serial', C.c_ulong), ('send_event', C.c_int),
                        ('display', C.c_void_p), ('window', C.c_ulong),
                        ('message_type', C.c_ulong), ('format', C.c_int), ('data', Data)]

        class Event(C.Union):
            _fields_ = [('client', Client), ('pad', C.c_long * 24)]

        event = Event()
        event.client.type = 33
        event.client.display = self.d
        event.client.window = window
        event.client.message_type = self.x.XInternAtom(self.d, b'WM_PROTOCOLS', False)
        event.client.format = 32
        event.client.data.l[0] = self.x.XInternAtom(self.d, b'WM_DELETE_WINDOW', False)
        self.x.XSendEvent(self.d, window, False, 0, C.byref(event))
        self.flush()


def run(args):
    repo, output, binary = args.repo.resolve(), args.output.resolve(), args.binary.resolve()
    if output == repo or repo in output.parents:
        raise ValueError('Generated capture output must be outside the repository')
    if not binary.is_file():
        raise ValueError('Build the native desktop binary before invoking this tool')
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ, DISPLAY=args.display, XAUTHORITY=str(args.authority.resolve()),
               LIBGL_ALWAYS_SOFTWARE='1', OMP_THREAD_LIMIT='1',
               MESA_SHADER_CACHE_DIR=str(output/'mesa-cache'))
    if args.runtime:
        env['XDG_RUNTIME_DIR'] = str(args.runtime.resolve())
    os.environ.update({k: env[k] for k in ['DISPLAY', 'XAUTHORITY']})
    events, captures = [], []
    started = time.monotonic()
    result = {'schema': 'rheon-native-desktop-capture-v1', 'status': 'in-progress',
              'repo': str(repo), 'binary': str(binary), 'binary_sha256': digest(binary),
              'source_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip(),
              'source_tree': subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], cwd=repo, text=True).strip(),
              'worktree_status': subprocess.check_output(['git', 'status', '--porcelain'], cwd=repo, text=True),
              'capture_tool_sha256': digest(__file__),
              'source_sha256': {str(p.relative_to(repo)): digest(p) for p in
                                sorted([repo/'Cargo.toml', repo/'Cargo.lock', *repo.glob('src/**/*.rs')])},
              'display': args.display, 'software_gl_requested': True,
              'browser_used': False, 'native_source_modified_for_capture': False,
              'numerical_state_exported': False,
              'metadata_precision': 'OCR of actual displayed values; not full-precision numerical state',
              'expected_accepted_steps': args.expected_steps, 'events': events, 'captures': captures}

    def command(argv):
        return subprocess.check_output(argv, env=env, text=True, stderr=subprocess.PIPE)

    def event(name, **fields):
        events.append({'elapsed_seconds': time.monotonic() - started, 'action': name, **fields})

    def screenshot(path):
        subprocess.run(['import', '-window', hex(window), str(path)], env=env, check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)

    def ocr(path):
        raw = command(['tesseract', str(path), 'stdout', '--psm', '11', 'tsv'])
        # Tesseract may prefix resolution notices before the TSV header.
        header = raw.find('level\tpage_num\t')
        if header < 0:
            raise RuntimeError('Tesseract did not emit TSV')
        words = [row for row in csv.DictReader(io.StringIO(raw[header:]), delimiter='\t')
                 if (row.get('text') or '').strip()]
        return words, ' '.join(w['text'] for w in words)

    def capture(name):
        path = output/(name+'.png')
        screenshot(path)
        words, text = ocr(path)
        (output/(name+'.txt')).write_text(text+'\n')
        with Image.open(path) as image:
            image.convert('RGB').save(output/(name+'.jpg'), quality=85)
            dimensions = list(image.size)
        record = {'name': name, 'png': path.name, 'png_sha256': digest(path),
                  'jpeg': name+'.jpg', 'jpeg_sha256': digest(output/(name+'.jpg')),
                  'dimensions': dimensions, 'displayed_text': text}
        captures.append(record)
        return path, words, text

    def click_word(words, text):
        hits = [w for w in words if w['text'].strip().casefold() == text.casefold()]
        if len(hits) != 1:
            raise RuntimeError(f'Expected one visible {text!r} control, found {len(hits)}')
        w = hits[0]
        pointer.click(left+int(w['left'])+int(w['width'])/2,
                      top+int(w['top'])+int(w['height'])/2)
        event('click', control=text)

    def step_identity(text):
        match = re.search(r'\bStep\s+(\d+)\s*/\s*(\d+)\b', text)
        return tuple(map(int, match.groups())) if match else None

    def changed(a, b):
        with Image.open(a) as one, Image.open(b) as two:
            box = (280, 45, min(one.width, two.width)-10, min(one.height, two.height)-120)
            delta = ImageChops.difference(one.convert('RGB').crop(box), two.convert('RGB').crop(box))
            return sum(ImageStat.Stat(delta).mean)/3

    app = pointer = window = None
    log = (output/'native.log').open('wb')
    try:
        result['display_info'] = command(['xdpyinfo'])
        app = subprocess.Popen([str(binary)], cwd=repo, env=env, stdout=log, stderr=subprocess.STDOUT)
        deadline = time.monotonic()+30
        while time.monotonic() < deadline:
            if app.poll() is not None:
                raise RuntimeError(f'Native app exited during startup: {app.returncode}')
            tree = command(['xwininfo', '-root', '-tree'])
            match = re.search(r'(0x[0-9a-f]+) "Rheon / Live 3D"', tree)
            if match:
                window = int(match[1], 16)
                break
            time.sleep(.2)
        if window is None:
            raise RuntimeError('Native app window did not appear')
        info = command(['xwininfo', '-id', hex(window)])
        result['window_info'] = info
        left = int(re.search(r'Absolute upper-left X:\s+(-?\d+)', info)[1])
        top = int(re.search(r'Absolute upper-left Y:\s+(-?\d+)', info)[1])
        pointer = Input(args.display)
        pointer.x.XSetInputFocus(pointer.d, window, 1, 0)
        pointer.flush()
        time.sleep(1)
        initial, words, text = capture('material-initial')
        if args.simulate_at:
            x, y = args.simulate_at
            with Image.open(initial) as image:
                if not (0 <= x < image.width and 0 <= y < image.height):
                    raise RuntimeError('Reviewed Simulate coordinate is outside the native window')
            pointer.click(left+x, top+y)
            event('click', control='Simulate', reviewed_window_point=[x, y])
        else:
            click_word(words, 'Simulate')
        deadline = time.monotonic()+args.timeout
        while time.monotonic() < deadline:
            if app.poll() is not None:
                raise RuntimeError(f'Native app exited during calculation: {app.returncode}')
            time.sleep(1)
            poll = output/'last-observed.png'
            screenshot(poll)
            _, text = ocr(poll)
            identity = step_identity(text)
            if 'Complete' in text and identity and identity[0] == identity[1] and identity[0] > 1:
                break
            if 'stopped:' in text or 'failed;' in text:
                raise RuntimeError('Native simulation stopped: '+text)
        else:
            raise RuntimeError('Timed out before the requested accepted material state: '+text)
        event('observed-complete', displayed_text=text)
        _, words, complete_text = capture('material-completed')
        complete = step_identity(complete_text)
        if not complete or complete[0] != complete[1] or 'Material fluid' not in complete_text:
            raise RuntimeError('Completed material step is not legible in the actual capture')
        if args.expected_steps is not None and complete[1] != args.expected_steps:
            raise RuntimeError('Actual accepted step count differs from the requested count')
        result['completed_step_identity'] = complete
        # Use the existing previous-step control. This disables Follow live and
        # holds an actual accepted historical frame before changing the camera.
        if args.previous_at:
            x, y = args.previous_at
            width, height = captures[-1]['dimensions']
            if not (0 <= x < width and 0 <= y < height):
                raise RuntimeError('Reviewed previous-step coordinate is outside the native window')
            pointer.click(left+x, top+y)
            event('click', control='Previous accepted step', reviewed_window_point=[x, y])
        else:
            follow = [w for w in words if w['text'] == 'Follow']
            if len(follow) != 1:
                raise RuntimeError('Cannot locate the native timeline control row')
            row_y = int(follow[0]['top']) + int(follow[0]['height'])/2
            steps = sorted((w for w in words if re.sub(r'[^A-Za-z]', '', w['text']) == 'Step' and
                            abs(int(w['top'])+int(w['height'])/2-row_y) < 12),
                           key=lambda w: int(w['left']))
            if len(steps) != 2:
                raise RuntimeError('Cannot identify both native previous/next-step controls')
            w = steps[0]
            pointer.click(left+int(w['left'])+int(w['width'])/2,
                          top+int(w['top'])+int(w['height'])/2)
        event('previous-accepted-step', expected_step=complete[0]-1)
        base, _, baseline_text = capture('material-accepted')
        held = step_identity(baseline_text)
        if held != (complete[0]-1, complete[1]):
            raise RuntimeError('Native previous-step control did not hold the prior accepted state')
        result['held_step_identity'] = held
        width, height = captures[-1]['dimensions']
        start = (left+width*.66, top+height*.42)
        pointer.drag(start, (start[0]+110, start[1]+55))
        event('primary-drag-orbit', start=start, delta=[110, 55])
        orbit, _, orbit_text = capture('material-accepted-orbit')
        difference = changed(base, orbit)
        if difference < 1 or step_identity(orbit_text) != held:
            raise RuntimeError('Camera orbit did not change real geometry while retaining accepted step')
        result['orbit_mean_pixel_difference'] = difference
        result['accepted_step_preserved_on_orbit'] = True
        result['initial_to_accepted_mean_pixel_difference'] = changed(initial, base)
        if result['initial_to_accepted_mean_pixel_difference'] < 1:
            raise RuntimeError('No actual material geometry appeared')
        result['source_unchanged'] = all(digest(repo/name) == sha for name, sha in result['source_sha256'].items())
        result['binary_unchanged'] = digest(binary) == result['binary_sha256']
        if not result['source_unchanged'] or not result['binary_unchanged']:
            raise RuntimeError('Source or binary changed during capture')
        result['status'] = 'PASS'
    except Exception as error:
        result['status'] = 'FAIL'
        result['error'] = str(error)
        raise
    finally:
        if pointer and window and app and app.poll() is None:
            pointer.close_window(window)
            try:
                app.wait(timeout=15)
            except subprocess.TimeoutExpired:
                app.terminate()
                app.wait(timeout=15)
                result['forced_termination'] = True
        if pointer:
            pointer.x.XCloseDisplay(pointer.d)
        log.close()
        result['native_exit_code'] = app.returncode if app else None
        result['elapsed_seconds'] = time.monotonic()-started
        (output/'provenance.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps({'status': result['status'], 'output': str(output),
                      'provenance_sha256': digest(output/'provenance.json')}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--display', default=os.environ.get('DISPLAY', ':90'))
    parser.add_argument('--authority', type=Path, required=True)
    parser.add_argument('--runtime', type=Path)
    parser.add_argument('--timeout', type=float, default=180)
    parser.add_argument('--expected-steps', type=int)
    parser.add_argument('--simulate-at', type=lambda value: tuple(map(int, value.split(','))),
                        help='Explicit X,Y window coordinate reviewed in the actual initial screenshot; otherwise use OCR')
    parser.add_argument('--previous-at', type=lambda value: tuple(map(int, value.split(','))),
                        help='Explicit X,Y previous-step control coordinate reviewed in the actual completed screenshot')
    run(parser.parse_args())
