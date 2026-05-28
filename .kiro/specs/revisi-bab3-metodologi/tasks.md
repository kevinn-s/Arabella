# Implementation Plan: Revisi Bab 3 Metodologi

## Overview

Implementasi spec ini menghasilkan satu berkas Markdown tunggal `Skripsi/bab3_metodologi.md` yang ditulis ulang sebagiannya agar setiap klaim teknis selaras dengan source code Arabella di `src/`, `Cargo.toml`, `examples/`, dan `tests/`. Pekerjaan terbagi menjadi tiga fase berurutan sesuai design: (1) ekstraksi fakta dari source code untuk membangun representasi internal yang dapat dipakai sebagai basis penulisan, (2) penulisan ulang narasi per subbab pada satu berkas Markdown target, dan (3) validasi pasca-tulis terhadap delapan correctness property yang didefinisikan di design.

Karena seluruh tugas penulisan menyentuh berkas yang sama (`Skripsi/bab3_metodologi.md`), tugas-tugas pada Fase 2 dijadwalkan secara sekuensial pada gelombang yang berbeda agar tidak saling menimpa, sementara Fase 1 (read-only ke source code) dan Fase 3 (read-only ke berkas output) dapat dijalankan paralel di dalam gelombangnya masing-masing.

## Tasks

