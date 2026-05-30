# BAB 3 METODE PENELITIAN

## 3.1 Diagram Alir Kerangka Berpikir

Kerangka berpikir dalam penelitian ini dirancang untuk memberikan tahapan yang sistematis dan terarah guna memecahkan masalah dependensi compute shader pada pipeline rendering vektor paralel. Penelitian eksperimental ini dibagi menjadi lima fase utama yang saling berkesinambungan, seperti yang dijabarkan secara tekstual di bawah ini:

1. **Fase 1: Studi Literatur dan Pengumpulan Data**
   - Melakukan analisis mendalam terhadap literatur grafis komputer terkait, mencakup metode shortcut tree, teknik rasterisasi scanline berbasis boundary fragments, fungsi implisit Loop-Blinn, serta arsitektur mesin rendering compute-centric modern seperti Vello.
   - Mengumpulkan berkas sampel uji dalam format Scalable Vector Graphics (SVG) dengan variasi tingkat kompleksitas geometri jalur primitif.

2. **Fase 2: Analisis Kebutuhan Sistem**
   - Mengidentifikasi keterbatasan lingkungan grafis non-compute pada perangkat low-end.
   - Merumuskan spesifikasi fungsional pustaka (library) rendering berbasis pemrograman sistem Rust dan API grafis WebGL 2.0.

3. **Fase 3: Perancangan Arsitektur dan Algoritma**
   - Merancang pembagian beban kerja hibrida: tahap preprocessing paralel masif pada Central Processing Unit (CPU) dan tahap rasterisasi pada Graphics Processing Unit (GPU).
   - Menyusun struktur data spasial berbasis tiling (ubin) serta merancang mekanisme kalkulasi winding number melalui binning DDA dua tahap, akumulator signed-area per scanline, dan propagasi backdrop kiri-ke-kanan.
   - Memformulasikan pemetaan kurva Bézier kuadratik dan kubik ke dalam fungsi implisit kanonik.

4. **Fase 4: Implementasi Purwarupa (Prototype)**
   - Membangun modul parser SVG dan arsitektur data memori di CPU menggunakan Rust.
   - Menyusun struktur data spasial berbasis tile yang dirancang agar pemrosesan antar-jalur saling independen, sehingga membuka jalan bagi paralelisasi tingkat data melalui pustaka Rayon pada pengembangan lanjutan; pada implementasi yang dievaluasi, optimasi paralelisme yang sudah aktif adalah paralelisme tingkat instruksi melalui SIMD pada hot path transformasi dan flattening, sedangkan feature `multithreading` berbasis Rayon masih bersifat opsional dan belum diaktifkan pada build baku.
   - Mengembangkan pemrosesan Vertex Shader dan Fragment Shader konvensional pada GPU menggunakan WebGL 2.0.

5. **Fase 5: Pengujian dan Evaluasi**
   - Melakukan validasi kebenaran output visual (correctness validation) dengan membandingkan citra hasil purwarupa terhadap renderer referensi peramban.
   - Melakukan pengujian performa (benchmarking) untuk mengukur metrik frame time per bingkai yang didekomposisi menjadi biaya tahap pra-pemrosesan di CPU dan biaya tahap rasterisasi di GPU secara terpisah.
   - Menganalisis secara kualitatif posisi arsitektur pustaka yang diusulkan terhadap mesin rendering berbasis CPU murni (Skia/Cairo) serta berbasis GPU komputasi penuh (Vello) pada dimensi paradigma rasterisasi, ketergantungan compute shader, dan target platform.

## 3.2 Analisis Kebutuhan

### 3.2.1 Analisis User

Sistem yang dikembangkan dalam penelitian ini merupakan sebuah pustaka antarmuka pemrograman aplikasi (Application Programming Interface / API) rendering grafis 2D. Oleh karena itu, pengguna langsung (user) dari sistem ini adalah pengembang perangkat lunak (software developer) yang membutuhkan kapabilitas eksekusi grafis vektor performa tinggi untuk diintegrasikan ke dalam aplikasi akhir mereka, seperti game engine, browser web emulasi, atau aplikasi seluler.

Pengguna sistem ini diwajibkan memiliki pemahaman dasar mengenai matematika komputasi grafis (seperti koordinat kartesian, vektor, dan kurva parametrik), alur kerja pipeline grafis tradisional (konsep shader, vertex buffer, dan framebuffer), serta memiliki pengalaman dalam implementasi kode menggunakan bahasa pemrograman sistem. Interaksi pengembang dengan pustaka dilakukan sepenuhnya secara terprogram (programmatic) melalui pemanggilan fungsi-fungsi pustaka dan penyerahan deskripsi geometri berbasis teks atau biner (seperti data jalur SVG), tanpa melibatkan interaksi komponen antarmuka grafis pengguna akhir (Graphical User Interface).

### 3.2.2 Analisis Aplikasi Sejenis

Untuk memvalidasi urgensi pengembangan sistem, dilakukan analisis komparatif terhadap tiga arsitektur rendering grafis yang menjadi acuan utama dalam penelitian ini:

1. **Vello (Linebender)** — Merupakan mesin rendering modern yang mengadopsi paradigma GPU compute-centric. Pustaka ini memindahkan seluruh komputasi sekuensial yang berat — seperti tessellation, pemotongan geometri (clipping), dan alokasi memori spasial — langsung ke GPU menggunakan serangkaian compute shader dispatches. Akselerasi paralelnya memanfaatkan algoritma parallel prefix-sum guna menurunkan kompleksitas serial $O(n)$ menjadi tugas paralel $O(\log n)$. Walaupun menghasilkan throughput masif pada perangkat high-end, Vello memerlukan dukungan WebGPU API atau Vulkan modern, sehingga tidak dapat beroperasi secara stabil pada segmen perangkat keras legacy atau low-end kelas konsumen.

2. **Massively-Parallel Vector Graphics (Ganacim dkk., 2014)** — Sistem ini memparalelkan tahap preprocessing segmen geometri masukan dan tahap rendering sampel piksel secara simultan di GPU. Komponen intinya memanfaatkan struktur data spasial hierarkis adaptif bernama Shortcut Tree (berbasis quadtree) untuk memberikan akses acak cepat terhadap nilai warna piksel. Namun, implementasi algoritma ini sangat bergantung pada arsitektur komputasi umum GPU yang spesifik (seperti teknologi NVIDIA CUDA) untuk menangani warping dan penjadwalan sampel, yang secara drastis membatasi portabilitas lintas platform.

3. **Efficient GPU Path Rendering Using Scanline Rasterization (Li dkk., 2016)** — Pendekatan ini mengadaptasi algoritma scanline rasterizer klasik agar berjalan paralel di atas GPU. Logika utamanya memisahkan pemrosesan antara piksel perbatasan jalur (boundary fragments berukuran $2 \times 2$ piksel) dan piksel bagian dalam (horizontal spans). Pendekatan ini meminimalkan biaya komputasi winding number global dengan melokalisasi komputasi cakupan anti-aliasing. Walaupun efisien, sistem ini tetap mengandalkan arsitektur compute pipeline untuk fase pengurutan (sorting) dan penggabungan (merging) fragmen sebelum tahap rasterisasi akhir dilakukan.

### 3.2.3 Rumusan dan Solusi Kebutuhan

Berdasarkan analisis kesenjangan (gap analysis) terhadap aplikasi sejenis, berikut adalah tabel pemetaan rumusan masalah dan solusi teknis yang diimplementasikan di dalam pustaka ini:

| No | Rumusan Masalah Keperluan Sistem | Solusi Teknis Terimplementasi |
|----|----------------------------------|-------------------------------|
| 1 | Kebutuhan akan metode rendering grafis vektor paralel secara masif yang tidak bergantung pada fitur compute shader. | Merancang pipeline hibrida yang membagi tugas secara tegas: mengeksekusi tahapan preprocessing spasial secara paralel masif di CPU, dan mengalokasikan rasterisasi piksel ke GPU. |
| 2 | Kebutuhan akan jaminan kompatibilitas platform yang luas, terutama pada lingkungan grafis terbatas (web lama dan perangkat low-end). | Membatasi implementasi GPU secara ketat hanya pada rasterization pipeline tradisional menggunakan Vertex Shader dan Fragment Shader standar WebGL 2.0. |
| 3 | Kebutuhan akan efisiensi performa tinggi dan minimalisasi overdraw komputasi pada skenario adegan (scene) vektor yang kompleks. | Menerapkan segmentasi layar berbasis tiling (ubin) berukuran tetap di CPU melalui pipeline tiga tahap: binning DDA dua tahap (outer DDA lintas baris ubin dan inner DDA lintas kolom ubin) untuk memecah segmen garis ke ubin yang dilintasinya, akumulator signed-area per scanline untuk menghitung winding number secara inkremental, dan propagasi backdrop kiri-ke-kanan saat emisi ubin, sehingga hanya ubin nontrivial yang dikirim ke memori GPU. |

