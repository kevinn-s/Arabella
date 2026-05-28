# Requirements Document

## Introduction

Dokumen ini menetapkan persyaratan penulisan berkas `Skripsi/bab4_implementasi_dan_hasil.md` dari nol. Berkas tersebut saat ini berisi 100% teks placeholder (lorem ipsum) dan harus diganti seluruhnya dengan narasi akademik yang mendeskripsikan implementasi sistem Arabella serta hasil pengujiannya.

Spec ini adalah spec dokumentasi akademik. Yang ditulis adalah konten naratif, bukan kode. Source code Arabella berfungsi sebagai sumber kebenaran (source of truth) yang tidak boleh dimodifikasi. Setiap klaim implementasi dalam Bab 4 harus dapat dilacak ulang ke berkas atau simbol kode di `src/`, `examples/`, `tests/`, atau ke entri di `Cargo.toml`.

Penulisan mempertahankan terminologi lintas-bab yang sudah konsisten dengan Bab 1, Bab 2, dan Bab 3 (yang akan direvisi secara terpisah melalui spec `revisi-bab3-metodologi`). Istilah kanonik yang dipakai mengikuti daftar Istilah_Wajib dan Istilah_Terlarang pada spec tersebut.

## Glossary

- **Bab_4**: Berkas `Skripsi/bab4_implementasi_dan_hasil.md` versi pasca-penulisan (output akhir spec ini).
- **Source_Of_Truth**: Source code Arabella di direktori `src/`, `examples/`, `tests/`, beserta `Cargo.toml` dan `.cargo/config.toml` di akar repositori.
- **Laporan_Analisis**: Berkas `Skripsi/analisis_project_dan_skripsi.md` yang mendokumentasikan ringkasan sistem, fitur, teknologi, dan struktur Bab 4 yang diusulkan (Bagian G).
- **Spec_Bab3**: Berkas `.kiro/specs/revisi-bab3-metodologi/requirements.md` yang menjadi referensi terminologi dan konsistensi.
- **Penulis**: Subjek aktif yang menulis Bab 4. Dalam dokumen requirements ini "Penulis" adalah sistem agen yang melaksanakan penulisan.
- **Klaim_Teknis**: Pernyataan dalam Bab 4 yang menyebut salah satu dari: (a) nama algoritma atau struktur data, (b) nilai parameter numerik konkret, (c) nama berkas, fungsi, struct, trait, modul, atau konstanta dalam kode, atau (d) perilaku runtime spesifik dari pustaka Arabella.
- **Rujukan_Kode**: Kombinasi jalur berkas relatif terhadap akar repositori ditambah salah satu dari: nama fungsi, nama konstanta, nama struct, atau rentang baris.
- **Placeholder_Numerik**: Teks berformat `[Tabel 4.x — diisi setelah pengujian dilakukan]` atau `[Gambar 4.x: deskripsi — dimasukkan kemudian]` yang menandai data empiris yang belum dikumpulkan.
- **Subbab_Wajib**: Heading subbab yang harus hadir di Bab 4: 4.1, 4.2 (4.2.1–4.2.8), 4.3, 4.4 (4.4.1–4.4.3), 4.5, 4.6.

## Requirements

### Requirement 1: Identitas Berkas Output

**User Story:** Sebagai dosen pembimbing, saya ingin satu berkas tunggal `Skripsi/bab4_implementasi_dan_hasil.md` hasil penulisan menggantikan placeholder lama, sehingga saya tidak perlu menelusuri beberapa lokasi untuk membaca Bab 4.

#### Acceptance Criteria

1. THE Penulis SHALL menulis hasil penulisan pada satu berkas tunggal yaitu `Skripsi/bab4_implementasi_dan_hasil.md` relatif terhadap akar repositori.
2. THE Penulis SHALL menyimpan berkas dengan ekstensi `.md` dan sintaks Markdown CommonMark yang dapat diurai tanpa galat.
3. THE Penulis SHALL meletakkan heading utama `# BAB 4 HASIL DAN PEMBAHASAN` pada baris pertama berkas dengan kapitalisasi penuh, satu spasi tunggal antar kata, tanpa karakter tambahan.
4. THE Penulis SHALL menghapus seluruh teks lorem ipsum yang ada pada berkas saat ini dan menggantinya dengan konten substantif.
5. THE Penulis SHALL TIDAK membuat berkas Bab 4 tambahan di lokasi lain pada repositori.