- [x] 1. Ekstraksi fakta dari source code Arabella
  - [x] 1.1 Ekstraksi fakta `Cargo.toml`
    - Baca akar `Cargo.toml`; catat `edition = "2024"`, daftar dependensi langsung (`fearless_simd`, `lyon_path`, `lyon_geom`, `kurbo`, `peniko`, `roxmltree`, `bytemuck`, `thiserror`, `hashbrown`, `smallvec`), blok `[features]` (`multithreading = ["std", "dep:rayon", "dep:thread_local"]`), dan blok `[target.'cfg(target_arch = "wasm32")'.dependencies]`.
    - Catat versi crate persis seperti tertulis dan ejaan kebab/underscore yang dipakai upstream.
    - _Source: `Cargo.toml`_
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 7.6, 8.6, 13.10, 15.3, 15.4_
  
  - [x] 1.2 Ekstraksi fakta DDA dan signed-area dari `src/blocks.rs`
    - Catat konstanta `TILE_W = 16` dan `TILE_H = 8` beserta nomor baris.
    - Catat signature dan deskripsi singkat fungsi `bin_line`, `Blocks::build_block`, `record_per_scanline_crossings`, dan kelompok fungsi outer DDA (empat arah diagonal + tiga kasus khusus single-row, vertical-degenerate, horizontal-degenerate) serta inner DDA (empat arah utama).
    - Catat format fixed-point `F24Dot8` dan `8.8` untuk akumulator winding per scanline.
    - _Source: `src/blocks.rs`_
    - _Requirements: 5.4, 8.2, 8.3, 8.5, 13.4, 13.5, 13.10, 15.3_
  
  - [x] 1.3 Ekstraksi fakta builder dari `src/builder.rs`
    - Catat signature `Builder::build_path` dan `Builder::generate_tiles`; catat alur propagasi backdrop kiri-ke-kanan per baris ubin saat emisi tile.
    - Catat field `Builder` (termasuk `blocks: Blocks` dan `cover_storage: CoverStorage`) untuk class diagram.
    - _Source: `src/builder.rs`_
    - _Requirements: 5.1, 5.4, 7.4, 7.5, 8.2, 13.6, 15.3_
  
  - [x] 1.4 Ekstraksi fakta struct `Tile` dari `src/tile.rs` dan `src/render/common.rs`
    - Catat tata letak `#[repr(C)]` struct `Tile` dengan urutan field: `x: u16`, `y: u16`, `width: u8`, `height: u8`, `_pad: [u8; 2]`, `backdrop: [i16; 8]`, `segments: [_; 2]`, `payload: u32`, `paint_and_rect_flag: u32`, `depth_index: u32`, dan total ukuran 44 byte.
    - Catat definisi terkait di `src/render/common.rs` (`GpuEncodedImage` dan struktur tekstur RGBA32F untuk segmen).
    - _Source: `src/tile.rs`, `src/render/common.rs`_
    - _Requirements: 7.2, 7.3, 7.4, 10.1, 10.2, 10.3, 10.4, 10.5, 13.10, 15.3_
  
  - [x] 1.5 Ekstraksi fakta shader dari `src/render/shaders/render_tile.frag` dan `render_tile.vert`
    - Catat keberadaan fungsi `line_box` (integral trapezoidal cakupan piksel) beserta tanda tangan parameternya.
    - Catat formula fill rule NonZero (`clamp(abs(winding), 0, 1)`) dan EvenOdd (`1 - abs(mod(abs(winding), 2) - 1)`) atau formulasi setara, serta konstanta `WINDING_UNIT = 256`.
    - Konfirmasi bahwa fragment shader memproses semua ubin nontrivial pada satu jalur kode tanpa cabang `if`/`switch` berbasis tipe ubin.
    - _Source: `src/render/shaders/render_tile.frag`, `src/render/shaders/render_tile.vert`_
    - _Requirements: 5.2, 5.3, 5.5, 8.2, 11.3, 13.7, 15.3_
  
  - [x] 1.6 Ekstraksi fakta flattening dari `src/path.rs` dan `src/flatten.rs`
    - Catat signature dan dokumentasi singkat `convert_cubics_to_quadratic_curves` dan `estimate_number_of_quadratic_curves` di `src/path.rs`.
    - Catat alur De Casteljau midpoint subdivision pada `src/flatten.rs` yang mengubah kuadratik menjadi segmen garis pada format `F24Dot8`.
    - _Source: `src/path.rs`, `src/flatten.rs`_
    - _Requirements: 8.2, 8.7, 13.3, 15.3_
  
  - [x] 1.7 Ekstraksi fakta `Scene` API dan `WebGlRenderer`
    - Catat signature `Scene::fill`, `Scene::stroke`, dan field `builder: Builder` di `src/scene.rs`.
    - Catat signature `WebGlRenderer::new`, `WebGlRenderer::render`, dan `initialize_tile_vao` di `src/render/webgl.rs`, termasuk vertex divisor 44 byte per tile yang dipakai untuk instancing.
    - _Source: `src/scene.rs`, `src/render/webgl.rs`_
    - _Requirements: 6.2, 7.1, 7.4, 7.5, 10.4, 13.8, 15.3_
  
  - [x] 1.8 Ekstraksi fakta parser SVG dari `src/pico_svg.rs`
    - Catat elemen yang ditangani (`<g>`, `<path>`) beserta atribut yang diparse (`fill`, `stroke`, `stroke-width`, `transform`).
    - Konfirmasi bahwa parser bukan SVG 1.1 Core penuh dan tidak menangani `text`, `defs`, gradient, pattern, filter, atau clipPath.
    - _Source: `src/pico_svg.rs`_
    - _Requirements: 4.6, 7.1, 7.4, 13.9, 15.3_
  
  - [x] 1.9 Ekstraksi fakta demo dan tes dari `examples/native_webgl/` dan `tests/test.rs`
    - Catat resolusi DPR-aware pada demo native: `width = inner_width × devicePixelRatio` dan `height = inner_height × devicePixelRatio` di `examples/native_webgl/src/main.rs`.
    - Catat empat metrik overlay FPS pada `examples/native_webgl/src/lib.rs` (CPU ms, GPU ms, jumlah paint, rasio zoom) beserta nama fungsi `update_overlay` (atau setara).
    - Catat konstanta `W: u16 = 1080` dan `H: u16 = 520` pada `tests/test.rs`.
    - _Source: `examples/native_webgl/src/main.rs`, `examples/native_webgl/src/lib.rs`, `tests/test.rs`_
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 13.10, 15.3_

