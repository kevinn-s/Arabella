# Requirements Document

## Introduction

Dokumen ini menetapkan persyaratan revisi terhadap berkas `Skripsi/bab3_metodologi.md` agar seluruh klaim metodologi pada Bab 3 sinkron dengan implementasi source code Arabella di direktori `src/` dan konfigurasi `Cargo.toml`. Output akhir spec adalah satu berkas Markdown tunggal `Skripsi/bab3_metodologi.md` yang telah ditulis ulang sebagiannya, mempertahankan struktur akademik kampus (Subbab 3.1 sampai 3.7) sekaligus menghilangkan ketidaksesuaian substantif yang telah didokumentasikan pada laporan analisis (`Skripsi/analisis_project_dan_skripsi.md`, Bagian D.2 dan G).

Spec ini adalah spec dokumentasi akademik. Yang ditulis adalah konten naratif, bukan kode. Source code Arabella berfungsi sebagai sumber kebenaran (source of truth) yang tidak boleh dimodifikasi. Setiap klaim teknis dalam Bab 3 yang telah direvisi harus dapat dilacak ulang ke berkas atau simbol kode di `src/` atau ke entri di `Cargo.toml`.

Revisi mempertahankan terminologi lintas-bab yang sudah konsisten dengan Bab 1 dan Bab 2 (misalnya "pipeline hibrida", "rasterization pipeline tradisional", "preprocessing", "viewport", "winding number"), dan mempertahankan gaya bahasa akademik formal Indonesia yang sudah berlaku di Bab 1 sampai Bab 3.

## Glossary

- **Bab_3**: Berkas `Skripsi/bab3_metodologi.md` versi pasca-revisi.
- **Bab_3_Lama**: Berkas `Skripsi/bab3_metodologi.md` versi sebelum revisi (versi git pra-spec).
- **Source_Of_Truth**: Source code Arabella di direktori `src/` beserta `Cargo.toml` di akar repositori.
- **Laporan_Analisis**: Berkas `Skripsi/analisis_project_dan_skripsi.md` yang mendokumentasikan ketidaksesuaian Bab 3 lama terhadap source code, beserta struktur Bab 3 yang diusulkan.
- **Penulis**: Subjek aktif yang menulis ulang Bab 3. Dalam dokumen requirements ini "Penulis" adalah sistem agen yang melaksanakan revisi.
- **Klaim_Teknis**: Pernyataan dalam Bab 3 yang menyebut salah satu dari: (a) nama algoritma atau struktur data, (b) nilai parameter numerik konkret (termasuk dimensi ubin, format fixed-point, jumlah byte), (c) nama berkas, fungsi, struct, trait, modul, atau konstanta dalam kode, atau (d) perilaku runtime spesifik dari pustaka Arabella.
- **Rujukan_Kode**: Kombinasi jalur berkas relatif terhadap akar repositori (misalnya `src/blocks.rs`) ditambah salah satu dari: nama fungsi (misalnya `record_per_scanline_crossings`), nama konstanta (misalnya `TILE_W`), atau rentang baris pada format `path:start-end`.
- **Istilah_Terlarang**: Daftar istilah yang berasal dari narasi Bab 3 lama yang tidak sesuai dengan implementasi dan harus hilang dari Bab 3, yaitu: "Ray Shooting", "Ray Shoot", "ray shooting", "ray shoot", "EMPTY", "INTERIOR", "EDGE" (saat dipakai sebagai label tipe ubin), "TileType", "winding_number" sebagai field skalar pada Tile, "fungsi implisit linear ax+by+c" sebagai narasi tahap GPU, "fungsi implisit kuadratik kanonik f(u,v)=u-v²" sebagai narasi tahap GPU, "fungsi implisit kubik PPGA" atau "Projective Geometric Algebra" sebagai narasi algoritma Arabella, "C(x,y)=0" sebagai persamaan inti Arabella, dan "OpenGL ES 3.0 yang ditranspilasikan ke WebGL 2.0".
- **Istilah_Wajib**: Daftar istilah konkret yang harus muncul di Bab 3 karena merepresentasikan implementasi sebenarnya, yaitu: "WebGL 2.0", "Rust edisi 2024", "F24Dot8" (atau "24.8 fixed-point"), "8.8 fixed-point", "DDA" atau "Digital Differential Analyzer", "outer DDA", "inner DDA", "signed-area" atau "akumulasi signed area" per scanline, "backdrop", "propagasi backdrop kiri-ke-kanan" (atau frasa setara), "flattening", "midpoint subdivision" atau "De Casteljau midpoint subdivision", "cubic-to-quadratic", "line_box" sebagai nama fungsi GPU, "trapezoidal" untuk integral cakupan piksel, "fearless_simd", "lyon_path", "lyon_geom", "kurbo", "peniko", "roxmltree", "bytemuck", "thiserror", "hashbrown", "smallvec", "fill rule NonZero" dan "fill rule EvenOdd", "16×8" sebagai ukuran ubin Arabella, dan "Rayon" yang dideskripsikan sebagai feature opsional.
- **Subbab_Wajib**: Heading subbab yang harus hadir di Bab 3 dengan format `## 3.X` dan `### 3.X.Y`, mengikuti struktur Laporan_Analisis Bagian G, yaitu: 3.1, 3.2 (3.2.1, 3.2.2, 3.2.3), 3.3 (3.3.1), 3.4 (3.4.1, 3.4.2, 3.4.3, 3.4.4), 3.5, 3.6, 3.7.
- **Konsistensi_Internal**: Properti bahwa narasi pada Subbab 3.4.2 (UC-03), 3.4.3 (Sequence Diagram), 3.4.4 (Class Diagram), dan 3.5 (Perancangan Algoritma) saling merujuk objek, langkah, dan istilah yang sama tanpa kontradiksi.

## Requirements

