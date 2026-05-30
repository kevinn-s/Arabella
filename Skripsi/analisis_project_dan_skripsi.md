# Laporan Analisis Project dan Skripsi

> ⚠️ **STATUS: DOKUMEN KERJA INTERNAL — USANG — BUKAN BAGIAN NASKAH SKRIPSI.**
> Dokumen ini adalah catatan kerja awal yang dipakai sebagai dasar revisi Bab 3 dan penulisan Bab 4/5. **Isinya sudah tidak mencerminkan kondisi terkini** — bagian E masih menyebut Bab 4, Bab 5, Abstrak, dan Daftar Pustaka sebagai "lorem ipsum / tidak relevan", padahal keempatnya kini sudah ditulis lengkap dan relevan. Banyak ketidaksesuaian pada bagian D juga sudah diperbaiki. **JANGAN dikumpulkan, dicetak, atau dilampirkan ke naskah skripsi final.** Jejak revisi yang aktual dan terkini ada pada `catatan_revisi_konsistensi.md`.

> Dokumen analisis menyeluruh terhadap source code project Arabella dan dokumen skripsi pada folder `Skripsi/`. Disusun sebagai dasar untuk revisi Bab 3 dan penulisan Bab 4.

---

## A. Ringkasan Sistem yang Dibangun

Project bernama **Arabella** (`Cargo.toml` → `name = "arabella"`) merupakan pustaka rendering grafis vektor dua dimensi berbasis arsitektur hibrida CPU–GPU yang tidak bergantung pada compute shader. Sistem dibangun menggunakan bahasa pemrograman **Rust edisi 2024** dan dieksekusi di lingkungan peramban melalui kompilasi ke target `wasm32-unknown-unknown` dengan API grafis **WebGL 2.0**. Pustaka mengekspos antarmuka pemrograman terprogram (programmatic API) yang memungkinkan pengembang memuat berkas SVG, membangun adegan (scene), kemudian merender frame per frame ke kanvas HTML.

Pipeline rendering Arabella terbagi atas dua fase besar yang saling berurutan, yaitu fase pra-pemrosesan di CPU dan fase rasterisasi di GPU. Pada fase CPU, jalur kurva diratakan (flattened) menjadi segmen garis melalui pendekatan rekursif midpoint subdivision gaya Blaze, kemudian seluruh segmen dipartisi secara spasial ke dalam grid ubin berukuran tetap menggunakan algoritma Digital Differential Analyzer (DDA). Setiap pasangan (segmen, ubin) menghasilkan satu rekam Block yang berisi titik ujung garis dalam koordinat ubin lokal pada format F24Dot8. Bersamaan dengan binning, sebuah akumulator winding analitik per scanline dihitung dalam representasi 8.8 fixed-point dan dipropagasikan baris demi baris untuk menghasilkan nilai backdrop awal setiap ubin. Pada fase GPU, setiap ubin yang nontrivial dirender sebagai instance quad melalui vertex shader, sementara fragment shader mengevaluasi kontribusi area analitik untuk setiap segmen garis di dalam ubin (fungsi `line_box`) dan menjumlahkannya dengan backdrop scanline untuk mendapatkan nilai winding final per piksel. Aturan pengisian (fill rule) NonZero dan EvenOdd diterapkan melalui klamping nilai winding tersebut menjadi koefisien cakupan (coverage).

## B. Identifikasi Fitur Utama Aplikasi

Berdasarkan inventaris kode pada `src/lib.rs`, `src/scene.rs`, dan modul-modul pendukung, fitur fungsional yang sudah terimplementasi meliputi:

1. **Pemuatan dan parsing SVG sederhana** melalui modul `pico_svg.rs` yang menangani elemen `<g>`, `<path>`, atribut `fill`, `stroke`, `stroke-width`, `transform`, serta turunan affine matrix/translate/scale. Parser ini diadaptasi dari proyek Vello, sebagaimana dinyatakan secara eksplisit di komentar berkas.
2. **Pengisian path (fill)** dengan dua aturan winding (NonZero dan EvenOdd) melalui `Scene::fill`.
3. **Penguatan stroke (stroke expansion)** dengan delegasi pada `kurbo::stroke_with`, sehingga stroke diekspansi menjadi outline tertutup yang kemudian direnderkan melalui pipeline fill yang sama. Properti stroke yang didukung adalah lebar, line cap, line join, dan miter limit.
4. **Transformasi affine batched menggunakan SIMD** (`f32x4`, `f32x8`) untuk mempercepat transformasi titik kontrol garis dan kurva kubik.
5. **Konversi kurva kubik ke kuadratik** melalui pendekatan iteratif gaya Vello pada `path.rs`, lalu flattening kuadratik menjadi segmen garis melalui De Casteljau midpoint subdivision di `flatten.rs`.
6. **Demo interaktif** pada `examples/native_webgl/` yang menyediakan pan, zoom, perpindahan adegan, serta overlay FPS berisi metrik CPU ms, GPU ms, jumlah operasi paint, dan rasio zoom. Demo memuat tiga aset uji: Ghostscript Tiger, SVG Logo, dan Bismillah.
7. **Pengujian otomatis berbasis WebGL** pada `tests/test.rs` yang merender Ghostscript Tiger pada kanvas 1080×520 piksel.

