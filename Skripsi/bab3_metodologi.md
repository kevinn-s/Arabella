# BAB 3 METODE PENELITIAN

## 3.1 Diagram Alir Kerangka Berpikir

Kerangka berpikir dalam penelitian ini dirancang untuk memberikan tahapan yang sistematis dan terarah guna memecahkan masalah dependensi compute shader pada pipeline rendering vektor paralel. Penelitian eksperimental ini dibagi menjadi lima fase utama yang saling berkesinambungan, seperti yang dijabarkan secara tekstual di bawah ini:

1. **Fase 1: Studi Literatur dan Pengumpulan Data**
   - Melakukan analisis mendalam terhadap literatur grafis komputer terkait, mencakup metode shortcut tree, teknik rasterisasi scanline berbasis boundary fragments, fungsi implisit Loop-Blinn, serta arsitektur mesin rendering compute-centric modern seperti Vello.
   - Mengumpulkan berkas sampel uji dalam format Scalable Vector Graphics (SVG) dengan variasi tingkat kompleksitas geometri jalur primitif.

2. **Fase 2: Analisis Kebutuhan Sistem**
   - Mengidentifikasi keterbatasan lingkungan grafis non-compute pada perangkat low-end.
   - Merumuskan spesifikasi fungsional pustaka (library) rendering berbasis pemrograman sistem Rust dan API grafis OpenGL ES 3.0 / WebGL 2.0.

3. **Fase 3: Perancangan Arsitektur dan Algoritma**
   - Merancang pembagian beban kerja hibrida: tahap preprocessing paralel masif pada Central Processing Unit (CPU) dan tahap rasterisasi pada Graphics Processing Unit (GPU).
   - Menyusun struktur data spasial berbasis tiling (ubin) serta merancang mekanisme kalkulasi winding number melalui teknik ray shooting.
   - Memformulasikan pemetaan kurva Bézier kuadratik dan kubik ke dalam fungsi implisit kanonik.

4. **Fase 4: Implementasi Purwarupa (Prototype)**
   - Membangun modul parser SVG dan arsitektur data memori di CPU menggunakan Rust.
   - Mengimplementasikan paralelisasi tingkat tile pada CPU menggunakan pustaka data paralel Rayon.
   - Mengembangkan pemrosesan Vertex Shader dan Fragment Shader konvensional pada GPU menggunakan OpenGL ES 3.0.

5. **Fase 5: Pengujian dan Evaluasi**
   - Melakukan validasi kebenaran output visual (correctness validation) dengan membandingkan citra hasil purwarupa terhadap renderer referensi sekuler.
   - Melakukan pengujian performa (benchmarking) untuk mengukur metrik frame time (kecepatan per bingkai) serta skalabilitas throughput sistem terhadap penambahan jumlah core CPU.
   - Menganalisis trade-off kualitatif dan kuantitatif pustaka yang diusulkan terhadap mesin rendering berbasis CPU murni (Skia/Cairo) serta berbasis GPU komputasi penuh (Vello).

## 3.2 Analisis Kebutuhan

### 3.2.1 Analisis User

Sistem yang dikembangkan dalam penelitian ini merupakan sebuah pustaka antarmuka pemrograman aplikasi (Application Programming Interface / API) rendering grafis 2D. Oleh karena itu, pengguna langsung (user) dari sistem ini adalah pengembang perangkat lunak (software developer) yang membutuhkan kapabilitas eksekusi grafis vektor performa tinggi untuk diintegrasikan ke dalam aplikasi akhir mereka, seperti game engine, browser web emulasi, atau aplikasi seluler.

Pengguna sistem ini diwajibkan memiliki pemahaman dasar mengenai matematika komputasi grafis (seperti koordinat kartesian, vektor, dan kurva parametrik), alur kerja pipeline grafis tradisional (konsep shader, vertex buffer, dan framebuffer), serta memiliki pengalaman dalam implementasi kode menggunakan bahasa pemrograman sistem. Interaksi pengembang dengan pustaka dilakukan sepenuhnya secara terprogram (programmatic) melalui pemanggilan fungsi-fungsi pustaka dan penyerahan deskripsi geometri berbasis teks atau biner (seperti data jalur SVG), tanpa melibatkan interaksi komponen antarmuka grafis pengguna akhir (Graphical User Interface).

