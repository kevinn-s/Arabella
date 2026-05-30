# Antisipasi Tanya-Jawab Sidang Skripsi — Arabella

> Dokumen persiapan internal (bukan bagian naskah final). Berisi perkiraan pertanyaan dosen pembimbing/penguji beserta poin-poin jawaban yang berpijak pada isi naskah dan source code Arabella yang sudah terverifikasi. Disusun 30 Mei 2026.
>
> Cara pakai: baca poin inti tiap jawaban, lalu hafalkan satu kalimat "jangkar" (ditandai **Jangkar:**) yang bisa diucapkan langsung saat ditanya. Jangan menghafal paragraf — pahami logikanya.

---

## A. Pertanyaan Paling Mungkin & Paling Menekan

### A1. "Judulnya 'pipeline rendering vektor PARALEL', tapi katanya eksekusinya masih single-thread. Mana paralelismenya?"

Ini pertanyaan paling kritis. Jawab dengan tenang dan jujur — jangan defensif.

**Poin inti:**
- Paralelisme dalam penelitian ini berada pada **dua tingkat yang berbeda**, dan keduanya **sudah aktif**:
  1. **Paralelisme tingkat instruksi (SIMD)** pada tahap pra-pemrosesan CPU. Ini nyata dan aktif pada build baku, di *hot path* transformasi affine (`transform_pair` dengan `f32x4`, `transform_quad` dengan `f32x8`) dan konversi kubik→kuadratik (`f32x8`), serta propagasi backdrop (`i16x8.add`). Diaktifkan lewat `target-feature=+simd128`.
  2. **Paralelisme tingkat data di GPU** pada tahap rasterisasi. Ribuan fragment diproses serentak oleh fragment shader pada *rasterization pipeline* konvensional. Ini juga paralelisme, hanya saja bukan compute shader.
- Yang **belum aktif** adalah **paralelisme tingkat thread (multithreading antar-jalur via Rayon)** pada CPU. Ini sudah disiapkan sebagai feature `multithreading` di `Cargo.toml` namun belum ditarik pada build baku — dan ini **dinyatakan jujur** di Bab 3.3.1, 3.5, 4.6, dan 5.2.1.
- Jadi "paralel" dalam judul bukan klaim kosong: arsitekturnya paralel di dua tingkat yang sudah berjalan, plus satu tingkat lagi yang strukturnya sudah disiapkan tinggal diaktifkan.

**Mengapa Rayon belum diaktifkan?** Kontribusi inti penelitian adalah **pembuktian kelayakan (feasibility)** arsitektur hibrida non-compute, bukan optimasi performa maksimal. Struktur data sudah dirancang agar pemrosesan antar-jalur independen (tidak ada ketergantungan data antar-path pada flattening dan binning), sehingga jalan menuju paralelisasi Rayon sudah terbuka. Mengaktifkannya adalah pekerjaan rekayasa lanjutan, bukan penelitian baru.

**Jangkar:** "Paralelismenya ada di dua tingkat yang sudah aktif — SIMD di CPU dan paralelisme fragmen di GPU. Multithreading antar-jalur lewat Rayon sengaja kami posisikan sebagai pengembangan lanjutan, dan ini kami nyatakan terbuka di naskah, karena fokus skripsi ini membuktikan kelayakan arsitektur non-compute, bukan mengejar performa puncak."

> Catatan kejujuran: justru karena keterbatasan ini ditulis terbuka di naskah (bukan ditutupi), posisi Anda lebih kuat — penguji tidak bisa menuduh ada klaim menyesatkan.

---

### A2. "Kalau paris-30k cuma 6,4 FPS, apa benar ini layak disebut 'skala besar'? Bukankah itu gagal?"

**Poin inti:**
- 6,4 FPS pada `paris-30k.svg` **bukan kegagalan, melainkan temuan yang justru memperkuat tesis**. Penelitian ini bertujuan mendemonstrasikan kelayakan + mengukur trade-off, bukan menjamin 60 FPS pada semua skala.
- Dekomposisi waktu menunjukkan: dari total 156,15 ms, **150,15 ms (96%) ada di CPU single-thread**, dan hanya 6,00 ms di GPU. Artinya:
  - GPU pada pipeline rasterisasi konvensional **tidak** menjadi bottleneck — ini membuktikan pendekatan non-compute di sisi GPU bekerja efisien bahkan pada 60.580 ubin.
  - Bottleneck murni ada di pra-pemrosesan CPU yang **belum diparalelkan** (single-thread). Ini persis celah yang sudah disiapkan untuk Rayon.
