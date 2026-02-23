import struct

import numpy as np
import pytest

import surfio_rs as surfio
import xtgeo


def compare_xtgeo_surface_with_surfio_header(
    xtgeo_surface: xtgeo.RegularSurface, surfio_header: surfio.IrapHeader
):
    assert xtgeo_surface.ncol == surfio_header.ncol
    assert xtgeo_surface.nrow == surfio_header.nrow
    assert xtgeo_surface.xori == surfio_header.xori
    assert xtgeo_surface.yori == surfio_header.yori
    assert xtgeo_surface.xinc == surfio_header.xinc
    assert xtgeo_surface.yinc == surfio_header.yinc
    assert xtgeo_surface.xmax == surfio_header.xmax
    assert xtgeo_surface.ymax == surfio_header.ymax
    assert xtgeo_surface.rotation == surfio_header.rot


def _headers_equal(h1, h2):
    if h1.ncol != h2.ncol or h1.nrow != h2.nrow:
        return False
    float_fields = [
        "xori",
        "yori",
        "xmax",
        "ymax",
        "xinc",
        "yinc",
        "rot",
        "xrot",
        "yrot",
    ]
    for f in float_fields:
        if not np.isclose(getattr(h1, f), getattr(h2, f), atol=1e-12, rtol=0):
            return False
    return True


def test_reading_empty_file_errors(tmp_path):
    irap_path = tmp_path / "test.irap"
    irap_path.write_text("")
    # mmap can raise OSError in this environment
    with pytest.raises(Exception, match="memory map|map|zero"):
        surfio.IrapSurface.from_ascii_file(str(irap_path))


def test_reading_short_header_results_in_value_error():
    with pytest.raises(ValueError, match="end of file"):
        _ = surfio.IrapSurface.from_ascii_string("-996 1")


def test_short_files_result_in_value_error():
    srf = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=4, xinc=1.0, yinc=1.0, xmax=2.0, ymax=3.0),
        values=np.random.normal(2000, 50, size=12).reshape((3, 4)).astype(np.float32),
    )
    file_buffer = srf.to_binary_buffer()
    truncated_buffer = file_buffer[:100]
    with pytest.raises(ValueError, match="fill|end of file|buffer"):
        _ = surfio.IrapSurface.from_binary_buffer(truncated_buffer)


def test_surfio_can_import_data_exported_from_surfio():
    srf = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=1.0, yinc=1.0, xmax=2.0, ymax=1.0),
        values=np.arange(6, dtype=np.float32).reshape((3, 2)),
    )
    buffer = srf.to_binary_buffer()
    srf_imported = surfio.IrapSurface.from_binary_buffer(buffer)

    assert np.allclose(srf.values, srf_imported.values)
    assert _headers_equal(srf.header, srf_imported.header)


def test_surfio_can_export_values_in_fortran_order():
    srf = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=1.0, yinc=1.0, xmax=2.0, ymax=1.0),
        values=np.asfortranarray(np.arange(6, dtype=np.float32).reshape((3, 2))),
    )
    buffer = srf.to_binary_buffer()
    srf_imported = surfio.IrapSurface.from_binary_buffer(buffer)

    assert np.allclose(srf.values, srf_imported.values)
    assert _headers_equal(srf.header, srf_imported.header)


def test_xtgeo_can_import_data_exported_from_surfio(tmp_path):
    srf = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=1.0, yinc=1.0, xmax=2.0, ymax=1.0),
        values=np.arange(6, dtype=np.float32).reshape((3, 2)),
    )
    srf.to_binary_file(str(tmp_path / "test.irap"))
    srf_imported = xtgeo.surface_from_file(tmp_path / "test.irap")

    assert np.allclose(srf.values, srf_imported.values)
    compare_xtgeo_surface_with_surfio_header(srf_imported, srf.header)


def test_exporting_nan() -> None:
    surface = surfio.IrapSurface(
        surfio.IrapHeader(ncol=1, nrow=1, xinc=2.0, yinc=2.0, xmax=2.0, ymax=2.0),
        values=np.array([[np.nan]], dtype=np.float32),
    )

    srf_export = surface.to_binary_buffer()
    assert struct.unpack("f", srf_export[107:103:-1])[0] >= 1e30