### Requirement 2: Kelengkapan Struktural Subbab Wajib

**User Story:** Sebagai mahasiswa yang menulis Bab 4 sesuai panduan kampus, saya ingin seluruh subbab wajib hadir dengan heading yang konsisten, sehingga struktur Bab 4 mengikuti panduan akademik.

#### Acceptance Criteria

1. THE Bab_4 SHALL memuat heading `## 4.1 Spesifikasi Lingkungan Implementasi` tepat satu kali pada level heading 2.
2. THE Bab_4 SHALL memuat heading `## 4.2 Implementasi Modul` tepat satu kali pada level heading 2, beserta subheading `### 4.2.1 Parser SVG`, `### 4.2.2 Scene API`, `### 4.2.3 Path Processing dan Flattening`, `### 4.2.4 Tile Binning DDA`, `### 4.2.5 Pembangkit Tile dan Akumulator Backdrop`, `### 4.2.6 Renderer WebGL`, `### 4.2.7 Shader Vertex dan Fragment`, dan `### 4.2.8 Demo Interaktif`, masing-masing tepat satu kali pada level heading 3.
3. THE Bab_4 SHALL memuat heading `## 4.3 Verifikasi Kebenaran Output` tepat satu kali pada level heading 2.
4. THE Bab_4 SHALL memuat heading `## 4.4 Pengujian Performa` tepat satu kali pada level heading 2, beserta subheading `### 4.4.1 Metodologi Pengukuran`, `### 4.4.2 Hasil Pengukuran Per Aset`, dan `### 4.4.3 Analisis Perbandingan dengan Baseline`, masing-masing tepat satu kali pada level heading 3.
5. THE Bab_4 SHALL memuat heading `## 4.5 Pembahasan Trade-off Arsitektur Non-Compute` tepat satu kali pada level heading 2.
6. THE Bab_4 SHALL memuat heading `## 4.6 Keterbatasan Implementasi Saat Ini` tepat satu kali pada level heading 2.
7. THE Bab_4 SHALL menyusun heading subbab level 2 pada urutan menaik monotonik 4.1, 4.2, 4.3, 4.4, 4.5, 4.6 tanpa nomor yang dilewati, diulang, atau dibalik.
8. THE Bab_4 SHALL menempatkan setiap subheading `### 4.x.y` setelah heading induknya `## 4.x` dan sebelum heading subbab level 2 berikutnya, dengan urutan `y` menaik monotonik.

### Requirement 3: Spesifikasi Lingkungan Implementasi (Subbab 4.1)

**User Story:** Sebagai penguji, saya ingin mengetahui lingkungan pengembangan dan parameter build yang dipakai, sehingga saya dapat mereplikasi hasil.

#### Acceptance Criteria

1. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 bahasa pemrograman Rust edisi 2024 dengan rujukan ke `Cargo.toml` (`edition = "2024"`).
2. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 target kompilasi `wasm32-unknown-unknown` dengan SIMD128 enabled, dengan rujukan ke `.cargo/config.toml` (`target-feature=+simd128`).
3. THE Bab_4 SHALL mendaftarkan pada Subbab 4.1 parameter profil release: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = false`, `strip = true`, dengan rujukan ke `[profile.release]` di `Cargo.toml`.
4. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 peramban target untuk evaluasi (Chrome dengan dukungan WebGL 2.0 dan WASM SIMD128).
5. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 toolchain build: `wasm-pack` untuk pengujian dan `cargo-run-wasm` untuk demo interaktif, dengan rujukan ke `examples/run_wasm/Cargo.toml`.

### Requirement 4: Implementasi Modul (Subbab 4.2) Mencerminkan Source Code

**User Story:** Sebagai dosen penguji, saya ingin setiap subbab implementasi modul mendeskripsikan modul yang benar-benar ada di source code, sehingga narasi dapat diverifikasi.

#### Acceptance Criteria

1. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.1 modul parser SVG (`src/pico_svg.rs`) dengan menyebutkan: struct `PicoSvg`, enum `Item` (varian `Fill`, `Stroke`, `Group`), subset elemen SVG yang didukung (`g`, `path`, atribut `fill`, `stroke`, `stroke-width`, `transform`), dan pustaka `roxmltree` sebagai parser XML.
2. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.2 Scene API (`src/scene.rs`) dengan menyebutkan: struct `Scene`, method `Scene::new`, `Scene::fill`, `Scene::stroke`, `Scene::reset`, encoding paint solid color melalui `encode_paint`, dan delegasi stroke expansion ke `kurbo::stroke_with`.
3. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.3 path processing (`src/path.rs`, `src/flatten.rs`) dengan menyebutkan: transformasi affine SIMD-batched (`transform_pair`, `transform_quad` menggunakan `f32x4` dan `f32x8`), konversi cubic-to-quadratic (`convert_cubics_to_quadratic_curves`, `estimate_number_of_quadratic_curves`), midpoint subdivision Blaze (`flatten_quadratic` di `src/flatten.rs`), format F24Dot8 (`f32_to_f24dot8`), dan flatness threshold.
4. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.4 tile binning DDA (`src/blocks.rs`) dengan menyebutkan: struct `Block`, `Blocks`, `TileBounds`, fungsi `bin_line` (outer DDA empat arah diagonal + kasus khusus), fungsi `bin_line_in_row` (inner DDA empat arah), fungsi `record_per_scanline_crossings` (akumulator signed-area 8.8 fixed-point), dan ukuran ubin 16×8 piksel (`TILE_W = 16`, `TILE_H = 8`).
5. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.5 pembangkit tile (`src/builder.rs`) dengan menyebutkan: struct `Builder`, `CoverStorage`, method `Builder::build_path` dan `Builder::generate_tiles`, propagasi backdrop kiri-ke-kanan, dan optimasi SIMD `i16x8` untuk akumulasi per baris.
6. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.6 renderer WebGL (`src/render/webgl.rs`) dengan menyebutkan: struct `WebGlRenderer`, pembuatan `WebGl2RenderingContext`, kompilasi shader (`create_shader_program`), VAO instanced (`initialize_tile_vao` dengan stride 44 byte), upload tekstur RGBA32F untuk segmen, dan upload vertex buffer untuk tile.
7. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.7 shader vertex dan fragment dengan menyebutkan: vertex shader instanced quad (`render_tile.vert`) yang memetakan posisi ubin ke NDC, fragment shader analitik (`render_tile.frag`) dengan fungsi `line_box` (integral trapezoidal cakupan piksel), `read_backdrop` (lookup per-scanline backdrop 8.8 fixed-point), dan penerapan fill rule NonZero (`clamp(abs(winding), 0, 1)`) serta EvenOdd (`1 - abs(mod(w, 2) - 1)`).
8. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.8 demo interaktif (`examples/native_webgl/`) dengan menyebutkan: fungsi `run_interactive`, event handler (pan, zoom, keyboard), overlay FPS empat metrik (CPU ms, GPU ms, jumlah ops, zoom), tiga aset uji (Ghostscript Tiger, SVG Logo, Bismillah), dan mekanisme `requestAnimationFrame` loop.

### Requirement 5: Verifikasi Kebenaran Output (Subbab 4.3)

**User Story:** Sebagai pembaca, saya ingin melihat bukti bahwa output rendering Arabella secara visual benar, sehingga saya yakin implementasi berfungsi.

#### Acceptance Criteria

1. THE Bab_4 SHALL menyebutkan pada Subbab 4.3 tiga aset uji yang digunakan: Ghostscript Tiger (`assets/Ghostscript_Tiger.svg`), SVG Logo (`assets/SVG_Logo.svg`), dan Bismillah (`assets/bismillah.svg`).
2. THE Bab_4 SHALL menyertakan placeholder gambar dengan format `[Gambar 4.x: Hasil rendering {nama aset} oleh Arabella — dimasukkan kemudian]` untuk setiap aset uji.
3. THE Bab_4 SHALL membahas pada Subbab 4.3 aspek correctness yang diverifikasi: fill solid color, stroke expansion, fill rule NonZero, dan ketidakhadiran artefak visual (streak, seam pada batas ubin).
4. THE Bab_4 SHALL menyebutkan pada Subbab 4.3 bahwa pengujian otomatis dilakukan melalui `wasm-bindgen-test` pada `tests/test.rs` dengan fungsi `test_renders_tiger_svg`.
5. THE Bab_4 SHALL menyebutkan pada Subbab 4.3 bahwa validasi visual dilakukan secara manual dengan membandingkan output Arabella terhadap rendering peramban (SVG native rendering) pada aset yang sama.

### Requirement 6: Pengujian Performa (Subbab 4.4)

**User Story:** Sebagai dosen penguji, saya ingin melihat metodologi pengukuran performa yang jelas dan hasil yang dapat direproduksi, sehingga klaim performa dapat dipertanggungjawabkan.

#### Acceptance Criteria

1. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.4.1 metodologi pengukuran: penggunaan `performance.now()` untuk mengukur waktu CPU dan GPU secara terpisah, sampling 60-frame rolling average untuk FPS, dan rujukan ke implementasi di `examples/native_webgl/src/lib.rs` (method `AppState::render`).
2. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.4.1 resolusi pengujian: demo interaktif pada resolusi window-fill DPR-aware, dan tes otomatis pada resolusi tetap 1080×520 piksel.
3. THE Bab_4 SHALL menyertakan pada Subbab 4.4.2 placeholder tabel dengan format `[Tabel 4.x — Hasil pengukuran performa per aset — diisi setelah pengujian dilakukan]` yang mencakup kolom: nama aset, jumlah paint ops, CPU ms, GPU ms, total frame time, dan FPS.
4. THE Bab_4 SHALL TIDAK memuat angka FPS, CPU ms, atau GPU ms karangan pada Subbab 4.4.2; seluruh nilai numerik performa yang belum diukur harus berupa Placeholder_Numerik.
5. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.4.3 perbandingan kualitatif terhadap Skia, Cairo, dan Vello dengan disclaimer eksplisit bahwa benchmark kuantitatif langsung terhadap renderer lain belum dilakukan pada implementasi saat ini.

### Requirement 7: Pembahasan Trade-off (Subbab 4.5)

**User Story:** Sebagai pembaca, saya ingin memahami trade-off arsitektur non-compute yang dipilih Arabella, sehingga saya dapat menilai kontribusi penelitian.

#### Acceptance Criteria

1. THE Bab_4 SHALL membahas pada Subbab 4.5 minimal tiga dimensi trade-off: (a) kompatibilitas platform (WebGL 2.0 tersedia di lebih banyak perangkat dibanding WebGPU), (b) kompleksitas implementasi (beban preprocessing di CPU vs compute shader dispatch), dan (c) karakteristik performa (latensi transfer CPU→GPU vs paralelisme GPU penuh).
2. THE Bab_4 SHALL merujuk pada Subbab 4.5 arsitektur Vello (compute-centric) dan Skia/Cairo (CPU-centric) sebagai titik perbandingan, menggunakan informasi dari Bab 2 (Subbab 2.2.3, 2.2.4, 2.2.8).
3. THE Bab_4 SHALL menyatakan pada Subbab 4.5 bahwa pendekatan Arabella mengeliminasi ketergantungan pada compute shader dengan konsekuensi bahwa seluruh komputasi tujuan umum (flattening, binning, winding) dieksekusi di CPU.

### Requirement 8: Keterbatasan Implementasi (Subbab 4.6)

**User Story:** Sebagai dosen penguji, saya ingin mengetahui secara jujur fitur apa saja yang belum berfungsi, sehingga evaluasi terhadap skripsi tetap valid dan proporsional.

#### Acceptance Criteria

1. THE Bab_4 SHALL mendaftarkan pada Subbab 4.6 minimal enam keterbatasan berikut: (a) gradien linear/radial/sweep belum aktif di fragment shader (encoding ada di `src/paint/encode.rs` namun shader hanya menjalankan `PAINT_TYPE_SOLID`), (b) image paint dan tinting belum aktif, (c) Rayon belum diaktifkan pada hot path (feature `multithreading` opsional), (d) subset SVG yang didukung terbatas pada `path`, `g`, `fill`, `stroke`, `transform`, (e) tiada sistem text rendering, (f) tiada filter effect (blur, drop shadow).
2. THE Bab_4 SHALL menyertakan rujukan kode untuk setiap keterbatasan yang diklaim pada Subbab 4.6 (misalnya `src/scene.rs:encode_paint` untuk gradien TODO, `Cargo.toml:[features]` untuk Rayon opsional).
3. THE Bab_4 SHALL menyatakan pada Subbab 4.6 bahwa keterbatasan-keterbatasan tersebut merupakan peluang pengembangan lanjutan (future work) dan tidak mengurangi validitas kontribusi inti penelitian (pipeline hibrida non-compute).

### Requirement 9: Traceability Setiap Klaim Teknis

**User Story:** Sebagai dosen penguji, saya ingin setiap klaim teknis di Bab 4 dapat saya verifikasi langsung ke berkas kode, sehingga saya dapat memastikan tidak ada klaim fabrikasi.

#### Acceptance Criteria

1. WHEN Bab_4 memuat klaim teknis tentang modul tertentu pada Subbab 4.2.1 sampai 4.2.8, THE Penulis SHALL menyertakan rujukan kode (jalur berkas + nama fungsi/struct/konstanta) yang dapat diverifikasi terhadap Source_Of_Truth.
2. WHEN Bab_4 memuat klaim parameter numerik (16, 8, 256, 44 byte, 1080×520, dll), THE Penulis SHALL menyertakan rujukan kode ke berkas yang mendefinisikan parameter tersebut.
3. THE Penulis SHALL TIDAK memasukkan ke Bab_4 klaim teknis yang tidak dapat ditelusuri ke berkas pada `src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, atau `.cargo/config.toml`.
4. THE Penulis SHALL TIDAK mengarang nama fungsi, nama struct, nama trait, nama modul, nama feature flag, nama crate, maupun nilai parameter numerik yang tidak muncul secara literal pada Source_Of_Truth.

