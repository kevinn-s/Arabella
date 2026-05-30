# BAB 3 METODE PENELITIAN

> Catatan versi: Berkas ini adalah **versi alternatif (v2)** Bab 3 dengan struktur yang ditata ulang mengikuti alur pipeline rendering (Metode Penelitian → Analisis → Perancangan). Versi asli yang mengikuti template UML klasik tetap tersedia pada `bab3_metodologi.md`. Kedua versi merujuk source code yang sama; perbedaan hanya pada organisasi penyajian. Diagram UML (use case, sequence, class) pada versi ini dilipat ke dalam Subbab 3.3.1, bagian "Pewarnaan" dibatasi pada warna solid sesuai kapabilitas shader aktual, sedangkan "Analisis User" dan "Perancangan Layar" dipertahankan masing-masing pada pengantar Subbab 3.2 dan Subbab 3.4.

Sebelum uraian dimulai, perlu didefinisikan istilah "klaim teknis" yang dipakai secara konsisten pada Bab 3 ini. Klaim teknis adalah pernyataan terverifikasi mengenai implementasi aktual pustaka Arabella yang menyebut salah satu dari: (a) nama algoritma atau struktur data, (b) nilai parameter numerik konkret (termasuk dimensi ubin, format fixed-point, atau jumlah byte), (c) nama berkas, fungsi, struct, trait, modul, atau konstanta dalam kode, atau (d) perilaku runtime spesifik dari pustaka. Setiap klaim teknis pada Bab 3 ini wajib dapat ditelusuri langsung ke source code melalui rujukan kode berformat `berkas:simbol` atau `berkas:start-end` relatif terhadap akar repositori, sehingga setiap pernyataan dapat divalidasi pembaca dengan membuka berkas yang dirujuk pada `src/`, `Cargo.toml`, `examples/`, atau `tests/`.

## 3.1 Metode Penelitian

Metode penelitian ini dirancang untuk memberikan tahapan yang sistematis dan terarah guna memecahkan masalah dependensi compute shader pada pipeline rendering vektor paralel. Penelitian eksperimental ini dibagi menjadi lima fase utama yang saling berkesinambungan, seperti yang dijabarkan secara tekstual di bawah ini:

1. **Fase 1: Studi Literatur dan Pengumpulan Data**
   - Melakukan analisis mendalam terhadap literatur grafis komputer terkait, mencakup metode shortcut tree, teknik rasterisasi scanline berbasis boundary fragments, fungsi implisit Loop-Blinn, serta arsitektur mesin rendering compute-centric modern seperti Vello.
   - Mengumpulkan berkas sampel uji dalam format Scalable Vector Graphics (SVG) dengan variasi tingkat kompleksitas geometri jalur primitif.

2. **Fase 2: Analisis Kebutuhan Sistem**
   - Mengidentifikasi keterbatasan lingkungan grafis non-compute pada perangkat low-end.
   - Merumuskan spesifikasi fungsional pustaka (library) rendering berbasis pemrograman sistem Rust dan API grafis WebGL 2.0.

3. **Fase 3: Perancangan Arsitektur dan Algoritma**
   - Merancang pembagian beban kerja hibrida: tahap preprocessing paralel masif pada Central Processing Unit (CPU) dan tahap rasterisasi pada Graphics Processing Unit (GPU).
   - Menyusun struktur data spasial berbasis tiling (ubin) serta merancang mekanisme kalkulasi winding number melalui binning DDA dua tahap, akumulator signed-area per scanline, dan propagasi backdrop kiri-ke-kanan.
   - Memformulasikan penyederhanaan kurva Bézier kubik dan kuadratik menjadi segmen garis pada format fixed-point.

4. **Fase 4: Implementasi Purwarupa (Prototype)**
   - Membangun modul parser SVG dan arsitektur data memori di CPU menggunakan Rust.
   - Menyusun struktur data spasial berbasis tile yang dirancang agar pemrosesan antar-jalur saling independen, sehingga membuka jalan bagi paralelisasi tingkat data melalui pustaka Rayon pada pengembangan lanjutan; pada implementasi yang dievaluasi, optimasi paralelisme yang sudah aktif adalah paralelisme tingkat instruksi melalui SIMD pada hot path transformasi dan flattening, sedangkan feature `multithreading` berbasis Rayon masih bersifat opsional dan belum diaktifkan pada build baku.
   - Mengembangkan pemrosesan Vertex Shader dan Fragment Shader konvensional pada GPU menggunakan WebGL 2.0.

5. **Fase 5: Pengujian dan Evaluasi**
   - Melakukan validasi kebenaran output visual (correctness validation) dengan membandingkan citra hasil purwarupa terhadap renderer referensi peramban.
   - Melakukan pengujian performa (benchmarking) untuk mengukur metrik frame time per bingkai yang didekomposisi menjadi biaya tahap pra-pemrosesan di CPU dan biaya tahap rasterisasi di GPU secara terpisah.
   - Menganalisis secara kualitatif posisi arsitektur pustaka yang diusulkan terhadap mesin rendering berbasis CPU murni (Skia/Cairo) serta berbasis GPU komputasi penuh (Vello) pada dimensi paradigma rasterisasi, ketergantungan compute shader, dan target platform.

## 3.2 Analisis

Sistem yang dikembangkan dalam penelitian ini merupakan sebuah pustaka antarmuka pemrograman aplikasi (Application Programming Interface / API) rendering grafis 2D. Oleh karena itu, pengguna langsung (user) dari sistem ini adalah pengembang perangkat lunak (software developer) yang membutuhkan kapabilitas eksekusi grafis vektor performa tinggi untuk diintegrasikan ke dalam aplikasi akhir mereka, seperti game engine, browser web emulasi, atau aplikasi seluler. Pengguna sistem ini diwajibkan memiliki pemahaman dasar mengenai matematika komputasi grafis (seperti koordinat kartesian, vektor, dan kurva parametrik), alur kerja pipeline grafis tradisional (konsep shader, vertex buffer, dan framebuffer), serta memiliki pengalaman dalam implementasi kode menggunakan bahasa pemrograman sistem. Interaksi pengembang dengan pustaka dilakukan sepenuhnya secara terprogram (programmatic) melalui pemanggilan fungsi-fungsi pustaka dan penyerahan deskripsi geometri berbasis teks atau biner (seperti data jalur SVG), tanpa melibatkan interaksi komponen antarmuka grafis pengguna akhir (Graphical User Interface). Karakteristik pengguna sebagai pengembang inilah yang menjadi konteks bagi analisis perbandingan, permasalahan, dan usulan pemecahan masalah pada tiga sub-bagian berikut.

### 3.2.1 Analisis Perbandingan dengan Aplikasi Sejenis

Untuk memvalidasi urgensi pengembangan sistem, dilakukan analisis komparatif terhadap tiga arsitektur rendering grafis yang menjadi acuan utama dalam penelitian ini:

1. **Vello (Linebender)** — Merupakan mesin rendering modern yang mengadopsi paradigma GPU compute-centric. Pustaka ini memindahkan seluruh komputasi sekuensial yang berat — seperti tessellation, pemotongan geometri (clipping), dan alokasi memori spasial — langsung ke GPU menggunakan serangkaian compute shader dispatches. Akselerasi paralelnya memanfaatkan algoritma parallel prefix-sum guna menurunkan kompleksitas serial $O(n)$ menjadi tugas paralel $O(\log n)$. Walaupun menghasilkan throughput masif pada perangkat high-end, Vello memerlukan dukungan WebGPU API atau Vulkan modern, sehingga tidak dapat beroperasi secara stabil pada segmen perangkat keras legacy atau low-end kelas konsumen.

2. **Massively-Parallel Vector Graphics (Ganacim dkk., 2014)** — Sistem ini memparalelkan tahap preprocessing segmen geometri masukan dan tahap rendering sampel piksel secara simultan di GPU. Komponen intinya memanfaatkan struktur data spasial hierarkis adaptif bernama Shortcut Tree (berbasis quadtree) untuk memberikan akses acak cepat terhadap nilai warna piksel. Namun, implementasi algoritma ini sangat bergantung pada arsitektur komputasi umum GPU yang spesifik (seperti teknologi NVIDIA CUDA) untuk menangani warping dan penjadwalan sampel, yang secara drastis membatasi portabilitas lintas platform.