### 3.2.2 Analisis Aplikasi Sejenis

Untuk memvalidasi urgensi pengembangan sistem, dilakukan analisis komparatif terhadap tiga arsitektur rendering grafis yang menjadi acuan utama dalam penelitian ini:

1. **Vello (Linebender)** — Merupakan mesin rendering modern yang mengadopsi paradigma GPU compute-centric. Pustaka ini memindahkan seluruh komputasi sekuensial yang berat — seperti tessellation, pemotongan geometri (clipping), dan alokasi memori spasial — langsung ke GPU menggunakan serangkaian compute shader dispatches. Akselerasi paralelnya memanfaatkan algoritma parallel prefix-sum guna menurunkan kompleksitas serial $O(n)$ menjadi tugas paralel $O(\log n)$. Walaupun menghasilkan throughput masif pada perangkat high-end, Vello memerlukan dukungan WebGPU API atau Vulkan modern, sehingga tidak dapat beroperasi secara stabil pada segmen perangkat keras legacy atau low-end kelas konsumen.

2. **Massively-Parallel Vector Graphics (Gan et al., 2014)** — Sistem ini memparalelkan tahap preprocessing segmen geometri masukan dan tahap rendering sampel piksel secara simultan di GPU. Komponen intinya memanfaatkan struktur data spasial hierarkis adaptif bernama Shortcut Tree (berbasis quadtree) untuk memberikan akses acak cepat terhadap nilai warna piksel. Namun, implementasi algoritma ini sangat bergantung pada arsitektur komputasi umum GPU yang spesifik (seperti teknologi NVIDIA CUDA) untuk menangani warping dan penjadwalan sampel, yang secara drastis membatasi portabilitas lintas platform.

3. **Efficient GPU Path Rendering Using Scanline Rasterization (Li et al., 2016)** — Pendekatan ini mengadaptasi algoritma scanline rasterizer klasik agar berjalan paralel di atas GPU. Logika utamanya memisahkan pemrosesan antara piksel perbatasan jalur (boundary fragments berukuran $2 \times 2$ piksel) dan piksel bagian dalam (horizontal spans). Pendekatan ini meminimalkan biaya komputasi winding number global dengan melokalisasi komputasi cakupan anti-aliasing. Walaupun efisien, sistem ini tetap mengandalkan arsitektur compute pipeline untuk fase pengurutan (sorting) dan penggabungan (merging) fragmen sebelum tahap rasterisasi akhir dilakukan.

### 3.2.3 Rumusan dan Solusi Kebutuhan

Berdasarkan analisis kesenjangan (gap analysis) terhadap aplikasi sejenis, berikut adalah tabel pemetaan rumusan masalah dan solusi teknis yang diimplementasikan di dalam pustaka ini:

| No | Rumusan Masalah Keperluan Sistem | Solusi Teknis Terimplementasi |
|----|----------------------------------|-------------------------------|
| 1 | Kebutuhan akan metode rendering grafis vektor paralel secara masif yang tidak bergantung pada fitur compute shader. | Merancang pipeline hibrida yang membagi tugas secara tegas: mengeksekusi tahapan preprocessing spasial secara paralel masif di CPU, dan mengalokasikan rasterisasi piksel ke GPU. |
| 2 | Kebutuhan akan jaminan kompatibilitas platform yang luas, terutama pada lingkungan grafis terbatas (web lama dan perangkat low-end). | Membatasi implementasi GPU secara ketat hanya pada rasterization pipeline tradisional menggunakan Vertex Shader dan Fragment Shader standar OpenGL ES 3.0 / WebGL 2.0. |
| 3 | Kebutuhan akan efisiensi performa tinggi dan minimalisasi overdraw komputasi pada skenario adegan (scene) vektor yang kompleks. | Menerapkan segmentasi layar berbasis tiling (ubin) berukuran tetap dan kalkulasi winding number dini di CPU melalui algoritma Ray Shooting paralel untuk memangkas piksel kosong (empty tiles) sebelum dikirim ke memori GPU. |