- [x] 2. Penulisan ulang narasi per subbab pada `Skripsi/bab3_metodologi.md`
  - [x] 2.1 Patch Subbab 3.1 (Diagram Alir Kerangka Berpikir)
    - Pertahankan kata dan urutan kalimat versi lama; hanya ganti penyebutan algoritma yang merujuk ray shooting atau klasifikasi tipe ubin di Fase 3 dan Fase 4 dengan istilah kanonik (binning DDA, akumulator signed-area, propagasi backdrop) yang konsisten dengan Subbab 3.5.
    - Pastikan baris pertama berkas adalah `# BAB 3 METODE PENELITIAN` dan heading `## 3.1 Diagram Alir Kerangka Berpikir` muncul tepat satu kali.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Requirements: 1.1, 1.2, 1.4, 2.1, 2.8, 3.1, 3.2, 14.1, 14.2, 14.3_
  
  - [x] 2.2 Patch Subbab 3.2 (Analisis Kebutuhan: 3.2.1, 3.2.2, 3.2.3)
    - Pertahankan teks Subbab 3.2.1 dan 3.2.2 versi lama tanpa perubahan substantif.
    - Tulis ulang Solusi Teknis baris ke-3 pada tabel Subbab 3.2.3 untuk mengganti penyebutan "Ray Shooting paralel" dengan deskripsi pipeline aktual (binning DDA dua tahap + akumulator signed-area + propagasi backdrop) tanpa mengubah Rumusan Masalah.
    - Pastikan heading `## 3.2 Analisis Kebutuhan`, `### 3.2.1 Analisis User`, `### 3.2.2 Analisis Aplikasi Sejenis`, dan `### 3.2.3 Rumusan dan Solusi Kebutuhan` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Requirements: 2.2, 2.8, 2.9, 3.1, 3.2, 14.1, 14.2_
  
  - [x] 2.3 Tulis ulang Subbab 3.3.1 (Spesifikasi Aplikasi)
    - Tulis ulang total: nyatakan Rust edisi 2024 (rujuk `edition = "2024"` di `Cargo.toml`); WebGL 2.0 sebagai target langsung pada `wasm32-unknown-unknown` (rujuk blok `[target.'cfg(target_arch = "wasm32")'.dependencies]`); daftarkan kesepuluh crate `fearless_simd`, `lyon_path`, `lyon_geom`, `kurbo`, `peniko`, `roxmltree`, `bytemuck`, `thiserror`, `hashbrown`, `smallvec` dengan ejaan persis seperti `Cargo.toml`; deskripsikan Rayon sebagai dependensi opsional di balik feature flag `multithreading` yang belum dipanggil pada hot path; deskripsikan dukungan SVG sebagai subset minimal (`g`, `path`, `fill`, `stroke`, `stroke-width`, `transform`) dari `src/pico_svg.rs`.
    - Definisikan istilah "klaim teknis" pada paragraf pengantar Subbab 3.3 (atau awal Subbab 3.5) sesuai AC 13.1.
    - Pastikan heading `## 3.3 Perancangan Aplikasi` dan `### 3.3.1 Spesifikasi Aplikasi` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.1, 1.8_
    - _Requirements: 2.3, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 13.1, 13.2, 13.9, 13.10, 13.11, 15.3, 15.4_
  
  - [x] 2.4 Patch Subbab 3.4.1 (Use Case Diagram)
    - Pertahankan struktur tiga use case (UC-01 Inisialisasi Context, UC-02 Input Data Vektor, UC-03 Render Frame); ganti penyebutan sub-proses "Preprocessing (CPU Tiling & Ray Shoot)" dengan istilah kanonik "pra-pemrosesan CPU (binning DDA + akumulator signed-area + propagasi backdrop)" dan sub-proses "Rendering (GPU Implicit Evaluation)" dengan "rasterisasi GPU (vertex shader instanced quad + fragment shader analitik)".
    - Pastikan heading `## 3.4 Perancangan Sistem` dan `### 3.4.1 Use Case Diagram` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Requirements: 2.4, 3.1, 3.2, 11.1, 11.2, 12.1, 12.2_
  
  - [x] 2.5 Tulis ulang Subbab 3.4.2 (Use Case Description, fokus UC-03)
    - Pertahankan tabel UC-01 dan UC-02 versi lama dengan koreksi terminologi minor.
    - Tulis ulang Alur Peristiwa Inti UC-03 menjadi enam tahap CPU berurutan: (a) flattening kurva ke segmen garis pada format F24Dot8, (b) outer DDA membagi setiap segmen lintas baris ubin, (c) inner DDA membagi lintas kolom ubin, (d) akumulasi signed-area per scanline pada format 8.8 fixed-point, (e) emisi ubin nontrivial, (f) propagasi backdrop kiri-ke-kanan saat emisi; lalu dua tahap GPU: (i) vertex shader instanced quad untuk setiap ubin nontrivial dan (ii) fragment shader analitik tunggal yang sama untuk seluruh ubin.
    - Sebut fill rule NonZero (`clamp(abs(winding), 0, 1)`) dan EvenOdd (`1 - abs(mod(abs(winding), 2) - 1)`) yang diterapkan pada `src/render/shaders/render_tile.frag`.
    - Sertakan rujukan kode `src/blocks.rs:bin_line`, `src/blocks.rs:Blocks::build_block`, dan `src/blocks.rs:record_per_scanline_crossings`.
    - JANGAN memuat cabang "Render Warna Solid"/"Evaluasi Fungsi Implisit" atau penanda klasifikasi tipe ubin.
    - Pastikan heading `### 3.4.2 Use Case Description` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.2, 1.3, 1.5, 1.6_
    - _Requirements: 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 11.1, 11.2, 11.3, 12.1, 12.2, 12.3, 12.5, 13.3, 13.4, 13.5, 13.7_
  
  - [x] 2.6 Tulis ulang Subbab 3.4.3 (Sequence Diagram)
    - Ganti seluruh konten lama dengan urutan lima pesan berurutan antar partisipan (Aplikasi Utama, Scene, Builder, WebGlRenderer, GPU): (i) `Scene::fill`/`Scene::stroke`, (ii) `Builder::build_path` (flattening + binning DDA), (iii) `Builder::generate_tiles` (propagasi backdrop kiri-ke-kanan + emisi seluruh ubin nontrivial tanpa percabangan tipe ubin), (iv) penyerahan vertex buffer dan tekstur segmen ke `WebGlRenderer::render`, (v) eksekusi vertex shader instanced quad lalu fragment shader analitik tunggal di GPU.
    - JANGAN memuat blok `alt`/`opt`/`loop` yang bercabang berdasarkan tipe ubin maupun pesan "Tandai Tipe: INTERIOR/EDGE/EMPTY".
    - Pastikan heading `### 3.4.3 Sequence Diagram` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.3, 1.5, 1.7_
    - _Requirements: 2.4, 3.1, 3.2, 6.1, 6.2, 6.3, 6.4, 6.5, 11.1, 11.2, 11.3, 13.3, 13.6, 13.8_
  
  - [x] 2.7 Tulis ulang Subbab 3.4.4 (Class Diagram)
    - Tampilkan sembilan kotak kelas UML: `Scene`, `Builder`, `CoverStorage`, `Block`, `Blocks`, `TileBounds`, `Tile`, `WebGlRenderer`, `PicoSvg`, masing-masing dengan daftar field utama beserta tipe seperti dideklarasikan di source code.
    - Tampilkan struct `Tile` dengan field `x`, `y`, `width`, `height`, `backdrop` (array delapan elemen 16-bit), `segments` (dua elemen offset+jumlah), `payload`, `paint_and_rect_flag`, `depth_index` sesuai `src/tile.rs`.
    - JANGAN memuat enum `TileType`, field `winding_number` skalar, field `curves: List<CurveRef>`, atau method `ray_shoot()`.
    - Cantumkan rujukan kode `src/scene.rs` (`Scene`), `src/builder.rs` (`Builder`, `CoverStorage`), `src/blocks.rs` (`Block`, `Blocks`, `TileBounds`), `src/tile.rs` (`Tile`), `src/render/webgl.rs` (`WebGlRenderer`), `src/pico_svg.rs` (`PicoSvg`).
    - Deskripsikan minimal tiga relasi UML (komposisi/agregasi/asosiasi) yang konsisten dengan deklarasi field di source code.
    - Pastikan heading `### 3.4.4 Class Diagram` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.2, 1.3, 1.4, 1.7, 1.8_
    - _Requirements: 2.4, 3.1, 3.2, 3.3, 3.4, 7.1, 7.2, 7.3, 7.4, 7.5, 11.1, 11.2, 12.5, 13.2, 13.6, 13.8, 13.9_
  
  - [x] 2.8 Tulis ulang Subbab 3.5 (Perancangan Algoritma)
    - Hapus seluruh narasi lama tentang fungsi implisit linear/kuadratik kanonik/kubik PPGA dan pseudocode `ALGORITMA PREPROCESSING_TILING_PARALEL` versi ray shooting.
    - Susun enam sub-bagian terpisah, masing-masing dengan minimal satu rujukan kode: (a) flattening kurva (cubic→quadratic gaya Vello via `convert_cubics_to_quadratic_curves` dan `estimate_number_of_quadratic_curves` di `src/path.rs`, lalu quadratic→garis via De Casteljau midpoint subdivision di `src/flatten.rs`); (b) binning DDA dua tahap (rujuk `src/blocks.rs`); (c) akumulator signed-area per scanline 8.8 fixed-point (rujuk `record_per_scanline_crossings` di `src/blocks.rs`); (d) propagasi backdrop kiri-ke-kanan (rujuk `Builder::generate_tiles` di `src/builder.rs`); (e) evaluasi cakupan piksel di GPU melalui integral trapezoidal `line_box` (rujuk `src/render/shaders/render_tile.frag`); (f) penerapan fill rule NonZero (clamp absolute) dan EvenOdd (triangle wave) pada fragment shader.
    - Nyatakan ukuran ubin Arabella adalah 16×8 piksel dengan rujukan eksplisit ke `TILE_W = 16` dan `TILE_H = 8` di `src/blocks.rs`.
    - Deskripsikan outer DDA sebagai pemecahan empat arah diagonal (down-right, down-left, up-right, up-left) plus tiga kasus khusus (single-row, vertikal degenerate, horizontal degenerate); inner DDA sebagai empat arah utama (right-down, right-up, left-down, left-up).
    - Nyatakan Rayon sebagai dependensi opsional yang belum dipanggil pada hot path binning maupun emisi ubin pada implementasi saat ini, sehingga klaim paralelisme CPU bersifat potensial.
    - Pastikan heading `## 3.5 Perancangan Algoritma` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.1, 1.2, 1.3, 1.5, 1.6_
    - _Requirements: 2.5, 2.8, 3.1, 3.2, 3.5, 3.6, 3.7, 3.8, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 11.1, 11.3, 12.1, 12.3, 12.5, 13.3, 13.4, 13.5, 13.6, 13.7, 13.10, 14.4, 15.3_
  
  - [x] 2.9 Tulis ulang Subbab 3.6 (Perancangan Layar)
    - Nyatakan demo interaktif `examples/native_webgl/` menggunakan resolusi window-fill DPR-aware dengan rumus `width = inner_width × devicePixelRatio` dan `height = inner_height × devicePixelRatio` (rujuk `examples/native_webgl/src/main.rs`).
    - Nyatakan pengujian wasm-bindgen-test pada `tests/test.rs` menggunakan resolusi kanvas tetap 1080×520 piksel (rujuk konstanta `W: u16 = 1080` dan `H: u16 = 520`).
    - Hapus klaim resolusi default 1920×1080 versi lama.
    - Deskripsikan overlay FPS dengan empat metrik enumeratif: waktu pra-pemrosesan CPU (ms), waktu render GPU (ms), jumlah operasi paint frame saat ini (bilangan bulat non-negatif), rasio zoom (bilangan riil positif), dengan rujukan ke `update_overlay` (atau setara) di `examples/native_webgl/src/lib.rs`.
    - Pastikan heading `## 3.6 Perancangan Layar` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.9_
    - _Requirements: 2.6, 9.1, 9.2, 9.3, 9.4, 9.5, 13.2, 13.10_
  
  - [x] 2.10 Tulis ulang Subbab 3.7 (Perancangan Database File)
    - Nyatakan daftar ubin disimpan sebagai vektor datar `Vec<Tile>` dengan setiap elemen `Tile` berukuran tepat 44 byte sesuai tata letak `#[repr(C)]` pada `src/tile.rs`.
    - Nyatakan segmen garis hasil binning diunggah ke GPU sebagai tekstur `RGBA32F` di mana satu texel menyimpan empat float `(p0.x, p0.y, p1.x, p1.y)` dalam ruang piksel ubin lokal (rentang dimensi ubin 16×8 piksel).
    - Deskripsikan vertex buffer instanced 44 byte/tile dengan tata letak berurutan: `x` (u16), `y` (u16), `width` (u8), `height` (u8), `_pad` (2 byte), `backdrop` (delapan i16), `segments` (dua elemen offset+jumlah), `payload` (u32), `paint_and_rect_flag` (u32), `depth_index` (u32); rujuk `src/render/webgl.rs:initialize_tile_vao` dan `src/tile.rs`.
    - JANGAN memakai frasa "koordinat quad mengambang", "floating-point array", atau "array vertex empat titik tunggal".
    - Pastikan heading `## 3.7 Perancangan Database File` muncul tepat satu kali dengan teks persis sama.
    - _Target file: `Skripsi/bab3_metodologi.md`_
    - _Source facts: hasil 1.4, 1.7_
    - _Requirements: 2.7, 2.8, 10.1, 10.2, 10.3, 10.4, 10.5, 10.6, 13.2, 13.8, 13.10, 15.3_