3. **Efficient GPU Path Rendering Using Scanline Rasterization (Li dkk., 2016)** — Pendekatan ini mengadaptasi algoritma scanline rasterizer klasik agar berjalan paralel di atas GPU. Logika utamanya memisahkan pemrosesan antara piksel perbatasan jalur (boundary fragments berukuran $2 \times 2$ piksel) dan piksel bagian dalam (horizontal spans). Pendekatan ini meminimalkan biaya komputasi winding number global dengan melokalisasi komputasi cakupan anti-aliasing. Walaupun efisien, sistem ini tetap mengandalkan arsitektur compute pipeline untuk fase pengurutan (sorting) dan penggabungan (merging) fragmen sebelum tahap rasterisasi akhir dilakukan.

### 3.2.2 Analisis Permasalahan

Ketiga arsitektur acuan pada Subbab 3.2.1 menunjukkan satu benang merah yang sama: kapabilitas paralelisme masif yang mereka tawarkan ditebus dengan ketergantungan pada fitur komputasi GPU tingkat lanjut — compute shader (Vello), CUDA warp scheduling (Ganacim dkk., 2014), atau compute pipeline untuk sorting/merging fragmen (Li dkk., 2016). Akibatnya, ketiga sistem tersebut tidak dapat beroperasi secara stabil pada lingkungan grafis terbatas, yaitu peramban web lama dan perangkat keras low-end yang hanya menyediakan rasterization pipeline tradisional (vertex shader dan fragment shader) tanpa dukungan compute. Dari analisis kesenjangan (gap analysis) ini, tiga permasalahan utama dapat dirumuskan:

1. Belum tersedianya metode rendering grafis vektor paralel secara masif yang **tidak bergantung pada fitur compute shader**, sehingga renderer berperforma tinggi yang ada tidak portabel ke lingkungan grafis non-compute.
2. Kebutuhan akan **jaminan kompatibilitas platform yang luas**, terutama pada lingkungan grafis terbatas (web lama dan perangkat low-end) yang hanya menyediakan pipeline rasterisasi tradisional.
3. Kebutuhan akan **efisiensi performa tinggi dan minimalisasi overdraw komputasi** pada skenario adegan (scene) vektor yang kompleks, tanpa memindahkan beban kerja sekuensial yang berat ke GPU.

### 3.2.3 Usulan Pemecahan Masalah

Berdasarkan ketiga rumusan permasalahan tersebut, penelitian ini mengusulkan pustaka rendering hibrida CPU–GPU bernama Arabella. Tabel 3.1 memetakan setiap rumusan permasalahan terhadap solusi teknis yang diimplementasikan di dalam pustaka.

**Tabel 3.1.** Pemetaan rumusan permasalahan dan solusi teknis terimplementasi.

| No | Rumusan Permasalahan | Solusi Teknis Terimplementasi |
|----|----------------------|-------------------------------|
| 1 | Kebutuhan akan metode rendering grafis vektor paralel secara masif yang tidak bergantung pada fitur compute shader. | Merancang pipeline hibrida yang membagi tugas secara tegas: mengeksekusi tahapan preprocessing spasial secara paralel masif di CPU, dan mengalokasikan rasterisasi piksel ke GPU. |
| 2 | Kebutuhan akan jaminan kompatibilitas platform yang luas, terutama pada lingkungan grafis terbatas (web lama dan perangkat low-end). | Membatasi implementasi GPU secara ketat hanya pada rasterization pipeline tradisional menggunakan Vertex Shader dan Fragment Shader standar WebGL 2.0. |
| 3 | Kebutuhan akan efisiensi performa tinggi dan minimalisasi overdraw komputasi pada skenario adegan (scene) vektor yang kompleks. | Menerapkan segmentasi layar berbasis tiling (ubin) berukuran tetap di CPU melalui pipeline tiga tahap: binning DDA dua tahap (outer DDA lintas baris ubin dan inner DDA lintas kolom ubin) untuk memecah segmen garis ke ubin yang dilintasinya, akumulator signed-area per scanline untuk menghitung winding number secara inkremental, dan propagasi backdrop kiri-ke-kanan saat emisi ubin, sehingga hanya ubin nontrivial yang dikirim ke memori GPU. |

## 3.3 Perancangan

Subbab ini memuat perancangan teknis pustaka Arabella. Penyajiannya ditata mengikuti alur pipeline rendering: dimulai dari gambaran umum arsitektur (3.3.1), lalu masukan geometri jalur (3.3.2), aturan pengisian (3.3.3), pewarnaan (3.3.4), tahap rasterisasi kasar di CPU (3.3.5), dan diakhiri tahap rasterisasi halus di GPU (3.3.6). Diagram interaksi (use case, sequence, dan class) yang pada template klasik berdiri sebagai subbab terpisah, di sini dilipat ke dalam 3.3.1 sebagai bagian dari gambaran umum arsitektur.

### 3.3.1 Gambaran Umum Arsitektur

#### Spesifikasi Pustaka

Pustaka rendering vektor paralel Arabella dirancang sebagai crate Rust yang menargetkan lingkungan peramban web melalui WebAssembly, dengan dependensi grafis yang dikunci pada WebGL 2.0 sebagai target langsung. Bahasa pemrograman utamanya adalah Rust edisi 2024, sebagaimana dideklarasikan oleh `edition = "2024"` pada blok `[package]` (`Cargo.toml:7`); pemilihan edisi ini didasarkan pada zero-cost abstractions, jaminan memory safety tanpa garbage collector, serta paradigma fearless concurrency yang krusial untuk paralelisme CPU. Target eksekusi utama adalah `wasm32-unknown-unknown` pada lingkungan peramban dengan API grafis WebGL 2.0 tanpa lapisan transpilasi tambahan; konfigurasi target wasm dideklarasikan pada blok `[target.'cfg(target_arch = "wasm32")'.dependencies]` (`Cargo.toml:57-87`) yang menarik fitur `WebGl2RenderingContext` dari crate `web-sys`. Tidak ada penarikan API WebGL 1.0 maupun WebGPU pada blok target wasm tersebut.

Pustaka Arabella mendeklarasikan tiga belas crate sebagai dependensi langsung pada blok `[dependencies]` (`Cargo.toml:29-42`). Yang paling berperan pada pipeline rendering adalah `fearless_simd` versi `"0.4.0"` (`Cargo.toml:31`) untuk akselerasi SIMD pada hot path pra-pemrosesan CPU, `bytemuck` versi `"1.25.0"` (`Cargo.toml:30`) untuk reinterpretasi tipe POD saat menyiapkan vertex buffer dan tekstur GPU, trio `lyon_path`/`lyon_geom`/`lyon_algorithms` (`Cargo.toml:37-39`) dan `kurbo` versi `"0.13.0"` (`Cargo.toml:41`) untuk representasi serta operasi geometri jalur, `peniko` versi `"0.6.1"` (`Cargo.toml:40`) untuk primitif paint dan warna, serta `roxmltree` versi `"0.20.0"` (`Cargo.toml:42`) yang dipakai `src/pico_svg.rs` untuk membaca dokumen SVG. Crate `rayon` versi `"1.11.0"` dideklarasikan sebagai dependensi opsional pada blok `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` (`Cargo.toml:48-50`) bersama `thread_local`, keduanya hanya ditarik ketika feature flag `multithreading = ["std", "dep:rayon", "dep:thread_local"]` (`Cargo.toml:94`) diaktifkan secara opt-in. Karena `multithreading` tidak termasuk dalam feature `default = ["std", "png"]`, build standar tidak menyertakan Rayon; dengan demikian klaim paralelisme CPU pada Bab 3 ini bersifat potensial — kapasitas yang sudah disiapkan pada manifest — bukan paralelisme yang sudah aktif pada implementasi yang dievaluasi.

Arabella menerima dokumen SVG sebagai format data geometri masukan melalui modul `src/pico_svg.rs`. Parser ini bukan implementasi SVG 1.1 Core lengkap, melainkan subset minimal yang hanya menangani dua nama elemen secara eksplisit pada dispatch tag-name di `Parser::rec_parse` (`src/pico_svg.rs:191`): elemen grup `g` dan elemen jalur `path`. Atribut presentation yang diparse terbatas pada `fill`, `stroke`, `stroke-width`, dan — khusus pada elemen `g` — `transform`. Setiap elemen di luar dua nama tersebut jatuh ke arm fallback `other => eprintln!("Unhandled node type {other}")` (`src/pico_svg.rs:228`) tanpa pemrosesan lebih lanjut. Konsekuensinya, fitur SVG umum seperti elemen teks, `defs`, gradient, pattern, filter, clipPath/mask, bentuk dasar non-`path`, serta penyisipan raster tidak ditangani; pemakai pustaka memodelkan setiap geometri sebagai elemen `<path d="…">`.