Komponen yang **belum berfungsi penuh** meskipun struktur datanya sudah ada:

- Encoding gradien linear, radial, dan sweep (`paint/encode.rs`) — kerangka data pada CPU sudah ada, tetapi shader fragment hanya menggunakan cabang `PAINT_TYPE_SOLID`.
- Encoding image dan tinting — definisi struktur `GpuEncodedImage` ada di `render/common.rs`, namun teksturnya tidak diunggah dari `Scene`.
- Multithreading binding melalui Rayon — terdaftar sebagai dependency `optional` di belakang feature flag `multithreading`, namun loop hot pada `Builder::build_path` dan `generate_tiles` saat ini bersifat single-threaded.

## C. Identifikasi Teknologi dan Arsitektur

Stack teknologi yang sebenarnya digunakan oleh purwarupa adalah sebagai berikut:

| Aspek | Teknologi yang dipakai pada source code |
|---|---|
| Bahasa | Rust edisi 2024 (Cargo.toml) |
| Target eksekusi | `wasm32-unknown-unknown` melalui wasm-bindgen |
| API grafis | WebGL 2.0 (web-sys `WebGl2RenderingContext`) |
| Shader | GLSL ES 3.00 (vertex + fragment, tradisional) |
| SIMD | fearless_simd 0.4 dengan target-feature `+simd128` |
| Geometri | lyon_path, lyon_geom, kurbo |
| Color | peniko |
| SVG parser | roxmltree + pico_svg (adaptasi dari Vello) |
| Multithreading | Rayon (opsional, native-only, belum diaktifkan di pipeline utama) |

Arsitektur dapat diringkas dalam empat lapis: (1) lapisan parser SVG yang mengekstrak `Item::Fill`, `Item::Stroke`, dan `Item::Group`; (2) lapisan scene CPU yang mengeksekusi flatten, DDA binning, akumulasi winding per scanline, dan emisi tile; (3) lapisan upload tekstur WebGL untuk segmen garis serta vertex buffer untuk tile; (4) lapisan rasterisasi GPU yang mengeksekusi vertex shader instanced quad dan fragment shader analitik berbasis integral garis.

## D. Analisis Kesesuaian Implementasi dan Skripsi

### D.1 Bagian yang Sinkron

Beberapa klaim metodologi pada Bab 3 sudah konsisten dengan implementasi:

1. Pemilihan Rust sebagai bahasa pemrograman utama (Bab 3.3.1).
2. Adopsi paradigma rasterization-only pipeline (vertex + fragment shader saja, tanpa compute/geometry/tessellation) di sisi GPU (Bab 1.3, Bab 3.3.1).
3. Penggunaan format masukan SVG (Bab 1.3, Bab 3.3.1).
4. Pembagian beban kerja antara CPU sebagai unit pra-pemrosesan dan GPU sebagai unit rasterisasi (Bab 1.4 dan Bab 3.4.2).
5. Strategi spasial berbasis tiling dengan ukuran tetap (Bab 3.5 poin 4) — meski parameter dimensi yang disebut tidak sesuai (lihat poin D.2).
6. Konversi cubic-to-quadratic untuk menyederhanakan evaluasi kurva (sejalan dengan filosofi Vello yang dirujuk di Bab 2.2.8).

### D.2 Bagian yang Tidak Sinkron

Terdapat sejumlah ketidaksesuaian substantif antara klaim teknis di Bab 3 dengan kode aktual yang harus diluruskan agar skripsi tetap dapat dipertanggungjawabkan secara ilmiah.