### Requirement 1: Identitas Berkas Output

**User Story:** Sebagai dosen pembimbing, saya ingin satu berkas tunggal `Skripsi/bab3_metodologi.md` hasil revisi menggantikan berkas lama, sehingga saya tidak perlu menelusuri beberapa lokasi untuk membaca Bab 3 yang mutakhir.

#### Acceptance Criteria

1. THE Penulis SHALL menulis hasil revisi pada satu berkas tunggal yaitu `Skripsi/bab3_metodologi.md` relatif terhadap akar repositori, sehingga di seluruh repositori hanya terdapat satu jalur berkas yang menyimpan Bab 3 versi pasca-revisi.
2. THE Penulis SHALL menyimpan berkas hasil revisi dengan ekstensi `.md` dan menggunakan sintaks Markdown CommonMark yang dapat diurai (parsable) tanpa galat oleh parser CommonMark, sebagaimana sudah berlaku pada Bab_3_Lama.
3. THE Penulis SHALL TIDAK membuat berkas tambahan pada lokasi lain di repositori yang memuat baris pertama `# BAB 3 METODE PENELITIAN` ataupun yang memiliki nama berkas mengandung substring "bab3", "bab_3", atau "bab-3" selain `Skripsi/bab3_metodologi.md`.
4. THE Penulis SHALL meletakkan heading utama `# BAB 3 METODE PENELITIAN` pada baris pertama berkas hasil revisi dengan kapitalisasi penuh seperti tertulis, satu spasi tunggal antar kata, tanpa karakter tambahan sebelum atau sesudah teks heading pada baris yang sama.
5. THE Penulis SHALL menulis seluruh hasil revisi langsung ke berkas `Skripsi/bab3_metodologi.md` sebagai versi final pasca-revisi, dan SHALL TIDAK meninggalkan salinan utuh, salinan parsial, cadangan, atau draf alternatif Bab 3 di lokasi manapun pada repositori.

### Requirement 2: Kelengkapan Struktural Subbab Wajib

**User Story:** Sebagai mahasiswa yang menulis Bab 3 sesuai panduan kampus, saya ingin seluruh subbab wajib hadir dengan heading yang konsisten, sehingga struktur Bab 3 tetap mengikuti panduan akademik.

#### Acceptance Criteria

1. THE Bab_3 SHALL memuat heading `## 3.1 Diagram Alir Kerangka Berpikir` tepat satu kali pada level heading 2 (`##`) dengan teks persis sama secara case-sensitive, termasuk nomor subbab dan spasi tunggal antar kata.
2. THE Bab_3 SHALL memuat heading `## 3.2 Analisis Kebutuhan` tepat satu kali pada level heading 2, beserta subheading `### 3.2.1 Analisis User`, `### 3.2.2 Analisis Aplikasi Sejenis`, dan `### 3.2.3 Rumusan dan Solusi Kebutuhan`, masing-masing tepat satu kali pada level heading 3 (`###`) dengan teks persis sama secara case-sensitive.
3. THE Bab_3 SHALL memuat heading `## 3.3 Perancangan Aplikasi` tepat satu kali pada level heading 2, beserta subheading `### 3.3.1 Spesifikasi Aplikasi` tepat satu kali pada level heading 3, dengan teks persis sama secara case-sensitive.
4. THE Bab_3 SHALL memuat heading `## 3.4 Perancangan Sistem` tepat satu kali pada level heading 2, beserta subheading `### 3.4.1 Use Case Diagram`, `### 3.4.2 Use Case Description`, `### 3.4.3 Sequence Diagram`, dan `### 3.4.4 Class Diagram`, masing-masing tepat satu kali pada level heading 3 dengan teks persis sama secara case-sensitive.
5. THE Bab_3 SHALL memuat heading `## 3.5 Perancangan Algoritma` tepat satu kali pada level heading 2 dengan teks persis sama secara case-sensitive.
6. THE Bab_3 SHALL memuat heading `## 3.6 Perancangan Layar` tepat satu kali pada level heading 2 dengan teks persis sama secara case-sensitive.
7. THE Bab_3 SHALL memuat heading `## 3.7 Perancangan Database File` tepat satu kali pada level heading 2 dengan teks persis sama secara case-sensitive.
8. THE Bab_3 SHALL menyusun heading subbab level 2 pada urutan menaik monotonik 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7 tanpa nomor yang dilewati, diulang, atau dibalik urutannya.
9. THE Bab_3 SHALL menempatkan setiap subheading `### 3.x.y` setelah heading induknya `## 3.x` dan sebelum heading subbab level 2 berikutnya, dengan urutan `y` menaik monotonik mulai dari 1 tanpa nomor yang dilewati atau diulang.

### Requirement 3: Eliminasi Istilah Terlarang

**User Story:** Sebagai pembaca yang akan memverifikasi konsistensi Bab 3 dengan source code, saya ingin seluruh istilah dari narasi lama yang tidak ada di kode dihapus, sehingga skripsi tidak menyatakan klaim yang bertentangan dengan implementasi.

#### Acceptance Criteria