#### Pembagian Beban Kerja Hibrida

Arsitektur Arabella membagi pekerjaan rendering geometri dua dimensi menjadi dua fase besar yang berurutan. **Fase pra-pemrosesan CPU** mencakup flattening kurva menjadi segmen garis, binning DDA dua tahap ke kisi ubin, akumulasi signed-area per scanline, dan propagasi backdrop kiri-ke-kanan saat emisi ubin. **Fase rasterisasi GPU** mengeksekusi vertex shader instanced quad dan fragment shader analitik untuk seluruh ubin nontrivial. Viewport dibagi secara spasial menjadi kisi ubin homogen berukuran tetap $16 \times 8$ piksel sesuai konstanta `TILE_W = 16` dan `TILE_H = 8` (`src/blocks.rs:6-7`, verbatim `pub const TILE_W: usize = 16;` dan `pub const TILE_H: usize = 8;`), yang dipakai konsisten oleh `Builder::new` saat menginisialisasi setiap rekord `Tile` (`src/builder.rs:60-61`). Variasi perilaku antar ubin pada tahap GPU sepenuhnya didorong data — jumlah segmen yang di-binning, isi backdrop per scanline, dan bit fill rule pada `paint_and_rect_flag` — sehingga seluruh ubin nontrivial diproses fragment shader tunggal melalui jalur kode yang sama tanpa cabang `if`/`switch` berbasis kategori ubin.

#### Interaksi Fungsional (Use Case)

Sistem ini memodelkan interaksi fungsional antara aktor tunggal (Developer) dengan batasan sistem pustaka melalui tiga use case utama. **UC-01 Inisialisasi Context** menyiapkan konteks grafis WebGL 2.0, mengompilasi serta menautkan program Vertex Shader dan Fragment Shader konvensional, lalu mengalokasikan objek memori awal; prasyaratnya adalah aplikasi pengembang sudah memiliki referensi permukaan jendela grafis aktif, dan kondisi akhirnya pustaka siap menerima data geometri. **UC-02 Input Data Vektor** menjembatani penyerahan berkas geometri dari aplikasi utama ke memori internal pustaka: developer memanggil fungsi penyerahan data path atau memuat berkas SVG, sistem mem-parsing masukan menjadi elemen kurva dasar (linear, kuadratik, kubik), lalu menyimpannya beserta informasi warna ke koleksi lintasan `Scene` di memori utama. **UC-03 Render Frame** adalah use case inti yang memicu loop rendering per bingkai hibrida; use case ini secara otomatis mengikutsertakan (include) sub-proses pra-pemrosesan CPU (binning DDA + akumulator signed-area + propagasi backdrop) dan sub-proses rasterisasi GPU (vertex shader instanced quad + fragment shader analitik), dengan kondisi akhir hasil visual ter-rasterisasi pada viewport target.

#### Alur Sekuensial Satu Bingkai (Sequence)

Daur hidup satu bingkai rendering melibatkan lima partisipan — Aplikasi Utama, `Scene`, `Builder`, `WebGlRenderer`, dan GPU — yang berinteraksi melalui lima pesan berurutan. Karena pipeline memproses seluruh ubin nontrivial melalui satu jalur kode tunggal tanpa percabangan berbasis tipe ubin, alur ini tidak memuat blok alternatif maupun perulangan bercabang berdasarkan klasifikasi ubin.

1. **Aplikasi Utama → `Scene`.** Aplikasi memanggil `Scene::fill(path, fill_rule, transform, brush)` (`src/scene.rs:70`) untuk jalur isian atau `Scene::stroke(path, style, transform, brush)` (`src/scene.rs:117`) untuk jalur garis. Pada jalur stroke, outline hasil ekspansi `kurbo::stroke_with` dialirkan kembali ke `Scene::fill` dengan `FillRule::NonZero` (`src/scene.rs:158`), sehingga kedua API menyatu pada pipeline isian yang sama.
2. **`Scene` → `Builder` (flattening + binning).** `Scene::fill` mendelegasikan tahap pertama pra-pemrosesan ke `Builder::build_path(path, fill_rule, transform)` (`src/builder.rs:84`): flattening kurva Bézier menjadi segmen garis F24Dot8 (`src/path.rs`, `src/flatten.rs`), lalu binning setiap segmen ke ubin $16 \times 8$ melalui outer dan inner DDA (`src/blocks.rs:bin_line`, `src/blocks.rs:Blocks::build_block`) sambil mengakumulasi signed-area per scanline pada format 8.8 fixed-point (`src/blocks.rs:record_per_scanline_crossings`).
3. **`Scene` → `Builder` (propagasi + emisi).** `Scene::fill` melanjutkan dengan `Builder::generate_tiles(paint_index, fill_rule, payload, paint_flag)` (`src/builder.rs:151`, situs panggil `src/scene.rs:93`) yang mempropagasikan backdrop kiri-ke-kanan per baris ubin lalu mengemisi seluruh ubin nontrivial ke `Vec<Tile>`.
4. **Aplikasi Utama → `WebGlRenderer`.** Aplikasi memicu `WebGlRenderer::render(&scene, &render_size)` (`src/render/webgl.rs:296`), yang membaca vertex buffer instanced via `scene.tiles()` (`src/scene.rs:155`) dan tekstur segmen RGBA32F via `scene.segments()` (`src/scene.rs:160`), lalu mengunggah keduanya ke GPU bersama uniform konfigurasi.
5. **`WebGlRenderer` → GPU.** `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len())` (`src/render/webgl.rs:393-398`) menjalankan vertex shader instanced quad satu instance per ubin nontrivial, lalu fragment shader analitik tunggal (`src/render/shaders/render_tile.frag`) menjumlahkan kontribusi `line_box` tiap segmen, menggabungkannya dengan backdrop per scanline, dan menerapkan fill rule. Hasil komposit warna dengan premultiplied alpha dialirkan ke framebuffer layar, mengakhiri siklus bingkai.

#### Struktur Kelas Inti (Class)

Arsitektur perangkat lunak Arabella diorganisasikan sebagai sejumlah struct Rust yang saling terkait. Tabel 3.2 meringkas struct inti beserta peran dan rujukan kodenya; field rinci tiap struct diuraikan kembali pada subbab teknis terkait (3.3.5 dan 3.3.6).

**Tabel 3.2.** Struct inti pustaka Arabella dan perannya.

| Struct | Peran | Rujukan kode |
|---|---|---|
| `Scene` | Fasad API publik untuk penyerahan jalur geometri (`fill`, `stroke`, `tiles`, `segments`, `reset`). | `src/scene.rs:35-41` |
| `Builder` | Pengelola pipeline pra-pemrosesan CPU (flattening, binning DDA, propagasi backdrop, emisi ubin). | `src/builder.rs:36-54` |
| `CoverStorage` | Akumulator winding 8.8 fixed-point per scanline dan tag cell yang disentuh DDA. | `src/builder.rs:360-369` |
| `Block` | Rekord per-pasangan (segmen, ubin) hasil binning DDA dengan endpoint ubin-lokal F24Dot8. | `src/blocks.rs:21-39` |
| `Blocks` | Kontainer pengakumulasi `Block` plus driver outer/inner DDA dan pengurutan. | `src/blocks.rs:51-55` |
| `TileBounds` | Batas ruang-ubin pra-komputasi per shape. | `src/blocks.rs:664-669` |
| `Tile` | Rekord vertex buffer instanced 44 byte (`#[repr(C)]`) yang diunggah ke GPU. | `src/tile.rs:9-23` |
| `WebGlRenderer` | Pengelola konteks WebGL 2.0 dan eksekusi pipeline GPU. | `src/render/webgl.rs:265-269` |
| `PicoSvg` | Parser SVG subset (`PicoSvg::load`) yang menghasilkan `Vec<Item>` untuk disuplai ke `Scene`. | `src/pico_svg.rs:84` |

### 3.3.2 Input Path

Masukan geometri pustaka Arabella adalah jalur (path) yang tersusun atas tiga primitif kurva parametrik: garis lurus, kurva Bézier kuadratik, dan kurva Bézier kubik. Ketiganya dikonsumsi oleh fungsi `fill_impl` (`src/path.rs:135`) yang menelusuri iterator `PathEvent` dan men-dispatch tiap varian event ke jalur penanganan masing-masing. Seluruh primitif pada akhirnya direduksi menjadi rangkaian segmen garis lurus pada format fixed-point F24Dot8 yang ditulis ke buffer kerja `Builder::line_buf` sebagai empat `i32` `[p0x, p0y, p1x, p1y]` per segmen; reduksi inilah yang memungkinkan tahap binning DDA bekerja seragam tanpa perlu mengevaluasi fungsi kurva di GPU.

