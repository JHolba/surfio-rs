import time
import surfio
import surfio_rs
import numpy as np
import os


def create_dummy_data(rows=200, cols=200):
    values = (
        np.random.rand(rows * cols).astype(np.float32).reshape(rows, cols, order="F")
    )

    header_cpp = surfio.IrapHeader(ncol=cols, nrow=rows)
    surface_cpp = surfio.IrapSurface(header_cpp, values)

    surface_rs = surfio_rs.Irap()
    surface_rs.header = surfio_rs.IrapHeader(ncol=cols, nrow=rows)
    surface_rs.values = values

    return surface_cpp, surface_rs


def benchmark(name, func, *args):
    start = time.time()
    func(*args)
    end = time.time()
    return end - start


def main():
    rows, cols = 5000, 5000
    print(f"Creating dummy data ({rows}x{cols})...")
    surface_cpp, surface_rs = create_dummy_data(rows, cols)

    ascii_file = "bench.irap"
    binary_file = "bench.grd"

    print(f"\n--- Benchmarking Write ASCII ---")
    t_cpp = benchmark(
        "C++ Write ASCII", surfio.IrapSurface.to_ascii_file, surface_cpp, ascii_file
    )
    print(f"surfio (C++):    {t_cpp:.4f} s")

    t_rs = benchmark(
        "Rust Write ASCII", surfio_rs.write_irap_ascii_file, ascii_file, surface_rs
    )
    print(f"surfio-rs (Rust): {t_rs:.4f} s")

    print(f"Ratio (C++/Rust): {t_cpp / t_rs:.2f}x")

    print(f"\n--- Benchmarking Read ASCII ---")
    t_cpp = benchmark("C++ Read ASCII", surfio.IrapSurface.from_ascii_file, ascii_file)
    print(f"surfio (C++):    {t_cpp:.4f} s")

    t_rs = benchmark("Rust Read ASCII", surfio_rs.read_irap_ascii_file, ascii_file)
    print(f"surfio-rs (Rust): {t_rs:.4f} s")

    print(f"Ratio (C++/Rust): {t_cpp / t_rs:.2f}x")

    print(f"\n--- Benchmarking Write Binary ---")
    t_cpp = benchmark(
        "C++ Write Binary", surfio.IrapSurface.to_binary_file, surface_cpp, binary_file
    )
    print(f"surfio (C++):    {t_cpp:.4f} s")

    t_rs = benchmark(
        "Rust Write Binary", surfio_rs.write_irap_binary_file, binary_file, surface_rs
    )
    print(f"surfio-rs (Rust): {t_rs:.4f} s")

    print(f"Ratio (C++/Rust): {t_cpp / t_rs:.2f}x")

    print(f"\n--- Benchmarking Read Binary ---")
    t_cpp = benchmark(
        "C++ Read Binary", surfio.IrapSurface.from_binary_file, binary_file
    )
    print(f"surfio (C++):    {t_cpp:.4f} s")

    t_rs = benchmark("Rust Read Binary", surfio_rs.read_irap_binary_file, binary_file)
    print(f"surfio-rs (Rust): {t_rs:.4f} s")

    print(f"Ratio (C++/Rust): {t_cpp / t_rs:.2f}x")

    # Clean up
    if os.path.exists(ascii_file):
        os.remove(ascii_file)
    if os.path.exists(binary_file):
        os.remove(binary_file)


if __name__ == "__main__":
    main()