- Pada skala kecil–menengah (el_gato 1266 FPS, tiger 103 FPS), pipeline ini jauh di atas ambang interaktif. Jadi pendekatannya **layak**; yang membatasi adalah satu tahap yang belum dioptimasi, bukan arsitekturnya.

**Jangkar:** "Justru 6,4 FPS itu data yang berguna: 96% waktunya di CPU single-thread, bukan di GPU. Itu membuktikan sisi GPU non-compute kami efisien, dan menunjuk dengan tepat ke mana optimasi berikutnya harus diarahkan — paralelisasi CPU yang fondasinya sudah kami siapkan."

---

### A3. "Validasi kebenarannya cuma visual dan manual. Kenapa tidak ada metrik kuantitatif (pixel diff / SSIM)?"

**Poin inti:**
- Cakupan validasi sudah dinyatakan jujur di Bab 4.3: satu tes otomatis (`test_renders_tiger_svg` di `tests/test.rs`), dua aset lain divalidasi manual side-by-side terhadap rendering peramban.
- Untuk purwarupa pembuktian kelayakan, validasi visual atas empat aspek correctness (ketepatan fill solid, ekspansi stroke, fill rule NonZero, ketiadaan seam antar-ubin) sudah memadai untuk menunjukkan pipeline menghasilkan output yang benar secara geometris.
- Metrik kuantitatif (pixel-diff/SSIM terhadap renderer referensi) memerlukan *ground truth* yang sepadan piksel-demi-piksel, yang sulit adil karena algoritma antialiasing tiap renderer berbeda. Itu lebih cocok sebagai bagian dari benchmark head-to-head — yang memang sudah diposisikan sebagai pengembangan lanjutan di Bab 5.2.3.
- Penting: untuk `paris-30k.svg` perbedaan warna sudah dijelaskan **bukan** karena rasterisasi salah, melainkan karena keterbatasan parser (transparansi berlapis & pewarisan warna default), sementara geometrinya tetap akurat (Bab 4.3 & 4.6).

**Jangkar:** "Validasi kami fokus pada kebenaran geometris dan empat aspek correctness yang bisa diamati langsung; metrik piksel kuantitatif kami posisikan menyatu dengan benchmark head-to-head sebagai pengembangan lanjutan, karena perbandingan piksel adil menuntut kontrol AA yang setara antar-renderer."

---

### A4. "Kenapa tidak ada perbandingan kuantitatif langsung dengan Skia, Cairo, atau Vello?"

**Poin inti:**
- Sudah dinyatakan eksplisit dan konsisten di Bab 1.3, 4.4.3, dan 5.2.3: perbandingan dalam skripsi ini **kualitatif** pada tiga dimensi — paradigma rasterisasi, ketergantungan compute shader, dan target platform.
- Alasan metodologis: benchmark head-to-head yang sahih menuntut perangkat keras identik, berkas uji identik, dan prosedur pengukuran seragam untuk keempat renderer. Tanpa itu, angka perbandingan menyesatkan. Syarat-syarat ini sudah dirinci di Bab 5.2.3.
- Data Tabel 4.4 adalah **pengukuran internal** (dekomposisi CPU/GPU Arabella sendiri), bukan klaim menang/kalah terhadap renderer lain — dan ini dinyatakan tegas.

**Jangkar:** "Kami sengaja tidak menarik klaim performa relatif tanpa pengukuran tatap muka yang terkontrol, karena itu tidak akan ilmiah. Perbandingan kami batasi pada paradigma arsitektur, dan benchmark kuantitatif kami tetapkan sebagai langkah validasi berikutnya dengan syarat-syarat yang sudah kami rinci."

---

## B. Pertanyaan Teknis Implementasi

### B1. "Kenapa ukuran ubin 16×8 piksel? Kenapa tidak 16×16 seperti Pathfinder/Vello?"

**Poin inti:**
- 16×8 dipilih agar selaras dengan representasi data: `TILE_H = 8` cocok dengan delapan akumulator signed-area per scanline yang muat dalam `[i16; 8]` (16 byte), dan langsung dipetakan ke dua atribut `ivec4` di vertex shader (backdrop_lo/hi).
- Lebar 16 (`TILE_W_F24DOT8 = 4096`) dan tinggi 8 (`TILE_H_F24DOT8 = 2048`) keduanya pangkat dua, sehingga pemetaan koordinat→indeks ubin bisa pakai pergeseran bit (`TILE_W_LOG2 = 4`, `TILE_H_LOG2 = 3`), bukan pembagian.
- Trade-off ukuran ubin sudah dibahas di Bab 2.1.8: ubin lebih besar → overhead manajemen turun tapi overdraw naik; lebih kecil → kerja lebih terlokalisasi tapi jumlah ubin naik. 16×8 adalah titik tengah yang pas dengan layout memori `Tile` 44-byte.