## 3.3 Perancangan Aplikasi

Subbab ini memuat spesifikasi teknis pustaka Arabella sebagai bagian dari perancangan aplikasi pipeline hibrida. Sebelum spesifikasi diuraikan, perlu didefinisikan istilah "klaim teknis" yang akan dipakai secara konsisten pada Bab 3 ini. Klaim teknis adalah pernyataan terverifikasi mengenai implementasi aktual pustaka Arabella yang menyebut salah satu dari: (a) nama algoritma atau struktur data, (b) nilai parameter numerik konkret (termasuk dimensi ubin, format fixed-point, atau jumlah byte), (c) nama berkas, fungsi, struct, trait, modul, atau konstanta dalam kode, atau (d) perilaku runtime spesifik dari pustaka. Setiap klaim teknis pada Subbab 3.3 sampai 3.7 wajib dapat ditelusuri langsung ke source code melalui rujukan kode berformat `berkas:simbol` atau `berkas:start-end` relatif terhadap akar repositori, sehingga setiap pernyataan dapat divalidasi pembaca dengan membuka berkas yang dirujuk pada `src/`, `Cargo.toml`, `examples/`, atau `tests/`.

### 3.3.1 Spesifikasi Aplikasi

Pustaka rendering vektor paralel Arabella dirancang sebagai crate Rust yang menargetkan lingkungan peramban web melalui WebAssembly, dengan dependensi grafis yang dikunci pada WebGL 2.0 sebagai target langsung. Spesifikasi komponen penyusun pustaka, beserta rujukan kode pada `Cargo.toml` (akar repositori) dan `src/`, dijabarkan sebagai berikut.

**Bahasa Pemrograman dan Edisi Rust.** Bahasa pemrograman utama Arabella adalah Rust edisi 2024, sebagaimana dideklarasikan oleh `edition = "2024"` pada blok `[package]` (`Cargo.toml:7`). Pemilihan edisi 2024 didasarkan pada dukungan stabil terhadap fitur ergonomi modern, zero-cost abstractions yang menjamin kecepatan setara C/C++, jaminan memory safety tanpa garbage collector, serta paradigma fearless concurrency yang krusial untuk paralelisme CPU.

**Target Eksekusi dan API Grafis.** Target eksekusi utama Arabella adalah `wasm32-unknown-unknown` pada lingkungan peramban dengan API grafis WebGL 2.0 sebagai target langsung tanpa lapisan transpilasi tambahan. Konfigurasi target wasm dideklarasikan pada blok `[target.'cfg(target_arch = "wasm32")'.dependencies]` (`Cargo.toml:57-87`) yang menarik fitur `WebGl2RenderingContext` dari crate `web-sys` untuk akses langsung ke konteks WebGL 2.0. Target dokumentasi `wasm32-unknown-unknown` juga dieksplisitkan pada `[package.metadata.docs.rs]` melalui `targets = ["wasm32-unknown-unknown"]` (`Cargo.toml:26`). Tidak ada penarikan API WebGL 1.0 (`WebGlRenderingContext` tanpa angka 2) maupun WebGPU (`WebGpu*`) pada blok target wasm tersebut.

**Dependensi Langsung.** Pustaka Arabella mendeklarasikan tiga belas crate sebagai dependensi langsung pada blok `[dependencies]` (`Cargo.toml:29-42`), dengan ejaan dan versi persis seperti tertulis pada manifest (diurutkan menurut posisi baris pada manifest):

1. `bytemuck` versi `"1.25.0"` dengan `features = ["derive", "extern_crate_alloc"]` (`Cargo.toml:30`) — utilitas reinterpretasi tipe POD untuk menyiapkan vertex buffer dan tekstur GPU.
2. `fearless_simd` versi `"0.4.0"` (`Cargo.toml:31`) — abstraksi SIMD portabel yang dipakai untuk akselerasi paralelisme tingkat instruksi pada hot path pra-pemrosesan CPU.
3. `png` versi `"0.18.1"`, ditandai `optional = true` (`Cargo.toml:32`) — encoder PNG yang ditarik oleh feature `png` (termasuk dalam `default`) untuk menyimpan keluaran kanvas sebagai berkas citra pada harness pengujian dan benchmark.
4. `hashbrown` versi `"0.17.0"` (`Cargo.toml:33`) — implementasi `HashMap` performansi tinggi.
5. `smallvec` versi `"1.15.1"` (`Cargo.toml:34`) — kontainer vektor dengan kapasitas inline yang dipakai untuk menampung koleksi kecil pada hot path tanpa alokasi heap.
6. `thiserror` versi `"2.0.18"` (`Cargo.toml:35`) — derive macro untuk tipe error idiomatik pada antarmuka publik.
7. `log` versi `"0.4.29"` (`Cargo.toml:36`) — fasad logging ringan yang dipakai untuk mencatat pesan diagnostik selama eksekusi.
8. `lyon_path` versi `"1.0.19"` (`Cargo.toml:37`) — representasi `Path` 2D yang dipakai untuk konstruksi geometri vektor.
9. `lyon_geom` versi `"1.0.19"` (`Cargo.toml:38`) — operasi geometri 2D pada segmen garis dan kurva Bézier.
10. `lyon_algorithms` versi `"1.0.20"` (`Cargo.toml:39`) — kumpulan algoritma jalur 2D (path algorithms) pelengkap `lyon_path` dan `lyon_geom`.
11. `peniko` versi `"0.6.1"` dengan `default-features = false` dan `features = ["libm"]` (`Cargo.toml:40`) — primitif paint dan warna (`AlphaColor`, `Srgb`) yang dipakai parser SVG dan jalur paint.
12. `kurbo` versi `"0.13.0"` (`Cargo.toml:41`) — pustaka geometri kurva 2D yang menyediakan tipe `BezPath`, `Affine`, dan `Point` untuk representasi internal jalur.
13. `roxmltree` versi `"0.20.0"` (`Cargo.toml:42`) — parser XML read-only yang dipakai oleh `src/pico_svg.rs` untuk membaca dokumen SVG.

Di antara ketiga belas crate tersebut, hanya `png` yang ditandai `optional` namun tetap aktif pada build baku karena ditarik oleh feature `png` yang termasuk dalam `default = ["std", "png"]`; dua belas crate lainnya bersifat wajib tanpa gerbang feature.

**Pustaka Konkurensi CPU (Opsional).** Crate `rayon` versi `"1.11.0"` dideklarasikan sebagai dependensi opsional pada blok `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` (`Cargo.toml:48-50`) bersama crate `thread_local` versi `"1.1.9"`, keduanya ditandai `optional = true`. Kedua crate tersebut hanya ditarik ketika feature flag `multithreading = ["std", "dep:rayon", "dep:thread_local"]` (`Cargo.toml:94`) diaktifkan secara opt-in. Karena `multithreading` tidak termasuk dalam feature `default = ["std", "png"]`, build standar tidak menyertakan Rayon. Pada implementasi yang dievaluasi dalam penelitian ini, feature `multithreading` belum dipanggil pada hot path pra-pemrosesan CPU; dengan demikian, klaim paralelisme CPU yang dijabarkan pada Subbab 3.5 bersifat potensial — yaitu kapasitas yang sudah disiapkan pada manifest dependensi — bukan paralelisme yang sudah aktif pada implementasi saat ini.

**Format Data Geometri Masukan.** Arabella menerima dokumen SVG sebagai format data geometri masukan melalui modul `src/pico_svg.rs`. Parser ini bukan implementasi SVG 1.1 Core lengkap, melainkan subset minimal yang hanya menangani dua nama elemen secara eksplisit pada dispatch tag-name di `Parser::rec_parse` (`src/pico_svg.rs:191`): elemen grup `g` dan elemen jalur `path`. Atribut presentation yang diparse terbatas pada `fill`, `stroke`, `stroke-width`, dan — khusus pada elemen `g` — `transform`. Setiap elemen di luar dua nama tersebut jatuh ke arm fallback `other => eprintln!("Unhandled node type {other}")` (`src/pico_svg.rs:228`) yang hanya mencetak peringatan ke stderr tanpa pemrosesan lebih lanjut. Konsekuensinya, fitur SVG yang umum seperti elemen teks (`text`, `tspan`), blok definisi terbagi (`defs`), gradient (linear/radial), pattern, filter, clipPath/mask, bentuk dasar non-`path` (`rect`, `circle`, `ellipse`, `polyline`, `polygon`), serta penyisipan raster (`image`/`symbol`/`use`) tidak ditangani oleh Arabella. Pemakai pustaka memodelkan setiap geometri sebagai elemen `<path d="…">` agar dapat diproses oleh pipeline rasterisasi.

