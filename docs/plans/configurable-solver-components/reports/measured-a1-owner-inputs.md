# Exact missing measured A1 inputs — owner handoff

**Status: blocked on original owner inputs; source/evidence handoff independently reviewed.** Retrieve the existing aa82 deployment and its actual runtime/parity/custody bytes. Do not train, substitute weights, fabricate an inventory, or relabel fixture results. This retrieval-requirements work downloaded no model binary and executed no original A1 exporter, verifier, model or runtime. It changes no numerical/UI owner. The known file hashes below are reported by pinned primary sources; the missing owner bytes have not been verified here.

Reference adapter/exporter/verifier source is lab commit `a8426061380e82951b4dba1bbfa38ff93dc7ebb2`. Original training source is `11408e93995a9f3c3f59e171ccd0889906531027`; water diagnostics source is `7bf9f0f5f069063140f67f005da39c7a516c416d`. Preserve those different roles. Original material is reported under `runs/sealed-water-v1-3`, deployment `runs/live-ai-comparison/deployment-2`, registered runtime `runs/cuda-pressure-runtime-1/manifest.json`, and the existing Tuldok owner. Public GitHub source does not contain the complete ignored deployment/runtime/packet. [Exporter](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/deployment.py), [original instructions](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/README.md), [training custody/results](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/sealed-water-pressure-v1/acceptance.json).

## 1. Existing deployment and actual runtime

Provide the whole original `deployment-2` directory, preserving every byte and relative path. Required root files are `deployment.json`, `weights.npz`, `normalization.json`, `history.json`, `receipt.json`, `cpu_bridge.py`, `installed-runtime.json`; under `sources/`, provide `source-witness.json`, `runtime-manifest.json` and **every original witness.sources entry**, not just the two imported entry modules. The exporter copies the training receipt/history and the frozen source closure; the bridge loads model/data and their original helper/custody sources. The actual original descriptor.files map supplies each file's expected hash and length.

| File / role | Known original expected SHA256 |
|---|---|
| `runs/live-ai-comparison/deployment-2/weights.npz` | `aa82bd122fde9406d9e6b92114b3e6ba77281f5ee8b9c726aa168dacbd93a62d` |
| `runs/live-ai-comparison/deployment-2/normalization.json` | `bee9be00a94f0241bc8baf6df2499314918b0a9e708b3f995bea5046c40a3296` |
| `runs/live-ai-comparison/deployment-2/deployment.json` | `be4eef1f051206205598e6876c83edaf5246138e17613f1aa787f3df183a761f` |
| `runs/live-ai-comparison/deployment-2/receipt.json` | `84674887178288e8eb6e48d3e6bf152113f24958357453c59c3a9029bd51fd7c` |
| `runs/live-ai-comparison/deployment-2/sources/source-witness.json` | `582d1f2c77cc2a37bf06035a868867b7118cd5933418a93b390bb92961fd8c56` |

`weights.npz` must be the declared final 2,000-update candidate, associated by `history.checkpoints["checkpoint-2000.npz"]` and the completed training Run. The exporter requires that checkpoint's declared SHA to equal weights aa82; this derived equality does not supply the missing checkpoint bytes/length. `normalization.json` is the unchanged train-only preprocessing, not newly fitted statistics. `receipt.json` is the byte-copy of stage-00's receipt. These target identities do not imply that every other file has a known hash here.

For `history.json`, the original deployed `cpu_bridge.py`, `installed-runtime.json`, the registered runtime summary and each remaining frozen source, fetch their **owner-supplied per-file hash/length inventory**. Those original whole-file values are unavailable in the provided public records; the JSON marks them null. Pinned reference source hashes in this handoff are evidence about code, not an invented replacement for the original be4 descriptor.files inventory.

The runtime manifest alone is insufficient. Provide the exact actual executable named by `deployment.python` / `installed-runtime.json.python`, its `python_sha256`, the named `site_packages` tree and every actual `runtime.files[*].path` asset with matching bytes/hash. The bridge checks the registered manifest hash against `sources/runtime-manifest.json.manifest_sha256`, measures every listed installed asset and total bytes, compares executable identity and Torch/NumPy package versions, then imports the copied sources and registered site-packages under `-I -S -B`. Runtime profile must be `managed_torch_python_v1`; its actual inventory is bounded by 32,768 files/6 GiB. **Actual absolute interpreter path, package versions, runtime manifest SHA and all installed-asset hashes are owner inventory, unknown here—not values to guess.** Host OS/drivers remain trusted external dependencies; this is not a complete sandbox/reproducibility claim. [Pinned bridge](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/cpu_bridge.py).

