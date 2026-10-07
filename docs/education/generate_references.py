"""Build ignored numerical lab/plot assets with the original checked generators."""
from pathlib import Path
import os
import shutil
import subprocess
import sys

BOOK = Path(__file__).resolve().parents[1] / 'research-book'


def generate():
    # Bound reference linear algebra; this changes no scenario or solver setting.
    env = os.environ | {'OPENBLAS_NUM_THREADS': '1'}
    for script in ['companion/experiments.py', 'companion/depth_experiments.py',
                   'companion/make_figures.py', 'expansion/reference.py',
                   'expansion/contact/make_figures.py']:
        subprocess.run([sys.executable, str(BOOK/script)], env=env, check=True)
    for name in ['equal-volume-sessile-caps.svg', 'couette-slip-and-power.svg',
                 'boundary-mechanisms.svg']:
        shutil.copy2(BOOK/'expansion/contact/figures'/name,
                     BOOK/'expansion/figures'/name)


if __name__ == '__main__':
    generate()
