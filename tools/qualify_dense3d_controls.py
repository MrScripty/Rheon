"""Fresh source-bound metadata qualification; never invokes the exporter pilot."""
import argparse
from fractions import Fraction
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
BASE = "fee7b4a139574f87b259796b1ba8698a41d31ac1"
V1_SHA256 = "063d17a3bbc92a27c26c0f5dd4588a9478091ebbf53ce0ccb818cee6a9393b98"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_hashes():
    return {name: digest(ROOT / name) for name in git("ls-files").splitlines()}


def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate native JSON key")
        result[key] = value
    return result


def rational_json(raw):
    return json.loads(raw, parse_float=Fraction, object_pairs_hook=unique_pairs,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))


def numeric_tree(value):
    if type(value) is list:
        return all(numeric_tree(item) for item in value)
    return type(value) in (int, Fraction)


def compare_tiny(directory):
    """Check actual tiny accepted captures without fabricating a v2 run binding."""
    raw_frames = (directory / "frames.jsonl").read_bytes()
    raw_controls = (directory / "intervals.json").read_bytes()
    require(raw_frames.endswith(b"\n"), "tiny JSONL final newline")
    frames = [rational_json(line) for line in raw_frames.splitlines()]
    intervals = rational_json(raw_controls)
    require(len(frames) == 9 and len(intervals) == 8, "tiny accepted roster")
    for index, frame in enumerate(frames):
        require(type(frame["frame"]) is int and frame["frame"] == index, "tiny frame order")
    keys = {"start_frame", "end_frame", "start_time_s", "end_time_s", "dt_s",
            "carrier_before", "carrier_after", "liquid_before", "liquid_after",
            "boundary_stamp", "inlet_stamp", "outward_speed_m_s", "inlet_fraction",
            "source_mode", "source_rate_m3_s", "body_acceleration_m_s2"}
    u64 = re.compile(r"(?:0|[1-9][0-9]{0,19})\Z")
    for k, record in enumerate(intervals, 1):
        before, after = frames[k - 1:k + 1]
        require(type(record) is dict and record.keys() == keys, "tiny interval keys")
        require(type(record["start_frame"]) is int and record["start_frame"] == k - 1,
                "tiny start frame")
        require(type(record["end_frame"]) is int and record["end_frame"] == k, "tiny end frame")
        for name, expected in (("start_time_s", before["time_s"]),
                               ("end_time_s", after["time_s"]), ("dt_s", after["dt_s"])):
            require(type(record[name]) is type(expected) and record[name] == expected,
                    "tiny adjacent clock " + name)
        require(record["dt_s"] == Fraction(1, 16)
                and record["end_time_s"] - record["start_time_s"] == record["dt_s"],
                "tiny actual accepted interval")
        for prefix, field in (("carrier", "carrier_stamp"), ("liquid", "liquid_stamp")):
            for suffix, frame, version in (("before", before, k - 1), ("after", after, k)):
                stamp = record[prefix + "_" + suffix]
                require(stamp == frame[field] and stamp.keys() == {"id", "version"},
                        "tiny adjacent stamp")
                for value in stamp.values():
                    require(type(value) is str and u64.fullmatch(value) is not None
                            and int(value) < 1 << 64, "tiny canonical u64")
                require(stamp["version"] == str(version), "tiny accepted version")
        require(record["boundary_stamp"] == {"id": "47", "version": "0"}, "tiny boundary")
        require(record["inlet_stamp"] == {"id": "53", "version": "0"}, "tiny inlet")
        for key in ("outward_speed_m_s", "inlet_fraction", "source_rate_m3_s", "body_acceleration_m_s2"):
            require(numeric_tree(record[key]), "tiny numeric control " + key)
        require(record["outward_speed_m_s"] == [[Fraction(-1, 4), Fraction(1, 4)], [0, 0], [0, 0]],
                "tiny native outward speeds")
        require(record["inlet_fraction"] == [[0, 0], [0, 0], [0, 0]], "tiny native inlet")
        require(record["source_mode"] == "none" and record["source_rate_m3_s"] == 0
                and record["body_acceleration_m_s2"] == [0, 0, 0], "tiny explicit absence")
    return {"scope": "actual tiny three-cell accepted interval association only",
            "geometry_counts": [3, 1, 1], "frames": len(frames), "intervals": len(intervals),
            "full_fixed_case_v2_dataset": False, "training_qualification": False,
            "frames_sha256": hashlib.sha256(raw_frames).hexdigest(),
            "intervals_sha256": hashlib.sha256(raw_controls).hexdigest()}