**Jangkar:** "Tinggi 8 dipilih supaya satu ubin punya tepat delapan akumulator scanline yang muat di `[i16;8]` dan langsung jadi dua atribut ivec4 di shader; lebar 16 menjaga keduanya pangkat dua agar pemetaan indeks pakai shift, bukan bagi."

### B2. "Kenapa fixed-point F24Dot8, bukan floating-point?"

**Poin inti (Bab 2.1.7):**
- **Determinisme**: hasil penjumlahan/perkalian integer tidak bergantung mode pembulatan FP yang bisa beda antar-hardware → geometri sama menghasilkan piksel identik lintas platform.
- **Presisi seragam** di seluruh rentang koordinat layar (FP kehilangan presisi saat magnitudo membesar).
- Relevan untuk winding number: pembulatan simetris di `f32_to_f24dot8` mencegah bias winding pada koordinat negatif.

### B3. "Jelaskan binning DDA dua tahap. Kenapa harus dua tahap?"

**Poin inti (Bab 3.5b, 4.2.4):**
- **Outer DDA** (`bin_line`) memecah segmen melintas **baris ubin** (tinggi 8), empat varian arah diagonal (down-right/down-left/up-right/up-left).
- **Inner DDA** (`bin_line_in_row`) memecah sub-segmen per-baris melintas **kolom ubin** (lebar 16), empat varian arah.
- Dua tahap karena partisi 2D dipecah jadi dua partisi 1D yang lebih sederhana dan bisa pakai akumulator galat integer bergaya Bresenham di tiap sumbu — lebih murah dan bebas galat pembulatan daripada perpotongan garis-kotak langsung.
- Kasus khusus ditangani terpisah: horizontal-degenerate (tidak menyumbang winding), single-row, vertical-degenerate, dan pembelahan rekursif bila melampaui `MAXIMUM_DELTA` (anti-overflow).

### B4. "Apa itu akumulator signed-area dan backdrop? Bagaimana antialiasing muncul?"

**Poin inti (Bab 2.1.10, 3.5c–d, 4.2.4):**
- Tiap segmen menyumbang **luas bertanda** ke akumulator per-scanline (`[i16; 8]`, format 8.8, ±256 = 1 winding penuh). Tanda mengikuti arah vertikal segmen (turun −256, naik +256).
- Karena luas bersifat kontinu, nilai cakupan pecahan di tepi bentuk **otomatis memberi antialiasing** tanpa supersampling.
- **Backdrop** = winding awal yang diwarisi ubin dari semua geometri di kirinya pada baris yang sama. Dipropagasikan kiri→kanan (`generate_tiles`), sehingga ubin interior yang tidak disentuh tepi tetap terisi benar hanya dari backdrop.
- Teknik ini lazim pada rasterizer analitik: Blaze, FreeType, Skia (sudah disitasi).

### B5. "Bagaimana fragment shader menghitung cakupan? Apa itu `line_box`?"

**Poin inti (Bab 3.5e–f, shader `render_tile.frag`):**
- `line_box` = konvolusi indikator setengah-bidang garis dengan filter kotak 1×1 di pusat piksel, dievaluasi trapezoidal; mengembalikan kontribusi bertanda di [−1, +1].
- Shader: mulai dari backdrop scanline (8.8 → float dibagi `WINDING_UNIT = 256.0`), tambahkan kontribusi `line_box` tiap segmen, lalu terapkan fill rule:
  - NonZero: `coverage = clamp(abs(winding), 0, 1)`
  - EvenOdd: `coverage = 1 - abs(mod(abs(winding), 2) - 1)`
- Satu jalur kode untuk semua ubin (tanpa cabang berbasis tipe ubin) — variasi perilaku didorong data.

> Catatan: di shader ada juga fungsi `line_tent` (filter tent 2×2) yang **tidak dipakai** pada jalur aktif — itu eksperimen yang sengaja ditinggalkan dengan komentar penjelas. Kalau ditanya: jalur produksi memakai `line_box`; `line_tent` adalah eksperimen AA yang didokumentasikan tapi tidak diaktifkan karena butuh halo-binning yang belum ada.

