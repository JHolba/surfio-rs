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


@pytest.mark.parametrize("func", [str, repr])
def test_irap_surface_string_representation(func):
    surface = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=2.0, yinc=2.0, xmax=2.0, ymax=2.0),
        np.zeros((3, 2), dtype=np.float32),
    )
    s = func(surface)
    assert isinstance(s, str)
    assert "Irap" in s


def test_reading_empty_file_errors(tmp_path):
    irap_path = tmp_path / "test.irap"
    irap_path.write_text("")
    # the underlying implementation may raise OSError for mmap edge-cases
    with pytest.raises(Exception, match="memory map|map|zero"):
        surfio.IrapSurface.from_ascii_file(str(irap_path))


def test_reading_short_header_results_in_value_error():
    with pytest.raises(ValueError, match="end of file"):
        _ = surfio.IrapSurface.from_ascii_string("-996 1")


def test_reading_negative_dimensions_results_in_value_error():
    with pytest.raises(Exception):
        _ = surfio.IrapSurface.from_ascii_string(
            """\
            -996 -1 0.0 0.0
            0.0 0.0 0.0 0.0
            1 0.0 0.0 0.0
            0  0  0  0  0  0  0
            0.000000
            """
        )


def test_short_files_result_in_value_error():
    with pytest.raises(ValueError, match="end of file|fill"):
        _ = surfio.IrapSurface.from_ascii_string(
            """\
                -996 5 0.0 0.0
                0.0 0.0 0.0 0.0
                5 0.0 0.0 0.0
                0  0  0  0  0  0  0
                0.000000
                """
        )


def test_non_floats_result_in_domain_error():
    with pytest.raises(ValueError, match="float|parsing"):
        _ = surfio.IrapSurface.from_ascii_string(
            """\
            -996 5 0.0 0.0
            0.0 0.0 0.0 0.0
            5 0.0 0.0 0.0
            0  0  0  0  0  0  0
            not_a_number
            """
        )


def test_incorrect_header_types_result_in_value_error():
    with pytest.raises(ValueError, match="invalid|digit"):
        _ = surfio.IrapSurface.from_ascii_string(
            """\
            -996 5 0.0 0.0
            0.0 0.0 0.0 0.0
            not_a_number 0.0 0.0 0.0
            0  0  0  0  0  0  0
            0.0
            """
        )


def test_reading_one_by_one_values():
    surface = surfio.IrapSurface.from_ascii_string(
        """\
        -996 1 2.0 3.0
        0.0 4.0 0.0 5.0
        1 0.0 0.0 0.0
        0  0  0  0  0  0  0
        1.000000
        """
    )
    assert surface.values == [[1.0]]


def test_9999900_is_mapped_to_nan():
    surface = surfio.IrapSurface.from_ascii_string(
        """\
        -996 1 2.0 3.0
        0.0 4.0 0.0 5.0
        1 0.0 0.0 0.0
        0  0  0  0  0  0  0
        9999900.0000
        """
    )
    assert np.isnan(surface.values[0, 0])


def test_reading_no_leading_decimals():
    surface = surfio.IrapSurface.from_ascii_string(
        """\
        -996 1 2.0 3.0
        0.0 4.0 0.0 5.0
        1 0.0 0.0 0.0
        0  0  0  0  0  0  0
        .5"""
    )
    assert surface.values == [[0.5]]


def test_reading_two_by_three_results_in_f_order_values():
    surface = surfio.IrapSurface.from_ascii_string(
        """\
        -996 2 2.0 2.0
        0.0 2.0 0.0 2.0
        3 0.0 0.0 0.0
        0  0  0  0  0  0  0
        1.000000 2.000000 3.000000
        4.000000 5.000000 6.000000
        """
    )
    assert surface.values.tolist() == [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]