def qualify(output):
    output = output.absolute()
    require(not output.exists(), "fresh evidence directory required")
    require(not git("status", "--porcelain", "--untracked-files=all"), "clean source required")
    subprocess.run(["git", "merge-base", "--is-ancestor", BASE, "HEAD"], cwd=ROOT, check=True)
    require(digest(ROOT / "tools/import_dense3d_sequence.py") == V1_SHA256, "v1 reader changed")
    original = git("ls-tree", "-r", "--name-only", BASE).splitlines()
    protected = [p for p in original if p.startswith("src/") and p.endswith(".rs")
                 and p != "src/dense3d_sequence.rs"]
    protected += ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tools/import_dense3d_sequence.py"]
    for name in protected:
        prior = subprocess.check_output(["git", "show", BASE + ":" + name], cwd=ROOT)
        require((ROOT / name).read_bytes() == prior, "protected source changed: " + name)
    output.mkdir(parents=True)
    head, tree, hashes = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}"), source_hashes()
    env = os.environ.copy()
    env["RHEON_TINY_CONTROLS_EVIDENCE"] = str(output / "native-tiny")
    commands = [
        ["cargo", "build", "--locked", "--no-default-features", "--example", "dense3d_sequence"],
        ["cargo", "test", "--locked", "--no-default-features", "--test", "dense3d_controls_contract"],
        ["cargo", "test", "--locked", "--no-default-features", "--test", "dense3d_sequence_contract", "--",
         "--skip", "nine_frames_round_trip_every_accepted_native_bit_and_decimal_u64_stamp"],
        ["cargo", "clippy", "--locked", "--no-default-features", "--example", "dense3d_sequence",
         "--test", "dense3d_controls_contract", "--test", "dense3d_sequence_contract", "--", "-D", "warnings"],
        ["rustfmt", "--edition", "2024", "--check", "src/dense3d_sequence.rs",
         "examples/dense3d_sequence.rs", "tests/dense3d_controls_contract.rs", "tests/dense3d_sequence_contract.rs"],
        [sys.executable, "-m", "unittest", "discover", "-s", "tools", "-p", "test_dense3d*.py"],
        [sys.executable, "-O", "-m", "unittest", "discover", "-s", "tools", "-p", "test_dense3d*.py"],
    ]
    receipts = []
    for index, command in enumerate(commands):
        log = output / (str(index) + ".log")
        with log.open("wb") as stream:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream,
                                    stderr=subprocess.STDOUT, timeout=300)
        receipts.append({"command": command, "returncode": result.returncode,
                         "log": log.name, "log_sha256": digest(log)})
        require(result.returncode == 0, "qualification command failed: " + log.name)
    tiny = compare_tiny(output / "native-tiny")
    require(hashes == source_hashes() and head == git("rev-parse", "HEAD")
            and not git("status", "--porcelain", "--untracked-files=all"), "source changed during qualification")
    artifacts = {str(p.relative_to(output)): digest(p) for p in sorted(output.rglob("*")) if p.is_file()}
    receipt = {"source_commit": head, "source_tree": tree, "base_commit": BASE, "source_clean": True,
               "source_sha256": hashes, "protected_source_files": protected,
               "commands": receipts, "artifacts_sha256": artifacts, "tiny_native": tiny,
               "v1_reader_sha256": V1_SHA256, "new_full_pilot_executed": False,
               "scope": "producer metadata implementation with native tiny and authored static tests"}
    (output / "qualification.json").write_text(json.dumps(receipt, indent=2, allow_nan=False) + "\n")
    return {"output": str(output), "source_commit": head, "source_tree": tree,
            "commands": len(commands), "tiny_native": tiny}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    print(json.dumps(qualify(parser.parse_args().output), indent=2, allow_nan=False))