Keep original runtime paths/material for an exact historical identity. Relocating/re-measuring the executable, package assets or path-bound manifests may change descriptor/runtime/source binding. Any admitted new binding must be explicit and preserve originals; do not hand-edit the old witness/descriptor or claim a changed descriptor still hashes to be4.

## 2. Original export/project and Run custody

Provide `runs/sealed-water-v1-3/project/` intact: original `source-witness.json`, `runtime-manifest.json`, `qualification.json`, config, all witness-listed sources, original project/recipe/stage definitions and all frozen `inputs/`. The context policy reads **project/inputs/splits.json** and checks its hash **and byte count** against stage-00 `receipt.closure.inputs.splits`; it does not trust an arbitrary replacement reservation.

The pinned project assembler names the source family: policy/support/scenes/model/data/train/test_inference modules; portable_model/portable_policy; original model/train/data; frozen_gn; custody-validator; historical reservation, config and qualification. This is a locator, not authority to recreate missing original source bytes: the original witness governs. Its inputs include the train/validation dataset/splits and reserved-test dataset/splits. `train.py` copies input splits byte-for-byte into its artifacts; original project/packet/split custody must agree. The inspected project/train/data sources are unchanged between the recorded training commit and the pinned lab revision. [Project assembler](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/sealed-water-v1/project.py), [packet reader](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/sealed-water-v1/data.py).

At minimum retrieve these exact missing Run/artifact records, not a newly authored summary:

| Original owner file | Reported expected SHA256 |
|---|---|
| `runs/sealed-water-v1-3/managed/stage-00/run.json` | `9ef94e4a21148a45dc86ae1f0094a229360de87ade3a87aba4c0e6cf2ad7a872` |
| `runs/sealed-water-v1-3/managed/stage-00/artifacts/receipt.json` | `84674887178288e8eb6e48d3e6bf152113f24958357453c59c3a9029bd51fd7c` |
| `runs/sealed-water-v1-3/managed/stage-01/run.json` | `3a42ee63d1bbdc7fdee857a48a1af2a428ae4a36a10e377467a0accb0ffb24d7` |
| `runs/sealed-water-v1-3/managed/stage-01/artifacts/receipt.json` | `3928b248e314c3765d87795daa5a983f738affa0e8f6d7be0374a46c9cdcf5a7` |
| `runs/sealed-water-v1-3/project/source-witness.json` | `582d1f2c77cc2a37bf06035a868867b7118cd5933418a93b390bb92961fd8c56` |
| `runs/sealed-water-v1-3/project/qualification.json` | `7169fb713d2627bfc0d5999091263a4deede29f31369ac3c7eb937ed6138e735` |

Stage-00 is the completed/drained **train** Run, with `optimizer_updates=2000`. The exporter checks its artifact-name→hash/length inventory for weights, normalization, history and receipt, source-witness closure, and final checkpoint declaration. Retrieve its full original 12-artifact set for custody. Stage-01 is the completed/drained **CPU restore** Run, `optimizer_updates=0`; retrieve its `run.json`, full receipt and **predictions.npz**, preferably its full 12-artifact set, with original inventory. The declared artifacts are weights, predictions, normalization, splits, history, receipt, summary and five checkpoint files (000/100/500/1000/2000). None of their unavailable whole-file hashes is invented here. [Training/restore owner](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/sealed-water-v1/train.py).

For the broader reserved accuracy/custody record, also preserve stage-02's existing reserved CPU inference Run/artifacts and `packets/reserved_test`, original managed aggregate receipt, material/capture/evaluation/build records referenced by acceptance. Those are distinct from the minimal stage-01 train/validation parity case; do not substitute stage-02 predictions into that verifier invocation.

| Additional original custody identity | SHA256 |
|---|---|
| `runs/sealed-water-v1-3/managed/stage-02/run.json` | `05fcb4eef5da7aa45f4d5984ab7b5a5fdea9d958aec1202af342080f0a57a64a` |
| `runs/sealed-water-v1-3/managed/stage-02/artifacts/receipt.json` | `a4185d6eff3d3cf1ac18317ac0f30334ecd645acda092779023bca1590c2085f` |
| `runs/sealed-water-v1-3/managed/receipt.json` | `dced06fbf71407c39d2ab6d65ffe456c011a8cbf4b51072f92563a6a9ce008b3` |
| `runs/sealed-water-v1-3/material.json` | `e0d32352138b89917381364cb9dae8527bd0796ea7c1497c45bc8aa6d75d61fd` |
| `runs/sealed-water-v1-3/capture/manifest.json` | `41ca720cb153321b65dcee50b2c97564628026e9d2348a8b4b4308be5812294d` |