### Requirement 10: Anti-Fabrikasi Numerik

**User Story:** Sebagai pembaca, saya ingin yakin bahwa angka performa yang disajikan adalah hasil pengukuran nyata atau ditandai secara eksplisit sebagai belum diukur, sehingga tidak ada data fiktif.

#### Acceptance Criteria

1. THE Bab_4 SHALL TIDAK memuat nilai FPS, CPU ms, GPU ms, atau metrik performa lain yang bukan berasal dari pengukuran aktual pada Subbab 4.4.2.
2. WHERE data performa belum tersedia, THE Bab_4 SHALL menggunakan Placeholder_Numerik dengan format `[Tabel 4.x — diisi setelah pengujian dilakukan]` atau `[Nilai — diisi setelah pengukuran]`.
3. WHERE Bab_4 menyebut nilai numerik yang berasal dari konstanta kode (misalnya `TILE_W = 16`, `TILE_H = 8`, stride 44 byte, resolusi 1080×520), THE Bab_4 SHALL menyertakan rujukan kode yang mendefinisikan konstanta tersebut.
4. THE Bab_4 SHALL TIDAK memuat grafik atau tabel dengan data numerik performa yang diisi dengan angka estimasi, proyeksi, atau asumsi tanpa label eksplisit yang menyatakan sifat estimatif data tersebut.