| No | Klaim pada Skripsi | Realita pada Source Code | Lokasi rujukan |
|---|---|---|---|
| 1 | "Edisi 2021" (Bab 3.3.1) | `edition = "2024"` | `Cargo.toml` |
| 2 | "Ubin homogen ukuran tetap 16×16 piksel" (Bab 3.5) | `TILE_W = 16`, `TILE_H = 8` (16×8 piksel) | `src/blocks.rs`, `src/builder.rs` |
| 3 | "Algoritma Ray Shooting menembakkan sinar vertikal dari koordinat tengah atas batas ubin" (Bab 3.5, Bab 3.4.2) | Tidak ada ray shooting. Yang dipakai adalah akumulasi signed-area per scanline (FreeType/Blaze style) yang dipropagasikan kiri-ke-kanan per baris ubin. | `src/builder.rs` (`generate_tiles`), `src/blocks.rs` (`record_per_scanline_crossings`) |
| 4 | "Klasifikasi tipe ubin EMPTY, INTERIOR, EDGE" (Bab 3.4.2, Bab 3.5, UC-03, Tile.type) | Tidak ada enum `TileType`. Ubin dengan akumulator nol dan tanpa segmen otomatis tidak diemit; selebihnya semua ubin melewati shader yang sama. Tidak ada cabang `Render Warna Solid` untuk INTERIOR atau cabang `Evaluasi Fungsi Implisit` untuk EDGE. | `src/builder.rs` (`generate_tiles` loop), `src/render/shaders/render_tile.frag` |
| 5 | "Fungsi implisit kuadratik kanonik $f(u,v) = u - v^2 = 0$" (Bab 3.5 poin 2) | Tidak digunakan sama sekali. Kurva kuadratik di-flatten menjadi segmen garis di CPU. | `src/flatten.rs`, `src/path.rs` |
| 6 | "Fungsi implisit kubik PPGA $f(p) = w_0(p)^3 - w_1 w_2 w_3$" (Bab 3.5 poin 3) | Tidak diimplementasikan. Kurva kubik dikonversi ke kuadratik lalu di-flatten ke garis. | `src/path.rs` (`convert_cubics_to_quadratic_curves`) |
| 7 | "Fungsi implisit linear $ax + by + c = 0$" (Bab 3.5 poin 1) | Tidak digunakan. Yang dipakai pada GPU adalah integral trapezoidal cakupan piksel per garis (`line_box`). | `src/render/shaders/render_tile.frag` |
| 8 | "OpenGL ES 3.0 yang ditranspilasikan ke WebGL 2.0" (Bab 3.3.1) | Implementasi langsung menargetkan WebGL 2.0 saja. Tidak ada jalur native OpenGL ES (tidak ada glow, glutin, sdl2, atau glfw). | `Cargo.toml` (cfg `target_arch = "wasm32"` only), `src/render/webgl.rs` |
| 9 | "Pustaka konkurensi CPU: Rayon" yang memparalelkan iterasi grid tile (Bab 3.3.1, Bab 3.5 algoritma) | Rayon dideklarasikan `optional` di balik feature `multithreading`. Loop binning di `Builder::build_path` adalah for-loop sekuensial. | `Cargo.toml` `[features]`, `src/builder.rs` |
| 10 | "Rust dengan pustaka Rayon, OpenMP, atau C++ Threads" (Bab 1.3) | Tidak ada OpenMP, tidak ada C++. Hanya Rust. | seluruh source tree |
| 11 | "Format SVG 1.1 Core" (Bab 3.3.1) | `pico_svg.rs` menangani subset minimal: `path` (`d`), `g`, `fill`, `stroke`, `transform`. Tidak mendukung `text`, `defs`, `linearGradient`, `radialGradient`, `pattern`, `filter`, `clipPath`, `use`, atau `style` blocks. | `src/pico_svg.rs` |
| 12 | "Pengisian solid + Stroke + UC-02 input data vektor" — abstrak Bab 3 menyiratkan kapabilitas penuh | Class `Scene` mendukung fill solid color dan stroke (dengan delegasi `kurbo::stroke_with`); gradient pada `Scene::encode_paint` masih TODO (mengembalikan magenta `0xFFFF00FF`). | `src/scene.rs` (`encode_paint`) |
| 13 | "Class diagram TileGrid, Tile.type, Tile.winding_number, Tile.curves" (Bab 3.4.4) | Struct aktual `Tile` di `src/tile.rs` dan `src/render/common.rs` memiliki field: `x, y, width, height, _pad, backdrop[8], segments[2], payload, paint_and_rect_flag, depth_index`. Tidak ada `winding_number` skalar, tidak ada `type`, tidak ada `curves: List<CurveRef>`. | `src/tile.rs`, `src/render/common.rs` |
| 14 | Resolusi uji default 1920×1080 (Bab 3.6) | Demo memakai resolusi window-fill DPR-aware; tes wasm-bindgen 1080×520. | `examples/native_webgl/src/lib.rs`, `tests/test.rs` |
| 15 | Klaim "Forma dari Google" sebagai engine compute-centric (Bab 1.1) | `forma` sebenarnya pernah diasosiasikan dengan riset Google (https://github.com/google/forma) — klaim ini perlu diverifikasi ulang dengan rujukan resmi. | Bab 1.1 |
| 16 | Bab 2.2.8 menyebut Vello memakai "tiling 16×16" | Vello hybrid pada CHANGELOG memakai tile 4×4 untuk Vello CPU sparse strips; Vello full memakai 16×16. Klaim 16×16 perlu dipertegas pada konteks varian mana. | Bab 2.2.8 |

## E. Identifikasi Bagian Skripsi yang Belum Lengkap

Pemeriksaan berkas skripsi menunjukkan kondisi penyelesaian sebagai berikut:

| Berkas | Status | Catatan |
|---|---|---|
| `abstrak.md` | **TIDAK RELEVAN** | Isinya membahas Web Forum Thread Summarization, Latent Semantic Analysis, klasterisasi forum — sama sekali tidak terkait rendering vektor. Harus ditulis ulang dari nol. |
| `kata_pengantar.md` | Placeholder generik | Perlu personalisasi nama rektor, dosen pembimbing, dll. |
| `bab1_pendahuluan.md` | **Selesai sebagian besar**. | Konten substantif sudah ada. Perlu klarifikasi minor pada penyebutan "Forma dari Google" dan referensi pustaka pendukung. |
| `bab2_landasan_teori.md` | **Selesai sebagian besar**. | Subbab 2.1 dan 2.2 sudah substantif. Belum ada subbab tentang algoritma DDA tile binning, Blaze flattening, atau winding number 8.8 fixed-point yang justru menjadi tulang punggung implementasi. Perlu disisipkan agar Bab 3 tidak terkesan tanpa landasan. |
| `bab3_metodologi.md` | **Selesai tetapi tidak sinkron dengan kode**. | Memerlukan revisi besar pada Bab 3.4.2 (UC-03), Bab 3.4.4 (Class Diagram), dan Bab 3.5 (Perancangan Algoritma) seperti diuraikan di bagian D.2. |
| `bab4_implementasi_dan_hasil.md` | **100% lorem ipsum**. | Belum ditulis. Prioritas tertinggi. |
| `bab5_kesimpulan.md` | **100% lorem ipsum**. | Belum ditulis. |
| `daftar_pustaka.md` | **TIDAK RELEVAN**. | Berisi referensi data structure dan e-learning (Aggarwal, Cheah, Guo, He dkk., Lai, Lim, dll). Tidak satupun terkait grafis komputer. Harus diganti seluruhnya. |

## F. Prioritas Pengerjaan Skripsi Selanjutnya

Berikut daftar tugas yang disusun menurut urgensi, dengan fokus pada Bab 3 dan Bab 4 sesuai instruksi.

### Prioritas 1 — Kritikal (memengaruhi keseluruhan kelayakan ilmiah)

1. **Revisi Bab 3 untuk mensinkronkan dengan implementasi.** Subbab yang harus direvisi: 3.3.1 (spesifikasi: edisi Rust 2024, WebGL 2.0 sebagai target utama, Rayon sebagai feature opsional), 3.4.2 (UC-03 alur tahap preprocessing dan rendering), 3.4.4 (class diagram Tile), 3.5 (perancangan algoritma — ganti narasi Ray Shooting menjadi DDA tile binning + per-scanline signed-area accumulator + propagasi backdrop kiri-ke-kanan; ganti narasi fungsi implisit GPU menjadi integral trapezoidal cakupan piksel per garis pada fragment shader).
2. **Penulisan Bab 4 dari nol.** Susun struktur yang merefleksikan implementasi sebenarnya, mencakup: subbab implementasi modul per modul (parser SVG, scene API, builder DDA, blocks tile binning, path flattening, render WebGL, shader vertex/fragment), subbab verifikasi correctness (perbandingan visual Ghostscript Tiger terhadap referensi), subbab pengujian performa (frame time CPU vs GPU pada tiga aset, hasil overlay FPS), subbab pembahasan trade-off arsitektur hibrida non-compute.
3. **Penulisan ulang Daftar Pustaka.** Susun daftar referensi sesuai sitasi yang sudah dipakai pada Bab 1 dan Bab 2 (Loop & Blinn 2005; Kokojima dkk. 2006; Gan dkk. 2014; Li dkk. 2016; Nehab & Hoppe 2008; Lengyel — Slug; Vello / Linebender; Pathfinder / Mozilla; Skia; Cairo; spesifikasi WebGL 2.0; spesifikasi SVG 1.1; dokumentasi Rust Rayon; Farin 2002 untuk Bezier).

### Prioritas 2 — Tinggi

4. **Penulisan ulang Abstrak dan Abstract** sesuai tema rendering vektor paralel hibrida.
5. **Penulisan Bab 5** simpulan dan saran berdasarkan hasil Bab 4 yang baru.
6. **Tambahan pada Bab 2** subbab tentang algoritma DDA, signed-area winding accumulator, dan integral trapezoidal cakupan garis (`line_box`) sebagai landasan teoretis Bab 3 dan Bab 4 yang baru.

### Prioritas 3 — Sedang

7. **Verifikasi klaim faktual minor di Bab 1.1** terkait atribusi Forma, Pathfinder, dan Vello.
8. **Klarifikasi pada Bab 2.2.8** mengenai ukuran ubin Vello untuk varian CPU (4×4) dan varian GPU (16×16).
9. **Personalisasi Kata Pengantar.**

## G. Rekomendasi Revisi Struktur

Rekomendasi penataan ulang Bab 3 dan Bab 4 agar isi skripsi merefleksikan kode dengan konsisten:

**Struktur Bab 3 yang diusulkan:**

- 3.1 Diagram alir kerangka berpikir (sudah baik)
- 3.2 Analisis kebutuhan
  - 3.2.1 Analisis pengguna
  - 3.2.2 Analisis aplikasi sejenis
  - 3.2.3 Rumusan dan solusi kebutuhan
- 3.3 Spesifikasi sistem
  - 3.3.1 Spesifikasi perangkat lunak (Rust 2024, WebGL 2.0, fearless_simd, lyon, kurbo, peniko, roxmltree)
  - 3.3.2 Spesifikasi target eksekusi (wasm32 + browser)
- 3.4 Perancangan arsitektur (use case, sequence, class) — diselaraskan dengan struct dan modul aktual
- 3.5 Perancangan algoritma
  - 3.5.1 Flattening kurva (cubic→quadratic→garis F24Dot8)
  - 3.5.2 Tile binning DDA (outer/inner DDA delapan arah, Blocks)
  - 3.5.3 Akumulator signed-area per scanline 8.8 fixed-point
  - 3.5.4 Propagasi backdrop kiri-ke-kanan per baris ubin
  - 3.5.5 Evaluasi cakupan piksel di GPU melalui integral trapezoidal `line_box`
  - 3.5.6 Penerapan fill rule (NonZero, EvenOdd) di fragment shader
- 3.6 Perancangan layar (sudah baik)
- 3.7 Perancangan tata letak memori (vertex buffer Tile, tekstur RGBA32F segments)

**Struktur Bab 4 yang diusulkan:**

- 4.1 Spesifikasi lingkungan implementasi
- 4.2 Implementasi modul
  - 4.2.1 Parser SVG (`pico_svg.rs`)
  - 4.2.2 Scene API (`scene.rs`) — fill, stroke, reset
  - 4.2.3 Path processing dan flattening (`path.rs`, `flatten.rs`)
  - 4.2.4 Tile binning DDA (`blocks.rs`)
  - 4.2.5 Pembangkit tile dan akumulator backdrop (`builder.rs`)
  - 4.2.6 Renderer WebGL (`render/webgl.rs`)
  - 4.2.7 Shader vertex dan fragment (`render/shaders/`)
  - 4.2.8 Demo interaktif (`examples/native_webgl/`)
- 4.3 Verifikasi kebenaran output (perbandingan visual terhadap renderer referensi pada tiga aset uji)
- 4.4 Pengujian performa
  - 4.4.1 Metodologi pengukuran (FPS overlay, perf.now() CPU/GPU)
  - 4.4.2 Hasil pengukuran per aset
  - 4.4.3 Analisis perbandingan dengan baseline
- 4.5 Pembahasan trade-off arsitektur non-compute
- 4.6 Keterbatasan implementasi saat ini (gradient, image, multithreading)