## 3.4 Perancangan Sistem

### 3.4.1 Use Case Diagram

Sistem ini memodelkan interaksi fungsional antara aktor tunggal (Developer) dengan modul eksternal sistem melalui batasan sistem pustaka (Vector Rendering Library). Aktor Developer mengendalikan daur hidup rendering melalui tiga komponen fungsional utama (use case):

1. **Inisialisasi Context (UC-01):** Menyediakan titik awal untuk alokasi memori subsistem grafis.
2. **Input Data Vektor (UC-02):** Berperan sebagai jembatan penyerahan berkas geometri dari aplikasi utama ke memori internal pustaka.
3. **Render Frame (UC-03):** Use case utama yang memicu fungsi loop rendering per frame hibrida, di mana use case ini secara otomatis mengikutsertakan (include) sub-proses pra-pemrosesan CPU (binning DDA + akumulator signed-area + propagasi backdrop) dan sub-proses rasterisasi GPU (vertex shader instanced quad + fragment shader analitik).

### 3.4.2 Use Case Description

Berikut adalah spesifikasi naratif terperinci untuk masing-masing use case yang terintegrasi di dalam sistem:

**Tabel UC-01: Deskripsi Use Case Inisialisasi Context**

| Komponen Deskripsi | Spesifikasi Fungsional |
|---|---|
| Nama Use Case | Inisialisasi Context |
| ID Use Case | UC-01 |
| Aktor Utama | Developer |
| Deskripsi Singkat | Proses penyiapan konteks grafis WebGL 2.0, kompilasi program shader konvensional, serta alokasi awal objek memori pada pustaka. |
| Kondisi Awal | Aplikasi pengembang telah berjalan sukses dan memiliki referensi permukaan jendela grafis (window context) aktif. |
| Kondisi Akhir | Pustaka siap menerima data geometri, program shader telah terkompilasi, dan lokasi memori GPU telah teralokasi. |
| Alur Peristiwa Inti (Basic Flow) | 1. Developer memanggil fungsi inisialisasi API pustaka. 2. Sistem memuat konfigurasi grafis dasar berbasis aturan WebGL 2.0. 3. Sistem memuat, mengompilasi, dan menautkan (linking) program Vertex Shader dan Fragment Shader konvensional. 4. Sistem mengembalikan kode status "Context Ready" kepada Developer. |

**Tabel UC-02: Deskripsi Use Case Input Data Vektor**

| Komponen Deskripsi | Spesifikasi Fungsional |
|---|---|
| Nama Use Case | Input Data Vektor |
| ID Use Case | UC-02 |
| Aktor Utama | Developer |
| Deskripsi Singkat | Developer memasukkan perintah rendering geometri atau memuat berkas SVG eksternal berisi deskripsi lintasan kurva dan atribut warna fill/stroke ke memori pustaka. |
| Kondisi Awal | Use case UC-01 telah berhasil dieksekusi secara sempurna. |
| Kondisi Akhir | Data lintasan geometri dikonversi menjadi representasi array primitif terstruktur dalam memori lokal CPU. |
| Alur Peristiwa Inti (Basic Flow) | 1. Developer memanggil fungsi penyerahan data path atau memuat berkas SVG. 2. Sistem mem-parsing berkas masukan menjadi elemen kurva dasar linear, kuadratik, dan kubik. 3. Sistem menyimpan data geometri primitif beserta informasi warna (fill color) ke objek koleksi lintasan (Scene struct) di memori utama. |

**Tabel UC-03: Deskripsi Use Case Render Frame**

| Komponen Deskripsi | Spesifikasi Fungsional |
|---|---|
| Nama Use Case | Render Frame |
| ID Use Case | UC-03 |
| Aktor Utama | Developer |
| Deskripsi Singkat | Proses menggambar seluruh data geometri vektor ke layar per bingkai menggunakan interaksi pipeline hibrida CPU-GPU paralel tanpa compute shader. |
| Kondisi Awal | Data geometri vektor telah tersimpan di memori (UC-02 sukses) dan fungsi render dipanggil secara berkala di loop aplikasi. |
| Kondisi Akhir | Hasil visual grafik vektor ter-rasterisasi secara presisi pada viewport target. |
| Alur Peristiwa Inti (Basic Flow) | **A. Pemicu (Trigger):** 1. Developer memanggil `Scene::fill` atau `Scene::stroke` untuk menyerahkan jalur, lalu memicu fungsi `render()` pada `WebGlRenderer` dari loop aplikasi utama. **B. Tahap Pra-Pemrosesan (CPU):** **(a)** Pustaka melakukan flattening setiap kurva Bézier menjadi rangkaian segmen garis pada format F24Dot8 (24.8 fixed-point) melalui jalur kubik ke kuadratik di `src/path.rs` dan kuadratik ke segmen garis via De Casteljau midpoint subdivision di `src/flatten.rs`. **(b)** Pustaka menjalankan outer DDA yang membagi setiap segmen garis lintas baris ubin berukuran 16×8 piksel (rujuk `src/blocks.rs:bin_line` dan `src/blocks.rs:Blocks::build_block`). **(c)** Pustaka menjalankan inner DDA yang membagi setiap potongan baris lintas kolom ubin pada baris yang sama. **(d)** Untuk setiap ubin yang disentuh, pustaka mengakumulasi signed-area per scanline pada format 8.8 fixed-point (delapan akumulator 16-bit per ubin, satu per scanline) melalui `src/blocks.rs:record_per_scanline_crossings`. **(e)** Pustaka mengemisi seluruh ubin nontrivial — yaitu ubin yang disentuh DDA atau yang akumulator winding-nya belum nol akibat propagasi dari kiri — sebagai rekord vertex buffer instanced beserta segmen tile-lokal pada tekstur RGBA32F. **(f)** Saat emisi berlangsung pada setiap baris ubin, pustaka mempropagasikan backdrop kiri-ke-kanan sehingga ubin di sebelah kanan menerima jumlah kumulatif kontribusi winding dari seluruh ubin sebelumnya pada baris yang sama (rujuk `Builder::generate_tiles` di `src/builder.rs`). **C. Tahap Rasterisasi (GPU):** **(i)** Vertex shader instanced quad mengeksekusi satu rekord per ubin nontrivial untuk memetakan empat corner quad ke koordinat klip NDC dan meneruskan atribut backdrop, offset segmen, serta paint flag ke fragment shader. **(ii)** Fragment shader analitik tunggal yang sama dijalankan untuk seluruh ubin nontrivial melalui satu jalur kode tanpa percabangan kondisional berbasis tipe ubin: shader membaca backdrop per scanline dari rekord ubin, menjumlahkan kontribusi `line_box` (integral trapezoidal cakupan piksel) untuk setiap segmen yang dibinning ke ubin tersebut, lalu menerapkan fill rule NonZero (`coverage = clamp(abs(winding), 0, 1)`) atau EvenOdd (`coverage = 1 - abs(mod(abs(winding), 2) - 1)`) sesuai bit fill rule pada paint flag, sebagaimana didefinisikan di `src/render/shaders/render_tile.frag`. 9. Hasil komposit warna akhir dengan premultiplied alpha dialirkan ke objek framebuffer layar. |

### 3.4.3 Sequence Diagram

Sequence diagram menggambarkan interaksi sekuensial berbasis pesan (message-passing) antar lima partisipan — **Aplikasi Utama**, **Scene**, **Builder**, **WebGlRenderer**, dan **GPU** — sepanjang siklus hidup satu bingkai rendering (single frame lifetime). Karena pipeline hibrida Arabella memproses seluruh ubin nontrivial melalui satu jalur kode tunggal tanpa percabangan kondisional berbasis tipe ubin, sequence diagram disusun sebagai lima pesan berurutan tanpa blok alternatif (`alt`), opsional (`opt`), maupun perulangan (`loop`) yang bercabang berdasarkan klasifikasi ubin.

**1. Aplikasi Utama → Scene: penyerahan jalur geometri.**
Aplikasi Utama memanggil `Scene::fill(path, fill_rule, transform, brush)` untuk jalur isian (rujuk `src/scene.rs:70`) atau `Scene::stroke(path, style, transform, brush)` untuk jalur garis (rujuk `src/scene.rs:117`). Pada jalur stroke, outline hasil ekspansi `kurbo::stroke_with` dialirkan kembali ke `Scene::fill` dengan `FillRule::NonZero` (rujuk `src/scene.rs:158`), sehingga kedua API menyatu pada pipeline isian yang sama.