- [x] 3. Checkpoint - Konsolidasi draf Bab 3
  - Pastikan seluruh tugas 2.1 sampai 2.10 telah ditulis pada satu berkas `Skripsi/bab3_metodologi.md`. Ensure all tests pass, ask the user if questions arise.

- [x] 4. Validasi pasca-tulis terhadap delapan correctness property
  - [x] 4.1 Validasi Property 1 — Pemindaian istilah terlarang (forbidden-term scan)
    - Jalankan pencarian regex case-insensitive untuk `Ray Shooting`, `Ray Shoot`, `ray shooting`, `ray shoot`, `ditranspilasikan ke WebGL`, `transpilasi OpenGL ES`, `OpenGL ES 3.0 yang ditranspilasikan`.
    - Jalankan pencarian token kata utuh case-sensitive untuk `TileType`, `winding_number` (dengan underscore), `PPGA`, `Projective Geometric Algebra`, `EMPTY`, `INTERIOR`, `EDGE` (dalam konteks tipe ubin), `Rust edisi 2021`, `edisi 2021`, `edition = "2021"`.
    - Jalankan pencarian persamaan untuk seluruh varian `ax+by+c=0`, `u-v²=0`, `u-v^2=0`, `f(u,v) = u - v²`, `C(x,y)=0`, `w_0³-w_1 w_2 w_3`, `w_0(p)^3 - w_1(p) · w_2(p) · w_3(p)` dengan atau tanpa spasi.
    - Kondisi PASS: 0 kemunculan untuk seluruh pola pada konteks yang dilarang.
    - **Property 1: Absence of Forbidden Terms**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11, 8.1, 11.2**
  
  - [x] 4.2 Validasi Property 2 — Pemindaian istilah wajib (required-term scan)
    - Jalankan pencarian kehadiran (minimal satu kemunculan) untuk seluruh anggota Istilah_Wajib: `WebGL 2.0`, `Rust edisi 2024`, `F24Dot8` (atau `24.8 fixed-point`), `8.8 fixed-point`, `DDA`, `outer DDA`, `inner DDA`, `signed-area`, `backdrop`, `propagasi backdrop`, `flattening`, `midpoint subdivision`, `cubic-to-quadratic`, `line_box`, `trapezoidal`, `fearless_simd`, `lyon_path`, `lyon_geom`, `kurbo`, `peniko`, `roxmltree`, `bytemuck`, `thiserror`, `hashbrown`, `smallvec`, `NonZero`, `EvenOdd`, `16×8`, `Rayon`.
    - Verifikasi bahwa istilah yang terikat subbab tertentu (kesepuluh nama crate pada Subbab 3.3.1; `line_box`, `record_per_scanline_crossings`, `TILE_W`, `TILE_H` pada Subbab 3.5; `1080×520` pada Subbab 3.6; `RGBA32F` pada Subbab 3.7) muncul minimal sekali pada subbab tersebut.
    - Kondisi PASS: minimal satu kemunculan per istilah pada lokasi yang ditetapkan.
    - **Property 2: Presence of Required Terms**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.5, 8.2, 8.3, 8.7**
  
  - [x] 4.3 Validasi Property 3 — Heading structure invariant
    - Verifikasi baris pertama berkas adalah `# BAB 3 METODE PENELITIAN`.
    - Verifikasi kehadiran tepat-sekali pada level Markdown yang ditentukan untuk seluruh Subbab_Wajib: `## 3.1 Diagram Alir Kerangka Berpikir`, `## 3.2 Analisis Kebutuhan`, `### 3.2.1 Analisis User`, `### 3.2.2 Analisis Aplikasi Sejenis`, `### 3.2.3 Rumusan dan Solusi Kebutuhan`, `## 3.3 Perancangan Aplikasi`, `### 3.3.1 Spesifikasi Aplikasi`, `## 3.4 Perancangan Sistem`, `### 3.4.1 Use Case Diagram`, `### 3.4.2 Use Case Description`, `### 3.4.3 Sequence Diagram`, `### 3.4.4 Class Diagram`, `## 3.5 Perancangan Algoritma`, `## 3.6 Perancangan Layar`, `## 3.7 Perancangan Database File`.
    - Verifikasi nomor heading level 2 menaik monotonik 3.1 → 3.7 tanpa lompatan/pengulangan/pembalikan, dan setiap subheading `### 3.X.Y` muncul setelah `## 3.X` dengan `Y` menaik mulai dari 1.
    - Kondisi PASS: seluruh heading hadir dengan teks dan urutan persis sama.
    - **Property 3: Structural Heading Invariant**
    - **Validates: Requirements 1.4, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9**
  
  - [x] 4.4 Validasi Property 4 — Konsistensi terminologi kanonik
    - Verifikasi bahwa istilah kanonik tunggal dipakai konsisten di seluruh berkas: `binning DDA`, `akumulator signed-area`, `propagasi backdrop`, `fragment shader`, `pra-pemrosesan` atau `preprocessing` (tidak campur dalam satu paragraf), `pipeline hibrida`, `rasterization pipeline tradisional` atau `pipeline rasterisasi konvensional`, `viewport`, `winding number`.
    - Pastikan tidak ada sinonim non-kanonik atau variasi ejaan untuk komponen yang sama yang muncul di subbab manapun.
    - Kondisi PASS: tidak ada sinonim non-kanonik untuk komponen pipeline yang sama.
    - **Property 4: Canonical Terminology Consistency**
    - **Validates: Requirements 11.1, 11.4, 12.1, 12.2, 12.3, 12.4, 12.5, 12.6**
  
  - [x] 4.5 Validasi Property 5 — Code-reference traceability
    - Untuk setiap klaim teknis pada Bab 3, verifikasi keberadaan rujukan kode dengan format `path:simbol` atau `path:start-end` di kalimat atau paragraf yang sama.
    - Untuk setiap rujukan, verifikasi bahwa berkas yang ditunjuk benar-benar ada di `src/`, `Cargo.toml`, `.cargo/config.toml`, `examples/*/src/`, `tests/`, atau `assets/`, dan simbol yang dirujuk benar-benar terdefinisi pada berkas tersebut.
    - Kondisi PASS: setiap klaim teknis memiliki rujukan kode valid yang dapat ditelusuri ke source code.
    - **Property 5: Technical Claim Traceability**
    - **Validates: Requirements 13.1, 13.2, 13.3, 13.4, 13.5, 13.6, 13.7, 13.8, 13.9, 13.10, 13.11, 13.12, 15.1, 15.2, 15.3, 15.4, 15.5**
  
  - [x] 4.6 Validasi Property 6 — Konsistensi naratif lintas subbab
    - Cross-check bahwa Subbab 3.4.2, 3.4.3, 3.4.4, dan 3.5 mendeskripsikan komponen pipeline yang sama tanpa kontradiksi: tidak ada subbab yang menyebut percabangan tipe ubin sementara subbab lain menggambarkan jalur kode tunggal; tidak ada subbab yang menampilkan field `winding_number` skalar atau enum `TileType` sementara subbab lain mendeklarasikan `backdrop` array delapan elemen.
    - Verifikasi urutan tahap pipeline (flattening → outer DDA → inner DDA → akumulasi signed-area → emisi ubin → propagasi backdrop → vertex shader → fragment shader) identik di setiap subbab tempat urutan tersebut disebutkan.
    - Kondisi PASS: keempat subbab konsisten satu sama lain.
    - **Property 6: Cross-Subsection Narrative Consistency**
    - **Validates: Requirements 5.1, 5.2, 5.3, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.3, 11.1, 11.2, 11.3, 11.4**
  
  - [x] 4.7 Validasi Property 7 — Konsistensi klaim numerik
    - Verifikasi nilai berulang konsisten di seluruh berkas: dimensi ubin Arabella = 16×8 piksel; ukuran rekord `Tile` = 44 byte; resolusi kanvas pengujian wasm-bindgen-test = 1080×520 piksel; format tekstur segmen = RGBA32F; format fixed-point segmen = F24Dot8; format fixed-point akumulator winding = 8.8 fixed-point.
    - Pastikan tidak ada nilai kontradiktif (16×16 untuk dimensi ubin, 1920×1080 untuk resolusi pengujian default, atau ukuran `Tile` selain 44 byte) di subbab manapun.
    - Kondisi PASS: setiap parameter numerik kanonik melaporkan nilai yang sama di seluruh dokumen.
    - **Property 7: Numerical Claim Consistency**
    - **Validates: Requirements 8.3, 8.4, 9.1, 9.2, 9.3, 9.4, 10.1, 10.2, 10.3, 10.4**
  
  - [x] 4.8 Validasi Property 8 — Identitas berkas tunggal
    - Pindai seluruh repositori untuk berkas dengan nama mengandung substring `bab3`, `bab_3`, atau `bab-3` (case-insensitive); verifikasi bahwa hanya `Skripsi/bab3_metodologi.md` yang baris pertamanya `# BAB 3 METODE PENELITIAN`.
    - Verifikasi tidak ada salinan utuh, salinan parsial, cadangan, atau draf alternatif Bab 3 di lokasi lain repositori.
    - Verifikasi ekstensi berkas adalah `.md` dan sintaks Markdown CommonMark dapat diurai tanpa galat.
    - Kondisi PASS: tepat satu berkas Bab 3 di seluruh repositori.
    - **Property 8: Single-File Output Identity**
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.5**