## 3.3 Perancangan Aplikasi

### 3.3.1 Spesifikasi Aplikasi

Pustaka rendering vektor paralel ini dirancang dengan spesifikasi teknis tingkat sistem untuk menjamin performa optimal dan reliabilitas tinggi. Spesifikasi komponen penyusun aplikasi didefinisikan sebagai berikut:

- **Bahasa Pemrograman Utama:** Rust (Edisi 2021). Pemilihan bahasa ini didasarkan pada karakteristik zero-cost abstractions untuk menjamin kecepatan setingkat C/C++, fitur guaranteed memory safety tanpa komponen Garbage Collector guna menghindari latensi berkala, serta paradigma konkurensi aman (fearless concurrency) yang krusial untuk paralelisasi CPU.
- **Pustaka Konkurensi CPU:** Rayon (work-stealing data parallelism library). Rayon digunakan untuk memparalelkan iterasi pemrosesan grid tile secara dinamis ke seluruh thread pool logika CPU yang tersedia.
- **Antarmuka Grafis (Graphics API):** OpenGL ES 3.0. Menggunakan abstraksi binding grafis lintas platform untuk memastikan pipeline dapat ditranspilasikan ke WebGL 2.0 di lingkungan peramban web tanpa memodifikasi logika shader inti.
- **Format Data Geometri:** Mengadopsi standar parsing Scalable Vector Graphics (SVG) 1.1 Core, yang dikonversikan menjadi representasi internal berupa array primitif terstruktur (flattened paths) di memori lokal.

## 3.4 Perancangan Sistem

### 3.4.1 Use Case Diagram

Sistem ini memodelkan interaksi fungsional antara aktor tunggal (Developer) dengan modul eksternal sistem melalui batasan sistem pustaka (Vector Rendering Library). Aktor Developer mengendalikan daur hidup rendering melalui tiga komponen fungsional utama (use case):

1. **Inisialisasi Context (UC-01):** Menyediakan titik awal untuk alokasi memori subsistem grafis.
2. **Input Data Vektor (UC-02):** Berperan sebagai jembatan penyerahan berkas geometri dari aplikasi utama ke memori internal pustaka.
3. **Render Frame (UC-03):** Use case utama yang memicu fungsi loop rendering per frame hibrida, di mana use case ini secara otomatis mengikutsertakan (include) sub-proses Preprocessing (CPU Tiling & Ray Shoot) dan sub-proses Rendering (GPU Implicit Evaluation).

### 3.4.2 Use Case Description

Berikut adalah spesifikasi naratif terperinci untuk masing-masing use case yang terintegrasi di dalam sistem:

**Tabel UC-01: Deskripsi Use Case Inisialisasi Context**