**2. Scene → Builder: flattening dan binning DDA.**
`Scene::fill` mendelegasikan tahap pertama pra-pemrosesan kepada `Builder::build_path(path, fill_rule, transform)` (rujuk `src/builder.rs:84`). Pada pesan ini Builder melakukan flattening kurva Bézier menjadi segmen garis pada format F24Dot8 (rujuk `src/path.rs` dan `src/flatten.rs`), kemudian membinning setiap segmen ke dalam ubin 16×8 piksel melalui outer DDA dan inner DDA (rujuk `src/blocks.rs:bin_line` dan `src/blocks.rs:Blocks::build_block`) sambil mengakumulasi signed-area per scanline pada format 8.8 fixed-point (rujuk `src/blocks.rs:record_per_scanline_crossings`).

**3. Scene → Builder: propagasi backdrop dan emisi ubin nontrivial.**
`Scene::fill` melanjutkan dengan memanggil `Builder::generate_tiles(paint_index, fill_rule, payload, paint_flag)` (rujuk `src/builder.rs:151` dan situs panggilan di `src/scene.rs:93`). Pada pesan ini Builder mempropagasikan backdrop kiri-ke-kanan per baris ubin sehingga setiap ubin di sebelah kanan menerima jumlah kumulatif kontribusi winding dari seluruh ubin sebelumnya pada baris yang sama, lalu mengemisi seluruh ubin nontrivial ke dalam vektor `Vec<Tile>` melalui satu jalur kode tunggal tanpa percabangan tipe ubin.

**4. Aplikasi Utama → WebGlRenderer: penyerahan vertex buffer dan tekstur segmen.**
Aplikasi Utama memicu `WebGlRenderer::render(&scene, &render_size)` (rujuk `src/render/webgl.rs:296`). WebGlRenderer membaca vertex buffer instanced melalui `scene.tiles()` (rujuk `src/scene.rs:155`) dan tekstur segmen RGBA32F melalui `scene.segments()` (rujuk `src/scene.rs:160`), lalu mengunggah keduanya ke GPU bersama uniform konfigurasi.

**5. WebGlRenderer → GPU: vertex shader instanced quad lalu fragment shader analitik tunggal.**
WebGlRenderer memanggil `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len())` (rujuk `src/render/webgl.rs:393-398`) sehingga vertex shader instanced quad mengeksekusi satu instance per ubin nontrivial untuk memetakan empat corner quad ke koordinat klip NDC. GPU kemudian mengeksekusi fragment shader analitik tunggal yang sama untuk seluruh ubin nontrivial pada satu jalur kode tanpa percabangan kondisional berbasis tipe ubin (rujuk `src/render/shaders/render_tile.frag`); fragment shader menjumlahkan kontribusi `line_box` (integral trapezoidal cakupan piksel) untuk setiap segmen yang dibinning ke ubin tersebut, menggabungkannya dengan backdrop per scanline, lalu menerapkan fill rule NonZero atau EvenOdd sesuai bit fill rule pada paint flag. Hasil komposit warna akhir dengan premultiplied alpha dialirkan kembali oleh GPU ke framebuffer layar yang dipegang Aplikasi Utama, mengakhiri siklus hidup bingkai.

### 3.4.4 Class Diagram

Arsitektur perangkat lunak pustaka Arabella diorganisasikan sebagai sembilan struct Rust yang saling terkait, masing-masing direpresentasikan sebagai kotak kelas UML berikut. Setiap kelas mendaftarkan field utama beserta tipenya persis seperti dideklarasikan pada source code, sehingga setiap atribut pada diagram dapat ditelusuri kembali ke berkas Rust yang dirujuk.

**1. `Scene` — fasad API publik untuk penyerahan jalur geometri** (`src/scene.rs:35-41`).

| Field | Tipe |
|---|---|
| `width` | `u16` |
| `height` | `u16` |
| `builder` | `Builder` |
| `paint_index_counter` | `u32` |

Method publik utama: `Scene::fill(path, fill_rule, transform, brush)` (`src/scene.rs:70`), `Scene::stroke(path, style, transform, brush)` (`src/scene.rs:117`), `Scene::tiles() -> &[Tile]` (`src/scene.rs:155`), `Scene::segments() -> &[f32]` (`src/scene.rs:160`), dan `Scene::reset()` (`src/scene.rs:165`).

**2. `Builder` — pengelola pipeline pra-pemrosesan CPU** (`src/builder.rs:36-54`).

| Field | Tipe |
|---|---|
| `tiles` | `TileMap<Tile>` |
| `segments` | `Vec<f32>` |
| `line_buf` | `Vec<i32>` |
| `blocks` | `Blocks` |
| `covers` | `RefCell<CoverStorage>` |
| `bbox` | `Box2D<f32>` |
| `level` | `Level` |
| `shape_index` | `u32` |

Method publik utama: `Builder::build_path(path, fill_rule, transform)` (`src/builder.rs:84`) untuk flattening dan binning DDA, serta `Builder::generate_tiles(paint_index, fill_rule, payload, paint_flag)` (`src/builder.rs:151`) untuk propagasi backdrop kiri-ke-kanan dan emisi seluruh ubin nontrivial.

**3. `CoverStorage` — akumulator winding 8.8 fixed-point per scanline dan tag cell yang disentuh DDA** (`src/builder.rs:360-369`).

| Field | Tipe |
|---|---|
| `tag` | `Vec<u32>` |
| `backdrops` | `Vec<[i16; TILE_H]>` |
| `col_count` | `usize` |
| `row_count` | `usize` |

Field `tag` adalah bit-vektor packed (satu bit per cell) yang menandai cell yang disentuh DDA, sedangkan `backdrops` menyimpan delapan akumulator i16 per cell pada format 8.8 fixed-point (1 winding penuh = 256 unit) yang diisi oleh `record_per_scanline_crossings` di `src/blocks.rs:710`.

**4. `Block` — rekord per-pasangan (segmen, ubin) hasil binning DDA** (`src/blocks.rs:21-39`).

| Field | Tipe |
|---|---|
| `p0x` | `i32` |
| `p0y` | `i32` |
| `p1x` | `i32` |
| `p1y` | `i32` |
| `x` | `u16` |
| `y` | `u16` |

Endpoint `(p0x, p0y)` dan `(p1x, p1y)` disimpan dalam koordinat ubin-lokal pada format F24Dot8 (rentang `[0, TILE_W * 256]` untuk x dan `[0, TILE_H * 256]` untuk y), sedangkan `(x, y)` adalah indeks kolom dan baris ubin global.

**5. `Blocks` — kontainer pengakumulasi `Block` plus driver outer/inner DDA** (`src/blocks.rs:51-55`).

| Field | Tipe |
|---|---|
| `data` | `Vec<Block>` |
| `sorted` | `bool` |

Method publik utama: `Blocks::build_block(covers, bounds, p0x, p0y, p1x, p1y)` (`src/blocks.rs:93`) sebagai entry point binning satu segmen, `Blocks::bin_line(...)` (`src/blocks.rs:107`) yang menjalankan outer DDA empat arah diagonal, serta `Blocks::sort_blocks()` (`src/blocks.rs:77`) yang mengurutkan rekord menurut `(y, x)` agar lookup linier per ubin dapat dilakukan via binary search.

**6. `TileBounds` — batas ruang-ubin pra-komputasi per shape** (`src/blocks.rs:664-669`).

| Field | Tipe |
|---|---|
| `min_col` | `i32` |
| `min_row` | `i32` |
| `col_count` | `usize` |
| `row_count` | `usize` |

Konstruktor `TileBounds::from_box2d(bounds)` (`src/blocks.rs:671`) mengubah `Box2D<f32>` (piksel) menjadi batas grid ubin via operasi `floor` pada minimum dan `ceil` pada maksimum dengan pembagi `TILE_W = 16` dan `TILE_H = 8`.

**7. `Tile` — rekord vertex buffer instanced 44 byte yang diunggah ke GPU** (`src/tile.rs:9-23`).

| Field | Tipe |
|---|---|
| `x` | `u16` |
| `y` | `u16` |
| `width` | `u8` |
| `height` | `u8` |
| `_pad` | `[u8; 2]` |
| `backdrop` | `[i16; 8]` |
| `segments` | `[f32; 2]` |
| `payload` | `u32` |
| `paint_and_rect_flag` | `u32` |
| `depth_index` | `u32` |

