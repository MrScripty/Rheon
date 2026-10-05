# Complete immutable liquid packet bindings

This additive inventory binds every tracked file in the frozen composition and
coupled evidence packets, including their root receipts and nested replay receipts.
The earlier packet verifiers exclude files named receipt.json from their local
file-hash inventory. That excluded their nested legacy replay receipt as well as
the intended master receipt. The original Git evidence commits still bind all
bytes. This supplement makes the complete per-file binding explicit without
modifying either published packet, their source/evidence identities or any
earlier historical receipt.

Composition evidence commit: 8e3ea72a820fbbcfde40f14146f6b9536a109608,
tree 4cdb0787c64672e374dfd6f02f9bfda2068dd18c.
Composition qualified source: f4af9b47223f4f6928f40f7f3b5283e6023bf620,
tree a209d4b8739f3bd353c4c685ac408c70aeae184c.

Coupled evidence commit: 9ef82f1d523ba20209365e4bace0bd75c374c709,
tree 81c492530d746fe2359e1b9bd9ba67813fd9f7a6.
Coupled qualified source: fd71f48df1f726ae130ba9b18ea02a6ecec26937,
tree 17e7506a426e112311183328175cdb9f12bd4a94.

verify.py compares every file's Git blob, SHA-256, frozen commit bytes and live
bytes; it also checks exact source/tree/parent identities and complete path sets.
It uses explicit checks that remain active under python -O. Numerical/native
qualification remains exactly that recorded in the frozen packets. No simulation
source changes or new numerical claims accompany this supplement.

    python3 evidence/liquid-bindings/verify.py
    python3 -O evidence/liquid-bindings/verify.py