def test_reading_two_by_three_results_in_f_order_values_from_file(tmp_path):
    irap_path = tmp_path / "test.irap"
    irap_path.write_text(
        """\
        -996 2 2.0 2.0
        0.0 2.0 0.0 2.0
        3 0.0 0.0 0.0
        0  0  0  0  0  0  0
        1.000000 2.000000 3.000000
        4.000000 5.000000 6.000000
        """
    )
    surface = surfio.IrapSurface.from_ascii_file(str(irap_path))
    assert surface.values.tolist() == [[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]


def test_header_can_have_high_precision():
    # xtgeo can produce precision in the header
    # only possible with double precision
    surface = surfio.IrapSurface.from_ascii_string(
        """\
        -996 2 1.0 1.0
        0.0 1.0 2.610356564800451e-73 1.0
        2 0.0 0.0 2.610356564800451e-73
        0  0  0  0  0  0  0
        0.000000 0.000000 0.000000 0.000000
        """
    )
    assert surface.header.yrot == pytest.approx(2.6e-73, abs=1e-74)


def test_import_and_export_are_inverse():
    surface = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=2.0, yinc=2.0, xmax=4.0, ymax=2.0),
        np.arange(6, dtype=np.float32).reshape((3, 2)),
    )
    roundtrip = surfio.IrapSurface.from_ascii_string(surface.to_ascii_string())
    assert _headers_equal(roundtrip.header, surface.header)
    assert np.array_equal(roundtrip.values, surface.values)


def test_import_and_export_file_are_inverse(tmp_path):
    irap_path = tmp_path / "test.irap"
    surface = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=2.0, yinc=2.0, xmax=4.0, ymax=2.0),
        np.arange(6, dtype=np.float32).reshape((3, 2)),
    )
    surface.to_ascii_file(str(irap_path))
    roundtrip = surfio.IrapSurface.from_ascii_file(str(irap_path))
    assert _headers_equal(roundtrip.header, surface.header)
    assert np.array_equal(roundtrip.values, surface.values)

def test_xtgeo_can_import_data_exported_from_surfio(tmp_path):
    srf = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=2, xinc=1.0, yinc=1.0, xmax=2.0, ymax=1.0),
        values=np.arange(6, dtype=np.float32).reshape((3, 2)),
    )
    srf.to_ascii_file(str(tmp_path / "test.irap"))
    srf_imported = xtgeo.surface_from_file(tmp_path / "test.irap", fformat="irap_ascii")

    assert np.allclose(srf.values, srf_imported.values)
    compare_xtgeo_surface_with_surfio_header(srf_imported, srf.header)

def test_exporting_nan_maps_to_9999900():
    surface = surfio.IrapSurface(
        surfio.IrapHeader(ncol=1, nrow=1, xinc=2.0, yinc=2.0, xmax=2.0, ymax=2.0),
        np.array([[np.nan]], dtype=np.float32),
    )

    assert "9999900.0000" in surface.to_ascii_string().split()[-1]


def test_exporting_produces_max_9_values_per_line():
    """
    This is the maximum supported by some applications
    """
    surface = surfio.IrapSurface(
        surfio.IrapHeader(ncol=10, nrow=1, xinc=2.0, yinc=2.0, xmax=2.0, ymax=2.0),
        np.zeros((10, 1), dtype=np.float32),
    )

    assert all(
        len(line.split()) <= 9
        for line in surface.to_ascii_string().splitlines()
    )


def test_surfio_can_export_values_in_fortran_order():
    srf = surfio.IrapSurface(
        surfio.IrapHeader(ncol=3, nrow=4, xinc=1.0, yinc=1.0, xmax=2.0, ymax=3.0),
        values=np.asfortranarray(np.arange(12, dtype=np.float32).reshape((3, 4))),
    )
    buffer = srf.to_ascii_string()
    srf_imported = surfio.IrapSurface.from_ascii_string(buffer)

    assert np.allclose(srf.values, srf_imported.values)
    assert _headers_equal(srf.header, srf_imported.header)


def test_can_mutate_imported_surface():
    surface = surfio.IrapSurface.from_ascii_string(
        """\
        -996 2 1.0 1.0
        0.0 1.0 2.610356564800451e-73 1.0
        2 0.0 0.0 2.610356564800451e-73
        0  0  0  0  0  0  0
        0.000000 0.000000 0.000000 0.000000
        """
    )
    surface.values[0][1] = 3.0
    assert surface.values[0][1] == 3.0