Atribut `#[repr(C)]` mengunci urutan field ke ABI C sehingga total ukuran tepat 44 byte. Field `backdrop` adalah array delapan elemen 16-bit yang membawa hasil propagasi backdrop untuk delapan scanline pada satu ubin (selaras dengan `TILE_H = 8`). Field `segments` adalah pasangan dua elemen yang merepresentasikan offset segmen dan jumlah segmen pada tekstur RGBA32F (offset dan count integer disandikan via reinterpret-bit dengan `f32::from_bits`).

**8. `WebGlRenderer` — pengelola konteks WebGL 2.0 dan eksekusi pipeline GPU** (`src/render/webgl.rs:265-269`).

| Field | Tipe |
|---|---|
| `programs` | `WebGlPrograms` |
| `gl` | `WebGl2RenderingContext` |

Method publik utama: `WebGlRenderer::new(canvas: &HtmlCanvasElement) -> Self` (`src/render/webgl.rs:271`) yang menginisialisasi konteks WebGL 2.0 dan memanggil `initialize_tile_vao` (`src/render/webgl.rs:525`) untuk mengonfigurasi vertex divisor 44 byte per ubin, serta `WebGlRenderer::render(&mut self, scene: &Scene, render_size: &RenderSize)` (`src/render/webgl.rs:296`) yang mengupload tekstur segmen RGBA32F dari `Scene::segments()`, mengupload vertex buffer instanced dari `Scene::tiles()`, lalu memicu `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len())` (`src/render/webgl.rs:393-398`).

**9. `PicoSvg` — parser SVG subset minimal** (`src/pico_svg.rs:28-34`).

| Field | Tipe |
|---|---|
| `items` | `Vec<Item>` |
| `size` | `Size` |

Method publik utama: `PicoSvg::load(xml_string: &str, scale: f64) -> Result<Self, Box<dyn core::error::Error>>` (`src/pico_svg.rs:84`). Parser hanya menangani elemen `<g>` dan `<path>` melalui dispatch tag-name pada `Parser::rec_parse` (`src/pico_svg.rs:191`); elemen lain jatuh ke arm fallback yang hanya mencetak peringatan ke stderr. Enum `Item` terdiri atas tiga varian — `Fill(FillItem)`, `Stroke(StrokeItem)`, dan `Group(GroupItem)` (`src/pico_svg.rs:36-44`) — yang masing-masing membawa `BezPath` dari `kurbo` beserta atribut warna, lebar stroke, atau transformasi affine grup.

**Relasi UML antar kelas.** Hubungan antar kotak kelas berikut diturunkan langsung dari deklarasi field pada source code, sehingga setiap relasi dapat diverifikasi dengan membuka berkas yang dirujuk.

1. **`Scene` ◆── `Builder` (komposisi).** `Scene` mengkomposisi tepat satu instance `Builder` melalui field `builder: Builder` pada `src/scene.rs:38`. Daur hidup `Builder` terikat penuh pada daur hidup `Scene`: `Scene::new` menginisialisasinya via `Builder::new(width, height, settings.level)` (`src/scene.rs:48`), dan `Scene::reset` mendelegasikan ke `Builder::reset` (`src/scene.rs:166`).
2. **`Builder` ◆── `Blocks` (komposisi).** `Builder` mengkomposisi tepat satu `Blocks` melalui field `blocks: Blocks` pada `src/builder.rs:47`. `Builder::build_path` memanggil `self.blocks.build_block(...)` per segmen flattened (`src/builder.rs:147`), dan `Builder::generate_tiles` memanggil `self.blocks.sort_blocks()` (`src/builder.rs:171`) sebelum loop emisi.
3. **`Builder` ◆── `CoverStorage` (komposisi melalui `RefCell`).** `Builder` mengkomposisi `CoverStorage` melalui field `covers: RefCell<CoverStorage>` pada `src/builder.rs:49`; pembungkus `RefCell` diperlukan agar `build_path` dapat meminjam `&mut covers` saat `&mut self.blocks.build_block(...)` aktif. `CoverStorage::new` dipanggil di `Builder::new` (`src/builder.rs:80`).
4. **`Builder` ◆── `TileMap<Tile>` (komposisi atas koleksi `Tile`).** `Builder` mengkomposisi koleksi `Tile` melalui field `tiles: TileMap<Tile>` pada `src/builder.rs:37`; loop emisi pada `Builder::generate_tiles` mendorong setiap ubin nontrivial dengan `self.tiles.push(Tile { ... })` (`src/builder.rs:223-237`).
5. **`Blocks` ◇── `Block` (agregasi).** `Blocks` mengagregasi nol-atau-lebih `Block` melalui field `data: Vec<Block>` pada `src/blocks.rs:53`. Setiap iterasi inner DDA pada `Blocks::push_to_tile` (`src/blocks.rs:625`) menambahkan satu `Block` ke `self.data`.
6. **`CoverStorage` ─── `TileBounds` (asosiasi melalui parameter).** `CoverStorage::reset_for_shape(&self.bbox)` dipanggil oleh `Builder::build_path` (`src/builder.rs:127`) untuk meresize `tag` dan `backdrops` sesuai dimensi `TileBounds` yang dihitung dari `Box2D<f32>` shape; relasi ini bersifat asosiasi runtime, bukan kepemilikan.
7. **`WebGlRenderer` ─── `Scene` (asosiasi runtime).** `WebGlRenderer::render(&mut self, scene: &Scene, render_size: &RenderSize)` (`src/render/webgl.rs:296`) menerima referensi `&Scene` sebagai parameter dan membaca vertex buffer melalui `scene.tiles()` (`src/scene.rs:155`) serta tekstur segmen melalui `scene.segments()` (`src/scene.rs:160`). `WebGlRenderer` tidak memiliki `Scene`; ia hanya memakainya untuk durasi satu pemanggilan `render`.
8. **`PicoSvg` ─── `Scene` (dependency aplikasi).** Pemakai pustaka memanggil `PicoSvg::load` (`src/pico_svg.rs:84`) untuk mengurai dokumen SVG menjadi `Vec<Item>`, lalu mengiterasi setiap `FillItem`/`StrokeItem` dan menyerahkan `BezPath`-nya ke `Scene::fill` atau `Scene::stroke`. Relasi ini bersifat dependency: `PicoSvg` dan `Scene` tidak saling memegang referensi struct, namun `PicoSvg` adalah jalur masukan kanonik untuk isi `Scene`.

## 3.5 Perancangan Algoritma

Pipeline hibrida Arabella memecah pekerjaan rendering geometri dua dimensi menjadi enam tahap CPU yang berurutan diikuti oleh dua tahap GPU. Pada tahap CPU, viewport dibagi secara spasial menjadi kisi ubin homogen berukuran tetap $16 \times 8$ piksel sesuai konstanta `TILE_W = 16` dan `TILE_H = 8` yang dideklarasikan pada `src/blocks.rs:6-7` (verbatim `pub const TILE_W: usize = 16;` dan `pub const TILE_H: usize = 8;`). Ukuran ubin yang sama dipakai konsisten oleh `Builder::new` saat menginisialisasi setiap rekord `Tile` (`src/builder.rs:60-61`). Variasi perilaku antar ubin pada tahap GPU sepenuhnya didorong data — jumlah segmen yang di-binning, isi backdrop per scanline, dan bit fill rule pada `paint_and_rect_flag` — sehingga seluruh ubin nontrivial diproses fragment shader tunggal melalui jalur kode yang sama tanpa cabang `if`/`switch` berbasis kategori ubin. Subbab ini menguraikan keenam tahap pra-pemrosesan tersebut sebagai enam sub-bagian terpisah, kemudian menjelaskan cara fragment shader mengevaluasi cakupan piksel dan menerapkan dua aturan fill rule yang didukung pustaka.

**(a) Flattening Kurva: Cubic-to-Quadratic dan Quadratic-to-Garis pada F24Dot8**

