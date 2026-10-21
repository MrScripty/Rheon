"""Independent VTK9.3.1 reader compares two cold exports with SQLite native bits.

Usage: python verify_recorded_carrier_vtk.py RUN.sqlite CONSTRUCTOR.vti STEP1.vti
The Rust example must first verify/select/export the archive. Not a physics receipt.
"""
import json
import sqlite3
import struct
import sys
import vtk


def values(array, kind):
    assert array is not None, 'missing VTK array'
    expected_type = {'f32': vtk.VTK_FLOAT, 'f64': vtk.VTK_DOUBLE}[kind]
    assert array.GetDataType() == expected_type, 'native precision/type'
    return [array.GetValue(i) for i in range(array.GetNumberOfValues())]


def bits(actual, data, kind):
    code = {'f32': 'f', 'f64': 'd'}[kind]
    assert struct.pack('<' + code * len(actual), *actual) == data, 'native field bits'


def check(db, path, ordinal):
    reader = vtk.vtkXMLImageDataReader()
    reader.SetFileName(path)
    reader.Update()
    assert reader.GetErrorCode() == 0, 'VTK XML reader error'
    image = reader.GetOutput()
    geometry, = db.execute('SELECT geometry FROM run').fetchone()
    nx, ny, nz, *physical = struct.unpack('<3Q6d', geometry)
    assert image.GetDimensions() == (nx + 1, ny + 1, nz + 1), 'non-cubic extents'
    assert image.GetSpacing() == tuple(physical[:3]), 'physical spacing'
    assert image.GetOrigin() == tuple(physical[3:]), 'physical origin'
    cells, fields = image.GetCellData(), image.GetFieldData()
    assert cells.GetArray('fraction') is None, 'no invented fraction'
    assert fields.GetArray('carrier_stamp') is None, 'no invented session stamp'
    generation, time = db.execute('SELECT generation,time FROM steps WHERE ordinal=?', (ordinal,)).fetchone()
    assert fields.GetArray('generation').GetValue(0) == struct.unpack('<Q', generation)[0]
    bits(values(fields.GetArray('TimeValue'), 'f64'), time, 'f64')
    faces = []
    rows = dict((name, (kind, data, evaluation)) for name, kind, data, evaluation in db.execute(
        'SELECT name,scalar_type,data,evaluation_time FROM fields WHERE ordinal=?', (ordinal,)))
    for d, label in enumerate('xyz'):
        name = 'velocity_' + label
        kind, data, _ = rows[name]
        actual = values(fields.GetArray(name), kind)
        bits(actual, data, kind)
        faces.append(actual)
        expected_dims = [nx, ny, nz]
        expected_dims[d] += 1
        dims = fields.GetArray(name + '_dimensions')
        assert [dims.GetValue(i) for i in range(3)] == expected_dims, 'face lattice'
        offset = [.5, .5, .5]
        offset[d] = 0.
        assert values(fields.GetArray(name + '_offset'), 'f64') == offset, 'stagger offset'
    kind, data, _ = rows['tracer']
    bits(values(cells.GetArray('tracer'), kind), data, kind)
    pressure = cells.GetArray('interval_pressure_pa')
    if ordinal == 0:
        assert pressure is None, 'constructor pressure absent'
        assert fields.GetArray('pressure_interval_s') is None
    else:
        kind, data, evaluation = rows['interval_pressure']
        bits(values(pressure, kind), data, kind)
        bits(values(fields.GetArray('pressure_interval_s'), 'f64'), evaluation + time, 'f64')
        assert fields.GetArray('pressure_gauge_cell').GetValue(0) == 0
    expected = []
    for k in range(nz):
        for j in range(ny):
            for i in range(nx):
                for d in range(3):
                    dims = [nx, ny, nz]
                    dims[d] += 1
                    upper = [i, j, k]
                    upper[d] += 1
                    lo = i + dims[0] * (j + dims[1] * k)
                    hi = upper[0] + dims[0] * (upper[1] + dims[1] * upper[2])
                    expected.append((faces[d][lo] + faces[d][hi]) / 2.)
    actual = values(cells.GetArray('velocity_cell_m_s'), 'f64')
    assert struct.pack('<' + 'd' * len(actual), *actual) == struct.pack('<' + 'd' * len(expected), *expected), 'independent cell velocity'


if __name__ == '__main__':
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    with sqlite3.connect(sys.argv[1]) as connection:
        assert connection.execute('PRAGMA integrity_check').fetchone() == ('ok',)
        check(connection, sys.argv[2], 0)
        check(connection, sys.argv[3], 1)
    print(json.dumps({'vtk': vtk.vtkVersion.GetVTKVersion(), 'ordinals': [0, 1], 'native_bits': 'verified', 'geometry_associations_intervals': 'verified'}))