| Komponen Deskripsi | Spesifikasi Fungsional |
|---|---|
| Nama Use Case | Inisialisasi Context |
| ID Use Case | UC-01 |
| Aktor Utama | Developer |
| Deskripsi Singkat | Proses penyiapan spesifikasi grafis (OpenGL ES Context), kompilasi program shader konvensional, serta alokasi awal objek memori pada pustaka. |
| Kondisi Awal | Aplikasi pengembang telah berjalan sukses dan memiliki referensi permukaan jendela grafis (window context) aktif. |
| Kondisi Akhir | Pustaka siap menerima data geometri, program shader telah terkompilasi, dan lokasi memori GPU telah teralokasi. |
| Alur Peristiwa Inti (Basic Flow) | 1. Developer memanggil fungsi inisialisasi API pustaka. 2. Sistem memuat konfigurasi grafis dasar berbasis aturan OpenGL ES. 3. Sistem memuat, mengompilasi, dan menautkan (linking) program Vertex Shader dan Fragment Shader konvensional. 4. Sistem mengembalikan kode status "Context Ready" kepada Developer. |

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
| Kondisi Akhir | Hasil visual grafik vektor ter-rasterisasi secara presisi pada permukaan layar target. |
| Alur Peristiwa Inti (Basic Flow) | **A. Pemicu (Trigger):** 1. Developer memanggil fungsi kolektif `render()` dari loop aplikasi utama. **B. Tahap Preprocessing (CPU Paralel):** 2. Pustaka menghitung batas spasial (bounding box) untuk setiap kurva jalur. 3. Pustaka memetakan referensi kurva ke dalam kisi ubin (grid tile) berukuran tetap secara paralel. 4. Pustaka mengeksekusi algoritma Ray Shooting pada setiap ubin untuk menghitung nilai winding number diskrit. 5. Pustaka mengklasifikasikan tipe ubin: Empty (diabaikan), Interior (terisi penuh), atau Edge (dilintasi kurva). **C. Tahap Rendering (GPU Rasterization):** 6. Pustaka mengirimkan data quads geometri dari ubin yang valid ke memori GPU. 7. Untuk Interior Tile, Fragment Shader diperintahkan langsung merender piksel sebagai warna solid. 8. Untuk Edge Tile, Fragment Shader mengeksekusi evaluasi fungsi implisit $C(x,y)=0$ per piksel untuk menentukan cakupan warna presisi. 9. Hasil komposit warna akhir dialirkan ke objek framebuffer layar. |

### 3.4.3 Sequence Diagram

Sequence diagram menggambarkan interaksi sekuensial berbasis pesan (message-passing) antar komponen sistem sepanjang siklus hidup satu bingkai rendering (single frame lifetime):

**1. Fase Inisialisasi & Data Input:**

- Aplikasi Utama (Dev) mengirim pesan sinkron Input Path (Kurva Bézier) ke objek CPU Preprocessor.
- CPU Preprocessor mengeksekusi operasi internal Simpan Data Geometri ke dalam struktur data Scene.

**2. Fase Rendering (Per Bingkai):**

- Aplikasi Utama (Dev) memicu fungsi loop dengan mengirim instruksi Panggil Render() ke komponen CPU Preprocessor.
- CPU Preprocessor memulai sub-rutin internal Tahap Preprocessing:
  - Mengeksekusi fungsi Bagi Layar jadi Grid Tile.
  - Masuk ke dalam blok perulangan paralel Loop [Setiap Tile (Paralel)].
  - Memanggil operasi internal Map Kurva ke Tile dilanjutkan dengan fungsi Ray Shooting (Hitung Winding Number).
  - Mengeksekusi logika kondisional alternatif alt: Jika memenuhi syarat [Winding Number != 0 & Tanpa Kurva], panggil fungsi Tandai Tipe: INTERIOR (Solid). Jika terdapat interseksi geometri [Ada Interseksi Kurva], panggil fungsi Tandai Tipe EDGE (Tepi).
  - Setelah perulangan selesai, CPU Preprocessor mengeksekusi fungsi Generate Vertex Buffer (Quads).
- CPU Preprocessor melakukan transfer data asinkron Upload Data Tile & Uniforms ke komponen GPU Rasterizer.
- GPU Rasterizer mengaktifkan Tahap Rendering pada sirkuit perangkat keras:
  - Mengeksekusi komponen Vertex Shader (Posisi Tile) untuk memetakan koordinat quad.
  - Masuk ke dalam blok perangkat keras Loop [Fragment Shader (Per Piksel)].
  - Mengeksekusi penanganan kondisional internal alt: Jika mendeteksi properti [Tipe Tile == EDGE], jalankan instruksi Evaluasi Fungsi Implisit C(x,y) untuk memanggil rutinitas Tentukan Warna (Dalam/Luar). Jika properti terdeteksi sebagai [Tipe Tile == INTERIOR], jalankan instruksi Render Warna Solid secara langsung.
  - Komponen GPU Rasterizer mengirimkan sinyal balik visual Tampilkan Frame ke Layar ke Aplikasi Utama (Dev) melalui mekanisme buffer swapping.

### 3.4.4 Class Diagram