Tahap flattening menyederhanakan setiap kurva Bézier kubik dan kuadratik menjadi rangkaian segmen garis lurus pada format fixed-point F24Dot8 (24.8 fixed-point yang disimpan sebagai `i32` dengan skala 1 piksel = 256 unit). Untuk kurva kubik, pustaka mengikuti strategi cubic-to-quadratic gaya Vello: jumlah sub-kuadratik diestimasi terlebih dahulu oleh `estimate_number_of_quadratic_curves` (`src/path.rs:352-375`) berdasarkan toleransi `TOL = 0.25` dan look-up table 16 entri yang membatasi sub-kuadratik maksimum pada `MAX_QUADS = 16` (`src/path.rs:7`); kemudian `convert_cubics_to_quadratic_curves` (`src/path.rs:391-475`) menghasilkan barisan titik kontrol sub-kuadratik melalui sampling kubik pada interval seragam $\Delta t = 0{,}5 / n$ menggunakan jalur SIMD `f32x8` dari `fearless_simd`. Setiap sub-kuadratik selanjutnya dilinearisasi menjadi segmen garis melalui De Casteljau midpoint subdivision di `flatten_quadratic` (`src/flatten.rs:20-29`) yang mendelegasikan rekursi ke `flatten_recursive` (`src/flatten.rs:31-58`). Pada setiap level rekursi, fungsi `is_flat_enough` (`src/flatten.rs:62-87`) menguji deviasi L1 antara midpoint chord $\frac{P_0 + P_2}{2}$ dan titik kontrol $P_1$ terhadap konstanta `FLATNESS_THRESHOLD: i32 = 32` (`src/flatten.rs:18`); jika lulus, kurva dianggap cukup datar dan satu segmen garis lurus dari $P_0$ ke $P_2$ ditulis ke buffer kerja `Builder::line_buf` sebagai empat `i32` `[p0x, p0y, p1x, p1y]` (`src/builder.rs:43-45`). Jika belum lulus, kurva dipecah dua di $t = 0{,}5$ memakai pembagian bilangan bulat F24Dot8 (`src/flatten.rs:46-57`) — bukan pergeseran aritmetika — agar tidak terjadi drift satu bit pada viewport bertanda.

**(b) Binning DDA Dua Tahap: Outer DDA dan Inner DDA**

Setelah `line_buf` terisi segmen garis F24Dot8, setiap segmen di-binning ke ubin yang dilintasinya melalui dua tahap Digital Differential Analyzer (DDA) yang seluruhnya berada di `src/blocks.rs`. Pintu masuk publik adalah `Blocks::build_block` (`src/blocks.rs:93-102`) yang melakukan tail-call ke `Blocks::bin_line` (`src/blocks.rs:107-160`) — tahap **outer DDA**. Outer DDA memecah satu segmen lintas baris ubin (tinggi `TILE_H`) menggunakan empat varian diagonal berdasarkan tanda `(dx, dy)`: down-right pada `outer_dda_down_right` (`src/blocks.rs:163-207`), down-left pada `outer_dda_down_left` (`src/blocks.rs:210-254`), up-right pada `outer_dda_up_right` (`src/blocks.rs:257-301`), dan up-left pada `outer_dda_up_left` (`src/blocks.rs:304-348`). Selain empat arah diagonal tersebut, outer DDA menangani tiga kasus khusus sebagai cabang kode terpisah: (i) horizontal-degenerate ketika `p0y == p1y` yang langsung ber-early-return tanpa emisi karena segmen horizontal murni tidak menyumbang winding (`src/blocks.rs:115-117`); (ii) single-row ketika seluruh segmen muat di satu baris ubin sehingga tidak perlu loop DDA dan dilewatkan langsung ke inner DDA (`src/blocks.rs:134-138` untuk arah turun, `src/blocks.rs:149-153` untuk arah naik); dan (iii) vertikal degenerate ketika `p0x == p1x` di dalam `bin_line_in_row` yang mendeteksi kolom ubin tunggal lalu memanggil `push_to_tile` tanpa loop DDA (`src/blocks.rs:366-374`).

Setelah outer DDA mengklip sub-segmen ke satu baris ubin, **inner DDA** pada `Blocks::bin_line_in_row` (`src/blocks.rs:353-391`) memecah sub-segmen lintas kolom ubin (lebar `TILE_W`) memakai empat varian arah utama: right-down pada `inner_dda_right_down` (`src/blocks.rs:394-451`), right-up pada `inner_dda_right_up` (`src/blocks.rs:454-505`), left-down pada `inner_dda_left_down` (`src/blocks.rs:508-560`), dan left-up pada `inner_dda_left_up` (`src/blocks.rs:563-619`). Setiap iterasi inner DDA memanggil `Blocks::push_to_tile` (`src/blocks.rs:625-657`) yang men-tag cell `(row, col)` sebagai disentuh, mendorong satu rekord `Block` berisi endpoint sub-segmen pada koordinat ubin-lokal F24Dot8 ke `self.data: Vec<Block>`, dan memanggil akumulator signed-area yang dijelaskan pada sub-bagian (c). Konstanta `TILE_W_F24DOT8 = 4096` dan `TILE_H_F24DOT8 = 2048` (`src/blocks.rs:10-11`) merepresentasikan ukuran ubin dalam unit F24Dot8 yang dipakai oleh seluruh varian DDA untuk menghitung sub-piksel offset.

**(c) Akumulator Signed-Area per Scanline pada 8.8 Fixed-Point**

Bersamaan dengan setiap pemanggilan `push_to_tile`, fungsi `record_per_scanline_crossings` (`src/blocks.rs:710-757`) mengakumulasi sumbangan signed area sub-segmen vertikal terhadap delapan strip scanline pada satu ubin. Akumulator disimpan sebagai `[i16; TILE_H]` — delapan akumulator i16 per cell, satu per scanline — pada format 8.8 fixed-point dengan konvensi tanda eksplisit: garis turun ($y_0 < y_1$) mengurangi akumulator sebesar $-256$ unit (setara $-1$ winding penuh), sedangkan garis naik ($y_0 > y_1$) menambah akumulator sebesar $+256$ unit (setara $+1$ winding penuh). Penjumlahan memakai `saturating_add` agar tidak terjadi overflow i16 untuk segmen yang melintasi banyak scanline. Skema ini adalah akumulator signed-area kanonik gaya Blaze (Gasiulis, 2024), FreeType (The FreeType Project, 2023), dan Skia (The Skia Project, 2023), yang dipilih agar fragment shader pada GPU dapat melakukan multisampling sub-piksel tanpa residu winding yang menggariskan seam antar ubin tetangga. Hasil array signed-area per cell disimpan pada `CoverStorage::backdrops: Vec<[i16; TILE_H]>` (`src/builder.rs:360-369`) yang dialokasikan ulang per shape oleh `CoverStorage::reset_for_shape` (`src/builder.rs:413`).

**(d) Propagasi Backdrop Kiri-ke-Kanan**

Tahap emisi ubin dijalankan oleh `Builder::generate_tiles` (`src/builder.rs:151-337`). Pada awal method, paint flag fill rule disandikan ke bit ke-24 (`src/builder.rs:158-162`) dan `Blocks::sort_blocks` mengurutkan rekord block menurut `(y, x)` agar lookup linier per ubin dapat memakai binary-search di `find_segment_range` (`src/builder.rs:171, 349-354`). Loop emisi kemudian menelusuri grid ubin baris demi baris: pada awal setiap baris ubin akumulator lokal `acc_arr: [i16; 8]` di-reset menjadi `[0; 8]` di tepi kiri (`src/builder.rs:194`), lalu loop kolom berjalan dari `col = 0` ke `col = covers.cols() - 1` sehingga akumulator winding bertambah secara monotonik kiri-ke-kanan sepanjang baris tersebut. Sebuah cell `(row, col)` diemisi sebagai `Tile` jika gerbang `tagged || acc_nonzero` bernilai benar (`src/builder.rs:196-203`): `tagged` menandakan DDA menyentuh cell ini, sedangkan `acc_nonzero` menandakan akumulator yang dipropagasi dari kolom-kolom kiri belum kembali ke nol. Definisi operasional inilah yang dipakai konsisten pada Subbab 3.4.2, 3.4.3, dan 3.4.4 untuk istilah "ubin nontrivial". Saat ubin diemisi, `Tile.backdrop = acc_arr` ditulis terlebih dahulu (`src/builder.rs:223-237`) — yaitu nilai akumulator sebelum kontribusi cell ini ditambahkan — sehingga backdrop yang diunggah ke GPU mencerminkan sumbangan kumulatif dari seluruh kolom di sebelah kirinya. Setelah emisi, jika cell `tagged`, akumulator diperbarui in-place dengan satu instruksi SIMD `i16x8.add` yang menggantikan delapan operasi skalar (`src/builder.rs:240-247`), lalu dibawa ke `col + 1` untuk diproses ubin berikutnya pada baris ubin yang sama.

**(e) Evaluasi Cakupan Piksel di GPU melalui Integral Trapezoidal `line_box`**