## 3. Exact existing CPU parity packet and prediction witness

Provide `runs/sealed-water-v1-3/packets/train_validation/dataset.npz` **and** its `splits.json`, plus original `dataset-receipt.json`/qualification custody. Individual whole-file hashes/lengths are not available here: obtain them from original `project/source-witness.json.inputs`, stage-00 `receipt.closure.inputs`, packet `splits.dataset_sha256` and packet receipt. Reservation hash `f5175321d24f4cb897b854ec4b60841d5c907613859cc4e2a60dfbb4720b59fa` and dataset-configuration hash `5a5731550caf3b2f4a30ac07926c34449e25f5f445e43e6e4e9c6c43c275562a` are different typed identities; neither is a substitute for a raw dataset/splits SHA.

The pinned verifier loads **stage-01/artifacts/receipt.json**, selects `cases[0]`, uses its `parent`, array-slot `frame` and `native_frame` to select the original base pressure and residual from that packet, and reads **stage-01/artifacts/predictions.npz** using receipt.prediction_inventory. Its comparison target is decoded `case_000`, flattened little-endian f64 pressure bytes. Historical case: counts `[16,16,16]`, native_frame15; expected raw pressure SHA256 `9af4715d737da4404093f8b80cf709ce9f676d364dd71148c160cd5873148361`. **That is not the whole predictions.npz SHA**, whose archive hash/length must come from the original Run inventory. Copy both packet files and both inference witness files; neither receipt nor an isolated pressure hash can recreate their arrays. [Exact verifier](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/verify_deployment.py).

Verify source itself was independently byte/Git-blob checked: SHA256 `bfed1dbccb50baee7a964ccdc47ed7ae40fc9c1c54a329fdf84f93dabc9af28c`, Git blob `1e48a6852ac409077d5ef9b8d558b247b0529270`. Its pinned source is [verify_deployment.py](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/verify_deployment.py); do not run the copied file from this arbitrary directory as if it were the original checkout: it resolves `experiments/sealed-water-v1/data.py` relative to its original lab layout.

## 4. Original retrieval coordinates and typed identity distinctions

Catalog `a7da4a8de53541a2a0e2aad08005bb71`; execution project `bb9315b6e176491bb6c1eb29a5e9dc89`; working scientific-field project `6460b40983b64da2a7ac21d40a6aa688`. Original Run IDs/material identities:

| Role | Run ID | Material SHA256 | Historical updates |
|---|---|---|---|
| train | `fa641802967e4d339ad2d0d82dd54b7c` | `f65b2965f055c95b7f504699e236c734467f5311bca74ad0f088cc270ef1bfd2` | 2000 CUDA |
| restore | `630e12fa31d348e7af584ce125a819bc` | `ddeea19a9bb2eb666b61c2d10a849eb7382e02c07b211c8a7dff56a215a03b48` | 0 CPU |
| test | `4ee8b8c359444202bccf3d67b320e622` | `f9de26dcd29be1a188446fc21eb1cc6f526b8745ce6bcc9b2a41d65c61780c3b` | 0 CPU |

All three were reported completed/drained, with 12/12/3 artifacts. Historical training hardware was an RTX5090 Laptop GPU; this is provenance, not a request to train or initialize CUDA now. Live comparison collection project `350d12fecfee4c51b38c75877896a58f`, record `899f4cc68ea64a91a4abc4ad9ac31b7e`, source hash `823c4782c531249f8d24c4d49f41090869ed29b088450dd31c837396a6bf9c72` retains diagnostic comparison fields. Historical live source also names Rheon lifecycle commit `66e50388ec7a788dbda0ad1227e330996edf506e`, and abbreviations `6c38078` (Rheon feature) / `d6a9292` (lab deployment); those abbreviations are retained as reported rather than invented full hashes. Run material hashes, record source hashes, catalog/project/Run IDs, source Git commits and file SHA256s are different identities. Ask the original owner/catalog for exact exports; no callable catalog download route or complete aa82 binary package was available in the bounded availability audit. [Historical training acceptance](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/sealed-water-pressure-v1/acceptance.json), [live acceptance](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/live-ai-pressure-comparison/reports/acceptance.json).

## 5. Exact original commands — proposals only, not executed

From the pinned lab checkout root, the original README's export arguments are:

