#!/usr/bin/env python3
"""Build/execute exact native source or recheck externally pinned I1 artifacts."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import tarfile

import oracle

HERE = Path(__file__).resolve().parent
FIXTURES = HERE / "fixtures.json"


def full_oid(value):
    oracle.require(isinstance(value, str) and re.fullmatch(r"[0-9a-f]{40}", value), "source-identity", "full lowercase 40-digit source commit required")
    return value


def output_directory(path, repo):
    out = Path(path).resolve()
    oracle.require(not out.is_relative_to(repo.resolve()), "output-policy", "fresh output must be outside checkout")
    oracle.require(not out.exists(), "fresh-output", "output already exists; refusing overwrite")
    out.mkdir(parents=True, exist_ok=False)
    return out


def write_json(path, data):
    temporary = path.with_name(path.name + ".partial")
    with temporary.open("x") as stream:
        json.dump(data, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")
        stream.flush(); os.fsync(stream.fileno())
    os.replace(temporary, path)


def command(arguments, cwd, out, name, env=None):
    stdout = out / (name + ".stdout"); stderr = out / (name + ".stderr")
    with stdout.open("xb") as so, stderr.open("xb") as se:
        result = subprocess.run(arguments, cwd=cwd, env=env, stdout=so, stderr=se, check=False)
    observation = {"argv": list(map(str, arguments)), "cwd": str(cwd), "returncode": result.returncode,
                   "stdout": {"path": str(stdout), "sha256": oracle.digest(stdout)},
                   "stderr": {"path": str(stderr), "sha256": oracle.digest(stderr)}}
    return observation


def git(repo, *arguments):
    result = subprocess.run(["git", "-C", str(repo), *arguments], capture_output=True, text=True, check=False)
    oracle.require(result.returncode == 0, "source-availability", "git failed: " + " ".join(arguments), "unavailable")
    return result.stdout.strip()


def _verify_origin(origin, origin_sha256, source, record_path, fixtures_sha256, fixture_id, repo):
    oracle.require(isinstance(origin_sha256, str) and re.fullmatch(r"[0-9a-f]{64}", origin_sha256), "origin-pin", "parent-supplied origin SHA256 required")
    oracle.require(oracle.digest(origin) == origin_sha256, "origin-integrity", "origin hash differs from externally supplied pin")
    obj = oracle.read_json(origin)
    oracle.require(obj.get("schema") == "rheon-submarine-i1-origin-v1" and obj.get("source_commit") == source,
                   "origin-source", "wrong origin schema/source")
    oracle.require(obj.get("terminal") == "completed", "origin-terminal", "capture is not durably completed")
    oracle.require(obj.get("fixtures_sha256") == fixtures_sha256, "origin-fixture", "origin fixture mismatch")
    oracle.require(obj.get("record_schema") == oracle.SCHEMA and obj.get("operator_profile") == oracle.PROFILE,
                   "origin-profile", "origin record schema/operator profile mismatch")
    oracle.require(re.fullmatch(r"[0-9a-f]{40}", obj.get("source_tree", "")) and obj.get("dirty") is False,
                   "origin-source", "missing clean source tree identity")
    oracle.require(obj["source_tree"] == git(repo, "rev-parse", source+"^{tree}"), "origin-source", "origin tree differs from actual pinned source")
    oracle.require(oracle.digest(obj["source_archive_path"]) == obj["source_archive_sha256"], "origin-source-archive", "source archive bytes mismatch")
    expected_archive = subprocess.run(["git", "-C", str(repo), "archive", "--format=tar", source], capture_output=True, check=False)
    oracle.require(expected_archive.returncode == 0 and hashlib.sha256(expected_archive.stdout).hexdigest() == obj["source_archive_sha256"], "origin-source-archive", "archive differs from actual source commit")
    exe = obj.get("executable", {})
    oracle.require(oracle.digest(exe["path"]) == exe.get("sha256"), "origin-executable", "executable bytes mismatch")
    record = obj.get("outputs", {}).get(fixture_id)
    oracle.require(isinstance(record, dict) and record.get("sha256") == oracle.digest(record_path), "origin-output", "actual record differs from captured output")
    oracle.require(Path(record["path"]).resolve() == Path(record_path).resolve(), "origin-output", "wrong output association")
    commands = obj.get("commands", [])
    oracle.require(commands and all(c.get("returncode") == 0 for c in commands), "origin-command", "missing/failed capture command")
    matching = [c for c in commands if "--case" in c["argv"] and c["argv"].index("--case")+1 < len(c["argv"]) and c["argv"][c["argv"].index("--case")+1] == fixture_id]
    oracle.require(len(matching) == 1, "origin-command", "no unique actual native case invocation")
    case_cmd = matching[0]["argv"]
    oracle.require(case_cmd[0] == exe["path"] and "--output" in case_cmd and "--fixtures" in case_cmd,
                   "origin-command", "case command missing binary/output/fixture binding")
    oracle.require(Path(case_cmd[case_cmd.index("--output")+1]).resolve() == Path(record_path).resolve().parent,
                   "origin-command", "case invocation output mismatch")
    oracle.require(oracle.digest(case_cmd[case_cmd.index("--fixtures")+1]) == fixtures_sha256,
                   "origin-command", "invoked fixture bytes mismatch")
    oracle.require(obj.get("validator", {}).get("files") == {p.name: oracle.digest(p) for p in [HERE/"run.py", HERE/"oracle.py", FIXTURES]}, "origin-validator", "capturing validator differs from current verifier files")
    for path, expected in obj.get("cargo_home_configs", {}).items():
        oracle.require(oracle.digest(path) == expected, "origin-build-config", "cargo-home config bytes changed")
    for c in commands:
        for label in ("stdout", "stderr"):
            oracle.require(oracle.digest(c[label]["path"]) == c[label]["sha256"], "origin-command-output", "command stream bytes mismatch")
    return {"status": "externally-pinned-origin-integrity-checked", "origin_sha256": origin_sha256,
            "assurance": "parent pin supplies provenance authority; hashes alone do not authenticate origin"}


def verify_origin(origin, origin_sha256, source, record_path, fixtures_sha256, fixture_id, repo):
    try:
        return _verify_origin(origin, origin_sha256, source, record_path, fixtures_sha256, fixture_id, repo)
    except (KeyError, TypeError, ValueError, AttributeError, IndexError) as exc:
        raise oracle.Failure("invalid", "origin-profile", "malformed origin member: " + str(exc)) from exc


def build_environment(out):
    # Remove ambient compiler/wrapper/feature overrides and credentials.
    allowed = ("PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR", "TEMP", "TMP", "LANG", "LC_ALL", "SYSTEMROOT")
    env = {key: os.environ[key] for key in allowed if key in os.environ}
    env.update({"CARGO_TARGET_DIR": str(out / "target"), "CARGO_BUILD_JOBS": "1", "RUSTUP_TOOLCHAIN": "1.92.0"})
    return env


def execute(args, repo, fixture_data, fixture_sha):
    source = full_oid(args.source)
    oracle.require(git(repo, "rev-parse", source+"^{commit}") == source, "source-identity", "source is not the pinned commit")
    example = git(repo, "ls-tree", "--name-only", source, "examples/submarine_stokes.rs")
    oracle.require(example == "examples/submarine_stokes.rs", "native-capability", "native submarine_stokes example absent at pinned source", "unavailable")
    oracle.require(args.native_contract_sha256 == fixture_sha, "contract-admission", "parent/native export agreement requires explicit fixture SHA256", "unavailable")
    cases = args.case or list(fixture_data["cases"])
    for case in cases:
        oracle.require(case in fixture_data["cases"], "fixture-id", "unknown fixture: " + case, "unsupported")
    oracle.require(len(set(cases)) == len(cases), "fixture-id", "duplicate requested case")
    out = output_directory(args.output, repo)
    manifest = {"schema": "rheon-submarine-i1-origin-v1", "terminal": "active", "source_commit": source,
                "record_schema": oracle.SCHEMA, "operator_profile": oracle.PROFILE,
                "source_tree": git(repo, "rev-parse", source+"^{tree}"), "dirty": False,
                "source_scope": "tracked-only-git-archive", "fixtures_sha256": fixture_sha,
                "executor": {"os": platform.platform(), "machine": platform.machine(), "python": sys.version},
                "validator": {"commit": git(repo, "rev-parse", "HEAD"), "dirty": bool(git(repo, "status", "--porcelain")), "files": {p.name: oracle.digest(p) for p in [HERE/"run.py", HERE/"oracle.py", FIXTURES]}},
                "commands": [], "outputs": {}}
    manifest_path = out / "origin.json"; write_json(manifest_path, manifest)
    try:
        archive = out / "source.tar"
        with archive.open("xb") as stream:
            result = subprocess.run(["git", "-C", str(repo), "archive", "--format=tar", source], stdout=stream, check=False)
        oracle.require(result.returncode == 0, "source-archive", "git archive failed", "unavailable")
        manifest["source_archive_sha256"] = oracle.digest(archive)
        manifest["source_archive_path"] = str(archive)
        snapshot = out / "source"; snapshot.mkdir()
        with tarfile.open(archive) as tar:
            oracle.require(all(not member.issym() and not member.islnk() for member in tar.getmembers()), "source-archive", "source archive contains unsupported links", "unsupported")
            tar.extractall(snapshot, filter="data")
        fixture_copy = out / "fixtures.json"; fixture_copy.write_bytes(FIXTURES.read_bytes())
        env = build_environment(out)
        manifest["build_environment"] = env
        cargo_home = Path(env.get("CARGO_HOME", str(Path(env.get("HOME", ""))/".cargo")))
        manifest["cargo_home_configs"] = {str(p): oracle.digest(p) for p in [cargo_home/"config", cargo_home/"config.toml"] if p.is_file()}
        for label, argv in [("rustc", ["rustc", "--version"]), ("cargo", ["cargo", "--version"]),
                            ("build", ["cargo", "build", "--verbose", "--locked", "--release", "--no-default-features", "--example", "submarine_stokes"])]:
            c = command(argv, snapshot, out, label, env); manifest["commands"].append(c)
            oracle.require(c["returncode"] == 0, "native-build", label+" command failed; streams retained", "unavailable")
            if label == "rustc":
                oracle.require(Path(c["stdout"]["path"]).read_text().startswith("rustc 1.92.0 "), "toolchain", "actual compiler is not pinned Rust 1.92.0", "unsupported")
        executable = out / "target/release/examples/submarine_stokes"
        oracle.require(executable.is_file(), "native-build", "native executable missing", "unavailable")
        manifest["executable"] = {"path": str(executable), "sha256": oracle.digest(executable), "bytes": executable.stat().st_size}
        assessments = {}
        for i, case in enumerate(cases):
            case_out = out / "cases" / case
            c = command([str(executable), "--fixtures", str(fixture_copy), "--case", case, "--output", str(case_out)], snapshot, out, "case-"+str(i), env)
            manifest["commands"].append(c)
            oracle.require(c["returncode"] == 0, "native-execution", "native case refused/failed: "+case, "unavailable")
            record_path = case_out / "record.json"
            record = oracle.read_json(record_path)
            oracle.require(record.get("fixture_id") == case, "fixture-id", "native record case mismatch")
            manifest["outputs"][case] = {"path": str(record_path), "sha256": oracle.digest(record_path), "bytes": record_path.stat().st_size}
            assessments[case] = oracle.validate_record(record, fixture_data["cases"][case], fixture_sha)
            manifest["terminal"] = "active"; write_json(manifest_path, manifest)
        manifest["terminal"] = "completed"; write_json(manifest_path, manifest)
        report = {"schema": "rheon-submarine-i1-assessment-v1", "status": "validated-discrete-captures", "source": source,
                  "origin_sha256": oracle.digest(manifest_path), "cases": assessments,
                  "N1": "pending-complete-native-controls-and-independent-review", "N2": "pending-complete-native-controls-and-independent-review",
                  "N3": "pending-continuum-mesh-domain-family", "N7": "pending-experiment-rights-model-uncertainty"}
        write_json(out / "assessment.json", report)
        return report
    except BaseException as exc:
        manifest["terminal"] = "cancelled" if isinstance(exc, KeyboardInterrupt) else "failed"
        manifest["failure"] = exc.data if isinstance(exc, oracle.Failure) else {"detail": type(exc).__name__+": "+str(exc)}
        write_json(manifest_path, manifest)
        raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", required=True)
    parser.add_argument("--source", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--repo", type=Path, default=HERE.parents[1])
    parser.add_argument("--case", action="append")
    parser.add_argument("--input", type=Path)
    parser.add_argument("--origin", type=Path)
    parser.add_argument("--origin-sha256")
    parser.add_argument("--native-contract-sha256")
    args = parser.parse_args(argv)
    try:
        oracle.require(args.suite == "stokes", "suite", "only steady aligned-cuboid Stokes is supported", "unsupported")
        source = full_oid(args.source)
        fixture_data = oracle.read_json(FIXTURES); fixture_sha = oracle.digest(FIXTURES)
        if args.input:
            oracle.require(not args.case, "profile", "analysis uses record fixture identity, not --case")
            record = oracle.read_json(args.input); case = record.get("fixture_id")
            oracle.require(isinstance(case, str) and case in fixture_data["cases"], "fixture-id", "unknown record fixture", "unsupported")
            report = oracle.validate_record(record, fixture_data["cases"][case], fixture_sha)
            if args.origin:
                report["provenance"] = verify_origin(args.origin, args.origin_sha256, source, args.input, fixture_sha, case, args.repo)
            else:
                oracle.require(args.origin_sha256 is None, "origin-pin", "origin pin without manifest")
                report["provenance"] = "unavailable; numerical properties only"
            out = output_directory(args.output, args.repo)
            write_json(out / "assessment.json", report)
        else:
            oracle.require(args.origin is None and args.origin_sha256 is None, "profile", "origin arguments require --input")
            report = execute(args, args.repo, fixture_data, fixture_sha)
        print(json.dumps(report, sort_keys=True, allow_nan=False))
        return 0
    except oracle.Failure as exc:
        print(json.dumps({"status": "failed", **exc.data}, sort_keys=True), file=sys.stderr)
        return 2
    except (OSError, KeyError, ValueError, TypeError) as exc:
        print(json.dumps({"status": "failed", "failure_kind": "unavailable", "claim_id": "execution", "detail": str(exc)}, sort_keys=True), file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print(json.dumps({"status": "cancelled", "failure_kind": "unavailable", "claim_id": "execution"}), file=sys.stderr)
        return 130


if __name__ == "__main__":
    sys.exit(main())