Arsitektur perangkat lunak dari pustaka diorganisasikan menggunakan paradigma terstruktur berorientasi objek-data (struktural struct-driven di Rust) yang direpresentasikan dalam susunan Class Diagram berikut:

- **Renderer (Class Utama API):**
  - Atribut: `+scene: Scene`, `+gpu_context: OpenGLContext`
  - Metode: `+render()`
  - Hubungan: Mengelola (manages) objek kelas TileGrid, serta memegang referensi asosiasi ke kelas Scene.

- **Scene (Container Objek):**
  - Atribut: `+paths: List<Path>`
  - Metode: `+add_path(data: Path)`
  - Hubungan: Bertindak sebagai agregator yang memiliki (contains) satu atau banyak instansiasi objek kelas Path.

- **Path (Deskripsi Geometri Lintasan):**
  - Atribut: `+segments: List<BezierCurve>`, `+fill_color: Color`
  - Metode: `+get_bounding_box(): Rect`

- **TileGrid (Struktur Data Spasial):**
  - Atribut: `+width: int`, `+height: int`, `+tiles: List<Tile>`
  - Metode: `+perform_preprocessing()`
  - Hubungan: Tersusun atas komposisi (contains) kumpulan elemen objek kelas Tile.

- **Tile (Unit Pemrosesan Raster):**
  - Atribut: `+x_index: int`, `+y_index: int`, `+winding_number: int`, `+curves: List<CurveRef>`, `+type: TileType`
  - Metode: `+ray_shoot(): int`
  - Hubungan: Memegang referensi indeks pointer (references) ke kelas Path guna menghindari duplikasi memori, serta memanfaatkan (uses) definisi nilai dari tipe enumerasi TileType.

- **TileType (Enumeration):**
  - Nilai Literal: `EMPTY`, `INTERIOR`, `EDGE`

## 3.5 Perancangan Algoritma

Pustaka ini memperlakukan seluruh bentuk geometri dua dimensi sebagai komposisi kurva parametrik terintegrasi yang ditransformasikan secara analitis menjadi fungsi implisit. Representasi implisit mendefinisikan kurva bukan sebagai fungsi berbasis waktu $r(t)$, melainkan sebagai batas geometris (geometric boundary) yang memisahkan bidang dua dimensi menjadi wilayah interior (dalam) dan eksterior (luar) yang memenuhi persamaan umum:

$$C(x, y) = 0$$

**1. Formulasi Kurva Primitif Linear**

Untuk segmen primitif garis lurus yang menghubungkan titik ujung koordinat $(x_0, y_0)$ dan $(x_1, y_1)$, persamaan fungsi implisit diturunkan melalui persamaan garis lurus standar:

$$C_{\text{linear}}(x, y) = ax + by + c = 0$$

Di mana nilai koefisien skalar ditentukan secara deterministik melalui operasi matriks:

$$a = y_0 - y_1$$
$$b = x_1 - x_0$$
$$c = x_0 y_1 - x_1 y_0$$

**2. Formulasi Kurva Kuadratik Kanonik**

Setiap kurva Bézier kuadratik memiliki deskripsi fungsi parametrik yang ditentukan oleh tiga titik kontrol $(P_0, P_1, P_2)$. Guna mereduksi biaya operasi aritmatika per piksel di dalam Fragment Shader, pustaka ini menerapkan teorema transformasi affine (Farin, 2002). Pemetaan affine mentransformasikan ruang koordinat lokal kurva kuadratik arbitrer ke dalam ruang koordinat parabola kanonik tunggal pada fase preprocessing CPU. Setelah pemetaan matriks transformasi terunggah ke GPU, evaluasi kedudukan piksel cukup dievaluasi menggunakan representasi fungsi implisit parabola Loop-Blinn yang sangat efisien:

$$f(u, v) = u - v^2 = 0$$

Evaluasi tanda matematika dari fungsi $f(u, v)$ menentukan klasifikasi apakah koordinat piksel berada di bawah atau di atas lengkungan kurva kuadratik tanpa memerlukan perkalian silang (cross-terms) derajat tinggi.