### B6. "Kenapa stroke tidak punya rasterizer sendiri?"

**Poin inti (Bab 4.2.2):**
- `Scene::stroke` mengekspansi garis jadi outline berisi via `kurbo::stroke_with`, lalu menyalurkannya ke `Scene::fill` dengan `FillRule::NonZero`. Jadi stroke dan fill berbagi satu rasterizer — menyederhanakan kode dan menjamin konsistensi AA antar keduanya.

---

## C. Pertanyaan Lingkup, Pilihan Teknologi, dan Kebaruan

### C1. "Apa kebaruan (novelty) penelitian ini? Bukankah tiling, signed-area, DDA semua sudah ada?"

**Poin inti:**
- Kebaruan **bukan** pada penemuan algoritma baru, melainkan pada **komposisi arsitektural**: membuktikan bahwa rendering vektor yang benar dapat dicapai **tanpa satu pun compute shader**, dengan memindahkan SELURUH komputasi tujuan umum (flattening, binning, winding) ke pra-pemrosesan CPU, dan membatasi GPU hanya pada vertex+fragment shader konvensional.
- Literatur modern (Vello, Pathfinder versi baru, Forma) hampir selalu mengandalkan compute shader. Arabella menunjukkan jalur alternatif yang **portabel ke WebGL 2.0** — lingkungan yang jauh lebih luas dukungannya daripada WebGPU.
- Kontribusinya adalah **bukti kelayakan + dokumentasi trade-off** pendekatan non-compute, bukan klaim performa juara.

**Jangkar:** "Kebaruannya pada komposisi: kami buktikan rendering vektor benar bisa jalan tanpa compute shader sama sekali, dengan seluruh komputasi umum dipindah ke CPU dan GPU dibatasi ke pipeline rasterisasi tradisional — sehingga portabel ke WebGL 2.0 yang dukungannya jauh lebih luas dari WebGPU."

### C2. "Kenapa Rust? Kenapa WebGL 2.0, bukan WebGPU atau OpenGL native?"

**Poin inti:**
- **Rust**: zero-cost abstraction (cepat setara C/C++), memory safety tanpa GC, dukungan SIMD portabel (`fearless_simd`), dan kompilasi mulus ke WebAssembly.
- **WebGL 2.0**: justru inti tesis — ini API yang **tidak mewajibkan compute** dan tersedia luas di perangkat low-end/lama. Memilih WebGPU akan membatalkan premis penelitian.
- **Bukan OpenGL native**: target eksekusi adalah peramban via `wasm32-unknown-unknown`; tidak ada jalur native (sudah diverifikasi: tidak ada glow/glutin/sdl2/glfw di dependensi).

### C3. "Kenapa parser SVG-nya cuma mendukung `g` dan `path`? Bukankah itu terlalu terbatas?"

**Poin inti (Bab 3.3.1, 4.6):**
- Parser (`pico_svg.rs`) memang subset minimal — sengaja, karena fokus penelitian pada **pipeline rasterisasi**, bukan kelengkapan parser SVG.
- Semua geometri dapat dimodelkan sebagai `<path>`, jadi subset ini cukup untuk menguji pipeline inti.
- Aset uji (tiger, el_gato, paris) sengaja dipilih yang berada dalam subset `g`/`path`. Aset di luar subset (SVG Logo pakai `defs`/`use`, Bismillah pakai `pattern`) justru dipakai untuk **mengilustrasikan batas parser** (Gambar 4.4–4.5), dengan penegasan bahwa selisihnya dari parser, bukan dari rasterizer.
- Perluasan subset sudah jadi butir pengembangan lanjutan (Bab 5.2.2).

### C4. "Judul menyebut 'skala besar'. Apa dasarnya?"

**Poin inti:**
- `paris-30k.svg` memuat 50.620 operasi paint / >50 ribu elemen path → menghasilkan 60.580 ubin nontrivial. Itu adalah uji-tekan (stress test) skala besar yang nyata.
- Temuan pada skala besar inilah yang mengungkap bottleneck CPU single-thread — yang merupakan hasil penelitian yang valid, bukan kegagalan.

---

## D. Pertanyaan "Jebakan" Saat Penguji Membaca Source Code

