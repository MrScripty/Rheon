"""Fresh source-bound Lean audit of the research-only conditional boundary/gradient statements.

Uses existing pinned third-party dependencies; never downloads or builds more.
Generated audit/probe files stay in a new external output directory.
"""
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'research'))
from check_proof_audit import audit, cli

SOURCE = Path(__file__).with_name('GradientInterface.lean')
PREFIX = 'Rheon.ObstacleGradient.'
SCOPE = 'conditional exact-real affine/trace and matched finite-gradient work algebra only'

def main(lean, dependencies, output):
    return audit(ROOT, SOURCE, PREFIX, SCOPE, Path(__file__), lean, dependencies, output)

if __name__ == '__main__':
    cli(ROOT, SOURCE, PREFIX, SCOPE, Path(__file__))