1. THE Bab_3 SHALL TIDAK memuat substring "Ray Shooting", "Ray Shoot", "ray shooting", maupun "ray shoot" pada bagian apapun dari berkas (heading, paragraf, caption, item daftar, blok kode, blok persamaan), dengan pencocokan case-insensitive.
2. THE Bab_3 SHALL TIDAK memuat token kata utuh "EMPTY", "INTERIOR", maupun "EDGE" sebagai label tipe ubin Arabella, dengan pencocokan case-sensitive pada heading, paragraf, caption, item daftar, blok kode, dan blok persamaan; istilah-istilah tersebut hanya boleh muncul jika digunakan dalam konteks generik di luar klasifikasi tipe ubin Arabella, dan dalam kasus tersebut wajib dijelaskan secara eksplisit.
3. THE Bab_3 SHALL TIDAK memuat token kata utuh "TileType" sebagai nama enum atau tipe data Arabella, dengan pencocokan case-sensitive di seluruh berkas.
4. THE Bab_3 SHALL TIDAK memuat string "winding_number" sebagai nama field skalar pada deskripsi struct atau class Tile Arabella, dengan pencocokan case-sensitive; penyebutan istilah konseptual "winding number" (dengan spasi, bukan underscore) tetap diperbolehkan.
5. THE Bab_3 SHALL TIDAK memuat persamaan dengan bentuk "ax+by+c=0", "ax + by + c = 0", maupun "a x + b y + c = 0" (semua varian dengan atau tanpa spasi), maupun frasa "fungsi implisit linear" sebagai deskripsi tahap rasterisasi GPU Arabella.
6. THE Bab_3 SHALL TIDAK memuat persamaan dengan bentuk "u-v^2=0", "u - v^2 = 0", "u-v²=0", "u - v² = 0", maupun "f(u,v) = u - v²" (semua varian dengan atau tanpa spasi), maupun frasa "fungsi implisit kuadratik kanonik" sebagai deskripsi tahap rasterisasi GPU Arabella.
7. THE Bab_3 SHALL TIDAK memuat frasa "fungsi implisit kubik PPGA", "PPGA", "Projective Geometric Algebra", maupun persamaan dengan bentuk "w_0^3 - w_1 w_2 w_3", "w0^3-w1w2w3", atau "f(p) = w_0(p)^3 - w_1(p) · w_2(p) · w_3(p)" (semua varian dengan atau tanpa spasi) sebagai deskripsi algoritma Arabella.
8. THE Bab_3 SHALL TIDAK memuat persamaan "C(x, y) = 0" maupun "C(x,y)=0" sebagai persamaan inti rendering Arabella, dengan pencocokan case-sensitive.
9. THE Bab_3 SHALL TIDAK memuat klaim "OpenGL ES 3.0 yang ditranspilasikan ke WebGL 2.0", "ditranspilasikan ke WebGL", "transpilasi OpenGL ES", maupun klaim setara yang menempatkan OpenGL ES sebagai jalur native Arabella atau sebagai sumber transpilasi shader.
10. THE Bab_3 SHALL TIDAK memuat klaim "Rust edisi 2021", "edisi 2021", "Rust Edition 2021", maupun "edition = \"2021\"" pada konteks edisi bahasa pemrograman Arabella.
11. THE Penulis SHALL memverifikasi bahwa kriteria 1 sampai 10 terpenuhi dengan menjalankan pencarian teks (case-insensitive untuk kriteria 1, case-sensitive untuk kriteria 2 sampai 10) terhadap seluruh isi `Skripsi/bab3_metodologi.md`, dan kondisi PASS adalah ketika tidak ada satupun kemunculan substring yang dilarang pada konteks yang dilarang.

### Requirement 4: Spesifikasi Aplikasi (Subbab 3.3.1) Mencerminkan Cargo.toml

**User Story:** Sebagai penguji yang akan membuka `Cargo.toml`, saya ingin daftar pustaka pada Subbab 3.3.1 cocok dengan dependensi yang benar-benar dideklarasikan, sehingga tidak ada klaim pustaka fiktif.

#### Acceptance Criteria

1. THE Bab_3 SHALL menyatakan secara eksplisit pada Subbab 3.3.1 bahwa bahasa pemrograman utama adalah "Rust edisi 2024" (mengacu pada `edition = "2024"` di `Cargo.toml`), tanpa menyebut edisi Rust lain pada konteks bahasa utama Arabella.
2. THE Bab_3 SHALL menyatakan pada Subbab 3.3.1 bahwa target eksekusi utama adalah `wasm32-unknown-unknown` pada lingkungan peramban dengan API grafis WebGL 2.0, dengan rujukan pada konfigurasi `[target.'cfg(target_arch = "wasm32")'.dependencies]` di `Cargo.toml`.
3. THE Bab_3 SHALL mendaftarkan kesepuluh nama crate berikut pada Subbab 3.3.1 sebagai dependensi langsung pustaka Arabella, dengan ejaan persis seperti di `Cargo.toml`: `fearless_simd`, `lyon_path`, `lyon_geom`, `kurbo`, `peniko`, `roxmltree`, `bytemuck`, `thiserror`, `hashbrown`, dan `smallvec`. Ketidakhadiran salah satu dari sepuluh nama tersebut menyebabkan kriteria ini tidak terpenuhi.
4. THE Bab_3 SHALL mendeskripsikan Rayon pada Subbab 3.3.1 atau Subbab 3.5 sebagai dependensi opsional di balik feature flag `multithreading` (mengacu pada `rayon = { version = "1.11.0", optional = true }` dan `multithreading = ["std", "dep:rayon", "dep:thread_local"]` di `Cargo.toml`), serta menyatakan bahwa feature tersebut belum dipanggil pada hot path pipeline pra-pemrosesan implementasi saat ini.
5. THE Bab_3 SHALL TIDAK menyebut "OpenMP", "C++ Threads", maupun "wasm-bindgen-rayon" sebagai pustaka konkurensi yang digunakan oleh Arabella.
6. WHERE Subbab 3.3.1 menyebut format data masukan, THE Bab_3 SHALL mendeskripsikan dukungan SVG sebagai subset minimal yang ditangani oleh `src/pico_svg.rs` (elemen `g`, `path`, atribut `fill`, `stroke`, `stroke-width`, `transform`), tanpa mengklaim dukungan SVG 1.1 Core penuh.

### Requirement 5: UC-03 Mencerminkan Pipeline DDA dan Akumulator Signed-Area

