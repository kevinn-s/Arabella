# Requirements Document

## Introduction

Dokumen ini menetapkan persyaratan penulisan berkas `Skripsi/bab4_implementasi_dan_hasil.md` dari nol. Berkas tersebut saat ini berisi 100% teks placeholder (lorem ipsum) dan harus diganti seluruhnya dengan narasi akademik berbahasa Indonesia formal yang mendeskripsikan implementasi pustaka Arabella beserta hasil pengujiannya.

Spec ini adalah spec dokumentasi akademik. Yang ditulis adalah konten naratif Markdown, bukan kode. Source code Arabella berfungsi sebagai sumber kebenaran (source of truth) yang TIDAK BOLEH dimodifikasi oleh spec ini. Setiap klaim teknis dalam Bab 4 harus dapat ditelusuri ulang ke berkas atau simbol kode di `src/`, `examples/`, `tests/`, atau ke entri di `Cargo.toml` dan `.cargo/config.toml`.

Bab 4 melaporkan dua kelas konten yang berbeda. Kelas pertama adalah deskripsi implementasi yang seluruhnya dapat ditulis sekarang berdasarkan source code (Subbab 4.1, 4.2.1–4.2.8, 4.4.1, 4.5, 4.6). Kelas kedua adalah hasil pengujian empiris yang membutuhkan pengukuran yang belum dilakukan (Subbab 4.3 untuk gambar hasil rendering, Subbab 4.4.2 dan 4.4.3 untuk angka performa). Konten kelas kedua harus ditulis sebagai kerangka naratif lengkap dengan penanda placeholder eksplisit; tidak boleh ada angka FPS, CPU ms, atau GPU ms karangan yang muncul di dokumen.

Penulisan mempertahankan terminologi lintas-bab yang sudah konsisten dengan Bab 1, Bab 2, dan Bab 3 yang sudah direvisi melalui spec `revisi-bab3-metodologi`. Istilah kanonik yang dipakai mengikuti daftar Istilah_Wajib dan Istilah_Terlarang pada spec tersebut, dan diteruskan secara eksplisit pada Glossary di bawah ini agar Bab 4 dapat divalidasi tanpa perlu membuka spec lain.

## Glossary

- **Bab_4**: Berkas `Skripsi/bab4_implementasi_dan_hasil.md` versi pasca-penulisan (output akhir spec ini).
- **Source_Of_Truth**: Source code Arabella di direktori `src/`, `examples/`, `tests/`, `assets/`, beserta `Cargo.toml`, `Cargo.lock`, dan `.cargo/config.toml` di akar repositori. Source_Of_Truth bersifat read-only untuk spec ini.
- **Laporan_Analisis**: Berkas `Skripsi/analisis_project_dan_skripsi.md` yang mendokumentasikan ringkasan sistem, fitur, teknologi, dan struktur Bab 4 yang diusulkan (Bagian G).
- **Bab_3_Final**: Berkas `Skripsi/bab3_metodologi.md` versi pasca-revisi yang dihasilkan oleh spec `revisi-bab3-metodologi`. Bab_4 wajib menjaga konsistensi terminologi, klaim numerik, dan rujukan kode dengan Bab_3_Final.
- **Spec_Bab3**: Direktori `.kiro/specs/revisi-bab3-metodologi/` yang berisi requirements, design, dan tasks untuk revisi Bab 3.
- **Penulis**: Subjek aktif yang menulis Bab 4. Dalam dokumen requirements ini "Penulis" adalah sistem agen yang melaksanakan penulisan.
- **Klaim_Teknis**: Pernyataan dalam Bab 4 yang menyebut salah satu dari: (a) nama algoritma atau struktur data, (b) nilai parameter numerik konkret (termasuk dimensi ubin, format fixed-point, ukuran rekord byte), (c) nama berkas, fungsi, struct, trait, modul, konstanta, feature flag, atau crate dalam kode, atau (d) perilaku runtime spesifik dari pustaka Arabella.
- **Rujukan_Kode**: Kombinasi jalur berkas relatif terhadap akar repositori yang ditulis dalam backtick, ditambah salah satu dari: nama fungsi, nama konstanta, nama struct, atau rentang baris, dengan format `\`berkas:simbol\`` (contohnya `` `src/blocks.rs:bin_line` ``) atau `\`berkas:start-end\`` (contohnya `` `src/blocks.rs:107-160` ``).
- **Placeholder_Tabel**: Penanda teks berformat persis `[Tabel 4.x — deskripsi — diisi setelah pengujian dilakukan]` dengan `4.x` sebagai nomor tabel dan `deskripsi` sebagai keterangan singkat tabel; menandai tabel data empiris yang belum dikumpulkan.
- **Placeholder_Gambar**: Penanda teks berformat persis `[Gambar 4.x: deskripsi — dimasukkan kemudian]` dengan `4.x` sebagai nomor gambar dan `deskripsi` sebagai keterangan singkat gambar; menandai ilustrasi visual yang belum tersedia.
- **Subbab_Wajib**: Heading subbab yang harus hadir di Bab 4: 4.1, 4.2 (dengan subheading 4.2.1 sampai 4.2.8), 4.3, 4.4 (dengan subheading 4.4.1 sampai 4.4.3), 4.5, dan 4.6.
- **Aset_Uji**: Tiga berkas SVG di direktori `assets/` yang dipakai sebagai aset uji rendering: `Ghostscript_Tiger.svg`, `SVG_Logo.svg`, dan `bismillah.svg`.
- **Istilah_Wajib**: Himpunan istilah yang wajib muncul minimal satu kali pada subbab tertentu sebagaimana dirinci pada AC Requirement 3, 4, 5, 6, 7, dan 8.
- **Istilah_Terlarang**: Himpunan istilah yang tidak boleh muncul pada Bab 4. Daftar diteruskan dari Spec_Bab3 dan dirinci pada AC Requirement 11.

## Requirements

### Requirement 1: Identitas Berkas Output Tunggal

**User Story:** Sebagai dosen pembimbing, saya ingin satu berkas tunggal `Skripsi/bab4_implementasi_dan_hasil.md` hasil penulisan menggantikan placeholder lama, sehingga saya tidak perlu menelusuri beberapa lokasi untuk membaca Bab 4.

#### Acceptance Criteria