#### 3.3.2.1 Garis

Primitif garis lurus adalah bentuk paling sederhana dan menjadi target akhir seluruh primitif lain. Pada `fill_impl`, event `PathEvent::Line { from, to }` (`src/path.rs:177`) mentransformasi kedua endpoint melalui jalur SIMD `transform_pair` lalu — bila bergeser melebihi `EPSILON` — memanggil `emit_line` (`src/path.rs:313`) yang mengubah koordinat float menjadi F24Dot8 via `f32_to_f24dot8` (`src/path.rs:334`) dan mendorong empat `i32` ke `line_buf`. Garis penutup kontur juga diemisi langsung sebagai garis pada penanganan `PathEvent::End` (`src/path.rs:260`) ketika atribut `close` bernilai benar dan titik akhir belum menyatu dengan titik awal.

Sebuah garis lurus dari $P_0 = (x_0, y_0)$ ke $P_1 = (x_1, y_1)$ dinyatakan secara parametrik sebagai

$$B(t) = (1 - t)\,P_0 + t\,P_1, \qquad t \in [0, 1].$$

#### 3.3.2.2 Kurva Bézier Kuadratik

Kurva Bézier kuadratik didefinisikan oleh tiga titik — titik awal $P_0$, titik kontrol $P_1$, dan titik akhir $P_2$ — dengan persamaan parametrik

$$B(t) = (1 - t)^2 P_0 + 2(1 - t)\,t\,P_1 + t^2 P_2, \qquad t \in [0, 1].$$

Event `PathEvent::Quadratic { from, ctrl, to }` (`src/path.rs:188`) mentransformasi ketiga titik, memperluas bounding box, lalu memecah kurva pada titik ekstrem-y bila ada (via `find_quadratic_extrema`) agar bounding box mencakup titik balik kurva. Setiap potongan kuadratik diteruskan ke `emit_quadratic_and_flatten` (`src/path.rs:291`) yang melinearisasinya menjadi segmen garis melalui De Casteljau midpoint subdivision (`flatten_quadratic`, `src/flatten.rs:20`). Detail algoritma flattening ini diuraikan pada Subbab 3.3.5.3.

#### 3.3.2.3 Kurva Bézier Kubik

Kurva Bézier kubik didefinisikan oleh empat titik — $P_0$, dua titik kontrol $P_1$ dan $P_2$, serta titik akhir $P_3$ — dengan persamaan parametrik

$$B(t) = (1 - t)^3 P_0 + 3(1 - t)^2 t\,P_1 + 3(1 - t)\,t^2 P_2 + t^3 P_3, \qquad t \in [0, 1].$$

Event `PathEvent::Cubic { from, ctrl1, ctrl2, to }` (`src/path.rs:211`) mentransformasi keempat titik sekaligus melalui jalur SIMD `transform_quad` yang memanfaatkan satu register `f32x8`. Berbeda dengan kuadratik, kurva kubik tidak dilinearisasi langsung; Arabella terlebih dahulu mengonversinya menjadi barisan sub-kurva kuadratik melalui pendekatan gaya Vello (`estimate_number_of_quadratic_curves` dan `convert_cubics_to_quadratic_curves` pada `src/path.rs`), baru kemudian tiap sub-kuadratik di-flatten menjadi segmen garis. Justifikasi dan parameter konversi ini diuraikan pada Subbab 3.3.5.2.

### 3.3.3 Aturan Fill

Aturan pengisian (fill rule) menentukan piksel mana yang dianggap berada "di dalam" sebuah jalur tertutup. Arabella mendukung dua aturan standar — Non-Zero dan Even-Odd — yang keduanya dievaluasi dari besaran winding number. Karena evaluasi cakupan dilakukan secara analitik kontinu di fragment shader, winding number pada Arabella bukan bilangan bulat diskret melainkan nilai riil hasil akumulasi backdrop per scanline dan kontribusi area tiap segmen.

#### 3.3.3.1 Winding Number

Winding number suatu titik terhadap jalur tertutup adalah jumlah berapa kali jalur tersebut melingkari titik, dengan tanda mengikuti arah lintasan. Secara konvensional, sebuah sinar ditembakkan dari titik uji; setiap perpotongan dengan tepi jalur menambah $+1$ bila tepi melintas ke satu arah dan $-1$ bila ke arah berlawanan. Pada Arabella, besaran ini dihitung secara inkremental: kontribusi tanda tiap segmen vertikal diakumulasi per scanline pada format 8.8 fixed-point ($256$ unit setara satu winding penuh) saat binning DDA (Subbab 3.3.5.6), lalu dipropagasikan kiri-ke-kanan antar ubin (Subbab 3.3.5.8). Di GPU, nilai backdrop ini menjadi basis kontinu yang ditambah kontribusi area sub-piksel tiap segmen melalui fungsi `line_box` (Subbab 3.3.6.3), menghasilkan winding riil $\omega$ per piksel. Konvensi tanda di CPU bersifat eksplisit: garis turun mengurangi akumulator $256$ unit dan garis naik menambah $256$ unit (`src/blocks.rs:record_per_scanline_crossings`).

#### 3.3.3.2 Aturan Non-Zero

Aturan Non-Zero menganggap sebuah titik berada di dalam jalur bila winding number-nya tidak sama dengan nol. Pada implementasi kontinu Arabella, aturan ini diwujudkan sebagai clamp nilai mutlak winding ke interval $[0, 1]$ untuk menghasilkan koefisien cakupan (coverage):

$$\text{coverage} = \mathrm{clamp}\bigl(|\omega|,\; 0{,}0,\; 1{,}0\bigr).$$

Bentuk GLSL-nya adalah `coverage = clamp(abs(winding), 0.0, 1.0);` pada `src/render/shaders/render_tile.frag:215`, dipilih ketika bit fill rule sama dengan `FILL_RULE_NONZERO = 0u`. Aturan ini menjadi default untuk jalur stroke karena `Scene::stroke` mengalirkan outline ekspansinya ke `Scene::fill` dengan `FillRule::NonZero` (`src/scene.rs:158`).

#### 3.3.3.3 Aturan Even-Odd

Aturan Even-Odd menganggap sebuah titik berada di dalam jalur bila winding number-nya ganjil dan di luar bila genap. Pada implementasi kontinu Arabella, aturan ini diwujudkan sebagai gelombang segitiga (triangle wave) pada nilai mutlak winding:

$$\text{coverage} = 1{,}0 - \bigl|\,\mathrm{mod}(|\omega|,\; 2{,}0) - 1{,}0\,\bigr|.$$

Bentuk GLSL-nya adalah `coverage = 1.0 - abs(mod(abs(winding), 2.0) - 1.0);` pada `src/render/shaders/render_tile.frag:218-219`, dipilih ketika bit fill rule sama dengan `FILL_RULE_EVENODD = 1u`. Kedua aturan diterapkan pada fragment shader yang sama melalui satu cabang biner `if (fill_rule == FILL_RULE_NONZERO) { … } else { … }` (`src/render/shaders/render_tile.frag:213-220`) yang hanya bergantung pada bit fill rule, bukan pada kategori ubin. Penyandian bit fill rule pada sisi CPU dilakukan `Builder::generate_tiles` (`src/builder.rs:158-162`) yang memetakan `peniko::Fill::NonZero` ke `FILL_RULE_NONZERO = 0` dan `peniko::Fill::EvenOdd` ke `FILL_RULE_EVENODD = 1` (`src/builder.rs:28-29`), lalu menggabungkan hasilnya ke `paint_and_rect_flag` melalui shift `FILL_RULE_SHIFT: u32 = 24` (`src/builder.rs:17`).

### 3.3.4 Pewarnaan

Tahap pewarnaan menentukan warna akhir yang ditulis ke framebuffer untuk setiap piksel yang dicakup oleh sebuah jalur. Pada implementasi Arabella yang dievaluasi, pewarnaan dibatasi pada **warna solid**; pipeline tipe paint yang lebih kompleks (gradien, image) memiliki kerangka struktur data warisan dari basis kode Vello pada modul `src/paint/`, namun belum tersambung ke jalur rasterisasi WebGL dan karena itu tidak dibahas sebagai bagian perancangan yang dievaluasi.

#### 3.3.4.1 Warna Solid