**3. Formulasi Kurva Kubik Berbasis Aljabar Geometris Proyektif**

Untuk penanganan kurva Bézier kubik yang memiliki sifat kompleksitas tinggi (kemungkinan memiliki titik balik serpentine, loop, atau cusp), pustaka ini mengadopsi pendekatan formulasi Contrast Rendering berbasis Projective Geometric Algebra (PPGA). Titik-titik kontrol kurva kubik dievaluasi di CPU untuk membangun empat fungsi bidang berbobot linear (weight planes) yang direpresentasikan sebagai fungsi koordinat $w_0(p)$, $w_1(p)$, $w_2(p)$, dan $w_3(p)$. Fungsi implisit homogen untuk piksel uji $p(x, y, 1)$ didefinisikan sebagai ekspresi:

$$f(p) = w_0(p)^3 - w_1(p) \cdot w_2(p) \cdot w_3(p) = 0$$

Shader GPU mengevaluasi interpolasi linear dari keempat bidang koordinat tersebut, meminimalkan kebutuhan komputasi akar polinomial penuh di sisi Fragment Shader.

**4. Strategi Pipeline Berbasis Tiling**

Pustaka ini menggantikan metode triangulasi global (seperti Constrained Delaunay Triangulation) yang memiliki batasan performa sekuensial tinggi di CPU dengan arsitektur berbasis tiling. Wilayah layar target (viewport) dibagi secara spasial menjadi kisi-kisi ubin (grid tile) homogen dengan ukuran tetap $16 \times 16$ piksel. Alur algoritma preprocessing hibrida yang dieksekusi secara paralel di CPU dijabarkan sebagai berikut:

```
ALGORITMA PREPROCESSING_TILING_PARALEL
Input: Daftar Objek Path dari berkas SVG, Dimensi Layar (Width, Height)
Output: Vertex Buffer Object berisi daftar Quad Tiles terklasifikasi

1.  Inisialisasi Grid: Hitung jumlah tile horizontal dan vertikal.
2.  ALOKASIKAN array dua dimensi GridTile berukuran (Width/16) x (Height/16).
3.  FOR EACH Path DALAM Daftar Objek Path SECARA PARALEL (Menggunakan Rayon) DO
4.    FOR EACH Kurva_Primitif DALAM Path DO
5.      Hitung Bounding Box dari Kurva_Primitif.
6.      Tentukan rentang indeks tile (min_tile_x sampai max_tile_x,
        min_tile_y sampai max_tile_y) yang berpotongan dengan Bounding Box.
7.      FOR y FROM min_tile_y TO max_tile_y DO
8.        FOR x FROM min_tile_x TO max_tile_x DO
9.          Masukkan referensi Kurva_Primitif ke dalam GridTile[x, y].curves_list.
10.       END FOR
11.     END FOR
12.   END FOR
13. END FOR
14. FOR EACH Tile DALAM GridTile SECARA PARALEL DO
15.   IF Tile.curves_list KOSONG THEN
16.     Tile.type = EMPTY
17.   ELSE
18.     // Eksekusi mekanisme Ray Shooting di CPU
19.     Winding_Number =
        HITUNG_WINDING_NUMBER_RAY_SHOOT(Tile.Sisi_Atas_Koordinat)
20.     Tile.winding_number = Winding_Number
21.
22.     IF Ada kurva yang benar-benar memotong geometri internal kotak Tile THEN
23.       Tile.type = EDGE
24.     ELSE IF Winding_Number != 0 THEN
25.       Tile.type = INTERIOR
26.     ELSE
27.       Tile.type = EMPTY
28.     END IF
29.   END IF
30. END FOR
31. Konstruksi Vertex Buffer: Kumpulkan semua Tile berstatus INTERIOR dan EDGE.
32. GENERATE koordinat quad (4 vertex) untuk setiap ubin valid ke dalam array biner siap unggah.
33. RETURN Array Vertex Buffer.
```