1. THE Penulis SHALL menempatkan seluruh unit konten Bab_4 — heading utama level 1, seluruh heading Subbab_Wajib, paragraf naratif, Placeholder_Tabel, dan Placeholder_Gambar — di dalam satu berkas tunggal dengan jalur `Skripsi/bab4_implementasi_dan_hasil.md` relatif terhadap akar repositori.
2. THE Penulis SHALL menyimpan berkas Bab_4 dengan ekstensi `.md`, encoding UTF-8 tanpa BOM, dan akhiran baris yang konsisten sehingga isinya dapat diurai sebagai Markdown CommonMark valid dan dicocokkan secara byte-deterministik.
3. THE Penulis SHALL meletakkan heading utama `# BAB 4 HASIL DAN PEMBAHASAN` pada baris pertama berkas Bab_4 dengan kapitalisasi penuh, prefiks `# ` (satu tanda pagar diikuti tepat satu spasi), tanpa BOM, tanpa karakter whitespace di awal maupun akhir baris heading, dan diakhiri tepat satu karakter newline tunggal sebelum konten berikutnya.
4. THE Penulis SHALL menghapus seluruh teks lorem ipsum yang ada pada berkas saat ini sehingga frasa `lorem ipsum`, `dolor sit amet`, `consectetur adipiscing`, `tempor incididunt`, `exercitation ullamco`, dan `duis aute irure` (case-insensitive) tidak muncul pada Bab_4 final; substansi naratif yang menggantikan teks placeholder tersebut diatur oleh Requirement 3 sampai Requirement 14.
5. THE Penulis SHALL TIDAK membuat berkas Bab 4 tambahan, salinan parsial, cadangan, atau draf alternatif di repositori, di mana "berkas Bab 4 tambahan" didefinisikan sebagai berkas yang memenuhi salah satu dari dua kriteria berikut: (a) baris pertama berkas mencocokkan persis `# BAB 4 HASIL DAN PEMBAHASAN`, atau (b) nama berkas mengandung substring `bab4` (case-insensitive); pemindaian kriteria ini SHALL mengabaikan seluruh berkas di dalam direktori `.kiro/specs/` agar berkas-berkas spec ini sendiri tidak menghasilkan false positive.
6. IF terdapat berkas lain di repositori (di luar direktori `.kiro/specs/`) yang baris pertamanya mencocokkan persis `# BAB 4 HASIL DAN PEMBAHASAN`, THEN THE Penulis SHALL menghapus berkas tersebut sehingga `Skripsi/bab4_implementasi_dan_hasil.md` menjadi satu-satunya pemilik heading tersebut.
7. IF berkas Bab_4 memiliki ekstensi `.md` namun gagal diurai sebagai Markdown CommonMark valid oleh parser referensi, THEN THE Bab_4 SHALL dianggap tidak memenuhi syarat Requirement 1 hingga galat sintaks tersebut diperbaiki.

### Requirement 2: Source-of-Truth Invariance

**User Story:** Sebagai pengembang yang juga merangkap sebagai mahasiswa, saya ingin penulisan Bab 4 tidak mengubah satu pun berkas source code, sehingga purwarupa yang dievaluasi pada Bab 4 sama persis dengan implementasi yang sudah saya selesaikan.

#### Acceptance Criteria