Warna solid adalah satu-satunya tipe paint yang dievaluasi penuh oleh fragment shader. Konstanta `PAINT_TYPE_SOLID = 0u` (`src/render/shaders/render_tile.frag`) menandai jalur ini. Warna disandikan sebagai satu `u32` RGBA8 yang dibawa melalui field `payload` pada rekord `Tile` dan diteruskan ke fragment shader sebagai `v_payload`. Di dalam `main()`, fungsi `unpack_rgba8` (`src/render/shaders/render_tile.frag`) membongkar keempat kanal warna dari `u32` tersebut:

$$r = \frac{(\text{packed} \gg 0)\;\&\;\text{0xFF}}{255}, \quad g = \frac{(\text{packed} \gg 8)\;\&\;\text{0xFF}}{255}, \quad b = \frac{(\text{packed} \gg 16)\;\&\;\text{0xFF}}{255}, \quad a = \frac{(\text{packed} \gg 24)\;\&\;\text{0xFF}}{255}.$$

Warna hasil unpack kemudian dikomposit dengan koefisien cakupan yang dihitung pada Subbab 3.3.3, menghasilkan keluaran premultiplied alpha `fragColor = vec4(paint.rgb * paint.a * coverage, paint.a * coverage)` pada akhir `main()`. Pada sisi CPU, warna `AlphaColor<Srgb>` dikonversi menjadi representasi premultiplied melalui `PremulColor::from_alpha_color` (`src/paint/paint.rs`), yang menyimpan baik bentuk RGBA8 maupun RGBAF32 agar tidak perlu dihitung ulang per ubin.

### 3.3.5 Rasterisasi Kasar (CPU)

Tahap rasterisasi kasar adalah seluruh pekerjaan pra-pemrosesan yang dieksekusi di CPU sebelum data dikirim ke GPU. Disebut "kasar" karena keluarannya bukan piksel akhir melainkan struktur data spasial tingkat ubin: daftar segmen tile-lokal, akumulator winding per scanline, dan daftar ubin nontrivial yang siap dirasterisasi halus oleh GPU. Subbab ini menguraikan tahap-tahap tersebut secara berurutan.

#### 3.3.5.1 Gambaran Umum

Rasterisasi kasar berjalan melalui rantai pemanggilan yang dipimpin `Builder`. Method `Builder::build_path` (`src/builder.rs:84`) menjalankan flattening seluruh kurva menjadi `line_buf` (Fase 2 pada `src/builder.rs:99-102`), menghitung bounding box dan batas ubin via `TileBounds::from_box2d` (Fase 3), mereset state per-shape (Fase 4), lalu melakukan binning DDA atas tiap segmen (Fase 5, loop `src/builder.rs:138-148`). Setelah seluruh segmen ter-binning, `Builder::generate_tiles` (`src/builder.rs:151`) mempropagasikan backdrop dan mengemisi ubin nontrivial. Keseluruhan tahap ini berjalan single-thread per pemanggilan `Scene::fill`/`Scene::stroke` pada build baku; satu-satunya paralelisme aktif adalah SIMD tingkat instruksi pada hot path transformasi dan flattening. Sembilan sub-bagian berikut merinci tiap langkah secara berurutan, dari konversi kurva hingga persiapan resource WebGL.

#### 3.3.5.2 Konversi Kurva Kubik ke Kuadratik

Kurva Bézier kubik tidak dilinearisasi langsung melainkan dikonversi lebih dulu menjadi barisan sub-kurva kuadratik, mengikuti strategi cubic-to-quadratic gaya Vello. Jumlah sub-kuadratik diestimasi oleh `estimate_number_of_quadratic_curves` (`src/path.rs:352-375`) berdasarkan toleransi `TOL = 0.25` dan look-up table 16 entri yang membatasi sub-kuadratik maksimum pada `MAX_QUADS = 16` (`src/path.rs:7`). Selanjutnya `convert_cubics_to_quadratic_curves` (`src/path.rs:391-475`) menghasilkan barisan titik kontrol sub-kuadratik dengan menyampel kurva kubik pada interval seragam $\Delta t = 0{,}5 / n$ menggunakan jalur SIMD `f32x8` dari `fearless_simd`. Strategi ini dipilih agar tahap flattening cukup menangani satu jenis kurva (kuadratik), menyederhanakan hot path tanpa kehilangan akurasi pada toleransi yang ditetapkan.

#### 3.3.5.3 Flattening Kurva Kuadratik (Midpoint Subdivision)

Setiap sub-kuadratik dilinearisasi menjadi segmen garis melalui De Casteljau midpoint subdivision pada `flatten_quadratic` (`src/flatten.rs:20-29`) yang mendelegasikan rekursi ke `flatten_recursive` (`src/flatten.rs:31-58`). Pada setiap level rekursi, fungsi `is_flat_enough` (`src/flatten.rs:62-87`) menguji deviasi L1 antara midpoint chord $\frac{P_0 + P_2}{2}$ dan titik kontrol $P_1$ terhadap konstanta `FLATNESS_THRESHOLD: i32 = 32` (`src/flatten.rs:18`). Bila lulus uji, kurva dianggap cukup datar dan satu segmen garis lurus dari $P_0$ ke $P_2$ ditulis ke `line_buf`. Bila belum lulus, kurva dipecah dua di $t = 0{,}5$ memakai pembagian bilangan bulat F24Dot8 (`src/flatten.rs:46-57`) — bukan pergeseran aritmetika — agar tidak terjadi drift satu bit pada viewport bertanda. Pendekatan rekursif adaptif ini hanya menambah segmen pada bagian kurva yang benar-benar melengkung, sehingga jumlah segmen tetap proporsional terhadap kelengkungan.

#### 3.3.5.4 Representasi Fixed-Point (F24Dot8)

Seluruh koordinat segmen garis disimpan pada format fixed-point F24Dot8, yaitu bilangan bertanda 32-bit (`i32`) dengan 24 bit bagian bulat dan 8 bit bagian pecahan, sehingga satu piksel setara $256$ unit. Konversi dari float dilakukan `f32_to_f24dot8` (`src/path.rs:334`) yang memakai pembulatan round-to-nearest simetris ($v \times 256$ lalu dibulatkan), bukan truncation `as i32`; pembulatan simetris ini penting karena truncation membulatkan nilai negatif ke arah nol dan membiaskan winding secara asimetris, meninggalkan residu akumulator scanline yang bocor sebagai garis-garis (streak) ke kanan. Ukuran ubin pun direpresentasikan dalam F24Dot8 melalui konstanta `TILE_W_F24DOT8 = 4096` dan `TILE_H_F24DOT8 = 2048` (`src/blocks.rs:10-11`), yaitu $16 \times 256$ dan $8 \times 256$, yang dipakai seluruh varian DDA untuk menghitung offset sub-piksel. Representasi fixed-point dipilih agar aritmetika binning bersifat deterministik dan eksak lintas platform, menghindari ketidakkonsistenan pembulatan floating-point.

#### 3.3.5.5 Digital Differential Analyzer (DDA)

Setelah `line_buf` terisi segmen garis F24Dot8, setiap segmen di-binning ke ubin yang dilintasinya melalui dua tahap Digital Differential Analyzer (DDA) yang seluruhnya berada di `src/blocks.rs`. Pintu masuk publiknya adalah `Blocks::build_block` (`src/blocks.rs:93-102`) yang melakukan tail-call ke `Blocks::bin_line` (`src/blocks.rs:107-160`). DDA dipilih karena memecah segmen lintas batas ubin dengan aritmetika inkremental tanpa pembagian per langkah, sehingga efisien dan eksak pada format fixed-point. Binning dipecah menjadi dua tingkat — outer DDA memecah lintas baris ubin, inner DDA memecah lintas kolom ubin — yang dijelaskan pada dua sub-bagian berikut.

##### 3.3.5.5.1 DDA Luar: Pembagian Baris

Outer DDA pada `Blocks::bin_line` memecah satu segmen garis lintas baris ubin (tinggi `TILE_H`) menggunakan empat varian diagonal berdasarkan tanda $(dx, dy)$: down-right pada `outer_dda_down_right` (`src/blocks.rs:163-207`), down-left pada `outer_dda_down_left` (`src/blocks.rs:210-254`), up-right pada `outer_dda_up_right` (`src/blocks.rs:257-301`), dan up-left pada `outer_dda_up_left` (`src/blocks.rs:304-348`). Selain empat arah diagonal, outer DDA menangani tiga kasus khusus sebagai cabang terpisah: (i) horizontal-degenerate ketika `p0y == p1y` yang langsung ber-early-return tanpa emisi karena segmen horizontal murni tidak menyumbang winding (`src/blocks.rs:115-117`); (ii) single-row ketika seluruh segmen muat di satu baris ubin sehingga langsung dilewatkan ke inner DDA (`src/blocks.rs:134-138` arah turun, `src/blocks.rs:149-153` arah naik); dan (iii) vertikal-degenerate ketika `p0x == p1x` yang ditangani di `bin_line_in_row` (`src/blocks.rs:366-374`).