### Requirement 11: Konsistensi Terminologi Lintas-Bab

**User Story:** Sebagai pembaca skripsi yang membaca Bab 1 sampai Bab 4 secara berurutan, saya ingin istilah lintas-bab tetap konsisten, sehingga Bab 4 tidak merusak referensi silang dari bab sebelumnya.

#### Acceptance Criteria

1. THE Bab_4 SHALL menggunakan istilah "pipeline hibrida" pada setiap penyebutan arsitektur Arabella secara keseluruhan.
2. THE Bab_4 SHALL menggunakan istilah "binning DDA" untuk tahap pemecahan segmen lintas ubin, "akumulator signed-area" untuk akumulator winding 8.8 fixed-point per scanline, dan "propagasi backdrop" untuk akumulasi kiri-ke-kanan saat emisi tile — konsisten dengan Spec_Bab3.
3. THE Bab_4 SHALL menggunakan istilah "pra-pemrosesan" atau "preprocessing" untuk fase CPU secara konsisten (tidak mencampur kedua varian dalam satu paragraf).
4. THE Bab_4 SHALL TIDAK memuat Istilah_Terlarang dari Spec_Bab3 (Ray Shooting, TileType, EMPTY/INTERIOR/EDGE sebagai label tipe ubin, fungsi implisit kanonik, PPGA, C(x,y)=0, OpenGL ES ditranspilasi, edisi 2021).
5. THE Bab_4 SHALL mempertahankan istilah "winding number" sebagai konsep, tanpa menjadikannya nama field skalar pada struct Tile.