1. THE Penulis SHALL TIDAK memodifikasi berkas apa pun di dalam Source_Of_Truth (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`), di mana "memodifikasi" berarti perubahan konten pada level byte antara titik mulai dan titik akhir eksekusi spec ini.
2. THE Penulis SHALL TIDAK menambah, menghapus, memindahkan, maupun menamai ulang berkas apa pun di dalam Source_Of_Truth; jumlah berkas dan himpunan jalur relatif berkas di dalam Source_Of_Truth SHALL identik antara titik mulai dan titik akhir eksekusi spec ini.
3. WHILE eksekusi spec ini berlangsung — yaitu periode dari mulainya tugas pertama pada `tasks.md` sampai selesainya tugas terakhir pada `tasks.md` — THE Penulis SHALL TIDAK mengubah konten berkas skripsi lain (`Skripsi/bab1_pendahuluan.md`, `Skripsi/bab2_landasan_teori.md`, `Skripsi/bab3_metodologi.md`, `Skripsi/bab5_kesimpulan.md`, `Skripsi/abstrak.md`, `Skripsi/kata_pengantar.md`, `Skripsi/daftar_pustaka.md`, `Skripsi/analisis_project_dan_skripsi.md`).
4. WHEN Penulis perlu memverifikasi nama simbol, signature fungsi, atau nilai konstanta untuk Klaim_Teknis, THE Penulis SHALL membaca berkas Source_Of_Truth secara strict read-only tanpa operasi tulis (write), tambah (append), pembuatan berkas baru, pembuatan berkas sementara, maupun perubahan metadata (mode akses, timestamp) di dalam direktori Source_Of_Truth.
5. IF Penulis menemukan inkonsistensi antara klaim yang ingin ditulis dan Source_Of_Truth — di mana "inkonsistensi" berarti perbedaan yang dapat diobservasi pada nama simbol, jalur berkas, signature fungsi, nilai konstanta, struktur tipe, atau perilaku runtime — THEN THE Penulis SHALL mengubah klaim agar sesuai Source_Of_Truth, dan IF klaim tidak memiliki referen apa pun di Source_Of_Truth, THEN THE Penulis SHALL menghapus klaim tersebut alih-alih membiarkannya pada Bab_4.
6. THE Penulis SHALL TIDAK menjalankan perintah yang memutasi Source_Of_Truth sebagai efek samping, termasuk namun tidak terbatas pada `cargo build`, `cargo test`, `cargo update`, atau perintah formatter yang menulis ulang berkas sumber, selama setiap perintah semacam itu menghasilkan perubahan byte pada berkas mana pun di dalam Source_Of_Truth.

### Requirement 3: Kelengkapan Struktural Subbab Wajib

**User Story:** Sebagai mahasiswa yang menulis Bab 4 sesuai panduan kampus, saya ingin seluruh subbab wajib hadir dengan heading yang konsisten, sehingga struktur Bab 4 mengikuti panduan akademik dan dapat diverifikasi secara deterministik.

#### Acceptance Criteria

1. THE Bab_4 SHALL memuat heading `## 4.1 Spesifikasi Lingkungan Implementasi` tepat satu kali pada level heading 2 dengan format ATX (`## ` diikuti tepat satu spasi tunggal sebelum nomor subbab dan tanpa karakter tambahan setelah teks heading), dengan kemunculan di dalam fenced code block dikecualikan dari penghitungan sebagaimana didefinisikan pada AC 9.
2. THE Bab_4 SHALL memuat heading `## 4.2 Implementasi Modul` tepat satu kali pada level heading 2 dengan format ATX, beserta subheading `### 4.2.1 Parser SVG`, `### 4.2.2 Scene API`, `### 4.2.3 Path Processing dan Flattening`, `### 4.2.4 Tile Binning DDA`, `### 4.2.5 Pembangkit Tile dan Akumulator Backdrop`, `### 4.2.6 Renderer WebGL`, `### 4.2.7 Shader Vertex dan Fragment`, dan `### 4.2.8 Demo Interaktif`, masing-masing tepat satu kali pada level heading 3 dengan format ATX (`### ` diikuti tepat satu spasi tunggal); kemunculan di dalam fenced code block dikecualikan sebagaimana AC 9.
3. THE Bab_4 SHALL memuat heading `## 4.3 Verifikasi Kebenaran Output` tepat satu kali pada level heading 2 dengan format ATX, dengan kemunculan di dalam fenced code block dikecualikan sebagaimana AC 9.
4. THE Bab_4 SHALL memuat heading `## 4.4 Pengujian Performa` tepat satu kali pada level heading 2 dengan format ATX, beserta subheading `### 4.4.1 Metodologi Pengukuran`, `### 4.4.2 Hasil Pengukuran Per Aset`, dan `### 4.4.3 Analisis Perbandingan dengan Baseline`, masing-masing tepat satu kali pada level heading 3 dengan format ATX; kemunculan di dalam fenced code block dikecualikan sebagaimana AC 9.
5. THE Bab_4 SHALL memuat heading `## 4.5 Pembahasan Trade-off Arsitektur Non-Compute` tepat satu kali pada level heading 2 dengan format ATX, dengan kemunculan di dalam fenced code block dikecualikan sebagaimana AC 9.
6. THE Bab_4 SHALL memuat heading `## 4.6 Keterbatasan Implementasi Saat Ini` tepat satu kali pada level heading 2 dengan format ATX, dengan kemunculan di dalam fenced code block dikecualikan sebagaimana AC 9.
7. THE Bab_4 SHALL menyusun heading subbab level 2 pada urutan menaik monotonik 4.1 → 4.2 → 4.3 → 4.4 → 4.5 → 4.6 tanpa nomor yang dilewati, diulang, atau dibalik, dan SHALL TIDAK memuat heading level 2 lain di luar enam Subbab_Wajib tersebut.
8. THE Bab_4 SHALL menempatkan setiap subheading `### 4.x.y` setelah heading induknya `## 4.x` dan sebelum heading subbab level 2 berikutnya, dengan urutan `y` menaik monotonik secara kontigu mulai dari 1 tanpa nomor yang dilewati, diulang, atau dibalik.
9. THE Bab_4 SHALL mengecualikan kemunculan teks heading di dalam fenced code block (blok yang dibatasi oleh ` ``` ` atau `~~~`) dari penghitungan kehadiran heading pada AC 1 sampai AC 8; hanya heading Markdown ATX di luar fenced code block yang dihitung sebagai heading struktural.

### Requirement 4: Spesifikasi Lingkungan Implementasi (Subbab 4.1)

**User Story:** Sebagai penguji yang ingin mereplikasi hasil, saya ingin mengetahui lingkungan pengembangan, parameter build, dan toolchain yang dipakai, sehingga saya dapat membangun ulang purwarupa secara identik.

#### Acceptance Criteria

1. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 bahasa pemrograman Rust edisi 2024 dengan Rujukan_Kode ke `Cargo.toml` (entri `edition = "2024"` pada blok `[package]`).
2. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 target kompilasi `wasm32-unknown-unknown` dengan SIMD128 enabled, dengan Rujukan_Kode ke `.cargo/config.toml` pada entri `target-feature=+simd128`.
3. THE Bab_4 SHALL mendaftarkan pada Subbab 4.1 lima parameter profil release: `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = false`, dan `strip = true`, dengan Rujukan_Kode ke blok `[profile.release]` di `Cargo.toml`.
4. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 minimal satu peramban target konkret beserta nomor versi minimumnya (misalnya `Google Chrome 113` atau `Mozilla Firefox 121`) yang sekaligus mendukung WebGL 2.0 dan WASM SIMD128.
5. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 toolchain build: `wasm-pack` untuk pengujian otomatis pada `tests/test.rs` dan `cargo-run-wasm` untuk demo interaktif, beserta rantai feature flag `webgl` yang mengaktifkan kompilasi WebGL — dengan Rujukan_Kode ke `examples/run_wasm/Cargo.toml` (`cargo-run-wasm = "0.4.0"`), ke alias `run_wasm = "run --release --package run_wasm --"` di `.cargo/config.toml`, ke definisi feature `webgl = ["dep:js-sys", "dep:web-sys", "dep:wasm-bindgen"]` pada blok `[features]` di `Cargo.toml`, dan ke entri `arabella = { path = "../..", features = ["webgl"] }` pada `examples/native_webgl/Cargo.toml`.
6. THE Bab_4 SHALL menyebutkan pada Subbab 4.1 layout workspace yang mencakup crate akar `arabella` dan dua crate contoh `examples/run_wasm` serta `examples/native_webgl`, dengan Rujukan_Kode ke blok `[workspace]` pada `Cargo.toml`.

### Requirement 5: Implementasi Modul (Subbab 4.2) Mencerminkan Source Code

**User Story:** Sebagai dosen penguji, saya ingin setiap subbab implementasi modul mendeskripsikan modul yang benar-benar ada di source code, sehingga setiap narasi dapat diverifikasi langsung ke berkas yang dirujuk.

#### Acceptance Criteria

1. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.1 modul parser SVG dengan menyebutkan minimal satu kali secara literal token-token berikut: struct `PicoSvg`, method `PicoSvg::load`, enum `Item` (varian `Fill`, `Stroke`, `Group`), dispatch tag-name pada `Parser::rec_parse` yang hanya menangani elemen `g` dan `path`, atribut presentation yang diparse (`fill`, `stroke`, `stroke-width`, dan `transform`), serta delegasi parsing XML ke crate `roxmltree`; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke `src/pico_svg.rs` pada paragraf yang sama dengan kemunculan setiap token tersebut.
2. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.2 Scene API dengan menyebutkan minimal satu kali secara literal token-token berikut: struct `Scene`, method `Scene::new`, `Scene::fill`, `Scene::stroke`, `Scene::reset`, encoding paint solid melalui konstanta `PAINT_TYPE_SOLID = 0`, dan delegasi stroke expansion ke `kurbo::stroke_with`; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke `src/scene.rs` pada paragraf yang sama dengan kemunculan setiap token tersebut.
3. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.3 path processing dengan menyebutkan minimal satu kali secara literal token-token berikut: transformasi affine SIMD-batched (`transform_pair`, `transform_quad` menggunakan `f32x4` dan `f32x8`), konversi cubic-to-quadratic (fungsi `convert_cubics_to_quadratic_curves` dan `estimate_number_of_quadratic_curves` dengan parameter `MAX_QUADS = 16` dan `TOL`), midpoint subdivision pada `flatten_quadratic` dan `flatten_recursive` dengan uji `is_flat_enough` terhadap `FLATNESS_THRESHOLD = 32`, serta format F24Dot8 melalui `f32_to_f24dot8`; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke `src/path.rs` atau `src/flatten.rs` pada paragraf yang sama dengan kemunculan setiap token tersebut.
4. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.4 tile binning DDA dengan menyebutkan minimal satu kali secara literal token-token berikut: konstanta `TILE_W = 16` dan `TILE_H = 8` beserta turunan F24Dot8-nya, struct `Block`, `Blocks`, dan `TileBounds`, fungsi `Blocks::build_block` dan `Blocks::bin_line` (outer DDA empat arah diagonal plus tiga kasus khusus single-row, vertikal degenerate, dan horizontal degenerate), fungsi `Blocks::bin_line_in_row` (inner DDA empat arah utama), serta fungsi `record_per_scanline_crossings` (akumulator signed-area pada format 8.8 fixed-point); THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke `src/blocks.rs` pada paragraf yang sama dengan kemunculan setiap token tersebut.
5. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.5 pembangkit tile dengan menyebutkan minimal satu kali secara literal token-token berikut: struct `Builder`, struct `CoverStorage` dengan field `tag` dan `backdrops: Vec<[i16; TILE_H]>`, method `Builder::build_path` dan `Builder::generate_tiles`, propagasi backdrop kiri-ke-kanan per baris ubin pada saat emisi tile, dan optimasi SIMD `i16x8` untuk akumulasi per baris; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke `src/builder.rs` pada paragraf yang sama dengan kemunculan setiap token tersebut.
6. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.6 renderer WebGL dengan menyebutkan minimal satu kali secara literal token-token berikut: struct `WebGlRenderer` dengan field `programs` dan `gl: WebGl2RenderingContext`, method `WebGlRenderer::new` dan `WebGlRenderer::render`, fungsi `initialize_tile_vao` dengan stride 44 byte yang diturunkan dari `core::mem::size_of::<Tile>()` plus `vertexAttribDivisor(_, 1)` untuk seluruh slot atribut, upload tekstur RGBA32F untuk segmen, dan pemanggilan `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len())`; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke `src/render/webgl.rs` pada paragraf yang sama dengan kemunculan setiap token tersebut.
7. THE Bab_4 SHALL menyebutkan pada Subbab 4.2.6 minimal satu kali secara literal struct `Tile` dengan atribut `#[repr(C)]` dan ukuran rekord 44 byte, dengan minimal satu Rujukan_Kode ke `src/tile.rs` pada paragraf yang sama dengan kemunculan token tersebut.
8. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.7 shader vertex dan fragment dengan menyebutkan minimal satu kali secara literal token-token berikut: vertex shader instanced quad pada `src/render/shaders/render_tile.vert` yang memetakan empat corner quad ke koordinat klip NDC, fragment shader analitik pada `src/render/shaders/render_tile.frag` yang mengakumulasi `line_box` (integral trapezoidal cakupan piksel) untuk setiap segmen yang dibinning ke ubin, konstanta `WINDING_UNIT = 256.0`, dan penerapan fill rule NonZero melalui `coverage = clamp(abs(winding), 0.0, 1.0)` serta EvenOdd melalui `coverage = 1.0 - abs(mod(abs(winding), 2.0) - 1.0)`; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke berkas shader yang bersangkutan pada paragraf yang sama dengan kemunculan setiap token tersebut.
9. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.2.8 demo interaktif dengan menyebutkan minimal satu kali secara literal token-token berikut: fungsi `run_interactive(width, height)` di `examples/native_webgl/src/lib.rs` yang dipanggil dari `examples/native_webgl/src/main.rs` setelah pembacaan `device_pixel_ratio()`, `inner_width()`, dan `inner_height()`; struct `AppState` beserta method `AppState::render` dan `AppState::update_overlay`; empat metrik overlay (waktu CPU dalam milidetik, waktu GPU dalam milidetik, jumlah operasi paint, rasio zoom); ketiga Aset_Uji yang dimuat (`Ghostscript Tiger`, `SVG Logo`, `Bismillah`); serta event handler untuk pan, zoom, dan keyboard; THE Bab_4 SHALL menyertakan minimal satu Rujukan_Kode ke berkas demo yang bersangkutan pada paragraf yang sama dengan kemunculan setiap token tersebut.

### Requirement 6: Verifikasi Kebenaran Output (Subbab 4.3)

**User Story:** Sebagai pembaca, saya ingin melihat bukti bahwa output rendering Arabella secara visual benar dan dapat diverifikasi melalui pengujian otomatis, sehingga saya yakin implementasi berfungsi sebagaimana mestinya.

#### Acceptance Criteria

1. THE Bab_4 SHALL menyebutkan pada Subbab 4.3 ketiga Aset_Uji dengan jalur lengkapnya yang sama persis dengan jalur di repositori (tanpa singkatan): `assets/Ghostscript_Tiger.svg`, `assets/SVG_Logo.svg`, dan `assets/bismillah.svg`.
2. THE Bab_4 SHALL menyertakan pada Subbab 4.3 satu Placeholder_Gambar untuk masing-masing Aset_Uji dengan format persis `[Gambar 4.x: Hasil rendering {nama aset} oleh Arabella — dimasukkan kemudian]`, dengan `{nama aset}` berisi salah satu nama tampilan kanonik berikut secara tepat: `Ghostscript Tiger` untuk `assets/Ghostscript_Tiger.svg`, `SVG Logo` untuk `assets/SVG_Logo.svg`, dan `Bismillah` untuk `assets/bismillah.svg`; ketiga `4.x` SHALL menggunakan penomoran yang menaik dan unik antar ketiga Placeholder_Gambar tersebut.
3. THE Bab_4 SHALL membahas pada Subbab 4.3 empat aspek correctness: (a) ketepatan fill solid color, (b) keberhasilan stroke expansion via `kurbo::stroke_with`, (c) penerapan fill rule NonZero, dan (d) ketiadaan artefak visual seperti streak atau seam pada batas ubin; setiap aspek SHALL diuraikan dalam minimal satu kalimat naratif terpisah yang menyatakan kriteria keberhasilan yang dapat diobservasi (misalnya "warna fill solid hasil rendering Arabella mencocokkan warna fill solid yang dirender peramban").
4. THE Bab_4 SHALL menyebutkan pada Subbab 4.3 bahwa pengujian otomatis dilakukan melalui `wasm-bindgen-test` pada `tests/test.rs` dengan fungsi `test_renders_tiger_svg` yang hanya mencakup `assets/Ghostscript_Tiger.svg` pada resolusi 1080×520 piksel, dengan Rujukan_Kode ke `tests/test.rs:test_renders_tiger_svg`; THE Bab_4 SHALL menyatakan bahwa dua Aset_Uji lainnya (`assets/SVG_Logo.svg` dan `assets/bismillah.svg`) divalidasi secara manual saja.
5. THE Bab_4 SHALL menyebutkan pada Subbab 4.3 bahwa validasi visual dilakukan secara manual dengan membandingkan output Arabella berdampingan (side-by-side) terhadap rendering peramban untuk ketiga Aset_Uji, dan SHALL menghubungkan perbandingan tersebut secara eksplisit ke keempat aspek correctness yang didefinisikan pada AC 3 (a–d).

### Requirement 7: Pengujian Performa (Subbab 4.4)

**User Story:** Sebagai dosen penguji, saya ingin melihat metodologi pengukuran performa yang jelas dan format hasil yang dapat direproduksi, sehingga klaim performa dapat dipertanggungjawabkan tanpa risiko angka karangan.

#### Acceptance Criteria

1. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.4.1 metodologi pengukuran berbasis `performance.now()` dengan satuan milidetik yang memisahkan waktu pra-pemrosesan CPU (blok `Scene::fill` dan `Scene::stroke` yang mendelegasikan ke `Builder::build_path`) dari waktu rasterisasi GPU (blok `WebGlRenderer::render` yang mengeksekusi upload tekstur dan `draw_arrays_instanced`), dengan sampling 60-frame rolling average untuk FPS via `fps_window`, dengan Rujukan_Kode ke `examples/native_webgl/src/lib.rs` (method `AppState::render`).
2. THE Bab_4 SHALL menyebutkan pada Subbab 4.4.1 dua resolusi pengujian: demo interaktif berbasis window-fill DPR-aware dengan rumus `width = inner_width × devicePixelRatio` dan `height = inner_height × devicePixelRatio` (Rujukan_Kode `examples/native_webgl/src/main.rs`), dan tes otomatis pada resolusi tetap 1080×520 piksel pada DPR 1.0 (Rujukan_Kode konstanta `W: u16 = 1080` dan `H: u16 = 520` di `tests/test.rs`).
3. THE Bab_4 SHALL menyertakan pada Subbab 4.4.2 satu Placeholder_Tabel berformat persis `[Tabel 4.x — Hasil pengukuran performa per aset — diisi setelah pengujian dilakukan]` yang didahului satu paragraf naratif pengantar; tabel placeholder tersebut SHALL mencakup kolom eksplisit Aset, Paint Ops, CPU ms, GPU ms, Total Frame Time ms, dan FPS, dan SHALL berisi tepat tiga baris data, satu baris untuk masing-masing Aset_Uji (`Ghostscript_Tiger.svg`, `SVG_Logo.svg`, dan `bismillah.svg`), dengan seluruh sel metrik dibiarkan kosong atau ditandai sebagai belum diisi (misalnya `—` atau `TBD`).
4. THE Bab_4 SHALL TIDAK memuat angka FPS, CPU ms, GPU ms, atau metrik performa numerik karangan pada Subbab 4.4.2; seluruh nilai numerik performa yang belum diukur SHALL diwakili oleh Placeholder_Tabel.
5. THE Bab_4 SHALL mendeskripsikan pada Subbab 4.4.3 perbandingan kualitatif terhadap Skia (CPU-centric SIMD), Cairo (CPU-centric scanline), dan Vello (GPU compute-centric) dengan rujukan ke Subbab 2.2.3, 2.2.4, dan 2.2.8 pada Bab 2, mencakup tiga dimensi perbandingan eksplisit: (a) paradigma rasterisasi, (b) ketergantungan compute shader, dan (c) target platform.
6. THE Bab_4 SHALL memuat pada Subbab 4.4.3 disclaimer eksplisit yang menyatakan bahwa benchmark kuantitatif langsung terhadap renderer lain belum dilakukan pada implementasi saat ini, bahwa data Tabel 4.x perlu dilengkapi sebelum perbandingan kuantitatif dapat ditarik, dan bahwa hasil pengukuran apa pun yang nantinya diisi akan bergantung pada peramban dan perangkat keras spesifik yang dideklarasikan pada Subbab 4.1.

### Requirement 8: Pembahasan Trade-off Arsitektur Non-Compute (Subbab 4.5)

**User Story:** Sebagai pembaca, saya ingin memahami trade-off arsitektur non-compute yang dipilih Arabella, sehingga saya dapat menilai kontribusi penelitian secara proporsional.

#### Acceptance Criteria

1. THE Bab_4 SHALL membahas pada Subbab 4.5 tiga dimensi trade-off masing-masing dalam paragraf terpisah berisi minimal tiga kalimat lengkap, ditulis berurutan dengan label literal yang sama persis: (a) Kompatibilitas Platform, dengan menyatakan bahwa WebGL 2.0 tersedia di lebih banyak perangkat dibanding WebGPU atau compute shader, dan menautkan klaim kompatibilitas ini ke peramban target yang didefinisikan pada Subbab 4.1; (b) Kompleksitas Implementasi, dengan menjelaskan bahwa beban pra-pemrosesan dipindahkan ke CPU sebagai pengganti compute shader dispatch; dan (c) Karakteristik Performa, dengan menjelaskan trade-off antara latensi transfer CPU→GPU dan paralelisme GPU penuh.
2. THE Bab_4 SHALL merujuk pada Subbab 4.5 arsitektur Vello (compute-centric), Skia (CPU-centric), dan Cairo (CPU-centric scanline) sebagai titik perbandingan, dengan menuliskan secara literal nomor sub-section Bab 2 berikut sebagai rujukan: `Subbab 2.2.3`, `Subbab 2.2.4`, dan `Subbab 2.2.8`.
3. THE Bab_4 SHALL menempatkan satu paragraf rekapitulasi sebagai paragraf terakhir pada Subbab 4.5 yang menyatakan bahwa pendekatan Arabella mengeliminasi ketergantungan pada compute shader dengan konsekuensi bahwa seluruh komputasi tujuan umum (flattening, binning DDA, akumulasi winding number) dieksekusi di CPU.

### Requirement 9: Keterbatasan Implementasi Saat Ini (Subbab 4.6)

**User Story:** Sebagai dosen penguji, saya ingin mengetahui secara jujur fitur apa saja yang belum berfungsi pada implementasi saat ini, sehingga evaluasi terhadap skripsi tetap valid dan proporsional terhadap kontribusi yang sebenarnya tercapai.

#### Acceptance Criteria

1. THE Bab_4 SHALL mendaftarkan pada Subbab 4.6 minimal enam keterbatasan dalam bentuk daftar tak berurutan Markdown dengan prefiks `- ` pada setiap butir: (a) gradien linear/radial/sweep belum aktif di fragment shader, dengan Rujukan_Kode ke `src/scene.rs` (fungsi `encode_paint` yang mengembalikan flag `PAINT_TYPE_SOLID`); (b) image paint dan tinting belum diunggah dari Scene, dengan Rujukan_Kode ke `src/render/common.rs` (struktur `GpuEncodedImage`); (c) Rayon belum diaktifkan pada hot path pra-pemrosesan, dengan Rujukan_Kode ke `Cargo.toml` (feature `multithreading` opsional di balik `default = ["std", "png"]`); (d) subset SVG yang didukung terbatas pada elemen `g` dan `path` plus atribut `fill`, `stroke`, `stroke-width`, dan `transform`, dengan Rujukan_Kode ke `src/pico_svg.rs:Parser::rec_parse`; (e) tiada sistem text rendering; dan (f) tiada filter effect (blur, drop shadow).
2. THE Bab_4 SHALL menyertakan Rujukan_Kode pada butir (a), (b), (c), dan (d) AC 1; butir (e) dan (f) AC 1 SHALL TIDAK memerlukan Rujukan_Kode karena keduanya mendeskripsikan ketiadaan fitur yang tidak memiliki referen kode.
3. THE Bab_4 SHALL menempatkan paragraf penutup sebagai paragraf terakhir Subbab 4.6 (paragraf yang langsung mendahului heading Subbab berikutnya atau akhir berkas) yang memuat ketiga elemen tekstual berikut: (1) salah satu kata kunci `future work` atau `pengembangan lanjutan` (case-insensitive); (2) pernyataan eksplisit bahwa keterbatasan-keterbatasan tersebut tidak menggugurkan validitas kontribusi inti penelitian; dan (3) frasa literal `pipeline hibrida non-compute`.
4. THE Bab_4 SHALL menampilkan paragraf penutup yang dideskripsikan pada AC 3 secara tanpa syarat, terlepas dari jumlah keterbatasan yang berhasil didaftarkan pada AC 1, sehingga kemunculan paragraf penutup tersebut dapat diuji secara independen dari isi daftar keterbatasan.

### Requirement 10: Traceability Setiap Klaim Teknis

**User Story:** Sebagai dosen penguji, saya ingin setiap klaim teknis di Bab 4 dapat saya verifikasi langsung ke berkas kode, sehingga saya dapat memastikan tidak ada klaim fabrikasi yang lolos ke dokumen final.

#### Acceptance Criteria

1. WHEN Bab_4 memuat Klaim_Teknis pada Subbab 4.1, 4.2.1 sampai 4.2.8, 4.3, 4.4.1, atau 4.6, THE Penulis SHALL menyertakan minimal satu Rujukan_Kode dalam paragraf yang sama dengan Klaim_Teknis tersebut, di mana "paragraf yang sama" didefinisikan sebagai blok teks Markdown yang terdiri dari satu atau lebih baris non-kosong berurutan dan dipisahkan dari blok teks lain oleh setidaknya satu baris kosong.
2. THE Penulis SHALL menulis setiap Rujukan_Kode dalam format `\`berkas:simbol\`` (contohnya `` `src/blocks.rs:bin_line` ``) atau `\`berkas:start-end\`` (contohnya `` `src/blocks.rs:107-160` ``), dengan jalur berkas relatif terhadap akar repositori tanpa prefiks `./` di depan jalur, dan untuk format rentang baris, indeks baris menggunakan basis 1, nilai `start` lebih kecil atau sama dengan `end`, dan rentang `(end - start + 1)` tidak melebihi 300 baris.
3. THE Penulis SHALL memastikan setiap Rujukan_Kode menunjuk berkas yang benar-benar ada di salah satu lokasi Source_Of_Truth (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, atau `.cargo/config.toml`); WHERE Rujukan_Kode menggunakan format `\`berkas:simbol\``, THE simbol SHALL terdefinisi pada berkas tersebut sebagai salah satu dari fungsi, struct, konstanta, method, enum, field, atau makro; WHERE Rujukan_Kode menggunakan format `\`berkas:start-end\``, THE rentang baris SHALL berada dalam batas berkas (yaitu `start ≥ 1` dan `end` tidak melebihi jumlah baris berkas yang dirujuk).
4. THE Penulis SHALL TIDAK memasukkan ke Bab_4 Klaim_Teknis yang tidak dapat ditelusuri ke Source_Of_Truth, di mana "dapat ditelusuri" berarti memenuhi format Rujukan_Kode pada AC 2 dan validitas berkas/simbol/rentang pada AC 3.
5. THE Penulis SHALL TIDAK mengarang nama fungsi, nama method, nama struct, nama trait, nama modul, nama enum, nama field, nama konstanta, nama feature flag, nama crate, nama berkas, maupun nilai parameter numerik yang tidak muncul secara literal pada Source_Of_Truth.
6. WHERE Klaim_Teknis pada Subbab 4.4.2, 4.4.3, atau 4.5 bersifat kualitatif (membahas trade-off arsitektur tanpa rujukan langsung ke simbol kode), THE Penulis SHALL menggantungkan klaim tersebut pada rujukan ke subbab Bab 1 atau Bab 2 yang relevan dengan menuliskan nomor subbab secara literal (misalnya `Subbab 1.2`, `Subbab 2.2.3`), dan SHALL TIDAK menyembunyikan nilai numerik konkret di dalam klaim kualitatif tanpa Rujukan_Kode.

### Requirement 11: Anti-Fabrikasi Numerik

**User Story:** Sebagai pembaca, saya ingin yakin bahwa angka performa dan metrik empiris yang disajikan adalah hasil pengukuran nyata atau ditandai secara eksplisit sebagai belum diukur, sehingga tidak ada data fiktif di dokumen final.

#### Acceptance Criteria

1. THE Bab_4 SHALL TIDAK memuat nilai FPS, CPU ms, GPU ms, total frame time, throughput, jumlah operasi paint, frame count rolling average, atau metrik performa numerik lain kecuali nilai tersebut muncul di dalam baris Placeholder_Tabel sebagaimana didefinisikan pada Requirement 7 AC 3, atau merupakan nilai konstanta kode yang dirujuk via Rujukan_Kode pada paragraf yang sama dengan kemunculan nilai tersebut.
2. IF data performa belum tersedia pada Subbab 4.3, 4.4.2, atau 4.4.3, THEN THE Bab_4 SHALL mengisinya dengan Placeholder_Tabel atau Placeholder_Gambar berformat persis seperti didefinisikan pada Glossary, dan SHALL TIDAK mengisi slot placeholder tersebut dengan angka substitusi, angka indikatif, atau angka yang dikutip dari literatur eksternal.
3. IF Bab_4 menyebut nilai numerik yang berasal dari konstanta kode (misalnya `TILE_W = 16`, `TILE_H = 8`, ukuran rekord `Tile` 44 byte, resolusi pengujian 1080×520, format tekstur RGBA32F, format `F24Dot8`, format `8.8 fixed-point`, `WINDING_UNIT = 256`), THEN THE Bab_4 SHALL menyertakan Rujukan_Kode yang mendefinisikan konstanta tersebut pada kalimat atau paragraf yang sama dengan kemunculan nilai numerik tersebut.
4. IF Bab_4 memuat grafik atau tabel berisi data numerik performa hasil estimasi, proyeksi, atau asumsi, THEN THE caption grafik atau tabel tersebut SHALL memuat secara eksplisit salah satu kata `estimasi`, `proyeksi`, atau `asumsi` (case-insensitive), dan THE Bab_4 SHALL menyertakan paragraf penjelas yang menyatakan basis estimatif data tersebut.
5. THE Bab_4 SHALL memastikan setiap parameter numerik kanonik melaporkan nilai yang sama di seluruh dokumen: dimensi ubin 16×8 piksel, ukuran rekord `Tile` 44 byte, resolusi pengujian 1080×520 piksel, format tekstur segmen RGBA32F, format fixed-point segmen F24Dot8, dan format fixed-point akumulator winding 8.8 fixed-point.
6. THE Bab_4 SHALL TIDAK memuat nilai kontradiktif untuk parameter numerik kanonik pada AC 5, dengan enumerasi eksplisit: dimensi ubin selain 16×8 (misalnya 16×16), ukuran rekord `Tile` selain 44 byte, resolusi pengujian selain 1080×520 (misalnya 1920×1080), format tekstur segmen selain RGBA32F, format fixed-point segmen selain F24Dot8, dan format fixed-point akumulator winding selain 8.8 fixed-point.

### Requirement 12: Konsistensi Terminologi Kanonik dan Eliminasi Istilah Terlarang

**User Story:** Sebagai pembaca skripsi yang membaca Bab 1 sampai Bab 4 secara berurutan, saya ingin istilah lintas-bab tetap konsisten, sehingga Bab 4 tidak merusak referensi silang yang sudah dibangun pada Bab 3 final.

#### Acceptance Criteria

1. WHEN Bab_4 merujuk konsep arsitektural yang sudah didefinisikan pada Bab_3_Final, THE Bab_4 SHALL menggunakan istilah kanonik berikut secara literal: "pipeline hibrida" untuk arsitektur Arabella secara keseluruhan, "binning DDA" untuk tahap pemecahan segmen lintas ubin, "akumulator signed-area" untuk akumulator winding 8.8 fixed-point per scanline, "propagasi backdrop" untuk akumulasi kiri-ke-kanan saat emisi tile, "fragment shader" untuk shader piksel WebGL, dan "pra-pemrosesan" atau "preprocessing" untuk fase CPU keseluruhan.
2. THE Bab_4 SHALL memuat minimal satu kemunculan dari salah satu varian "pra-pemrosesan" atau "preprocessing" (case-insensitive) di seluruh dokumen agar konsistensi terminologi fase CPU lintas-bab tetap terjaga.
3. THE Bab_4 SHALL TIDAK mencampur varian "pra-pemrosesan" dan "preprocessing" dalam satu paragraf yang sama, di mana "paragraf yang sama" didefinisikan sebagai blok teks Markdown CommonMark berupa satu atau lebih baris non-kosong berurutan yang dipisahkan dari blok teks lain oleh setidaknya satu baris kosong.
4. THE Bab_4 SHALL TIDAK memuat istilah berikut yang termasuk Istilah_Terlarang yang diteruskan dari Spec_Bab3:
   - `Ray Shooting`, `Ray Shoot`, `ray shooting`, `ray shoot` sebagai frasa utuh nama algoritma (case-insensitive);
   - `TileType` sebagai token kata utuh nama enum atau jenis ubin (case-sensitive);
   - `EMPTY`, `INTERIOR`, `EDGE` sebagai token kata utuh label tipe ubin atau cabang fragment shader (case-sensitive);
   - `winding_number` sebagai token kata utuh nama field skalar pada struct Tile (case-sensitive, dengan underscore);
   - `fungsi implisit linear`, `fungsi implisit kuadratik kanonik`, `fungsi implisit kubik`, `PPGA`, `Projective Geometric Algebra` sebagai frasa utuh (case-insensitive untuk frasa berbahasa Indonesia, case-sensitive untuk akronim `PPGA`);
   - persamaan `ax+by+c=0`, `u-v²=0`, `u-v^2=0`, `C(x,y)=0`, `w_0³-w_1 w_2 w_3` sebagai formula evaluasi GPU (dengan atau tanpa spasi, case-sensitive pada simbol matematis);
   - frasa `OpenGL ES 3.0 yang ditranspilasikan`, `ditranspilasikan ke WebGL`, `transpilasi OpenGL ES` (case-insensitive);
   - frasa `Rust edisi 2021`, `edisi 2021`, dan token `edition = "2021"` (case-sensitive untuk literal `edition = "2021"`).
5. THE Bab_4 SHALL memuat minimal satu kemunculan istilah "winding number" (tanpa underscore, dengan satu spasi tunggal antar kata, case-insensitive) sebagai konsep yang dirujuk pada narasi, dan SHALL TIDAK menjadikannya nama field skalar pada struct Tile.

### Requirement 13: Konektivitas Naratif dengan Bab 3

**User Story:** Sebagai pembaca, saya ingin Bab 4 terhubung secara naratif dengan Bab 3 final, sehingga transisi antar bab terasa koheren dan tidak mengulang materi yang sama tanpa keperluan.

#### Acceptance Criteria

1. THE Bab_4 SHALL memuat pada paragraf pertama setelah heading Subbab 4.1 atau setelah heading Subbab 4.2 minimal satu kalimat penghubung yang mengandung token literal `Bab 3` dan merujuk perancangan pada Bab 3 (contohnya bertema "Berdasarkan perancangan arsitektur pipeline hibrida yang telah diuraikan pada Bab 3, bab ini menyajikan implementasi purwarupa beserta hasil pengujiannya.").
2. THE Bab_4 SHALL menyebutkan urutan delapan tahap pipeline yang sama dengan UC-03 pada Bab 3 — flattening → outer DDA → inner DDA → akumulator signed-area → emisi tile → propagasi backdrop → vertex shader → fragment shader — dengan setiap tahap muncul minimal satu kali menggunakan terminologi kanonik yang sama, baik di dalam paragraf pengantar Subbab 4.2 maupun secara kumulatif tersebar di Subbab 4.2.1 sampai 4.2.7.
3. THE Bab_4 SHALL memuat pada paragraf pengantar Subbab 4.2 minimal satu kemunculan token literal `Subbab 3.4.4` atau frasa `class diagram pada Bab 3` yang merujuk class diagram Bab 3, sehingga rujukan visual ke struct utama (`Scene`, `Builder`, `CoverStorage`, `Block`, `Blocks`, `TileBounds`, `Tile`, `WebGlRenderer`, `PicoSvg`) dapat ditelusuri lintas bab.
4. THE Bab_4 SHALL TIDAK memuat blok verbatim sepanjang lebih dari 30 kata berurutan yang identik dengan blok teks pada Bab_3_Final, sehingga duplikasi naratif dengan Bab 3 dapat diuji secara mekanis.
5. THE Bab_4 SHALL memuat pada paragraf pengantar Subbab 4.2 minimal satu kalimat eksplisit yang menyatakan pergeseran fokus dari "perancangan" pada Bab 3 ke "wujud implementasi konkrit dan hasil pengujian" pada Bab 4.

### Requirement 14: Gaya Bahasa dan Format Akademik

**User Story:** Sebagai mahasiswa yang menulis skripsi sesuai panduan kampus, saya ingin Bab 4 menggunakan gaya bahasa akademik formal Indonesia yang konsisten dengan bab-bab sebelumnya.

#### Acceptance Criteria

1. THE Bab_4 SHALL ditulis dalam bahasa Indonesia formal akademik dengan kalimat lengkap (subjek-predikat-objek atau subjek-predikat-keterangan) dan SHALL TIDAK memuat token bahasa percakapan berikut sebagai kata utuh di luar fenced code block: `bisa`, `gak`, `enggak`, `nih`, `dong`, `kok`, `kan`, `aja`, `udah`, dan `mau`; sebagai gantinya THE Bab_4 SHALL menggunakan bentuk formal seperti `dapat` alih-alih `bisa`.
2. THE Bab_4 SHALL menggunakan backtick (`` ` ``) untuk membungkus nama identifier (struct, fungsi, konstanta, feature flag, nama berkas, jalur berkas, dan ekspresi kode) dan SHALL menggunakan italic (`*frasa*` atau `_frasa_`) untuk membungkus istilah teknis berbahasa Inggris yang digunakan sebagai frasa terminologi disiplin (misalnya *midpoint subdivision*, *fragment shader* ketika digunakan sebagai istilah disiplin dan bukan sebagai rujukan struct atau identifier konkret).
3. THE Bab_4 SHALL menyertakan Placeholder_Gambar berformat persis seperti definisi pada Glossary untuk setiap ilustrasi yang belum tersedia (screenshot rendering, diagram arsitektur modul) pada Subbab 4.3.
4. THE Bab_4 SHALL menyertakan Placeholder_Tabel berformat persis seperti definisi pada Glossary untuk setiap tabel data empiris yang belum dikumpulkan pada Subbab 4.4.2.
5. THE Bab_4 SHALL mempertahankan kosakata dan format penomoran subbab yang sudah dipakai di Bab 1, Bab 2, dan Bab 3 final (`pustaka`, `perangkat lunak`, `pra-pemrosesan`, `ubin`, format `4.x` dan `4.x.y`).
6. THE Bab_4 SHALL menggunakan kalimat pasif atau bentuk impersonal akademik dan SHALL TIDAK menggunakan kata ganti orang pertama (`saya`, `kami`, `kita`) maupun kata ganti orang kedua (`Anda`, `kamu`) sebagai kata utuh di dalam narasi Bab 4 di luar fenced code block.