##### 3.3.5.5.2 DDA Dalam: Pembagian Kolom

Setelah outer DDA mengklip sub-segmen ke satu baris ubin, inner DDA pada `Blocks::bin_line_in_row` (`src/blocks.rs:353-391`) memecah sub-segmen lintas kolom ubin (lebar `TILE_W`) memakai empat varian arah utama: right-down pada `inner_dda_right_down` (`src/blocks.rs:394-451`), right-up pada `inner_dda_right_up` (`src/blocks.rs:454-505`), left-down pada `inner_dda_left_down` (`src/blocks.rs:508-560`), dan left-up pada `inner_dda_left_up` (`src/blocks.rs:563-619`). Setiap iterasi inner DDA memanggil `Blocks::push_to_tile` (`src/blocks.rs:625-657`) yang men-tag cell `(row, col)` sebagai disentuh, mendorong satu rekord `Block` berisi endpoint sub-segmen pada koordinat ubin-lokal F24Dot8 ke `self.data: Vec<Block>`, lalu memanggil akumulator signed-area yang dijelaskan pada Subbab 3.3.5.6.

#### 3.3.5.6 Akumulasi Crossing per Scanline

Bersamaan dengan setiap pemanggilan `push_to_tile`, fungsi `record_per_scanline_crossings` (`src/blocks.rs:710-757`) mengakumulasi sumbangan signed area sub-segmen vertikal terhadap delapan strip scanline pada satu ubin. Akumulator disimpan sebagai `[i16; TILE_H]` — delapan akumulator i16 per cell, satu per scanline — pada format 8.8 fixed-point dengan konvensi tanda eksplisit: garis turun ($y_0 < y_1$) mengurangi akumulator sebesar $-256$ unit (setara $-1$ winding penuh), sedangkan garis naik ($y_0 > y_1$) menambah akumulator sebesar $+256$ unit (setara $+1$ winding penuh). Penjumlahan memakai `saturating_add` agar tidak terjadi overflow i16 untuk segmen yang melintasi banyak scanline. Skema ini adalah akumulator signed-area kanonik gaya Blaze (Gasiulis, 2024), FreeType (The FreeType Project, 2023), dan Skia (The Skia Project, 2023), yang dipilih agar fragment shader pada GPU dapat melakukan multisampling sub-piksel tanpa residu winding yang menggariskan seam antar ubin tetangga. Hasil array signed-area per cell disimpan pada `CoverStorage::backdrops: Vec<[i16; TILE_H]>` (`src/builder.rs:360-369`) yang dialokasikan ulang per shape oleh `CoverStorage::reset_for_shape` (`src/builder.rs:413`).

#### 3.3.5.7 Pencatatan dan Pengurutan Segmen

Setiap pasangan (segmen, ubin) dicatat sebagai satu rekord `Block` (`src/blocks.rs:21-39`) yang menyimpan endpoint `(p0x, p0y)` dan `(p1x, p1y)` dalam koordinat ubin-lokal F24Dot8 (rentang $[0, \text{TILE\_W} \times 256]$ untuk x dan $[0, \text{TILE\_H} \times 256]$ untuk y) beserta indeks kolom dan baris ubin global `(x, y)`. Seluruh rekord dikumpulkan pada `Blocks::data: Vec<Block>`. Pada awal `Builder::generate_tiles`, method `Blocks::sort_blocks` (`src/blocks.rs:77`) mengurutkan rekord menurut kunci `(y, x)` sehingga seluruh block milik satu ubin menempati rentang kontigu. Pengurutan ini memungkinkan lookup rentang segmen per ubin dilakukan via binary search pada `find_segment_range` (`src/builder.rs:171, 349-354`), menggantikan pencarian linier dan menjaga biaya emisi tetap rendah meski jumlah segmen besar.

#### 3.3.5.8 Generasi Tile (Propagasi Backdrop)

Tahap emisi ubin dijalankan `Builder::generate_tiles` (`src/builder.rs:151-337`). Setelah penyandian bit fill rule (`src/builder.rs:158-162`) dan pengurutan block (`src/builder.rs:171`), loop emisi menelusuri grid ubin baris demi baris. Pada awal setiap baris ubin, akumulator lokal `acc_arr: [i16; 8]` di-reset menjadi `[0; 8]` di tepi kiri (`src/builder.rs:194`), lalu loop kolom berjalan dari `col = 0` hingga `col = covers.cols() - 1` sehingga akumulator winding bertambah monotonik kiri-ke-kanan sepanjang baris. Sebuah cell `(row, col)` diemisi sebagai `Tile` bila gerbang `tagged || acc_nonzero` bernilai benar (`src/builder.rs:196-203`): `tagged` menandakan DDA menyentuh cell ini, sedangkan `acc_nonzero` menandakan akumulator yang dipropagasi dari kolom kiri belum kembali nol. Definisi operasional inilah yang dipakai untuk istilah "ubin nontrivial" sepanjang Bab 3. Saat ubin diemisi, `Tile.backdrop = acc_arr` ditulis lebih dulu (`src/builder.rs:223-237`) — yaitu nilai akumulator sebelum kontribusi cell ini ditambahkan — sehingga backdrop yang diunggah ke GPU mencerminkan sumbangan kumulatif seluruh kolom di kirinya. Setelah emisi, jika cell `tagged`, akumulator diperbarui in-place dengan satu instruksi SIMD `i16x8.add` yang menggantikan delapan operasi skalar (`src/builder.rs:240-247`), lalu dibawa ke `col + 1`. Propagasi kiri-ke-kanan ini menggantikan kebutuhan akan ray shooting per ubin: nilai winding di tepi kiri tiap ubin sudah tersedia sebagai akumulasi crossing seluruh ubin sebelumnya pada baris yang sama.

#### 3.3.5.9 Persiapan Resource WebGL

Hasil akhir rasterisasi kasar adalah tiga buffer in-memory yang menjembatani CPU dan GPU. Karena pustaka bersifat stateless dan dialokasikan ulang per bingkai, tidak ada basis data persisten; "perancangan tata letak data" di sini berarti perancangan tata letak biner ketiga buffer tersebut.

Pertama, **daftar ubin sebagai vektor datar `Vec<Tile>`** yang dipegang field `tiles` milik `Builder` dan dieksposkan via `Scene::tiles()` (`src/scene.rs:155-157`). Tipe `Tile` berstatus `#[repr(C)]` (`src/tile.rs:9-23`) sehingga setiap rekord menempati tepat 44 byte yang identik di RAM maupun Video RAM; ukuran ini diverifikasi runtime oleh `core::mem::size_of::<Tile>()` yang dipakai sebagai stride VAO (`src/render/webgl.rs:529`). Atribut `#[derive(Pod, Zeroable)]` mengizinkan `bytemuck::cast_slice` mengonversi `&[Tile]` menjadi `&[u8]` tanpa salinan, sehingga `Vec<Tile>` dapat di-`buffer_data_with_u8_array` langsung ke vertex buffer (`src/render/webgl.rs:381-388`). Tabel 3.3 merinci tata letak byte 44-byte tersebut.

**Tabel 3.3.** Tata letak byte vertex buffer instanced 44 byte/tile (`src/tile.rs:9-23`).

| Offset (byte) | Field | Tipe | Lebar (byte) | Slot atribut (`initialize_tile_vao`) |
|--------------:|-------|------|-------------:|---------------------------------------|
| 0 | `x` | `u16` | 2 | bagian dari atribut 0 (`UNSIGNED_INT`, offset 0) |
| 2 | `y` | `u16` | 2 | bagian dari atribut 0 (`UNSIGNED_INT`, offset 0) |
| 4 | `width` | `u8` | 1 | bagian dari atribut 1 (`UNSIGNED_INT`, offset 4) |
| 5 | `height` | `u8` | 1 | bagian dari atribut 1 (`UNSIGNED_INT`, offset 4) |
| 6 | `_pad` | `[u8; 2]` | 2 | padding eksplisit untuk alignment `backdrop` ke 8 byte |
| 8 | `backdrop` | `[i16; 8]` | 16 | atribut 2 (`SHORT × 4`, offset 8) dan atribut 3 (`SHORT × 4`, offset 16) |
| 24 | `segments` | `[f32; 2]` (di-reinterpret-bit sebagai `UNSIGNED_INT × 2`) | 8 | atribut 4 (`UNSIGNED_INT × 2`, offset 24) |
| 32 | `payload` | `u32` | 4 | bagian dari atribut 5 (`UNSIGNED_INT × 3`, offset 32) |
| 36 | `paint_and_rect_flag` | `u32` | 4 | bagian dari atribut 5 (`UNSIGNED_INT × 3`, offset 32) |
| 40 | `depth_index` | `u32` | 4 | bagian dari atribut 5 (`UNSIGNED_INT × 3`, offset 32) |
| **Total** | | | **44** | stride VAO = `core::mem::size_of::<Tile>() = 44` |