**User Story:** Sebagai pembaca Subbab 3.4.2, saya ingin alur UC-03 menggambarkan tahap pra-pemrosesan dan rasterisasi yang sebenarnya, sehingga deskripsi use case selaras dengan kode di `src/builder.rs`, `src/blocks.rs`, dan shader fragment.

#### Acceptance Criteria

1. THE Bab_3 SHALL menyusun ulang Alur Peristiwa Inti UC-03 pada Subbab 3.4.2 sehingga memuat tepat enam tahap berurutan dengan urutan: (a) flattening kurva ke segmen garis pada format F24Dot8, (b) outer DDA yang membagi setiap segmen lintas baris ubin, (c) inner DDA yang membagi lintas kolom ubin, (d) akumulasi signed-area per scanline pada format 8.8 fixed-point, (e) emisi ubin yang nontrivial, dan (f) propagasi backdrop kiri-ke-kanan saat emisi.
2. THE Bab_3 SHALL menggambarkan tahap rasterisasi GPU pada UC-03 sebagai dua langkah berurutan: (i) eksekusi vertex shader instanced quad untuk setiap ubin yang nontrivial, kemudian (ii) eksekusi fragment shader analitik tunggal yang sama untuk seluruh ubin tanpa cabang kondisional berbasis tipe ubin.
3. THE Bab_3 SHALL TIDAK memuat cabang "Render Warna Solid" untuk tipe ubin INTERIOR, cabang "Evaluasi Fungsi Implisit" untuk tipe ubin EDGE, maupun penanda klasifikasi tipe ubin lainnya pada deskripsi UC-03 di Subbab 3.4.2.
4. THE Bab_3 SHALL menyebutkan minimal dua rujukan kode pada Subbab 3.4.2 atau Subbab 3.5 dengan format `berkas:simbol`, yaitu salah satu dari `src/blocks.rs:bin_line` atau `src/blocks.rs:Blocks::build_block` untuk tahap binning DDA, dan `src/blocks.rs:record_per_scanline_crossings` untuk tahap akumulator signed-area.
5. WHERE UC-03 menyebut fill rule, THE Bab_3 SHALL menyebut dua aturan yaitu NonZero dan EvenOdd dengan minimal satu kalimat penjelas formula winding untuk masing-masing aturan (NonZero memakai operasi `clamp(abs(winding), 0, 1)`, EvenOdd memakai operasi `1 - abs(mod(abs(winding), 2) - 1)` atau formulasi setara), serta menyatakan bahwa kedua aturan diterapkan pada `src/render/shaders/render_tile.frag`.
6. WHERE Bab_3 menyajikan diagram pipeline atau tabel pipeline pada UC-03, THE Bab_3 SHALL menandai tahap (a) sampai (f) pada AC 5.1 sebagai tahap CPU dan tahap (i) sampai (ii) pada AC 5.2 sebagai tahap GPU, sehingga pembagian beban kerja CPU-GPU dapat diidentifikasi secara visual pada diagram atau tabel tersebut.

### Requirement 6: Sequence Diagram Selaras dengan UC-03 Revisi

**User Story:** Sebagai pembaca Subbab 3.4.3, saya ingin sequence diagram tidak lagi memuat percabangan berdasarkan tipe ubin, sehingga sequence diagram konsisten dengan UC-03 yang baru.

#### Acceptance Criteria

1. THE Bab_3 SHALL menulis ulang Subbab 3.4.3 sehingga tidak memuat blok alternatif (`alt`), blok opsional (`opt`), maupun blok perulangan (`loop`) yang bercabang berdasarkan tipe ubin (termasuk `alt: Tipe == EDGE`, `alt: Tipe == INTERIOR`, atau varian sintaks lain dengan label tipe ubin).
2. THE Bab_3 SHALL mendeskripsikan urutan pesan pada Subbab 3.4.3 sebagai tepat lima tahap berurutan dengan partisipan yang dinamai eksplisit (Aplikasi Utama, Scene, Builder, WebGlRenderer, GPU): (i) Aplikasi Utama memanggil `Scene::fill` atau `Scene::stroke`; (ii) Scene memicu `Builder::build_path` untuk flattening dan binning DDA; (iii) Builder memicu `Builder::generate_tiles` untuk propagasi backdrop kiri-ke-kanan dan emisi seluruh ubin nontrivial tanpa percabangan tipe ubin; (iv) Aplikasi Utama menyerahkan vertex buffer dan tekstur segmen ke `WebGlRenderer::render`; (v) GPU mengeksekusi vertex shader instanced quad lalu fragment shader analitik tunggal.
3. THE Bab_3 SHALL TIDAK memuat pesan apapun yang menandai tipe ubin pada Subbab 3.4.3, termasuk "Tandai Tipe: INTERIOR", "Tandai Tipe EDGE", "Set Type = EMPTY", maupun varian linguistik lain yang menetapkan label tipe ubin.
4. THE Penulis SHALL mengganti seluruh konten Subbab 3.4.3 versi Bab_3_Lama dengan urutan pesan baru yang dideskripsikan pada AC 6.2.
5. IF setelah penggantian masih terdapat kalimat sisa pada Subbab 3.4.3 yang merujuk percabangan tipe ubin, klasifikasi EMPTY/INTERIOR/EDGE, atau ray shooting, THEN THE Penulis SHALL menghapus kalimat tersebut sebelum berkas dianggap final.

### Requirement 7: Class Diagram Mencerminkan Struct Aktual

**User Story:** Sebagai pembaca Subbab 3.4.4, saya ingin class diagram memuat tipe data yang benar-benar ada di source code, sehingga diagram dapat ditelusuri ke berkas Rust.

#### Acceptance Criteria

