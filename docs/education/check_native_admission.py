"""Fail early on stale admission inputs; never create or refresh source pins."""
import sys
from aligned_strain_packet import ROOT, REVIEWED_INVENTORY, source_core, git
from aligned_strain_oracle import Refusal

def main():
    try:
        sources = source_core(ROOT)
    except Refusal as error:
        print(f'Native educational admission refused: {error}', file=sys.stderr)
        print('This is a source identity check, not a failed numerical comparison.', file=sys.stderr)
        print('Review changes against the admitted source, explicitly approve a new complete inventory, '
              'and run fresh native/rational qualification on the committed integration source. '
              'Do not copy old receipts, refresh hashes automatically, or skip this gate.', file=sys.stderr)
        return 1
    print(f'Native admission inputs match {len(sources)} paths at {git(ROOT, "rev-parse", "HEAD")}')
    print(f'Admission record: {REVIEWED_INVENTORY if (ROOT/REVIEWED_INVENTORY).exists() else "historical pinned core"}')
    print('This is not qualification. Fresh native/rational and publication checks remain required.')
    return 0

if __name__ == '__main__':
    sys.exit(main())