Kedua, **tekstur segmen `RGBA32F`** dengan satu texel per garis. Segmen garis hasil binning diunggah bukan sebagai vertex attribute melainkan sebagai tekstur dua dimensi `RGBA32F`, sehingga tiap fragment dapat melakukan pengambilan acak via `texelFetch`. Satu rekord garis = empat float `(p0.x, p0.y, p1.x, p1.y)` = satu texel RGBA32F (komentar verbatim `src/render/webgl.rs:74`, dikuatkan `src/builder.rs:40-42`). Unggah dilakukan `upload_data_to_rgba32f_texture` (`src/render/webgl.rs:423-446`) dengan `internalformat = RGBA32F`, `format = RGBA`, `type = FLOAT`. Karena koordinat dinyatakan dalam ruang piksel ubin-lokal, rentang komponen $x \in [0, 16]$ dan $y \in [0, 8]$ sesuai dimensi ubin $16 \times 8$.

Ketiga, **vertex buffer instanced 44 byte/tile** yang merupakan salinan tepat `Vec<Tile>` di Video RAM, dipakai mode instancing oleh `WebGlRenderer::render_tiles` melalui `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len())` (`src/render/webgl.rs:393-398`). Tata letaknya dikonfigurasi `initialize_tile_vao` (`src/render/webgl.rs:525`) dengan stride dari `core::mem::size_of::<Tile>()` (`src/render/webgl.rs:529`), dan status per-instance dipasang via `vertexAttribDivisor(_, 1)` untuk seluruh enam slot atribut (`src/render/webgl.rs:534, 539, 544, 549, 554, 559`). Tidak ada vertex buffer kedua berisi geometri quad — empat vertex per ubin diturunkan analitik oleh vertex shader dari field `x`, `y`, `width`, `height`, sehingga total transfer per bingkai dibatasi `tiles.len() × 44` byte ditambah ukuran tekstur segmen. Setelah `WebGlRenderer::render` selesai, alokasi sisi CPU dapat di-`reset` via `Scene::reset` (`src/scene.rs:165-168`) tanpa kehilangan informasi yang relevan untuk bingkai berikutnya.

### 3.3.6 Rasterisasi Halus (GPU)

Tahap rasterisasi halus adalah pekerjaan per-piksel di GPU yang mengubah daftar ubin nontrivial hasil rasterisasi kasar menjadi warna piksel akhir pada framebuffer. Disebut "halus" karena pada tahap inilah cakupan sub-piksel (anti-aliasing) dievaluasi secara analitik dan aturan fill diterapkan. Seluruh ubin nontrivial diproses oleh satu pasang shader yang sama — vertex shader instanced quad dan fragment shader analitik — tanpa percabangan berbasis tipe ubin.

#### 3.3.6.1 Vertex Shader: Instancing Tile

Vertex shader `src/render/shaders/render_tile.vert` dieksekusi satu instance per ubin nontrivial melalui `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, …)`, menghasilkan empat vertex (corner quad) per instance. Atribut per-instance dibaca dari rekord `Tile` 44 byte: `a_xy` (lokasi 0) dibongkar menjadi indeks kolom dan baris ubin (`tile_x_idx = a_xy & 0xFFFFu`, `tile_y_idx = a_xy >> 16u`), sedangkan `a_size` (lokasi 1) menjadi lebar dan tinggi ubin. Posisi tiap corner dihitung dari `gl_VertexID` melalui ekspresi `corner = vec2(float(gl_VertexID & 1), float((gl_VertexID >> 1) & 1))`, lalu dipetakan ke ruang piksel:

$$\text{pixel\_pos} = \bigl(\text{tile\_x\_idx} \cdot \text{TILE\_WIDTH} + \text{corner}_x \cdot \text{tile\_w},\;\; \text{tile\_y\_idx} \cdot \text{TILE\_HEIGHT} + \text{corner}_y \cdot \text{tile\_h}\bigr),$$

dengan `TILE_WIDTH = 16u` dan `TILE_HEIGHT = 8u` yang dideklarasikan `#define` di kepala shader. Posisi piksel kemudian dikonversi ke koordinat klip NDC dengan `ndc = (pixel_pos / u_size) * 2.0 - 1.0`, dengan opsi pembalikan sumbu-y melalui uniform `u_negate_ndc`. Selain `gl_Position`, vertex shader meneruskan atribut yang dibutuhkan fragment shader sebagai variabel `flat`: kedua paruh backdrop (`v_backdrop_lo`, `v_backdrop_hi`), pasangan offset+count segmen (`v_segment`), `v_payload`, `v_paint_flag`, titik asal ubin dalam piksel (`v_tile_origin_pixels`), dan `v_depth_index`, sementara `v_local_xy` diinterpolasi agar fragment menerima posisi piksel-nya. Pendekatan instancing ini membuat geometri quad tidak perlu disimpan eksplisit pada buffer; cukup empat vertex yang diturunkan analitik per ubin.

#### 3.3.6.2 Pengambilan Segmen dari Tekstur

Fragment shader memperoleh segmen garis milik ubinnya dari tekstur `RGBA32F`, bukan dari buffer geometri. Field `v_segment` membawa pasangan `(offset, count)`: `seg_offset = v_segment.x` adalah indeks awal pembacaan dan `seg_count = v_segment.y` adalah jumlah segmen yang harus dibaca. Untuk tiap segmen, fungsi `read_line` (`src/render/shaders/render_tile.frag`) memanggil `texelFetch` pada koordinat yang dihitung `segments_idx_to_coord`, lalu mengurai texel `vec4` menjadi dua titik ujung `p0 = t.xy` dan `p1 = t.zw` dalam koordinat tile-lokal. Pengambilan ini terjadi di dalam loop sekuensial `for (uint s = 0u; s < seg_count; s++)` sehingga setiap fragment hanya membaca segmen yang relevan untuk ubinnya, menjaga biaya per-piksel proporsional terhadap kepadatan geometri lokal.

#### 3.3.6.3 Analytic Coverage (Box Filter)

Inti rasterisasi halus adalah fungsi `line_box` (`src/render/shaders/render_tile.frag:90`), yang oleh blok komentar di atasnya dideskripsikan sebagai *"Analytic line-area contribution (BOX filter, radius 0.5)"*. Fungsi ini mengembalikan fraksi bertanda dari kotak unit-piksel yang terletak "di sebelah kanan" garis dalam rentang-y garis, diskala oleh tanda winding garis, dengan hasil pada interval $[-1, +1]$. Secara matematis `line_box` adalah konvolusi indikator setengah-bidang garis dengan filter kotak $1 \times 1$ yang berpusat pada koordinat piksel, dievaluasi secara trapezoidal: arah-y garis menentukan tanda $\pm 1$; rentang-y di-clip ke kotak piksel $[\text{pixel.y} - 0{,}5,\; \text{pixel.y} + 0{,}5]$; rentang-x sub-segmen ter-clip dirangkum melalui `avg_x = (xc_lo + xc_hi) * 0.5` dan `h_cov = px_hi - avg_x`. Hasil akhirnya adalah produk panjang vertikal sub-segmen ter-clip dengan rerata cakupan horizontal pada kotak piksel. Nilai backdrop per scanline dibaca dari atribut tile dan dikonversi dari 8.8 fixed-point ke float dengan dibagi `WINDING_UNIT = 256.0` (`src/render/shaders/render_tile.frag:24`), menjadi basis akumulator winding kontinu yang lalu ditambah kontribusi `line_box` setiap segmen pada loop sekuensial. Pendekatan box filter analitik ini menghasilkan anti-aliasing tanpa supersampling: cakupan sub-piksel dihitung tepat dari geometri garis, bukan dari pencuplikan berganda.

#### 3.3.6.4 Penerapan Aturan Fill