### Requirement 12: Konektivitas Naratif dengan Bab 3

**User Story:** Sebagai pembaca, saya ingin Bab 4 terhubung secara naratif dengan Bab 3, sehingga transisi antar bab terasa koheren.

#### Acceptance Criteria

1. THE Bab_4 SHALL memuat pada paragraf pembuka Subbab 4.1 atau 4.2 kalimat penghubung yang merujuk perancangan pada Bab 3 (misalnya "Berdasarkan perancangan arsitektur pipeline hibrida yang telah diuraikan pada Bab 3, bab ini menyajikan implementasi purwarupa beserta hasil pengujiannya.").
2. THE Bab_4 SHALL merujuk urutan tahap pipeline yang sama dengan yang dideskripsikan pada UC-03 Bab 3 (flatten → outer DDA → inner DDA → akumulator signed-area → emisi tile → propagasi backdrop → vertex shader → fragment shader).
3. THE Bab_4 SHALL merujuk class diagram Bab 3 saat mendeskripsikan struct utama pada Subbab 4.2 (misalnya "Struct `Builder` yang telah dirancang pada Subbab 3.4.4 diimplementasikan pada berkas `src/builder.rs`...").

### Requirement 13: Gaya Bahasa dan Format Akademik

**User Story:** Sebagai mahasiswa yang menulis skripsi sesuai panduan kampus, saya ingin Bab 4 menggunakan gaya bahasa akademik formal Indonesia yang konsisten dengan bab-bab sebelumnya.

#### Acceptance Criteria

1. THE Bab_4 SHALL ditulis dalam bahasa Indonesia formal akademik dengan kalimat lengkap (subjek-predikat-objek), tanpa singkatan kasual, dan tanpa bahasa percakapan.
2. THE Bab_4 SHALL menggunakan istilah teknis berbahasa Inggris dalam format italic atau backtick (misalnya `flatten_quadratic`, *midpoint subdivision*) sesuai konvensi yang sudah berlaku di Bab 1–3.
3. THE Bab_4 SHALL menyertakan placeholder gambar dengan format `[Gambar 4.x: deskripsi — dimasukkan kemudian]` untuk setiap ilustrasi yang belum tersedia (screenshot rendering, diagram arsitektur modul).
4. THE Bab_4 SHALL menyertakan placeholder tabel dengan format `[Tabel 4.x: deskripsi — diisi setelah pengujian dilakukan]` untuk setiap tabel data empiris yang belum dikumpulkan.
