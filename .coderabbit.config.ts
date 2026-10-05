import { defineConfig } from "@coderabbitai/config";

export default defineConfig((context) => {
  if (context.repo.fullName !== "MrScripty/Rheon" || context.pr?.number !== 17) {
    return {};
  }

  return {
    reviews: {
      review_details: true,
      path_filters: [
        // Preserve the existing default exclusions; these are not part of the 33 JSON files.
        "!**/*.log",
        "!**/*.png",
        "!**/*.csv",

        // PR #17 only: the 33 approved generated JSON outputs, listed exactly.
        "!evidence/free-surface/demo/complete.json",
        "!evidence/free-surface/demo/jacobi-pcg-v1/pulse.json",
        "!evidence/free-surface/demo/sgs-pcg-v1/pulse.json",
        "!evidence/free-surface/legacy-replay/demo-16/run.json",
        "!evidence/free-surface/legacy-replay/demo-64/run.json",
        "!evidence/free-surface/legacy-replay/demo-plume/run.json",
        "!evidence/free-surface/legacy-replay/receipt.json",
        "!evidence/liquid-composition/legacy-replay/demo-16/run.json",
        "!evidence/liquid-composition/legacy-replay/demo-64/run.json",
        "!evidence/liquid-composition/legacy-replay/demo-plume/run.json",
        "!evidence/liquid-composition/legacy-replay/receipt.json",
        "!evidence/liquid-step/demo/complete.json",
        "!evidence/liquid-step/demo/jacobi-pcg-v1-n16-c025/run.json",
        "!evidence/liquid-step/demo/jacobi-pcg-v1-n32-c025/run.json",
        "!evidence/liquid-step/demo/jacobi-pcg-v1-n64-c0125/run.json",
        "!evidence/liquid-step/demo/jacobi-pcg-v1-n64-c025/run.json",
        "!evidence/liquid-step/demo/sgs-pcg-v1-n16-c025/run.json",
        "!evidence/liquid-step/demo/sgs-pcg-v1-n32-c025/run.json",
        "!evidence/liquid-step/demo/sgs-pcg-v1-n64-c0125/run.json",
        "!evidence/liquid-step/demo/sgs-pcg-v1-n64-c025/run.json",
        "!evidence/liquid-step/legacy-replay/demo-16/run.json",
        "!evidence/liquid-step/legacy-replay/demo-64/run.json",
        "!evidence/liquid-step/legacy-replay/demo-plume/run.json",
        "!evidence/liquid-step/legacy-replay/receipt.json",
        "!evidence/liquid-volume/legacy-replay/demo-16/run.json",
        "!evidence/liquid-volume/legacy-replay/demo-64/run.json",
        "!evidence/liquid-volume/legacy-replay/demo-plume/run.json",
        "!evidence/liquid-volume/legacy-replay/receipt.json",
        "!evidence/liquid-volume/pre-area-guard-release/legacy-replay/demo-16/run.json",
        "!evidence/liquid-volume/pre-area-guard-release/legacy-replay/demo-64/run.json",
        "!evidence/liquid-volume/pre-area-guard-release/legacy-replay/demo-plume/run.json",
        "!evidence/liquid-volume/pre-area-guard-release/legacy-replay/receipt.json",
        "!evidence/liquid-volume/pre-area-guard-release/provenance.json",
      ],
    },
  };
});