### D1. "Di `src/lib.rs` ada `TILE_WIDTH = 4.0` dan `TILE_HEIGHT = 4.0`, dan di `webgl.rs` ada `tile_height: 4u32`. Tapi naskah bilang ubin 16×8. Mana yang benar?"

**Jawaban jujur:**
- Ubin yang **benar-benar dipakai** adalah 16×8, sebagaimana didefinisikan `TILE_W = 16`/`TILE_H = 8` di `blocks.rs` dan `builder.rs`, dan `#define TILE_WIDTH 16u`/`TILE_HEIGHT 8u` di kedua shader. Jalur rasterisasi memakai konstanta inilah.
- Konstanta `4.0` di `lib.rs` dan field `tile_height: 4u32` di config UBO adalah **sisa kode lama (dead code)** yang tidak ikut dalam perhitungan cakupan/indeks ubin pada jalur aktif. Shader menghitung baris scanline dari `#define TILE_HEIGHT 8u`, bukan dari `u_tile_height`.
- **Rencana tindak:** sebaiknya dibersihkan sebelum sidang agar tidak memancing pertanyaan (lihat bagian F).

**Jangkar:** "Ubin efektifnya 16×8 — itu yang dipakai blocks, builder, dan kedua shader. Yang 4 itu sisa konstanta lama yang tidak masuk jalur perhitungan aktif; output tetap benar, dan saya sudah menandainya untuk dibersihkan."

### D2. "Naskah bilang output 'premultiplied alpha', tapi blend func-nya `SRC_ALPHA, ONE_MINUS_SRC_ALPHA` — itu mode non-premultiplied. Tidak konsisten?"

**Jawaban jujur:**
- Fragment shader memang **mengeluarkan warna premultiplied**: `fragColor = vec4(paint.rgb * paint.a * coverage, paint.a * coverage)`.
- Blend func yang konsisten untuk sumber premultiplied semestinya `ONE, ONE_MINUS_SRC_ALPHA`. Saat ini dipakai `SRC_ALPHA, ONE_MINUS_SRC_ALPHA`.
- Untuk aset uji yang dominan warna **solid opak (alpha = 1)**, kedua mode menghasilkan output identik, sehingga tiger dan el_gato tampak benar. Perbedaan baru muncul pada bidang semi-transparan — yang juga bersinggungan dengan keterbatasan transparansi berlapis di Bab 4.6.
- **Rencana tindak:** dua pilihan — (a) ganti blend ke `ONE, ONE_MINUS_SRC_ALPHA` agar benar-benar premultiplied, atau (b) lunakkan kalimat di naskah. Lihat bagian F.

**Jangkar:** "Shader mengeluarkan warna premultiplied; untuk warna opak hasilnya identik dengan blend yang dipakai. Untuk semi-transparan, blend func-nya idealnya `ONE, ONE_MINUS_SRC_ALPHA` — itu perbaikan kecil yang sudah saya catat."

### D3. "Komentar di `bench_webgl` menyebut aset Tiger/SVG Logo/Bismillah, tapi yang di-benchmark Tiger/el_gato/paris. Yang mana benar?"

**Jawaban jujur:**
- Yang **di-benchmark (timed)** adalah Tiger, el_gato, paris (lihat `let assets = vec![...]`). Yang capture-only (PNG ilustrasi keterbatasan) adalah SVG Logo & Bismillah.
- Komentar doc di kepala file adalah **komentar usang** yang belum diperbarui; logika kode sudah benar dan naskah (Bab 4.3/4.6) sudah benar. Hanya komentarnya yang basi.

### D4. "Anda bilang 120 frame, tapi `budget_for` memberi 20 sampel untuk aset berat. Bagaimana?"

**Jawaban (sudah diperbaiki di naskah):**
- Bab 4.4.1 kini mendeskripsikan anggaran **adaptif**: aset <5.000 ops → 30 warm-up + 120 sampel; aset >5.000 ops → 5 warm-up + 20 sampel.
- `paris-30k` (50.620 ops) diukur pada 5+20; el_gato dan tiger pada 30+120.
- Beban CPU deterministik (geometri tetap, tanpa keacakan antar-frame), sehingga 20 sampel sudah menghasilkan rerata stabil.

**Jangkar:** "Anggaran sampelnya adaptif sesuai berat aset; paris pakai 20 sampel karena beban CPU-nya deterministik, jadi reratanya tetap stabil. Naskah sudah saya selaraskan dengan kode."

---

## E. Pertanyaan Konseptual / Teori