Setelah seluruh kontribusi segmen terjumlah, winding final $\omega$ dikonversi menjadi koefisien cakupan sesuai aturan fill yang dipilih melalui bit fill rule pada `paint_and_rect_flag` (mask `FILL_RULE_MASK = 0x07000000u`, shift `FILL_RULE_SHIFT = 24u`, `src/render/shaders/render_tile.frag:15-21`). Untuk **Non-Zero** dipakai $\text{coverage} = \mathrm{clamp}(|\omega|, 0, 1)$ dan untuk **Even-Odd** dipakai gelombang segitiga $\text{coverage} = 1 - |\mathrm{mod}(|\omega|, 2) - 1|$, persis seperti dijabarkan pada Subbab 3.3.3.2 dan 3.3.3.3. Bila `coverage <= 0.0`, fragment di-`discard` sehingga piksel kosong tidak menulis ke framebuffer. Cabang biner ini hanya bergantung pada bit fill rule, bukan kategori ubin, sehingga seluruh ubin nontrivial tetap berbagi satu jalur kode.

#### 3.3.6.5 Pewarnaan dan Kompositing

Langkah terakhir mengalikan warna paint dengan koefisien cakupan dan mengeluarkan hasilnya dalam representasi premultiplied alpha. Warna solid dibongkar dari `v_payload` melalui `unpack_rgba8` (Subbab 3.3.4.1), lalu keluaran akhir dihitung sebagai `fragColor = vec4(paint.rgb * paint.a * coverage, paint.a * coverage)`. Keluaran premultiplied ini diserasikan dengan konfigurasi blending `blend_func(ONE, ONE_MINUS_SRC_ALPHA)` pada sisi renderer, sehingga komposisi bidang semi-transparan benar secara matematis; untuk warna solid opak ($\alpha = 1$) hasilnya identik dengan blending konvensional. Dengan demikian satu fragment shader analitik menuntaskan seluruh rantai dari winding, cakupan, fill rule, hingga warna akhir per piksel dalam satu pass tanpa compute shader.

## 3.4 Perancangan Layar

Sistem yang dirancang dan dibangun dalam penelitian ini sepenuhnya berupa pustaka backend perangkat lunak (software library/API) murni yang menyediakan fungsionalitas rendering modular komputasi grafis. Pustaka ini mengekspos fungsi-fungsi programatik untuk dipanggil oleh aplikasi induk, sehingga Arabella tidak memiliki komponen antarmuka grafis pengguna langsung (User Interface / UI Layout) maupun desain cetak layar (mockup) yang menjadi bagian dari kontrak rilis pustaka. Konsekuensinya, "perancangan layar" pada Subbab ini dimaknai sebagai spesifikasi *surface* rendering yang dipakai oleh dua harness pendamping pustaka, yaitu demo interaktif berbasis browser dan harness pengujian otomatis berbasis `wasm-bindgen-test`, bukan sebagai mockup antarmuka pengguna.

**(a) Demo Interaktif `examples/native_webgl/` — Resolusi Window-Fill DPR-Aware**

Demo interaktif yang dipakai untuk pengamatan visual dan eksplorasi performa berada pada *crate* `examples/native_webgl/`. Pada target `wasm32`, ukuran kanvas WebGL ditentukan secara dinamis dari ukuran viewport browser yang dikalikan rasio piksel perangkat (*device pixel ratio*, DPR), bukan dari konstanta resolusi tetap. Saat inisialisasi pada `examples/native_webgl/src/main.rs:22-27`, handle `web_sys::window()` digunakan untuk membaca tiga besaran: `window.device_pixel_ratio()` menghasilkan rasio DPR sebagai `f64`, `window.inner_width()` dan `window.inner_height()` menghasilkan dimensi viewport browser dalam satuan CSS pixel sebagai `f64`. Resolusi kanvas piksel-perangkat kemudian dihitung dengan rumus:

$$
\text{width} = \mathrm{inner\_width} \times \mathrm{devicePixelRatio}
\qquad\text{dan}\qquad
\text{height} = \mathrm{inner\_height} \times \mathrm{devicePixelRatio},
$$

lalu di-cast ke `u16` untuk diteruskan ke `run_interactive(width, height)`. Rumus DPR-aware yang sama digunakan ulang oleh handler resize di `examples/native_webgl/src/lib.rs` agar kanvas tetap memenuhi seluruh viewport ketika ukuran jendela browser berubah. Implikasinya, demo interaktif tidak memiliki resolusi default tunggal; resolusi kanvasnya bersifat *window-fill* dan adaptif terhadap ukuran serta densitas piksel layar perangkat penonton.

**(b) Overlay FPS Demo — Empat Metrik Enumeratif**

Demo interaktif menumpangkan satu *overlay* HTML ringan di atas kanvas WebGL untuk menyajikan diagnostik runtime. String overlay disusun oleh metode `update_overlay` pada `impl AppState` (`examples/native_webgl/src/lib.rs:316-341`), yang dipanggil setiap sepuluh frame dari `AppState::render` agar pembaruan DOM tidak menjadi sumber jitter pengukuran. Selain FPS rerata dan nama *asset* aktif, overlay menampilkan empat metrik enumeratif berikut yang menjadi kontrak desain layar diagnostik:

1. **Waktu pra-pemrosesan CPU per frame (ms)**, bertipe `f64` dalam satuan milidetik, dibaca dari field `self.last_cpu_ms` (`examples/native_webgl/src/lib.rs:335`) yang merekam durasi blok pra-pemrosesan CPU (pemanggilan `Scene::fill` / `Scene::stroke`) lewat selisih `performance.now()`.
2. **Waktu render GPU per frame (ms)**, bertipe `f64` dalam satuan milidetik, dibaca dari field `self.last_gpu_ms` (`examples/native_webgl/src/lib.rs:336`) yang merekam durasi blok rasterisasi GPU (pemanggilan `Renderer::render`) lewat selisih `performance.now()`.
3. **Jumlah operasi paint frame saat ini**, bertipe `usize` non-negatif (bilangan bulat $\ge 0$), dibaca dari `asset.paint_ops.len()` (`examples/native_webgl/src/lib.rs:338`) sebagai indikator kompleksitas adegan SVG yang sedang dirender.
4. **Rasio zoom**, bilangan riil positif yang merepresentasikan skala efektif viewport setelah komposisi pan-zoom, dihitung dari magnitudo kolom pertama matriks transformasi tampilan dengan ekspresi `scale_x = self.view.m11.hypot(self.view.m12)` (`examples/native_webgl/src/lib.rs:325`) dan disubstitusikan ke format string overlay sebagai argumen ke-`{:.2}×` pada `examples/native_webgl/src/lib.rs:337`.

Empat metrik ini secara bersama memungkinkan pemisahan beban pra-pemrosesan CPU dan beban rasterisasi GPU per frame, sekaligus memberi konteks kompleksitas adegan (jumlah operasi paint) dan kondisi tampilan (rasio zoom) yang relevan untuk interpretasi nilai FPS yang ditayangkan.

**(c) Harness `wasm-bindgen-test` — Resolusi Kanvas Tetap 1080 × 520 Piksel**

Berbeda dengan demo interaktif yang adaptif, harness pengujian otomatis berbasis `wasm-bindgen-test` pada `tests/test.rs` memakai resolusi kanvas yang tetap dan deterministik agar hasil rasterisasi piksel dapat dibandingkan secara reproducible lintas eksekusi. Pengujian aktif `test_renders_tiger_svg` (atribut `#[wasm_bindgen_test]` pada `tests/test.rs:147`) mendeklarasikan dua konstanta lokal `const W: u16 = 1080;` (`tests/test.rs:151`) dan `const H: u16 = 520;` (`tests/test.rs:152`), keduanya di-cast ke `u32` saat diteruskan ke `RenderSize { width: W as u32, height: H as u32 }` dan ke `create_canvas(W as u32, H as u32, 1.0)` di tubuh fungsi tes yang sama. Resolusi kanvas pengujian kanonik dengan demikian adalah $1080 \times 520$ piksel pada DPR efektif $1{,}0$, dan menjadi target frame buffer tunggal tempat pustaka mengalirkan hasil rasterisasi piksel akhir dari sirkuit WebGL untuk diverifikasi secara visual maupun programatik.

Tidak terdapat elemen interaksi visual berupa tombol, menu kontrol, slider, maupun teks status di dalam kanvas pengujian guna menjaga netralitas pengukuran *frame time* tanpa interferensi rendering komponen UI pihak ketiga; satu-satunya elemen non-kanvas pada demo interaktif adalah overlay diagnostik `update_overlay` yang dideskripsikan pada poin (b) di atas dan secara sengaja dipisahkan dari jalur pengukuran tes otomatis.