Setiap ubin yang diemisi menjadi satu instance vertex buffer 44 byte yang dieksekusi oleh fragment shader analitik tunggal di `src/render/shaders/render_tile.frag`. Pada awal `void main()`, nilai backdrop per scanline dibaca dari atribut tile dan dikonversi dari 8.8 fixed-point ke float dengan dibagi konstanta `WINDING_UNIT = 256.0` (`src/render/shaders/render_tile.frag:24`), menghasilkan basis akumulator winding kontinu. Untuk setiap segmen garis yang di-binning ke ubin, fungsi `line_box` (`src/render/shaders/render_tile.frag:90`) — yang dideskripsikan blok komentar di atasnya sebagai *"Analytic line-area contribution (BOX filter, radius 0.5)"* — mengembalikan kontribusi cakupan piksel bertanda dalam interval $[-1, +1]$. Secara matematis `line_box` adalah konvolusi indikator setengah-bidang garis dengan filter kotak $1 \times 1$ yang berpusat pada koordinat piksel, dan dievaluasi secara trapezoidal: arah-y garis menentukan tanda $\pm 1$, rentang-y di-clip ke kotak piksel $[\text{pixel.y} - 0{,}5, \text{pixel.y} + 0{,}5]$, dan rentang-x sub-segmen ter-clip dijumlahkan dengan rumus `h_cov = px_hi - avg_x` di mana `avg_x = (xc_lo + xc_hi) * 0.5`. Hasil akhirnya adalah produk panjang vertikal sub-segmen ter-clip dengan rerata cakupan horizontal pada kotak piksel, yang dijumlahkan ke akumulator `winding` melalui loop sekuensial `for (uint s = 0u; s < seg_count; s++)`.

**(f) Penerapan Fill Rule NonZero (Clamp Absolute) dan EvenOdd (Triangle Wave)**

Setelah `winding` final terhitung, fragment shader memilih formula coverage berdasarkan dua bit fill rule yang diekstrak dari `paint_and_rect_flag` melalui mask `FILL_RULE_MASK = 0x07000000u` dan shift `FILL_RULE_SHIFT = 24u` (`src/render/shaders/render_tile.frag:15-21`). Aturan **NonZero** (`FILL_RULE_NONZERO = 0u`) memakai clamp pada nilai mutlak winding:

$$\text{coverage} = \mathrm{clamp}(|\omega|,\, 0{,}0,\, 1{,}0)$$

dengan bentuk GLSL `coverage = clamp(abs(winding), 0.0, 1.0);` pada `src/render/shaders/render_tile.frag:215`. Aturan **EvenOdd** (`FILL_RULE_EVENODD = 1u`) memakai gelombang segitiga pada nilai mutlak winding:

$$\text{coverage} = 1{,}0 - \bigl| \,\mathrm{mod}(|\omega|,\, 2{,}0) - 1{,}0 \,\bigr|$$

dengan bentuk GLSL `coverage = 1.0 - abs(mod(abs(winding), 2.0) - 1.0);` pada `src/render/shaders/render_tile.frag:218-219`. Kedua aturan diterapkan pada fragment shader yang sama melalui satu cabang biner `if (fill_rule == FILL_RULE_NONZERO) { … } else { … }` (`src/render/shaders/render_tile.frag:213-220`); cabang ini hanya bergantung pada bit fill rule, bukan pada kategori ubin. Ekspresi penyandian fill rule pada sisi CPU dilakukan oleh `Builder::generate_tiles` (`src/builder.rs:158-162`) yang memetakan `peniko::Fill::NonZero` ke konstanta `FILL_RULE_NONZERO = 0` dan `peniko::Fill::EvenOdd` ke `FILL_RULE_EVENODD = 1` (`src/builder.rs:28-29`) sebelum bit hasilnya digabung ke `paint_and_rect_flag` melalui shift `FILL_RULE_SHIFT: u32 = 24` (`src/builder.rs:17`).

**Catatan Paralelisme: Rayon sebagai Dependensi Opsional**

Klaim paralelisme CPU pada uraian di atas perlu dipahami sebagai kapasitas potensial, bukan paralelisme yang sudah aktif pada implementasi yang dievaluasi. Rayon dideklarasikan sebagai dependensi opsional di balik feature flag `multithreading` pada `Cargo.toml`, yaitu `rayon = { version = "1.11.0", optional = true }` (`Cargo.toml:48-50`) yang ditarik bersama `thread_local` melalui `multithreading = ["std", "dep:rayon", "dep:thread_local"]` (`Cargo.toml:94`). Karena feature `multithreading` tidak termasuk pada `default = ["std", "png"]`, build standar tidak menyertakan Rayon, dan loop binning di `Builder::build_path` (`src/builder.rs:138-148`) maupun loop emisi ubin di `Builder::generate_tiles` (`src/builder.rs:193-247`) saat ini tidak memanggil API Rayon mana pun pada hot path. Akibatnya, paralelisme antar shape atau antar baris ubin masih berupa kapasitas yang sudah disiapkan kerangka build-system-nya, sementara eksekusi pra-pemrosesan pada implementasi saat ini berjalan single-thread per pemanggilan `Scene::fill` atau `Scene::stroke`.

## 3.6 Perancangan Layar

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

Berbeda dengan demo interaktif yang adaptif, harness pengujian otomatis berbasis `wasm-bindgen-test` pada `tests/test.rs` memakai resolusi kanvas yang tetap dan deterministik agar hasil rasterisasi piksel dapat dibandingkan secara reproducible lintas eksekusi. Pengujian aktif `test_renders_tiger_svg` (atribut `#[wasm_bindgen_test]` pada `tests/test.rs:147`) mendeklarasikan dua konstanta lokal `const W: u16 = 1080;` (`tests/test.rs:151`) dan `const H: u16 = 520;` (`tests/test.rs:152`), keduanya di-cast ke `u32` saat diteruskan ke `RenderSize { width: W as u32, height: H as u32 }` dan ke `create_canvas(W as u32, H as u32, 1.0)` di tubuh fungsi tes yang sama. Resolusi kanvas pengujian kanonik dengan demikian adalah $1080 \times 520$ piksel pada DPR efektif $1{,}0$, dan menjadi target frame buffer tunggal tempat pustaka mengalirkan hasil rasterisasi piksel akhir dari sirkuit WebGL untuk diverifikasi secara visual maupun programatik. Klaim resolusi default $1920 \times 1080$ pada draf metodologi sebelumnya tidak didukung oleh source code dan karena itu tidak dipakai pada Subbab ini.

Tidak terdapat elemen interaksi visual berupa tombol, menu kontrol, slider, maupun teks status di dalam kanvas pengujian guna menjaga netralitas pengukuran *frame time* tanpa interferensi rendering komponen UI pihak ketiga; satu-satunya elemen non-kanvas pada demo interaktif adalah overlay diagnostik `update_overlay` yang dideskripsikan pada poin (b) di atas dan secara sengaja dipisahkan dari jalur pengukuran tes otomatis.

## 3.7 Perancangan Database File

Arsitektur pustaka rendering hibrida ini dirancang untuk beroperasi secara stateless dengan performa kecepatan ekstrem dan latensi rendering per-bingkai seminimal mungkin. Demi mencapai tujuan performa tersebut, sistem ini tidak mengimplementasikan subsistem database persisten atau penyimpanan berkas database lokal (seperti SQLite atau struktur file terindeks sejenis); seluruh struktur data bersifat transien dan dialokasikan ulang per bingkai tanpa interaksi dengan media penyimpanan sekunder.

Seluruh daur hidup pengelolaan data geometri dan spasial dikelola secara dinamis di dalam memori akses acak (Volatile Random Access Memory / RAM) menggunakan struktur data internal bahasa Rust yang efisien selama durasi eksekusi aplikasi berjalan. Tiga buffer utama menjembatani sisi CPU dengan sisi GPU: (a) daftar ubin sebagai vektor datar `Vec<Tile>` di RAM, (b) tekstur segmen `RGBA32F` di Video RAM, dan (c) vertex buffer instanced 44 byte per ubin yang merupakan salinan tepat dari `Vec<Tile>` di Video RAM. Detail pemetaan alokasi memori internal ketiga buffer tersebut diatur dengan spesifikasi sebagai berikut:

1. **Daftar Ubin sebagai Vektor Datar `Vec<Tile>`.** Hasil keseluruhan pra-pemrosesan CPU — yaitu binning DDA, akumulator signed-area, dan propagasi backdrop — dikumpulkan ke dalam satu vektor datar `Vec<Tile>` yang dipegang oleh field `tiles` milik `Builder` dan dieksposkan ke `WebGlRenderer` melalui `Scene::tiles()` (`src/scene.rs:155-157`, yang mengembalikan `&self.builder.tiles.as_slice()`). Tipe elemen `Tile` dideklarasikan dengan atribut `#[repr(C)]` pada `src/tile.rs:9-23`, sehingga compiler tidak diperkenankan menyusun ulang field dan setiap rekord menempati tepat 44 byte yang sama persis di RAM maupun di Video RAM. Ukuran 44 byte ini diturunkan dari penjumlahan deklaratif field per `#[repr(C)]` (`x: u16` 2 byte, `y: u16` 2 byte, `width: u8` 1 byte, `height: u8` 1 byte, `_pad: [u8; 2]` 2 byte, `backdrop: [i16; 8]` 16 byte, `segments: [f32; 2]` 8 byte, `payload: u32` 4 byte, `paint_and_rect_flag: u32` 4 byte, dan `depth_index: u32` 4 byte) dan diverifikasi runtime oleh ekspresi `core::mem::size_of::<Tile>()` yang dipakai sebagai stride VAO pada `src/render/webgl.rs:529`. Atribut `#[derive(Pod, Zeroable)]` pada baris yang sama mengizinkan konversi `bytemuck::cast_slice::<Tile>(&tiles)` menjadi `&[u8]` tanpa salinan, sehingga `Vec<Tile>` dapat di-`buffer_data_with_u8_array` langsung ke vertex buffer GPU saat `WebGlRenderer::render` dipanggil (`src/render/webgl.rs:381-388`).

2. **Tekstur Segmen `RGBA32F` dengan Satu Texel per Garis.** Segmen garis hasil binning DDA dua tahap diunggah ke GPU bukan sebagai vertex attribute, melainkan sebagai tekstur dua dimensi dengan format internal `RGBA32F`, sehingga setiap fragment dapat melakukan pengambilan acak (random access) terhadap segmen yang relevan untuk ubinnya melalui `texelFetch`. Konvensi tata letak texel didokumentasikan verbatim pada komentar `src/render/webgl.rs:74` (yaitu "Each LINE record is 4 floats (p0.x, p0.y, p1.x, p1.y) = 1 RGBA32F texel.") dan dikuatkan oleh komentar paralel pada `src/builder.rs:40-42` yang menegaskan keempat float disimpan dalam satuan piksel pada koordinat ubin lokal. Implementasi unggah dilakukan oleh fungsi pembantu `upload_data_to_rgba32f_texture` (`src/render/webgl.rs:423-446`) yang memanggil `tex_image_2d` dengan parameter `internalformat = RGBA32F`, `format = RGBA`, dan `type = FLOAT`, sehingga setiap texel membawa empat komponen presisi tunggal yang dipetakan satu-ke-satu ke pasangan `(p0.x, p0.y, p1.x, p1.y)`. Karena koordinat dinyatakan dalam ruang piksel ubin lokal, rentang nilai komponen $x$ berada pada $[0, 16]$ dan rentang nilai komponen $y$ berada pada $[0, 8]$ sesuai dimensi ubin Arabella sebesar $16 \times 8$ piksel. Dengan tata letak ini, fragment shader `render_tile.frag` cukup membandingkan posisi `pixel_in_tile` terhadap dua titik ujung pada texel yang dibaca, tanpa perlu rekonstruksi koordinat tambahan.

3. **Vertex Buffer Instanced 44 byte/tile.** Pada sisi GPU, vektor `Vec<Tile>` disalin sebagai-adanya ke satu objek vertex buffer (`tiles_buffer`) yang dipakai dalam mode instancing oleh `WebGlRenderer::render_tiles` (`src/render/webgl.rs:393-398`, pemanggilan `draw_arrays_instanced(TRIANGLE_STRIP, 0, 4, tiles.len() as i32)`). Tata letak vertex buffer dikonfigurasi oleh fungsi `initialize_tile_vao` pada `src/render/webgl.rs:525` dengan stride yang ditetapkan langsung dari ukuran rekord `Tile` melalui `core::mem::size_of::<Tile>()` di `src/render/webgl.rs:529`, sehingga keseluruhan 44 byte dari satu rekord `Tile` membentuk satu blok per-instance. Status per-instance ini dipasang melalui pemanggilan `vertexAttribDivisor(_, 1)` untuk seluruh enam slot atribut (`src/render/webgl.rs:534, 539, 544, 549, 554, 559`), yang membuat setiap iterasi instance pada `draw_arrays_instanced` membaca tepat satu rekord `Tile` dari vertex buffer dan mengeluarkan satu quad terinstance (empat vertex via `TRIANGLE_STRIP`) yang dipetakan ke piksel-piksel ubin oleh vertex shader. Tata letak byte sekuensial dari satu rekord `Tile` di vertex buffer mengikuti urutan deklarasi `#[repr(C)]` di `src/tile.rs:9-23` dan dirinci pada Tabel 3.1 berikut.

**Tabel 3.1.** Tata letak byte vertex buffer instanced 44 byte/tile di Arabella (`src/tile.rs:9-23`).

| Offset (byte) | Field                  | Tipe                                                                          | Lebar (byte) | Slot atribut (`initialize_tile_vao`)                          |
|--------------:|------------------------|-------------------------------------------------------------------------------|-------------:|---------------------------------------------------------------|
| 0             | `x`                    | `u16`                                                                         | 2            | bagian dari atribut 0 (`UNSIGNED_INT`, offset 0)              |
| 2             | `y`                    | `u16`                                                                         | 2            | bagian dari atribut 0 (`UNSIGNED_INT`, offset 0)              |
| 4             | `width`                | `u8`                                                                          | 1            | bagian dari atribut 1 (`UNSIGNED_INT`, offset 4)              |
| 5             | `height`               | `u8`                                                                          | 1            | bagian dari atribut 1 (`UNSIGNED_INT`, offset 4)              |
| 6             | `_pad`                 | `[u8; 2]`                                                                     | 2            | padding eksplisit untuk alignment `backdrop` ke 8 byte        |
| 8             | `backdrop`             | `[i16; 8]` (delapan i16, satu per scanline ubin)                              | 16           | atribut 2 (`SHORT × 4`, offset 8) dan atribut 3 (`SHORT × 4`, offset 16) |
| 24            | `segments`             | dua elemen offset+jumlah (di RAM bertipe `[f32; 2]`, di-reinterpret-bit oleh GPU sebagai `UNSIGNED_INT × 2`) | 8            | atribut 4 (`UNSIGNED_INT × 2`, offset 24)                     |
| 32            | `payload`              | `u32`                                                                         | 4            | bagian dari atribut 5 (`UNSIGNED_INT × 3`, offset 32)         |
| 36            | `paint_and_rect_flag`  | `u32`                                                                         | 4            | bagian dari atribut 5 (`UNSIGNED_INT × 3`, offset 32)         |
| 40            | `depth_index`          | `u32`                                                                         | 4            | bagian dari atribut 5 (`UNSIGNED_INT × 3`, offset 32)         |
| **Total**     |                        |                                                                               | **44**       | stride VAO = `core::mem::size_of::<Tile>() = 44`              |

Field `backdrop: [i16; 8]` membawa hasil propagasi backdrop kiri-ke-kanan untuk delapan scanline ubin (selaras dengan `TILE_H = 8`), sehingga setiap scanline memiliki akumulator winding number 16-bit independen yang dievaluasi oleh fragment shader saat menerapkan fill rule NonZero atau EvenOdd. Field `segments` membawa pasangan `(offset, count)` yang merujuk slice kontigu pada tekstur segmen `RGBA32F` di poin (2): nilai `offset` menjadi indeks awal pembacaan `texelFetch` dan nilai `count` membatasi banyaknya iterasi pembacaan segmen di shader. Field `payload`, `paint_and_rect_flag`, dan `depth_index` menyalurkan informasi non-geometris (indeks paint, bit flag bentuk persegi panjang, dan z-order) yang dipakai vertex dan fragment shader untuk komposisi akhir. Karena keseluruhan 44 byte ini bertindak sebagai data per-instance dengan divisor satu, tidak ada vertex buffer kedua yang berisi koordinat geometri quad — empat vertex yang membentuk quad per ubin diturunkan secara analitik oleh vertex shader dari field `x`, `y`, `width`, dan `height` pada rekord `Tile` aktif, sehingga total transfer per bingkai dibatasi oleh `tiles.len() × 44` byte ditambah ukuran tekstur segmen `RGBA32F`.

Setelah `WebGlRenderer::render` selesai, alokasi `Vec<Tile>` dan `Vec<f32>` segmen pada sisi CPU dapat di-`reset` melalui `Scene::reset` (`src/scene.rs:165-168`) tanpa kehilangan informasi yang relevan untuk bingkai berikutnya, karena seluruh kontrak data bersifat per-bingkai dan tidak ada bagian dari ketiga buffer di atas yang perlu dipertahankan ke disk. Dengan demikian, "perancangan database file" pada Arabella dimaknai sebagai perancangan tata letak biner ketiga buffer in-memory tersebut beserta korespondensinya dengan tekstur dan vertex buffer GPU, bukan sebagai skema basis data persisten.