### E1. "Apa beda winding number NonZero dan EvenOdd? Kenapa dukung keduanya?"
- NonZero: titik di dalam jika winding ≠ 0. EvenOdd: di dalam jika winding ganjil. Keduanya standar fill rule SVG; mendukung keduanya menjaga kompatibilitas dengan dokumen SVG nyata. Implementasi: dua formula coverage di fragment shader, dipilih oleh bit fill rule pada `paint_and_rect_flag`.

### E2. "Apa itu flattening dan kenapa kubik dikonversi ke kuadratik dulu?"
- Flattening = konversi kurva mulus jadi rangkaian garis lurus dalam toleransi galat. Kubik→kuadratik dulu karena kuadratik hanya 1 titik kontrol → uji kedataran & pembelahan lebih murah, sementara himpunan kuadratik tetap mendekati kubik pada toleransi `TOL = 0.25` (Bab 2.1.6, 4.2.3).

### E3. "Kenapa compute shader bisa lebih cepat, dan apa yang Anda korbankan dengan tidak memakainya?"
- Compute shader memberi akses baca-tulis memori GPU arbitrer + paralelisme masif → cocok untuk prefix-sum, sorting, dsб (cara Vello). Yang dikorbankan Arabella: tidak bisa memparalelkan komputasi tujuan umum di GPU, sehingga beban itu pindah ke CPU (jadi bottleneck pada skala besar). Yang didapat: portabilitas ke WebGL 2.0. Ini trade-off inti yang dibahas Bab 4.5.

### E4. "Apa itu SIMD dan SIMD128? Bedanya dengan multithreading?"
- SIMD = satu instruksi atas banyak data (paralelisme tingkat instruksi, dalam satu thread). SIMD128 = set instruksi WebAssembly dengan register 128-bit (`f32x4`, `i16x8`, dll). Multithreading = paralelisme tingkat thread antar unit pemrosesan. Arabella aktif pakai SIMD; multithreading (Rayon) belum (Bab 2.1.12, 3.5).

---

## F. Daftar Perbaikan Sebelum Sidang (status)

Tiga celah kode "jebakan" (D1–D3) **sudah diperbaiki** (lihat catatan revisi #12):

1. ~~Bersihkan dead code ukuran ubin~~ — **SELESAI.** Konstanta `TILE_WIDTH/HEIGHT = 4.0` dihapus dari `src/lib.rs`; `tile_height` di Config UBO dikoreksi ke 8 + komentar; struct legacy `common::Tile` ditandai tidak dipakai. (D1)
2. ~~Selaraskan blend func dengan klaim premultiplied~~ — **SELESAI.** Diubah ke `ONE, ONE_MINUS_SRC_ALPHA` di `webgl.rs` + komentar. (D2)
3. ~~Perbarui komentar usang harness~~ — **SELESAI.** Komentar kepala `bench_webgl/src/lib.rs` kini menyebut Tiger/el_gato/paris. (D3)

Verifikasi: `cargo check -p arabella --lib --target wasm32-unknown-unknown --features webgl` berhasil (exit 0).

Sisa yang masih opsional (tidak wajib):

4. **Cek konsistensi judul bab** — heading "BAB 2 TINJAUAN REFERENSI" vs template prodi (biasanya "TINJAUAN PUSTAKA").
5. **Bukti "tiga pengulangan"** sudah dihapus dari naskah (revisi #11), jadi tidak perlu disiapkan lagi.

---

## G. Strategi Umum Menjawab

- **Akui keterbatasan dengan percaya diri.** Skripsi ini kuat justru karena jujur. Jika ditanya kelemahan, akui + jelaskan kenapa itu konsisten dengan lingkup (feasibility, bukan performa juara).
- **Selalu kembalikan ke kontribusi inti:** "pembuktian kelayakan pipeline rendering vektor hibrida non-compute pada WebGL 2.0."
- **Pisahkan tiga hal yang sering tertukar:** (a) yang sudah terbukti (kelayakan + dekomposisi performa internal), (b) yang belum dilakukan tapi terdokumentasi (Rayon, benchmark head-to-head, perluasan parser), (c) yang di luar lingkup (filter, text, gradient).
- **Jangan mengklaim lebih dari data.** Tidak ada klaim "lebih cepat dari Skia/Vello" — pertahankan itu.
- **Kalau tidak tahu jawaban detail kode**, katakan akan menunjukkan baris kodenya — semua klaim teknis di Bab 3–4 punya rujukan `berkas:simbol`, manfaatkan itu.