Kalkulasi fungsi `HITUNG_WINDING_NUMBER_RAY_SHOOT` dilakukan dengan menembakkan sinar imajiner vertikal ke arah atas dari koordinat tengah atas batas ubin. Setiap interseksi antara sinar dengan kurva geometri akan memperbarui nilai winding number ($\omega$) secara analitis: jika arah lintasan kurva memotong sinar dari kiri ke kanan (atau counter-clockwise) bernilai $+1$, dan jika melintas sebaliknya (clockwise) bernilai $-1$.

Klasifikasi akhir mengoptimalkan kerja GPU secara drastis: ubin berstatus EMPTY langsung dieliminasi dari alur transfer memori, ubin INTERIOR digambar oleh GPU sebagai dua buah segitiga (quad) dengan pewarnaan solid fill langsung tanpa evaluasi matematika, dan hanya ubin berstatus EDGE yang memicu komputasi evaluasi fungsi implisit kurva pada sirkuit Fragment Shader GPU per piksel.

## 3.6 Perancangan Layar

Sistem yang dirancang dan dibangun dalam penelitian ini sepenuhnya berupa pustaka backend perangkat lunak (software library/API) murni yang menyediakan fungsionalitas rendering modular komputasi grafis. Pustaka ini mengekspos fungsi-fungsi programatik untuk dipanggil oleh aplikasi induk, sehingga sistem ini tidak memiliki komponen antarmuka grafis pengguna langsung (User Interface / UI Layout) maupun desain cetak layar (mockup).

Namun, sebagai instrumen validasi visual ilmiah, verifikasi fungsionalitas pengujian pustaka diintegrasikan ke dalam sebuah harness program pengujian minimalis berbasis kanvas rendering kosong (blank test canvas execution). Struktur jendela pengujian ini didefinisikan sebagai berikut:

- Jendela uji berupa sebuah bingkai jendela grafis kosong beresolusi variabel (contoh target uji default: $1920 \times 1080$ piksel) yang dialokasikan oleh subsistem windowing sistem operasi.
- Area kanvas pengujian berfungsi sebagai target frame buffer tunggal tempat pustaka mengalirkan hasil rasterisasi piksel akhir dari sirkuit OpenGL ES.
- Tidak terdapat elemen interaksi visual seperti tombol, menu kontrol, slider, atau teks status di dalam kanvas tersebut guna menjaga netralitas pengukuran performa frame time tanpa interferensi proses rendering komponen UI pihak ketiga.

## 3.7 Perancangan Database File

Arsitektur pustaka rendering hibrida ini dirancang untuk beroperasi secara stateless dengan performa kecepatan ekstrem dan latensi rendering per-bingkai seminimal mungkin. Demi mencapai tujuan performa tersebut, sistem ini tidak mengimplementasikan subsistem database persisten atau penyimpanan berkas database lokal (seperti SQLite atau struktur file terindeks sejenis).

Seluruh daur hidup pengelolaan data geometri dan spasial dikelola secara dinamis di dalam memori akses acak (Volatile Random Access Memory / RAM) menggunakan struktur data internal bahasa Rust yang efisien selama durasi eksekusi aplikasi berjalan. Detail pemetaan alokasi memori internal diatur dengan spesifikasi sebagai berikut:

1. **Geometry Array Storage:** Lintasan geometris hasil ekstraksi parser SVG disimpan ke dalam struktur vektor memori linear berurutan (`Vec<Path>`) untuk menjamin sifat cache locality saat diakses oleh loop thread paralel di CPU.
2. **Transient Spatial Grid Buffer:** Hasil pemetaan koordinat dan tipe ubin disimpan dalam array memori linear dua dimensi berukuran tetap yang dialokasikan ulang per bingkai, memastikan tidak ada biaya overhead penulisan ke media penyimpanan sekunder (disk).
3. **GPU Vertex Array Memory Layout:** Data hasil preprocessing langsung dikonversikan menjadi representasi biner kompak berupa struktur koordinat quad mengambang (floating-point array) yang langsung disalin ke objek memori GPU (Vertex Buffer Object / VBO dan Vertex Array Object / VAO) melalui bus PCIe, lalu segera dibebaskan dari memori RAM setelah siklus rendering bingkai tersebut selesai.