- [x] 5. Final checkpoint - Iterasi revisi sampai seluruh property PASS
  - Jika ada validasi 4.1 sampai 4.8 yang FAIL, kembali ke task 2.x yang relevan untuk perbaikan, lalu jalankan ulang validasi yang gagal sampai PASS. Ensure all tests pass, ask the user if questions arise.

## Notes

- Spec ini menghasilkan satu berkas Markdown akademik (`Skripsi/bab3_metodologi.md`); tidak ada kode runtime yang ditulis, sehingga tidak ada unit test atau property-based test berbasis kode. Validasi pada Fase 3 bersifat deterministik (regex + pemeriksaan struktural) dan merupakan bagian inti deliverable, bukan tugas opsional.
- Setiap tugas penulisan pada Fase 2 menyentuh berkas yang sama (`Skripsi/bab3_metodologi.md`); pada Task Dependency Graph, tugas-tugas tersebut dijadwalkan pada gelombang yang berbeda agar tidak saling menimpa.
- Tugas ekstraksi fakta pada Fase 1 bersifat read-only ke source code dan dapat dijalankan paralel.
- Tugas validasi pada Fase 3 bersifat read-only ke berkas output dan dapat dijalankan paralel.
- Setiap klaim teknis pada Bab 3 wajib menyertakan rujukan kode berformat `path:simbol` atau `path:start-end`, dan rujukan tersebut wajib menunjuk berkas yang benar-benar ada di repositori (`src/`, `Cargo.toml`, `.cargo/config.toml`, `examples/*/src/`, `tests/`, atau `assets/`).
- Setiap pekerjaan menulis ulang harus mempertahankan kosakata dan format penomoran subbab yang sudah dipakai di Bab_3_Lama (`pustaka`, `perangkat lunak`, `pra-pemrosesan`, `ubin`, format `3.x` dan `3.x.y`) sesuai Requirement 14.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9"] },
    { "id": 1, "tasks": ["2.1"] },
    { "id": 2, "tasks": ["2.2"] },
    { "id": 3, "tasks": ["2.3"] },
    { "id": 4, "tasks": ["2.4"] },
    { "id": 5, "tasks": ["2.5"] },
    { "id": 6, "tasks": ["2.6"] },
    { "id": 7, "tasks": ["2.7"] },
    { "id": 8, "tasks": ["2.8"] },
    { "id": 9, "tasks": ["2.9"] },
    { "id": 10, "tasks": ["2.10"] },
    { "id": 11, "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8"] }
  ]
}
```