1. THE Bab_3 SHALL mendaftarkan komponen `Scene`, `Builder`, `CoverStorage`, `Block`, `Blocks`, `TileBounds`, `Tile`, `WebGlRenderer`, dan `PicoSvg` sebagai kotak kelas UML pada Subbab 3.4.4, masing-masing mencantumkan nama struct dan daftar field utama beserta tipenya seperti yang dideklarasikan pada source code.
2. THE Bab_3 SHALL mendeskripsikan struct `Tile` pada Subbab 3.4.4 dengan field `x`, `y`, `width`, `height`, `backdrop` (array delapan elemen 16-bit untuk delapan scanline pada satu ubin), `segments` (dua elemen yang merepresentasikan offset segmen dan jumlah segmen), `payload`, `paint_and_rect_flag`, dan `depth_index`.
3. THE Bab_3 SHALL TIDAK menampilkan enum `TileType`, field `winding_number` skalar, field `curves: List<CurveRef>`, maupun method `ray_shoot()` pada class diagram di Subbab 3.4.4.
4. THE Bab_3 SHALL mencantumkan rujukan kode untuk setiap kotak kelas yang didaftarkan pada AC 7.1: `src/scene.rs` untuk `Scene`, `src/builder.rs` untuk `Builder` dan `CoverStorage`, `src/blocks.rs` untuk `Block`, `Blocks`, dan `TileBounds`, `src/tile.rs` untuk `Tile`, `src/render/webgl.rs` untuk `WebGlRenderer`, dan `src/pico_svg.rs` untuk `PicoSvg`.
5. THE Bab_3 SHALL mendeskripsikan minimal tiga relasi antar kelas pada Subbab 3.4.4 menggunakan notasi UML standar (komposisi, agregasi, atau asosiasi) yang konsisten dengan deklarasi field di source code, misalnya `Scene` mengkomposisi `Builder` (field `builder: Builder` di `src/scene.rs`), `Builder` mengkomposisi `Blocks` dan `CoverStorage`, dan `WebGlRenderer` mengasosiasi `Scene` melalui parameter `Scene` pada method `WebGlRenderer::render`.

### Requirement 8: Perancangan Algoritma (Subbab 3.5) Mencerminkan Pipeline Aktual

**User Story:** Sebagai dosen penguji, saya ingin Subbab 3.5 mendeskripsikan algoritma yang benar-benar dijalankan oleh kode, sehingga skripsi dapat dipertahankan saat sidang.

#### Acceptance Criteria

1. THE Bab_3 SHALL menghapus seluruh narasi "fungsi implisit linear ax+by+c=0", "fungsi implisit kuadratik kanonik f(u,v)=u-v²", dan "fungsi implisit kubik PPGA w_0³-w_1 w_2 w_3" dari Subbab 3.5, dan TIDAK memuat substring tersebut maupun variannya pada Subbab 3.5.
2. THE Bab_3 SHALL menyajikan Subbab 3.5 sebagai enam paragraf atau sub-bagian terpisah yang berjalan berurutan, masing-masing menyebutkan secara eksplisit minimal satu nama berkas sumber atau nama fungsi: (a) flattening kurva (cubic ke quadratic gaya Vello, lalu quadratic ke segmen garis melalui midpoint subdivision gaya Blaze) dengan rujukan ke `src/path.rs` dan `src/flatten.rs`; (b) tile binning DDA dua tahap dengan rujukan ke `src/blocks.rs`; (c) akumulator signed-area per scanline 8.8 fixed-point dengan rujukan ke fungsi `record_per_scanline_crossings` di `src/blocks.rs`; (d) propagasi backdrop kiri-ke-kanan dengan rujukan ke `Builder::generate_tiles` di `src/builder.rs`; (e) evaluasi cakupan piksel di GPU melalui integral trapezoidal `line_box` dengan rujukan ke `src/render/shaders/render_tile.frag`; (f) penerapan fill rule NonZero (clamp absolute value) serta EvenOdd (triangle wave) pada fragment shader.
3. THE Bab_3 SHALL menyatakan pada Subbab 3.5 bahwa ukuran ubin Arabella adalah 16×8 piksel, dengan rujukan eksplisit ke konstanta `TILE_W = 16` dan `TILE_H = 8` di `src/blocks.rs` (dan/atau `src/builder.rs`).
4. IF Bab_3 memuat penyebutan ukuran ubin Arabella, THEN THE Bab_3 SHALL menggunakan dimensi 16×8 piksel di seluruh Subbab 3.4, 3.5, 3.6, dan 3.7, dan SHALL TIDAK menyatakan ukuran ubin Arabella sebagai 16×16 piksel pada subbab manapun.
5. THE Bab_3 SHALL mendeskripsikan outer DDA pada Subbab 3.5 sebagai pemecahan setiap segmen garis lintas baris ubin pada empat arah diagonal (down-right, down-left, up-right, up-left) ditambah tiga kasus khusus yang ditangani sebagai cabang kode terpisah (single-row, vertikal degenerate, dan horizontal degenerate), dan inner DDA sebagai pemecahan lintas kolom ubin pada empat arah utama (right-down, right-up, left-down, left-up).
6. THE Bab_3 SHALL menyatakan pada Subbab 3.5 bahwa Rayon dideklarasikan sebagai dependensi opsional di `Cargo.toml` namun belum dipanggil pada hot path binning maupun emisi ubin pada implementasi saat ini, dengan konsekuensi bahwa klaim paralelisme CPU yang dijelaskan pada Subbab 3.5 bersifat potensial (kapasitas yang sudah disiapkan), bukan paralelisme yang sudah aktif pada implementasi yang dievaluasi.
7. WHERE Subbab 3.5 menyebut konversi cubic-to-quadratic, THE Bab_3 SHALL menyebutkan secara eksplisit nama fungsi `convert_cubics_to_quadratic_curves` dan/atau `estimate_number_of_quadratic_curves` di `src/path.rs`.

### Requirement 9: Perancangan Layar (Subbab 3.6) Mencerminkan Demo dan Tes Aktual

