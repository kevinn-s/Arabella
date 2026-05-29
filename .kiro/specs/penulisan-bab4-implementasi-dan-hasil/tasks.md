# Implementation Plan: Penulisan Bab 4 Implementasi dan Hasil

## Overview

Rencana implementasi ini menerjemahkan desain menjadi serangkaian tugas penulisan dan validasi konkret terhadap satu berkas keluaran tunggal `Skripsi/bab4_implementasi_dan_hasil.md`. "Implementasi" di sini berarti penulisan narasi akademik Markdown berbahasa Indonesia formal, bukan penulisan kode aplikasi. Source code Arabella (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`) bersifat **read-only / source-of-truth** dan TIDAK BOLEH dimodifikasi oleh tugas mana pun (Requirement 2).

Pendekatan eksekusi: (1) bangun kerangka heading lengkap dan hapus lorem ipsum, (2) tulis subbab siap-tulis berbasis pembacaan langsung source code dengan rujukan kode inline, (3) tulis subbab tunggu-pengukuran sebagai narasi metodologi lengkap dengan placeholder eksplisit (tanpa angka karangan), lalu (4) jalankan validasi deterministik (regex + struktural + cross-reference) yang memetakan langsung ke sepuluh Correctness Properties pada desain.

Karena seluruh tugas penulisan mengedit berkas yang sama, tugas-tugas tersebut dijadwalkan berurutan antar-wave; tugas validasi bersifat read-only sehingga dapat dijalankan paralel setelah konten selesai. Validasi properti diimplementasikan sebagai pencarian teks/struktural deterministik — bukan property-based testing iteratif acak — sesuai bagian Testing Strategy pada desain.

## Tasks

- [x] 1. Bangun kerangka dokumen dan hapus placeholder lama
  - [x] 1.1 Tulis ulang `Skripsi/bab4_implementasi_dan_hasil.md` dari nol dengan kerangka heading lengkap
    - Tulis baris pertama persis `# BAB 4 HASIL DAN PEMBAHASAN` (ATX, satu spasi tunggal, kapitalisasi penuh, tanpa BOM, tanpa whitespace awal/akhir)
    - Tulis enam heading level 2 berurutan menaik: `## 4.1 Spesifikasi Lingkungan Implementasi`, `## 4.2 Implementasi Modul`, `## 4.3 Verifikasi Kebenaran Output`, `## 4.4 Pengujian Performa`, `## 4.5 Pembahasan Trade-off Arsitektur Non-Compute`, `## 4.6 Keterbatasan Implementasi Saat Ini`
    - Tulis subheading level 3 di bawah 4.2: `### 4.2.1 Parser SVG` sampai `### 4.2.8 Demo Interaktif` (kontigu 1–8)
    - Tulis subheading level 3 di bawah 4.4: `### 4.4.1 Metodologi Pengukuran`, `### 4.4.2 Hasil Pengukuran Per Aset`, `### 4.4.3 Analisis Perbandingan dengan Baseline`
    - Pastikan TIDAK ada frasa lorem ipsum (`lorem ipsum`, `dolor sit amet`, `consectetur adipiscing`, `tempor incididunt`, `exercitation ullamco`, `duis aute irure`) yang tersisa; encoding UTF-8 tanpa BOM; Markdown CommonMark valid
    - Konfirmasi tidak ada berkas Bab 4 duplikat di luar `.kiro/specs/` (Req 1.5, 1.6) dan TIDAK memodifikasi berkas Source_Of_Truth maupun berkas skripsi lain (Req 2)
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9_

- [x] 2. Tulis Subbab 4.1 dan paragraf pengantar Subbab 4.2
  - [x] 2.1 Tulis isi Subbab 4.1 Spesifikasi Lingkungan Implementasi
    - Sebutkan Rust edisi 2024 (rujukan `Cargo.toml:edition`), target `wasm32-unknown-unknown` + `+simd128` (rujukan `.cargo/config.toml`), lima parameter `[profile.release]` (`lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = false`, `strip = true`)
    - Sebutkan minimal satu peramban target konkret + versi minimum (mis. Google Chrome 113) yang mendukung WebGL 2.0 dan WASM SIMD128; toolchain `wasm-pack` dan `cargo-run-wasm` dengan feature `webgl`; layout workspace (`arabella` + dua crate contoh)
    - Sisipkan kalimat penghubung ke `Bab 3` pada paragraf pembuka (Req 13.1); sertakan Rujukan_Kode pada setiap Klaim_Teknis; tanpa istilah terlarang dan tanpa angka performa karangan
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 10.1, 10.2, 10.3, 13.1_

  - [x] 2.2 Tulis paragraf pengantar Subbab 4.2 Implementasi Modul
    - Nyatakan urutan delapan tahap pipeline kanonik (flattening → outer DDA → inner DDA → akumulator signed-area → emisi tile → propagasi backdrop → vertex shader → fragment shader) konsisten dengan UC-03 Bab 3
    - Sertakan rujukan ke `Subbab 3.4.4` atau frasa `class diagram pada Bab 3`, dan satu kalimat eksplisit pergeseran fokus dari "perancangan" (Bab 3) ke "wujud implementasi konkrit dan hasil pengujian" (Bab 4)
    - _Requirements: 13.1, 13.2, 13.3, 13.5_

- [x] 3. Tulis Subbab 4.2.1–4.2.8 (Implementasi Modul per berkas)
  - [x] 3.1 Tulis Subbab 4.2.1 Parser SVG dari `src/pico_svg.rs`
    - Sebutkan `PicoSvg`, `PicoSvg::load`, enum `Item` (`Fill`, `Stroke`, `Group`), dispatch tag-name `Parser::rec_parse` (hanya `g` dan `path`), atribut `fill`/`stroke`/`stroke-width`/`transform`, delegasi XML ke `roxmltree`
    - Sertakan minimal satu Rujukan_Kode ke `src/pico_svg.rs` pada paragraf yang sama dengan setiap token
    - _Requirements: 5.1, 10.1, 10.5_

  - [x] 3.2 Tulis Subbab 4.2.2 Scene API dari `src/scene.rs`
    - Sebutkan `Scene`, `Scene::new`, `Scene::fill`, `Scene::stroke`, `Scene::reset`, encoding paint solid `PAINT_TYPE_SOLID = 0`, delegasi stroke expansion ke `kurbo::stroke_with`
    - Sertakan Rujukan_Kode ke `src/scene.rs` pada paragraf yang sama dengan setiap token
    - _Requirements: 5.2, 10.1, 10.5_

  - [x] 3.3 Tulis Subbab 4.2.3 Path Processing dan Flattening dari `src/path.rs` dan `src/flatten.rs`
    - Sebutkan transformasi affine SIMD-batched (`transform_pair`, `transform_quad`, `f32x4`, `f32x8`), `convert_cubics_to_quadratic_curves`, `estimate_number_of_quadratic_curves` (`MAX_QUADS = 16`, `TOL`), `flatten_quadratic`/`flatten_recursive` dengan `is_flat_enough` terhadap `FLATNESS_THRESHOLD = 32`, format F24Dot8 via `f32_to_f24dot8`
    - Sertakan Rujukan_Kode ke `src/path.rs` atau `src/flatten.rs` pada paragraf yang sama dengan setiap token
    - _Requirements: 5.3, 10.1, 10.5_

  - [x] 3.4 Tulis Subbab 4.2.4 Tile Binning DDA dari `src/blocks.rs`
    - Sebutkan `TILE_W = 16`, `TILE_H = 8` (+ turunan F24Dot8), struct `Block`/`Blocks`/`TileBounds`, `Blocks::build_block`, `Blocks::bin_line` (outer DDA empat arah + tiga kasus khusus), `Blocks::bin_line_in_row` (inner DDA empat arah), `record_per_scanline_crossings` (akumulator signed-area 8.8 fixed-point)
    - Pakai nilai numerik kanonik konsisten (16×8) dengan Rujukan_Kode pendukung; sertakan Rujukan_Kode ke `src/blocks.rs` pada paragraf yang sama dengan setiap token
    - _Requirements: 5.4, 10.1, 10.5, 11.3, 11.5_

  - [x] 3.5 Tulis Subbab 4.2.5 Pembangkit Tile dan Akumulator Backdrop dari `src/builder.rs`
    - Sebutkan `Builder`, `CoverStorage` (field `tag`, `backdrops: Vec<[i16; TILE_H]>`), `Builder::build_path`, `Builder::generate_tiles`, propagasi backdrop kiri-ke-kanan per baris ubin, optimasi SIMD `i16x8`
    - Gunakan istilah kanonik `propagasi backdrop`; sertakan Rujukan_Kode ke `src/builder.rs` pada paragraf yang sama dengan setiap token
    - _Requirements: 5.5, 10.1, 10.5_

  - [x] 3.6 Tulis Subbab 4.2.6 Renderer WebGL dari `src/render/webgl.rs` dan `src/tile.rs`
    - Sebutkan `WebGlRenderer` (field `programs`, `gl: WebGl2RenderingContext`), `WebGlRenderer::new`, `WebGlRenderer::render`, `initialize_tile_vao` (stride 44 byte dari `core::mem::size_of::<Tile>()` + `vertexAttribDivisor(_, 1)`), upload tekstur RGBA32F, `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len())`
    - Sebutkan struct `Tile` `#[repr(C)]` ukuran 44 byte dengan Rujukan_Kode ke `src/tile.rs`; sertakan Rujukan_Kode ke `src/render/webgl.rs` pada paragraf yang sama dengan setiap token
    - _Requirements: 5.6, 5.7, 10.1, 10.5, 11.3, 11.5_

  - [x] 3.7 Tulis Subbab 4.2.7 Shader Vertex dan Fragment dari berkas shader
    - Sebutkan vertex shader instanced quad `src/render/shaders/render_tile.vert` (corner quad → NDC), fragment shader analitik `src/render/shaders/render_tile.frag` (akumulasi `line_box` integral trapezoidal), `WINDING_UNIT = 256.0`, fill rule NonZero `coverage = clamp(abs(winding), 0.0, 1.0)` dan EvenOdd `coverage = 1.0 - abs(mod(abs(winding), 2.0) - 1.0)`
    - Sertakan Rujukan_Kode ke berkas shader yang bersangkutan pada paragraf yang sama dengan setiap token
    - _Requirements: 5.8, 10.1, 10.5, 11.3_

  - [x] 3.8 Tulis Subbab 4.2.8 Demo Interaktif dari `examples/native_webgl/src/{lib,main}.rs`
    - Sebutkan `run_interactive(width, height)` dipanggil dari `main.rs` setelah `device_pixel_ratio()`/`inner_width()`/`inner_height()`, struct `AppState` + `AppState::render` + `AppState::update_overlay`, empat metrik overlay (CPU ms, GPU ms, jumlah paint ops, rasio zoom), tiga Aset_Uji (`Ghostscript Tiger`, `SVG Logo`, `Bismillah`), event handler pan/zoom/keyboard
    - Sertakan Rujukan_Kode ke berkas demo yang bersangkutan pada paragraf yang sama dengan setiap token
    - _Requirements: 5.9, 10.1, 10.5_

- [x] 4. Checkpoint - Validasi struktural dan kelengkapan Subbab 4.1–4.2
  - Pastikan heading 4.1–4.2.8 lengkap dan terurut, setiap Klaim_Teknis 4.1/4.2 memiliki Rujukan_Kode, tidak ada istilah terlarang. Ensure all tests pass, ask the user if questions arise.

- [x] 5. Tulis Subbab 4.3 dan 4.4 (verifikasi correctness + performa)
  - [x] 5.1 Tulis Subbab 4.3 Verifikasi Kebenaran Output
    - Sebutkan ketiga Aset_Uji dengan jalur lengkap (`assets/Ghostscript_Tiger.svg`, `assets/SVG_Logo.svg`, `assets/bismillah.svg`) dan satu Placeholder_Gambar per aset berformat persis `[Gambar 4.x: Hasil rendering {nama aset} oleh Arabella — dimasukkan kemudian]` (penomoran menaik unik)
    - Bahas empat aspek correctness (fill solid color, stroke expansion via `kurbo::stroke_with`, fill rule NonZero, ketiadaan artefak streak/seam) masing-masing minimal satu kalimat; sebutkan pengujian otomatis `wasm-bindgen-test` `tests/test.rs:test_renders_tiger_svg` (1080×520, hanya Tiger) dan validasi manual side-by-side terhadap rendering peramban untuk ketiga aset
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 14.3_

  - [x] 5.2 Tulis Subbab 4.4.1 Metodologi Pengukuran
    - Deskripsikan pengukuran berbasis `performance.now()` (ms) yang memisahkan waktu pra-pemrosesan CPU (`Scene::fill`/`Scene::stroke` → `Builder::build_path`) dari rasterisasi GPU (`WebGlRenderer::render`: upload tekstur + `draw_arrays_instanced`), sampling 60-frame rolling average via `fps_window`, dengan Rujukan_Kode ke `examples/native_webgl/src/lib.rs` (`AppState::render`)
    - Sebutkan dua resolusi: demo window-fill DPR-aware (`width = inner_width × devicePixelRatio`, `examples/native_webgl/src/main.rs`) dan tes otomatis tetap 1080×520 DPR 1.0 (`W: u16 = 1080`, `H: u16 = 520` di `tests/test.rs`)
    - _Requirements: 7.1, 7.2, 10.1, 11.3, 11.5_

  - [x] 5.3 Tulis Subbab 4.4.2 Hasil Pengukuran Per Aset (placeholder tabel)
    - Tulis satu paragraf naratif pengantar lalu sisipkan Placeholder_Tabel berformat persis `[Tabel 4.x — Hasil pengukuran performa per aset — diisi setelah pengujian dilakukan]` dengan kolom Aset, Paint Ops, CPU ms, GPU ms, Total Frame Time ms, FPS dan tepat tiga baris (satu per Aset_Uji), seluruh sel metrik kosong/`—`/`TBD`
    - TIDAK menulis angka FPS/CPU ms/GPU ms karangan apa pun
    - _Requirements: 7.3, 7.4, 11.1, 11.2, 14.4_

  - [x] 5.4 Tulis Subbab 4.4.3 Analisis Perbandingan dengan Baseline
    - Tulis perbandingan kualitatif terhadap Skia (CPU-centric SIMD), Cairo (CPU-centric scanline), Vello (GPU compute-centric) dengan rujukan literal `Subbab 2.2.3`, `Subbab 2.2.4`, `Subbab 2.2.8`, mencakup tiga dimensi (paradigma rasterisasi, ketergantungan compute shader, target platform)
    - Tulis disclaimer eksplisit bahwa benchmark kuantitatif langsung belum dilakukan, data Tabel 4.x perlu dilengkapi, dan hasil bergantung peramban/perangkat keras pada Subbab 4.1
    - _Requirements: 7.5, 7.6, 10.6_

- [x] 6. Tulis Subbab 4.5 dan 4.6 (trade-off + keterbatasan)
  - [x] 6.1 Tulis Subbab 4.5 Pembahasan Trade-off Arsitektur Non-Compute
    - Tulis tiga paragraf berlabel literal: (a) Kompatibilitas Platform (WebGL 2.0 di lebih banyak perangkat dibanding WebGPU/compute shader, tautkan ke peramban target 4.1), (b) Kompleksitas Implementasi (beban pra-pemrosesan ke CPU sebagai pengganti compute shader dispatch), (c) Karakteristik Performa (trade-off latensi transfer CPU→GPU vs paralelisme GPU penuh), masing-masing minimal tiga kalimat
    - Rujuk `Subbab 2.2.3`, `Subbab 2.2.4`, `Subbab 2.2.8`; tutup dengan paragraf rekapitulasi bahwa Arabella mengeliminasi ketergantungan compute shader dengan konsekuensi seluruh komputasi tujuan umum (flattening, binning DDA, akumulasi winding number) dieksekusi di CPU
    - _Requirements: 8.1, 8.2, 8.3, 10.6_

  - [x] 6.2 Tulis Subbab 4.6 Keterbatasan Implementasi Saat Ini
    - Tulis daftar tak berurutan minimal enam butir (prefiks `- `): (a) gradien belum aktif di fragment shader (`src/scene.rs` `encode_paint` → `PAINT_TYPE_SOLID`), (b) image paint/tinting belum diunggah (`src/render/common.rs` `GpuEncodedImage`), (c) Rayon belum aktif di hot path (`Cargo.toml` feature `multithreading` di balik `default = ["std", "png"]`), (d) subset SVG terbatas `g`/`path` + `fill`/`stroke`/`stroke-width`/`transform` (`src/pico_svg.rs:Parser::rec_parse`), (e) tiada text rendering, (f) tiada filter effect; butir (a)–(d) wajib Rujukan_Kode, (e)–(f) tanpa rujukan
    - Tulis paragraf penutup tanpa syarat yang memuat `future work`/`pengembangan lanjutan`, pernyataan bahwa keterbatasan tidak menggugurkan validitas kontribusi inti, dan frasa literal `pipeline hibrida non-compute`
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 10.1_

- [x] 7. Checkpoint - Dokumen lengkap, siap divalidasi
  - Pastikan keenam Subbab_Wajib terisi penuh dan tidak ada Source_Of_Truth yang termodifikasi. Ensure all tests pass, ask the user if questions arise.

- [x] 8. Validasi deterministik (pemetaan Correctness Properties)
  - [x] 8.1 Validasi Property 1 (single-file output identity)
    - **Property 1: Single-File Output Identity**
    - Pencarian rekursif berkas bernama `bab4`/`bab_4`/`bab-4` (abaikan `.kiro/specs/`); pastikan hanya `Skripsi/bab4_implementasi_dan_hasil.md` yang baris pertamanya `# BAB 4 HASIL DAN PEMBAHASAN`, berekstensi `.md`, dan terurai sebagai CommonMark valid
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.5**

  - [x] 8.2 Validasi Property 2 (absence of lorem ipsum)
    - **Property 2: Absence of Lorem Ipsum**
    - Pencarian case-insensitive `Lorem ipsum`, `dolor sit amet`, `consectetur adipiscing`, `eiusmod tempor`, `Excepteur sint occaecat` → kondisi PASS = 0 hit
    - **Validates: Requirements 1.4**

  - [x] 8.3 Validasi Property 3 (structural heading invariant)
    - **Property 3: Structural Heading Invariant**
    - Verifikasi baris pertama berkas, kehadiran tepat satu kali setiap heading wajib pada level yang benar (regex `^##\s+4\.\d+\s+.*$` dan `^###\s+4\.\d+\.\d+\s+.*$`), urutan menaik monotonik 4.1→4.6 dan 4.x.y kontigu, kecualikan kemunculan di fenced code block
    - **Validates: Requirements 1.3, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8**

  - [x] 8.4 Validasi Property 4 (absence of forbidden terms)
    - **Property 4: Absence of Forbidden Terms**
    - Pencarian Istilah_Terlarang (Data Model 4): `Ray Shooting`/`ray shoot` (CI), `TileType`/`winding_number`/`PPGA`/`Projective Geometric Algebra` (CS, token utuh), `EMPTY`/`INTERIOR`/`EDGE` sebagai label tipe ubin, frasa `fungsi implisit ...`/`ditranspilasikan ke WebGL`/`Rust edisi 2021`/`edisi 2021`/`edition = "2021"`, persamaan `ax+by+c=0`/`u-v²=0`/`u-v^2=0`/`C(x,y)=0`/`w_0³-w_1 w_2 w_3` → PASS = 0 hit
    - **Validates: Requirements 11.4**

  - [x] 8.5 Validasi Property 5 (presence of required terms per subsection)
    - **Property 5: Presence of Required Terms Per Subsection**
    - Untuk setiap pasangan (subbab, istilah wajib) pada Data Model 3, pencarian sub-string pada blok teks subbab terkait → PASS = minimal satu kemunculan per pasangan (4.1, 4.2.1–4.2.8, 4.3, 4.4.1, 4.4.3, 4.5, 4.6)
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 5.1, 5.3, 5.4, 5.5, 6.1, 6.2, 6.5, 7.1, 7.2, 7.3, 8.1, 8.3**

  - [x] 8.6 Validasi Property 6 (technical claim traceability)
    - **Property 6: Technical Claim Traceability**
    - Untuk setiap Klaim_Teknis pada 4.1/4.2.1–4.2.8/4.6, verifikasi minimal satu Rujukan_Kode `path:simbol` atau `path:start-end` pada paragraf yang sama; cross-verifikasi via `grep_search` bahwa berkas dan simbol benar-benar ada di Source_Of_Truth (mis. `PicoSvg`, `Scene::fill`, `Builder::generate_tiles`, `bin_line`, `record_per_scanline_crossings`, `Tile`, `WebGlRenderer`, `initialize_tile_vao`, `line_box`, `TILE_W`, `TILE_H`, `FLATNESS_THRESHOLD`, `WINDING_UNIT`, `PAINT_TYPE_SOLID`, `run_interactive`, `test_renders_tiger_svg`)
    - **Validates: Requirements 8.2, 9.1, 9.2, 9.3, 9.4**

  - [x] 8.7 Validasi Property 7 (numerical anti-fabrication)
    - **Property 7: Numerical Anti-Fabrication**
    - Pencarian regex `\d+(\.\d+)?\s*(fps|FPS|ms|MS|millisecond|milidetik)`; setiap hit di 4.4.2/4.4.3 wajib berada di dalam Placeholder_Tabel, dan hit lain wajib berupa konstanta kode yang dirujuk pada paragraf yang sama (`1080×520`→`tests/test.rs`, `44 byte`→`src/tile.rs`, `16×8`→`src/blocks.rs`)
    - **Validates: Requirements 6.4, 10.1, 10.2, 10.3, 10.4**

  - [x] 8.8 Validasi Property 8 (canonical terminology consistency)
    - **Property 8: Canonical Terminology Consistency**
    - Verifikasi kehadiran istilah kanonik (Data Model 5: `binning DDA`, `akumulator signed-area`, `propagasi backdrop`, `fragment shader`, `pra-pemrosesan`/`preprocessing`, `pipeline hibrida`, `viewport`, `winding number`), tiada sinonim non-kanonik, tiada paragraf mencampur `pra-pemrosesan` dan `preprocessing`, `winding number` ada sebagai konsep sedangkan `winding_number` (underscore) tidak ada, istilah teknis Inggris dibungkus italic/backtick
    - **Validates: Requirements 11.1, 11.2, 11.3, 11.5, 13.2**

  - [x] 8.9 Validasi Property 9 (placeholder format invariant)
    - **Property 9: Placeholder Format Invariant**
    - Verifikasi tepat tiga Placeholder_Gambar di 4.3 (`\[Gambar 4\.\d+: Hasil rendering .*\]` untuk Ghostscript Tiger, SVG Logo, Bismillah), minimal satu Placeholder_Tabel di 4.4.2 (`\[Tabel 4\.\d+ — Hasil pengukuran performa per aset — diisi setelah pengujian dilakukan\]`) berkolom kanonik, placeholder gambar lain mengikuti pola `\[Gambar 4\.\d+: .* — dimasukkan kemudian\]`
    - **Validates: Requirements 5.2, 6.3, 13.3, 13.4**

  - [x] 8.10 Validasi Property 10 (cross-chapter narrative link)
    - **Property 10: Cross-Chapter Narrative Link**
    - Verifikasi paragraf pembuka 4.1 atau 4.2 merujuk `Bab 3`, urutan delapan tahap pipeline pada 4.2.5/4.2.7 identik dengan urutan kanonik Bab 3, dan minimal satu rujukan tekstual ke `Subbab 3.4.4`/`3.4.3`/`3.5` pada deskripsi struct utama 4.2
    - **Validates: Requirements 12.1, 12.2, 12.3**

  - [x] 8.11 Validasi kualitatif gaya bahasa akademik
    - Pencarian token percakapan terlarang sebagai kata utuh di luar fenced code block (`bisa`, `gak`, `enggak`, `nih`, `dong`, `kok`, `kan`, `aja`, `udah`, `mau`) dan kata ganti orang pertama/kedua (`saya`, `kami`, `kita`, `Anda`, `kamu`); verifikasi backtick untuk identifier dan italic untuk istilah disiplin; tinjauan kalimat S-P-O
    - _Requirements: 14.1, 14.2, 14.5, 14.6_

- [x] 9. Checkpoint akhir - Seluruh validasi PASS
  - Iterasi pada `Skripsi/bab4_implementasi_dan_hasil.md` hingga seluruh Property 1–10 dan validasi gaya bahasa PASS, tanpa pernah memodifikasi Source_Of_Truth. Ensure all tests pass, ask the user if questions arise.

## Notes

- Tugas bertanda `*` bersifat opsional (validasi deterministik) dan dapat dilewati untuk MVP, namun direkomendasikan dijalankan karena memetakan langsung ke Correctness Properties pada desain.
- Validasi properti di sini adalah pencarian teks/struktural deterministik (regex + cross-reference), bukan property-based testing iteratif acak — sesuai bagian "Mengapa Property-Based Testing Tidak Berlaku" pada desain.
- Setiap tugas penulisan mengedit berkas keluaran tunggal yang sama; karena itu tugas penulisan dijadwalkan berurutan antar-wave, sedangkan tugas validasi read-only dapat berjalan paralel.
- Requirement 2 (Source-of-Truth Invariance) berlaku global untuk seluruh tugas: `src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml` dibaca strict read-only dan tidak boleh dimodifikasi, ditambah, dihapus, atau dipindah.
- Tidak boleh menjalankan perintah yang memutasi Source_Of_Truth (mis. `cargo build`/`cargo test`/`cargo fmt` yang menulis ulang berkas).
- Checkpoint memastikan validasi inkremental sebelum melanjutkan ke subbab berikutnya.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["2.1"] },
    { "id": 2, "tasks": ["2.2"] },
    { "id": 3, "tasks": ["3.1"] },
    { "id": 4, "tasks": ["3.2"] },
    { "id": 5, "tasks": ["3.3"] },
    { "id": 6, "tasks": ["3.4"] },
    { "id": 7, "tasks": ["3.5"] },
    { "id": 8, "tasks": ["3.6"] },
    { "id": 9, "tasks": ["3.7"] },
    { "id": 10, "tasks": ["3.8"] },
    { "id": 11, "tasks": ["5.1"] },
    { "id": 12, "tasks": ["5.2"] },
    { "id": 13, "tasks": ["5.3"] },
    { "id": 14, "tasks": ["5.4"] },
    { "id": 15, "tasks": ["6.1"] },
    { "id": 16, "tasks": ["6.2"] },
    { "id": 17, "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "8.10", "8.11"] }
  ]
}
```
