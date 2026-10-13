#!/usr/bin/env python3
"""Read Rust fixture exports with actual VTK, not a local XML parser.

Generate first: cargo run --locked --example vtk_export_fixture -- /tmp/rheon-vtk
Then: python tools/verify_vtk_roundtrip.py /tmp/rheon-vtk
Consumer qualification dependency: vtk==9.3.1 (includes NumPy dependency).
Metadata *_utf8 fields are UInt8 bytes, without a trailing NUL.
"""
import json
from pathlib import Path
import sys

import numpy as np
import vtk

if not __debug__:
    raise SystemExit("VTK qualification requires assertions; run Python without -O")
from vtk.util.numpy_support import vtk_to_numpy


def read(path, reader_type):
    reader = reader_type()
    errors = []
    reader.AddObserver(vtk.vtkCommand.ErrorEvent, lambda *_: errors.append(str(path)))
    reader.SetFileName(str(path))
    reader.Update()
    assert not errors, f"VTK rejected {path}"
    assert reader.GetErrorCode() == 0, f"VTK error code reading {path}"
    return reader.GetOutput()


def array(owner, name):
    result = owner.GetArray(name)
    assert result is not None, f"Missing array {name}"
    return vtk_to_numpy(result)


def exact(actual, expected, dtype):
    expected = np.asarray(expected, dtype=dtype)
    assert actual.dtype == expected.dtype, (actual.dtype, expected.dtype)
    assert actual.shape == expected.shape, (actual.shape, expected.shape)
    # Bit comparison also preserves signed zero and the low bits of Float64.
    assert actual.tobytes() == expected.tobytes(), (actual, expected)


def metadata(data, generation, time):
    field = data.GetFieldData()
    exact(array(field, "run_id"), [2**64 - 1], np.uint64)
    exact(array(field, "generation"), [generation], np.uint64)
    exact(array(field, "TimeValue"), [time], np.float64)
    exact(array(field, "rheon_vtk_contract_version"), [1], np.uint32)
    assert array(field, "source_revision_utf8").tobytes().decode("utf-8") == "fixture<&\"'>–源"
    assert array(field, "coordinate_frame_utf8").tobytes().decode().startswith("lab; Cartesian XYZ")