**User Story:** Sebagai pembaca Subbab 3.6, saya ingin deskripsi layar pengujian sesuai dengan demo `examples/native_webgl/` dan tes `tests/test.rs`, sehingga klaim resolusi konsisten dengan kode.

#### Acceptance Criteria

1. THE Bab_3 SHALL menyatakan pada Subbab 3.6 bahwa demo interaktif (`examples/native_webgl/`) menggunakan resolusi window-fill yang DPR-aware dengan rumus eksplisit `width = inner_width × devicePixelRatio` dan `height = inner_height × devicePixelRatio`, mengacu pada `examples/native_webgl/src/main.rs` (atau lokasi setara pada source code).
2. THE Bab_3 SHALL menyatakan pada Subbab 3.6 bahwa pengujian wasm-bindgen-test pada `tests/test.rs` menggunakan resolusi kanvas tetap 1080×520 piksel (mengacu pada konstanta `W: u16 = 1080` dan `H: u16 = 520` di `tests/test.rs`).
3. THE Bab_3 SHALL TIDAK menyebut resolusi 1920×1080 piksel sebagai resolusi default pengujian Arabella pada Subbab 3.6, dan IF Bab_3_Lama memuat klaim 1920×1080 sebagai resolusi default, THEN THE Penulis SHALL menggantinya dengan resolusi aktual sesuai AC 9.1 dan AC 9.2.
4. THE Bab_3 SHALL mendeskripsikan overlay FPS pada Subbab 3.6 dengan menyebut empat metrik berikut secara enumeratif: (i) waktu pra-pemrosesan CPU dalam milidetik (rentang nilai non-negatif), (ii) waktu render GPU dalam milidetik (rentang nilai non-negatif), (iii) jumlah operasi paint pada frame saat ini (bilangan bulat non-negatif), dan (iv) rasio zoom (bilangan riil positif), dengan rujukan ke fungsi `update_overlay` (atau setara) di `examples/native_webgl/src/lib.rs`.
5. WHERE Subbab 3.6 mencantumkan klaim resolusi atau metrik overlay FPS, THE Bab_3 SHALL menyertakan rujukan kode (`examples/native_webgl/src/main.rs`, `examples/native_webgl/src/lib.rs`, atau `tests/test.rs`) yang konsisten dengan AC 9.1, AC 9.2, dan AC 9.4, sehingga setiap klaim numerik atau metrik dapat ditelusuri ke berkas sumbernya.

### Requirement 10: Perancangan Database File (Subbab 3.7) Mencerminkan Tata Letak Memori Aktual

**User Story:** Sebagai pembaca Subbab 3.7, saya ingin deskripsi tata letak memori cocok dengan struktur data pada source code, sehingga uraian alokasi memori dapat diverifikasi.

#### Acceptance Criteria

1. THE Bab_3 SHALL menyatakan pada Subbab 3.7 bahwa daftar ubin yang siap digambar disimpan sebagai vektor datar `Vec<Tile>` di memori CPU, dengan setiap elemen `Tile` berukuran tepat 44 byte sesuai tata letak `#[repr(C)]` pada `src/tile.rs`.
2. THE Bab_3 SHALL menyatakan pada Subbab 3.7 bahwa segmen garis yang telah dibinning ke ubin diunggah ke GPU sebagai tekstur dengan format `RGBA32F`, di mana satu texel menyimpan empat float `(p0.x, p0.y, p1.x, p1.y)` yang merepresentasikan dua titik ujung satu segmen pada ruang piksel ubin lokal (rentang koordinat di dalam batas dimensi ubin 16×8 piksel).
3. THE Bab_3 SHALL menyatakan pada Subbab 3.7 bahwa setiap ubin diinstance sebagai satu rekord vertex buffer berukuran tepat 44 byte dengan tata letak berurutan: `x` (u16), `y` (u16), `width` (u8), `height` (u8), `_pad` (2 byte padding), `backdrop` (delapan elemen i16 untuk delapan scanline), `segments` (dua elemen yang merepresentasikan offset segmen dan jumlah segmen), `payload` (u32), `paint_and_rect_flag` (u32), dan `depth_index` (u32).
4. THE Bab_3 SHALL menyebutkan rujukan kode `src/render/webgl.rs:initialize_tile_vao` pada Subbab 3.7 sebagai sumber penetapan tata letak vertex divisor 44 byte per tile, dan rujukan kode `src/tile.rs` sebagai sumber definisi struct `Tile`.
5. WHERE Subbab 3.7 menyebut tata letak vertex buffer untuk Tile, THE Bab_3 SHALL mendeskripsikan tata letak tersebut sebagai struct `Tile` dengan campuran tipe u16, u8, i16, u32, dan f32 yang dibagikan sebagai atribut vertex divisor instanced; THE Bab_3 SHALL TIDAK mendeskripsikan tata letak sebagai "koordinat quad mengambang", "floating-point array", maupun "array vertex empat titik tunggal".
6. IF Subbab 3.7 memuat klaim tata letak memori (ukuran byte, format tekstur, urutan field), THEN THE Penulis SHALL menyertakan rujukan kode pada klaim tersebut yang menunjuk berkas `src/tile.rs`, `src/render/webgl.rs`, atau `src/render/common.rs` yang mendefinisikan tata letak yang diklaim.

### Requirement 11: Konsistensi Internal Antar Subbab

**User Story:** Sebagai pembaca Bab 3, saya ingin terminologi pada UC-03, sequence diagram, class diagram, dan algoritma tidak bertentangan satu sama lain, sehingga dokumen dapat dibaca tanpa ambiguitas.

#### Acceptance Criteria