```bash
.venv/bin/python experiments/live-water-comparison-v1/deployment.py \
  --project runs/sealed-water-v1-3/project \
  --artifacts runs/sealed-water-v1-3/managed/stage-00/artifacts \
  --run runs/sealed-water-v1-3/managed/stage-00/run.json \
  --runtime runs/cuda-pressure-runtime-1/manifest.json \
  --output runs/live-ai-comparison/deployment-NEW
```

The existing successful directory was deployment-2. NEW denotes a fresh output name; the exporter refuses an existing destination. These arguments are for exporting **already completed original material**, not rerunning training.

The exact documented parity invocation is:

```bash
.venv/bin/python experiments/live-water-comparison-v1/verify_deployment.py \
  --deployment runs/live-ai-comparison/deployment-NEW/deployment.json \
  --packet runs/sealed-water-v1-3/packets/train_validation \
  --inference runs/sealed-water-v1-3/managed/stage-01/artifacts \
  --output runs/live-ai-comparison/deployment-verification-NEW.json
```

Boundary checks, only after the actual owner material/runtime is supplied and execution is authorized:

```bash
.venv/bin/python -m unittest discover -s experiments/live-water-comparison-v1 -p 'test_*.py'
```

The verifier launches the descriptor's actual Python with `-I -S -B cpu_bridge.py --deployment <descriptor>`, PATH `/usr/bin:/bin`, empty CUDA_VISIBLE_DEVICES, TULDOK_TORCH_DEVICE=cpu, and OMP/MKL/OPENBLAS thread counts1. It sends the actual case request (ID16 for historical native_frame15), a zero-residual request, and ID1 cold request. Expected CPU float32 model/no CUDA initialization/no optimizer updates; first prediction bytes exact to case_000, zero-residual pressure exactly the prior/base bytes, and one unsupported_context rejection before cold inference. Exit2 is expected for that deliberate cold request, not a successful autonomous run. Historical stderr_bytes0 is recorded; the verifier reports stderr bytes rather than asserting they are always zero. Before executing, compare checkpoint/normalization/descriptor against known target hashes: a passing verifier with substituted mutually consistent inputs would not by itself identify aa82.

## 6. Scientific contract and acceptance limit

Context policy is native_prefix15, first physical prediction16, derived from original retained zero-based snapshot indices `[15,63]`. Fifteen is conditioning/warmup, not a15-frame recurrent neural input. The seven channels are residual/prior pressure/x/y/z/rho/dt; normalization remains train-only. The original bridge emits **full pressure** (previous pressure plus gauged correction once), not a correction for the host to add again. Water is rho1000 kg/m3, declared mu0.001 Pa·s, dt1/60, cubic cells; per-axis counts may differ. The legacy request has no viscosity/origin key: its metadata alone cannot prove the current executor's actual viscosity/origin compatibility. New admitted composition must preserve host physical/support/history gates. Protocol bounds (5–128 per axis,≤65536 cells,scalar h range1/128–1m) are not an accuracy claim beyond the recorded training population (5–16 cells/axis,8/16 cells per metre, sealed free-slip boxes).

Owner retrieval and exact parity can qualify honest operation **including failure**. Existing live status is comparison_workflow_verified_model_accuracy_failed:49 warm-start predictions, zero qualifying AI checks; reserved teacher evidence also has0/128 qualifying proposals. Preserve these outcomes and unsupported cold setup records. Do not lower gates, count shared native prefix as AI success, promise learned accuracy/speedup, or replace aa82 with the unrelated committed final/trajectory-v2 weights. This handoff supplies no second compatible measured revision identity: A1's actual revision/consumer qualification and current executor admission still require their named evidence. No new training or fabricated accuracy pass is needed or authorized.

## Evidence files and unknowns

[compact checklist](measured-a1-owner-inputs.json) is the compact machine-readable checklist. Its reported-known hash maps come from pinned acceptance/export copy semantics; fields for unavailable owner inventory deliberately use null until original files/receipts provide them. The checkpoint-2000 equality is separately marked as derived. Source-backed code examples are not recreated original data or inventories.

The eleven original raw evidence inputs and lengths/hashes are retained under `/tmp/rheon-measured-a1-availability-20261016/availability-receipt.json`; all eleven were rehashed for this handoff. Additional pinned text-only bridge/project/data/train/model sources are recorded in [supplemental source receipt](measured-a1-owner-inputs-sources.json). No source/model execution or binary download occurred. Root independently rechecked the eleven primary sources, five supplemental source hashes/Git blobs, exact verifier and original training source identity. This documentation records retrieval requirements; no missing owner bytes or A1 outcome were verified.