def verify(directory):
    expected = json.loads((directory / "expected-session.json").read_text())
    accepted = read(directory / "accepted.vti", vtk.vtkXMLImageDataReader)
    constructor = read(directory / "constructor.vti", vtk.vtkXMLImageDataReader)
    for data in (accepted, constructor):
        assert data.GetDimensions() == (4, 3, 5)
        assert data.GetNumberOfCells() == 24
        assert data.GetOrigin() == (-1.25, 2.5, -0.75)
        assert data.GetSpacing() == (0.25, 0.5, 0.125)
        # Independently selected cell (i=2,j=1,k=3), x-fast index 23.
        assert data.GetCell(23).GetBounds() == (-0.75, -0.5, 3.0, 3.5, -0.375, -0.25)
    metadata(constructor, 0, 0.0)
    assert constructor.GetCellData().GetArray("interval_pressure_pa") is None
    assert constructor.GetFieldData().GetArray("pressure_interval_s") is None
    assert constructor.GetFieldData().GetArray("pressure_gauge_cell") is None
    assert array(constructor.GetFieldData(), "pressure_semantics_utf8").tobytes().decode().startswith("unavailable:")
    exact(array(constructor.GetCellData(), "fraction"), [(i + 1) / 32 for i in range(24)], np.float64)
    metadata(accepted, 1, expected["time"])
    field, cells = accepted.GetFieldData(), accepted.GetCellData()
    assert array(field, "stage_utf8").tobytes() == b"accepted"
    exact(array(field, "pressure_interval_s"), [[expected["time"] - expected["dt"], expected["time"]]], np.float64)
    exact(array(field, "pressure_gauge_cell"), [0], np.uint64)
    exact(array(field, "carrier_stamp"), [[2**64 - 3, 1]], np.uint64)
    exact(array(field, "volume_stamp"), [[2**64 - 2, 1]], np.uint64)
    for name in ("fraction", "pressure", "tracer"):
        label = "interval_pressure_pa" if name == "pressure" else name
        dtype = np.float32 if name == "tracer" else np.float64
        exact(array(cells, label), expected[name], dtype)
    assert array(cells, "interval_pressure_pa")[0] == 0.0
    assert np.ptp(array(cells, "tracer")) > 0, "Fixture must exercise nonuniform tracer"
    for axis, dimensions, offset in (("x", [4, 2, 4], [0, .5, .5]), ("y", [3, 3, 4], [.5, 0, .5]), ("z", [3, 2, 5], [.5, .5, 0])):
        exact(array(field, f"velocity_{axis}"), expected[axis], np.float32)
        exact(array(field, f"velocity_{axis}_dimensions"), [dimensions], np.uint64)
        exact(array(field, f"velocity_{axis}_offset"), [offset], np.float64)
    # Independent NumPy reconstruction from the native arrays, with distinct
    # dimensions and component directions; verifies offsets and ordering.
    x = np.asarray(expected["x"], dtype=np.float32).astype(np.float64).reshape((4, 2, 4))
    y = np.asarray(expected["y"], dtype=np.float32).astype(np.float64).reshape((4, 3, 3))
    z = np.asarray(expected["z"], dtype=np.float32).astype(np.float64).reshape((5, 2, 3))
    velocity = np.stack(((x[:, :, :-1] + x[:, :, 1:]) * .5,
                         (y[:, :-1, :] + y[:, 1:, :]) * .5,
                         (z[:-1, :, :] + z[1:, :, :]) * .5), axis=-1).reshape((-1, 3))
    exact(array(cells, "velocity_cell_m_s"), velocity, np.float64)
    particles = read(directory / "particles.vtp", vtk.vtkXMLPolyDataReader)
    initial = read(directory / "particles-initial.vtp", vtk.vtkXMLPolyDataReader)
    empty = read(directory / "particles-empty.vtp", vtk.vtkXMLPolyDataReader)
    assert empty.GetNumberOfPoints() == empty.GetNumberOfVerts() == 0
    metadata(empty, 0, 0.0)
    metadata(initial, 0, 0.0)
    metadata(particles, 17, 1.25)
    for data in (initial, empty):
        assert data.GetFieldData().GetArray("preintegration_pressure_pa") is None
        assert data.GetFieldData().GetArray("diagnostic_time_s") is None
        assert array(data.GetFieldData(), "diagnostic_semantics_utf8").tobytes().decode().startswith("unavailable:")
    assert particles.GetNumberOfPoints() == particles.GetNumberOfVerts() == 2
    for i in range(2):
        assert particles.GetCell(i).GetPointId(0) == i
    exact(vtk_to_numpy(particles.GetPoints().GetData()), [[-.125, 2.75, 1.2345678901234567], [.5, -1, 2]], np.float64)
    points, field = particles.GetPointData(), particles.GetFieldData()
    exact(array(points, "particle_id"), [0, 1], np.uint64)
    exact(array(points, "material"), [0, 1], np.uint8)
    exact(array(points, "velocity_m_s"), [[.12345678901234567, -2, 3], [-0.0, 1/3, -4.25]], np.float64)
    exact(array(points, "mass_kg"), [.12345678901234567, 2.5], np.float64)
    exact(array(points, "temperature_k"), [291.125, 302.625], np.float64)
    assert points.GetArray("preintegration_pressure_pa") is None
    exact(array(field, "diagnostic_time_s"), [1.125], np.float64)
    exact(array(field, "preintegration_density_kg_m3"), [998.1234567890123, 1.25], np.float64)
    exact(array(field, "preintegration_pressure_pa"), [12.123456789012345, 101325.125], np.float64)
    exact(array(field, "preintegration_acceleration_m_s2"), [[-9.81, 0, 1/7], [1, -2, 3]], np.float64)
    semantics = array(field, "diagnostic_semantics_utf8").tobytes().decode()
    assert "pre-integration" in semantics and "not sampled at accepted positions" in semantics
    for data, stage in ((initial, b"constructor"), (particles, b"accepted")):
        assert array(data.GetFieldData(), "stage_utf8").tobytes() == stage
    # Actual canonical MaterialSession publication, independent of the authored
    # particle fixture. Expected values are dumped directly from native accessors.
    expected_material = json.loads((directory / "expected-material.json").read_text())
    material_initial = read(directory / "material-constructor.vtp", vtk.vtkXMLPolyDataReader)
    material = read(directory / "material-accepted.vtp", vtk.vtkXMLPolyDataReader)
    metadata(material_initial, 0, 0.0)
    metadata(material, 1, expected_material["time"])
    assert array(material_initial.GetFieldData(), "stage_utf8").tobytes() == b"constructor"
    assert material_initial.GetFieldData().GetArray("preintegration_pressure_pa") is None
    assert material.GetNumberOfPoints() == 64
    assert array(material.GetFieldData(), "stage_utf8").tobytes() == b"accepted"
    exact(vtk_to_numpy(material.GetPoints().GetData()), expected_material["position"], np.float64)
    for name, label in (("velocity", "velocity_m_s"), ("mass", "mass_kg"), ("temperature", "temperature_k"), ("material", "material")):
        exact(array(material.GetPointData(), label), expected_material[name], np.uint8 if name == "material" else np.float64)
    for name, label in (("density", "preintegration_density_kg_m3"), ("pressure", "preintegration_pressure_pa"), ("acceleration", "preintegration_acceleration_m_s2")):
        exact(array(material.GetFieldData(), label), expected_material[name], np.float64)
        assert material.GetPointData().GetArray(label) is None
    exact(array(material.GetFieldData(), "diagnostic_time_s"), [expected_material["diagnostic_time"]], np.float64)
    assert expected_material["diagnostic_time"] < expected_material["time"]
    print(f"VTK {vtk.vtkVersion.GetVTKVersion()}: seven files read; native precision, staggered ordering, gauge, stage, identity, optional diagnostics and canonical MaterialSession passed")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: verify_vtk_roundtrip.py EXTERNAL_FIXTURE_DIRECTORY")
    verify(Path(sys.argv[1]))