1. WHEN Bab_3 menyebut komponen pipeline yang sama pada Subbab 3.4.2, 3.4.3, 3.4.4, dan 3.5, THE Bab_3 SHALL menggunakan istilah kanonik tunggal untuk setiap komponen dengan minimal mendefinisikannya pada penyebutan pertama, dan SHALL TIDAK menggunakan sinonim, singkatan tidak resmi, atau variasi ejaan untuk komponen tersebut pada subbab lain. Set istilah kanonik minimal mencakup: "binning DDA" (untuk tahap pemecahan segmen lintas ubin), "akumulator signed-area" (untuk akumulator winding 8.8 fixed-point per scanline), "propagasi backdrop" (untuk akumulasi kiri-ke-kanan saat emisi tile), dan "fragment shader" (untuk shader piksel WebGL).
2. THE Bab_3 SHALL TIDAK memuat klasifikasi tiga tipe ubin (EMPTY, INTERIOR, EDGE) pada narasi, diagram (use case, sequence, class), pseudocode, maupun blok algoritma di Subbab 3.4 dan Subbab 3.5.
3. THE Bab_3 SHALL menyatakan pada Subbab 3.4.2 dan Subbab 3.5 bahwa fragment shader Arabella memproses semua ubin nontrivial (sebagaimana didefinisikan pada tahap binning di Subbab 3.4.2) melalui jalur kode tunggal tanpa percabangan kondisional (`if`/`switch`) yang berdasarkan tipe ubin.
4. IF setelah revisi awal masih terdapat penyebutan istilah non-kanonik atau variasi ejaan untuk komponen pipeline pada Subbab 3.4 atau 3.5, THEN THE Penulis SHALL menggantinya dengan istilah kanonik dari AC 11.1 sebelum berkas dianggap final.

### Requirement 12: Konsistensi Terminologi Lintas-Bab

**User Story:** Sebagai pembaca skripsi yang membaca Bab 1 sampai Bab 3 secara berurutan, saya ingin istilah lintas-bab tetap konsisten, sehingga revisi Bab 3 tidak merusak referensi silang dari bab sebelumnya.

#### Acceptance Criteria

1. THE Bab_3 SHALL menggunakan istilah "pipeline hibrida" pada setiap penyebutan arsitektur Arabella secara keseluruhan.
2. THE Bab_3 SHALL menggunakan salah satu dari dua frasa berikut secara konsisten pada setiap penyebutan pipeline rasterisasi GPU yang diadopsi Arabella: "rasterization pipeline tradisional" atau "pipeline rasterisasi konvensional"; frasa selain dua tersebut tidak diperbolehkan untuk merujuk konsep yang sama.
3. THE Bab_3 SHALL memilih satu dari dua varian istilah "preprocessing" atau "pra-pemrosesan" untuk fase CPU, dan menggunakannya secara konsisten dalam satu paragraf atau bagian narasi (tidak mencampur kedua varian dalam satu paragraf).
4. THE Bab_3 SHALL menggunakan istilah "viewport" pada setiap penyebutan wilayah layar target.
5. WHILE Bab_3 menjelaskan kalkulasi winding pada Subbab 3.4.2, 3.4.4, atau 3.5, THE Bab_3 SHALL menggunakan istilah "winding number" sebagai konsep, tanpa menjadikannya nama field skalar pada deklarasi struct atau class Tile.
6. IF terdapat istilah lintas-bab yang sudah didefinisikan pada Bab 1 atau Bab 2 namun tidak terdaftar pada AC 12.1 sampai 12.5, THEN THE Bab_3 SHALL mempertahankan bentuk istilah pertama-kali-didefinisikan tersebut tanpa mengganti dengan sinonim atau variasi.

### Requirement 13: Traceability Setiap Klaim Teknis

**User Story:** Sebagai dosen penguji, saya ingin setiap klaim teknis di Bab 3 dapat saya verifikasi langsung ke berkas kode, sehingga saya dapat memastikan tidak ada klaim fabrikasi.

#### Acceptance Criteria

1. THE Penulis SHALL mendefinisikan "klaim teknis" pada awal Bab 3 (misalnya pada paragraf pengantar Subbab 3.3 atau 3.5) sebagai pernyataan yang menyebut salah satu dari: (a) nama algoritma atau struktur data, (b) nilai parameter numerik konkret (ukuran ubin, format fixed-point, jumlah byte, dimensi resolusi), (c) nama berkas, fungsi, struct, trait, modul, atau konstanta dalam kode, atau (d) perilaku runtime spesifik dari pustaka Arabella.
2. THE Penulis SHALL menggunakan format rujukan kode yang seragam pada setiap klaim teknis di Bab 3, yaitu jalur berkas relatif terhadap akar repositori, ditambah salah satu dari: nama fungsi, nama konstanta, nama struct, atau rentang baris pada format `path:start-end`.
3. WHEN Bab_3 memuat klaim teknis tentang algoritma flattening pada Subbab 3.4.2, 3.4.4, atau 3.5, THE Penulis SHALL menyertakan rujukan kode ke `src/flatten.rs` dan/atau `src/path.rs` dengan menyebut nama fungsi atau rentang baris yang relevan.
4. WHEN Bab_3 memuat klaim teknis tentang DDA tile binning, THE Penulis SHALL menyertakan rujukan kode ke `src/blocks.rs` dengan menyebut nama fungsi (`bin_line`, `Blocks::build_block`, `outer_dda_*`, `inner_dda_*`) atau rentang baris yang relevan.
5. WHEN Bab_3 memuat klaim teknis tentang akumulator signed-area per scanline, THE Penulis SHALL menyertakan nama fungsi `record_per_scanline_crossings` beserta rujukan ke `src/blocks.rs`.
6. WHEN Bab_3 memuat klaim teknis tentang propagasi backdrop dan emisi ubin, THE Penulis SHALL menyertakan rujukan ke fungsi `Builder::generate_tiles` di `src/builder.rs`.
7. WHEN Bab_3 memuat klaim teknis tentang evaluasi cakupan piksel di GPU, THE Penulis SHALL menyertakan nama fungsi `line_box` beserta rujukan ke `src/render/shaders/render_tile.frag`.
8. WHEN Bab_3 memuat klaim teknis tentang renderer WebGL, THE Penulis SHALL menyertakan rujukan kode ke `src/render/webgl.rs` dengan menyebut nama fungsi (`WebGlRenderer::new`, `WebGlRenderer::render`, `initialize_tile_vao`) atau rentang baris yang relevan.
9. WHEN Bab_3 memuat klaim teknis tentang parser SVG, THE Penulis SHALL menyertakan rujukan kode ke `src/pico_svg.rs` dengan menyebut nama fungsi atau rentang baris yang relevan.
10. WHEN Bab_3 memuat klaim parameter numerik (misalnya 16, 8, 256, 8.8, F24Dot8, 44 byte), THE Penulis SHALL menyertakan rujukan kode ke berkas yang mendefinisikan parameter tersebut dengan menyebut nama konstanta, nama tipe, atau rentang baris yang relevan.
11. THE Penulis SHALL TIDAK memasukkan ke Bab_3 klaim teknis (sesuai definisi di AC 13.1) yang tidak dapat ditelusuri ke berkas pada `src/`, ke `Cargo.toml`, atau ke berkas pada `assets/`.
12. IF setelah revisi awal masih terdapat satu atau lebih klaim teknis yang tidak dapat diverifikasi terhadap berkas yang dirujuk, THEN THE Penulis SHALL melakukan revisi terhadap Bab_3 sehingga setiap klaim teknis yang tersisa memenuhi AC 13.2 sampai 13.11 secara penuh, tanpa menyisakan klaim tidak terverifikasi pada subbab manapun.

### Requirement 14: Ruang Lingkup Revisi yang Dipertahankan

**User Story:** Sebagai mahasiswa yang menjaga koherensi skripsi, saya ingin Subbab yang sudah baik tidak diubah strukturnya secara berlebihan, sehingga revisi tetap minimal-invasif.

#### Acceptance Criteria

1. THE Penulis SHALL mempertahankan kata dan urutan kalimat Subbab 3.1 dan 3.2 dari Bab_3_Lama tanpa perubahan kata maupun urutan kalimat, kecuali pada referensi silang yang nomor subbab atau terminologinya berubah akibat revisi pada Subbab 3.3, 3.4, 3.5, 3.6, atau 3.7.
2. IF Subbab 3.1 menyebut Fase 3 atau Fase 4 yang merujuk ray shooting atau klasifikasi tipe ubin, THEN THE Penulis SHALL mengganti penyebutan tersebut dengan nama tahap, urutan tahap, dan istilah algoritma yang sama persis dengan yang dipakai pada Subbab 3.5 (binning DDA, akumulator signed-area, propagasi backdrop).
3. THE Penulis SHALL mempertahankan minimal kosakata berikut yang sudah dipakai di Bab_3_Lama: "pustaka", "perangkat lunak", "pra-pemrosesan", "ubin"; mempertahankan format penomoran subbab `3.x` dan `3.x.y`; serta mempertahankan gaya bahasa akademik formal Indonesia (kalimat lengkap dengan subjek-predikat-objek, tanpa singkatan kasual).
4. IF Penulis perlu menambahkan struktur baru di luar Subbab_Wajib (3.1 sampai 3.7), THEN penambahan tersebut SHALL dibatasi pada level sub-sub-subbab `### 3.5.y` di dalam Subbab 3.5 untuk mengorganisasi tahap algoritma sesuai struktur pada Laporan_Analisis Bagian G, dan SHALL TIDAK menambah subbab level 2 baru selain 3.1 sampai 3.7.

### Requirement 15: Larangan Membuat Informasi di Luar Source Code

**User Story:** Sebagai dosen pembimbing, saya ingin pasti bahwa tidak ada klaim fiktif yang ditambahkan dalam revisi, sehingga kredibilitas akademik skripsi terjaga.

#### Acceptance Criteria

1. THE Penulis SHALL menulis ulang Bab_3 hanya berdasarkan informasi yang dapat diverifikasi melalui kutipan eksplisit atau pernyataan yang dapat dilacak ke Source_Of_Truth atau ke Laporan_Analisis.
2. IF sebuah klaim teknis (sesuai definisi AC 13.1) tidak dapat diverifikasi dari Source_Of_Truth, THEN THE Penulis SHALL menghapus klaim tersebut dari Bab_3 atau menggantinya dengan pernyataan yang dapat diverifikasi.
3. THE Penulis SHALL TIDAK mengarang nama fungsi, nama struct, nama trait, nama modul, nama feature flag, nama crate, nama konstanta, maupun nilai parameter numerik yang tidak muncul secara literal pada Source_Of_Truth (`src/`, `Cargo.toml`, `.cargo/config.toml`, `examples/*/src/`, `tests/`, atau `assets/`).
4. IF sebuah pustaka disebut sebagai dependensi langsung Arabella di Subbab 3.3.1 atau Subbab 3.5, THEN THE Penulis SHALL memastikan nama crate tersebut terdaftar di `Cargo.toml`. Pustaka yang hanya disebut dalam konteks perbandingan aplikasi sejenis (misalnya pada Subbab 3.2.2) dikecualikan dari persyaratan ini.
5. WHEN Bab_3 mengutip cuplikan kode, jalur berkas, atau cuplikan konfigurasi (misalnya entri `Cargo.toml`, deklarasi struct, atau header fungsi), THE Penulis SHALL menjamin kesetiaan verbatim cuplikan tersebut terhadap isi berkas yang sebenarnya pada Source_Of_Truth, tanpa modifikasi karakter, ejaan, atau tipe data.
