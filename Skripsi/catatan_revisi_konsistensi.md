# Catatan Revisi Konsistensi Skripsi

> Dokumen ini mencatat masalah konsistensi antar-bab yang ditemukan saat review skripsi Arabella, beserta solusi yang sudah diterapkan. Disusun sebagai jejak audit revisi (tanggal review: 30 Mei 2026).

## Ringkasan

Dua belas perbaikan diterapkan: enam terkait konsistensi antar-bab (#1–#6), satu personalisasi Kata Pengantar (#7), tiga perbaikan akurasi/kerapian tambahan (#8 jumlah dependensi, #9 sitasi Blaze/FreeType, #10 penandaan dokumen kerja usang), satu koreksi akurasi metodologi benchmark (#11 anggaran sampel adaptif), dan satu pembersihan kode pra-sidang (#12 dead code ukuran ubin, blend func premultiplied, komentar usang harness). Seluruh perbaikan menyangkut keselarasan antar-dokumen, akurasi rujukan/metodologi, dan keselarasan kode-naskah, bukan kebenaran teknis inti — substansi dan kejujuran ilmiah skripsi sudah baik sebelum revisi.

---

## #1 — Kontradiksi aset uji (Bab 4.2.8 vs Bab 4.3)

**Masalah.** Bab 4.2.8 menyatakan demo interaktif memuat tiga aset yang "identik dengan aset verifikasi pada Subbab 4.3, yaitu Ghostscript Tiger, SVG Logo, dan Bismillah." Padahal:
- Demo (`examples/native_webgl/src/lib.rs:load_assets`) memuat: Ghostscript Tiger, SVG Logo, Bismillah.
- Verifikasi/benchmark (Bab 4.3, `examples/bench_webgl`) memakai: Ghostscript Tiger, el_gato, paris-30k.

Daftar yang disebut tidak cocok dengan aset 4.3, sehingga kalimat "identik" salah dan menimbulkan kontradiksi dalam satu bab.

**Solusi.** Kalimat di Bab 4.2.8 ditulis ulang: demo memuat Tiger, SVG Logo, Bismillah; ditegaskan bahwa daftar ini TIDAK identik dengan aset verifikasi 4.3 (hanya Tiger yang beririsan), dan SVG Logo + Bismillah justru dipakai mengilustrasikan keterbatasan parser pada Subbab 4.6. Demo diposisikan sebagai sarana eksplorasi visual, bukan jalur pengukuran kuantitatif.

**Berkas terdampak.** `bab4_implementasi_dan_hasil.md`

---

## #2 — Janji perbandingan tidak ditepati (Bab 1.3)

**Masalah.** Bab 1.3 menjanjikan "dua kelompok pembanding: renderer CPU murni seperti Cairo atau Skia sebagai *baseline performa sekuensial*, dan Vello sebagai baseline kualitatif." Namun Bab 4.4.3 dan Bab 5 menyatakan secara jujur bahwa benchmark kuantitatif terhadap Skia/Cairo/Vello belum dilakukan. Penguji dapat menanyakan baseline performa yang dijanjikan tetapi tidak ada.

**Solusi.** Bab 1.3 dilunakkan: perbandingan dinyatakan kualitatif pada dimensi paradigma rasterisasi, ketergantungan compute shader, dan target platform. Cairo/Skia menjadi acuan paradigma rasterisasi sekuensial dan Vello acuan GPU compute-centric. Benchmark head-to-head ditegaskan sebagai pengembangan lanjutan, konsisten dengan Bab 4.4.3 dan Bab 5.

**Berkas terdampak.** `bab1_pendahuluan.md`

---

## #3 — Klaim skalabilitas multi-core tidak teruji (Bab 3.1 Fase 5)

**Masalah.** Bab 3.1 (Fase 5) menyebut pengukuran "skalabilitas throughput sistem terhadap penambahan jumlah core CPU." Karena Rayon/`multithreading` tidak aktif (eksekusi single-thread), pengujian ini mustahil dan memang tidak ada di Bab 4. Terjadi mismatch metode vs hasil.

**Solusi.** Klaim skalabilitas core CPU dihapus dari Fase 5, diganti dekomposisi frame time CPU/GPU yang benar-benar diukur. Klaim "trade-off kuantitatif" juga dilunakkan menjadi analisis kualitatif (paradigma, ketergantungan compute shader, target platform).

**Berkas terdampak.** `bab3_metodologi.md`

---

## #4 — Inkonsistensi "OpenGL ES 3.0" (Bab 3.1 & tabel 3.2.3)

**Masalah.** Bab 3.1 (Fase 2 & 4) dan tabel 3.2.3 menulis "OpenGL ES 3.0 / WebGL 2.0", sedangkan Bab 3.3.1 menegaskan implementasi menargetkan WebGL 2.0 langsung tanpa transpilasi. Verifikasi kode: tidak ada jalur native OpenGL ES (tidak ada glow/glutin/sdl2/glfw); target hanya `wasm32`.

**Solusi.** Seluruh penyebutan "OpenGL ES 3.0" di Bab 3.1 dan tabel 3.2.3 diselaraskan menjadi "WebGL 2.0" saja. Tidak ada lagi sisa "OpenGL ES" di bab manapun (referensi tersisa hanya di file analisis kerja internal, bukan naskah bab).

**Berkas terdampak.** `bab3_metodologi.md`

---

## #5 — Rayon dinyatakan seolah aktif (Bab 3.1 Fase 4)

**Masalah.** Bab 3.1 Fase 4 menyatakan "Mengimplementasikan paralelisasi tingkat tile pada CPU menggunakan Rayon," padahal Bab 3.3.1, 3.5, 4.6, dan 5.2.1 menyatakan jujur bahwa Rayon belum aktif (feature opsional, tidak ditarik pada build baku). Terjadi tabrakan antar-bab.

**Solusi.** Fase 4 ditulis ulang: struktur data spasial dirancang agar pemrosesan antar-jalur independen (membuka jalan paralelisasi Rayon di masa depan); paralelisme yang sudah aktif adalah SIMD pada hot path transformasi/flattening; feature `multithreading` berbasis Rayon masih opsional dan belum aktif pada build baku. Kini konsisten dengan seluruh bab lain.

**Berkas terdampak.** `bab3_metodologi.md`

---

## #6 — Salah kutip nama penulis (Bab 3.2.2)

**Masalah.** Bab 1 dan Daftar Pustaka memakai "Ganacim dkk. (2014)", tetapi Bab 3.2.2 menulis "Gan et al., 2014" — penulis pertama paper tersebut adalah Ganacim, bukan Gan. Selain itu gaya sitasi "et al." pada "Li et al." tidak konsisten dengan gaya "dkk." di Bab 1.

**Solusi.** "Gan et al., 2014" → "Ganacim dkk., 2014"; "Li et al., 2016" → "Li dkk., 2016". Gaya sitasi kini seragam memakai "dkk." sesuai Bab 1.

**Berkas terdampak.** `bab3_metodologi.md`

---

## #7 — Personalisasi Kata Pengantar

**Masalah.** Kata Pengantar masih placeholder generik ("Rektor", "Dean", "Head of Study Program", "Dosen Pembimbing").

**Solusi.** Diisi dengan nama dan jabatan resmi:
- Rektor: Dr. Nelly, S.Kom., M.M., CSCA
- Dean School of Computer Science: Prof. Dr. Ir. Derwin Suhartono, S.Kom., MTI
- Head of Computer Science Department: Ir. Andry Chowanda, S.Kom., MM, Ph.D., MBCS, CCP, CME, IPM, SMIEEE
- Dosen Pembimbing (Deputy Head of Global Class Program – Computer Science): Kenny Jingga, S.Kom., M.T.
- Tempat/tanggal: Tangerang, 30 Mei 2026
- Nama penulis: Kevin Sukohardjo, Surya Saddam Saputra, Arlin Lutfi Widarma

**Berkas terdampak.** `kata_pengantar.md`

---

## #8 — Jumlah dependensi tidak akurat (Bab 3.3.1)

**Masalah.** Bab 3.3.1 menyebut "sepuluh crate dependensi langsung", padahal blok `[dependencies]` di `Cargo.toml` memuat tiga belas crate. Tiga yang tidak disebut: `png`, `log`, dan `lyon_algorithms`. Rujukan nomor baris pada beberapa entri juga bergeser dari posisi aktualnya.

**Solusi.** Daftar ditulis ulang menjadi tiga belas crate, diurutkan menurut posisi baris pada manifest, dengan nomor baris rujukan diselaraskan ke `Cargo.toml` aktual (`Cargo.toml:29-42`). Ditambahkan keterangan bahwa `png` ditandai `optional` namun tetap aktif pada build baku karena ditarik feature `png` yang termasuk `default`, sedangkan dua belas crate lainnya wajib.

**Berkas terdampak.** `bab3_metodologi.md`

---

## #9 — "Blaze" dan "FreeType" tanpa sitasi (Bab 2.1.10, 3.5, 4.2.4)

**Masalah.** "Blaze" dan "FreeType" dirujuk berulang sebagai acuan gaya akumulator signed-area, tetapi tidak memiliki entri di Daftar Pustaka (Skia sudah ada).

**Solusi.** Dua entri ditambahkan ke `daftar_pustaka.md` (dijaga urutan alfabetisnya):
- Gasiulis, A. (2024). *Blaze: Multi-threaded, CPU-based vector graphics rasterizer* [Perangkat lunak]. GitHub.
- The FreeType Project. (2023). *FreeType: A freely available software library to render fonts*.

Sitasi dalam teks ditambahkan pada ketiga lokasi: Bab 2.1.10 (FreeType, Skia), Bab 3.5 dan Bab 4.2.4 (Blaze, FreeType, Skia). Nama pengembang Blaze (Aurimas Gasiulis) telah dikonfirmasi penulis.

**Berkas terdampak.** `daftar_pustaka.md`, `bab2_landasan_teori.md`, `bab3_metodologi.md`, `bab4_implementasi_dan_hasil.md`

---

## #10 — Dokumen kerja internal usang ikut di folder Skripsi

**Masalah.** `analisis_project_dan_skripsi.md` adalah dokumen kerja awal yang sudah usang (bagian E masih menyebut Bab 4/5, Abstrak, dan Daftar Pustaka sebagai lorem ipsum/tidak relevan, padahal semuanya kini sudah lengkap). Berisiko membingungkan bila ikut dikumpulkan/dicetak.

**Solusi.** Ditambahkan banner status mencolok di bagian atas file yang menandainya sebagai dokumen kerja internal yang USANG dan BUKAN bagian naskah final, serta mengarahkan pembaca ke `catatan_revisi_konsistensi.md` untuk jejak revisi terkini. File tidak dihapus agar nilai historisnya terjaga. Jika ingin benar-benar dikeluarkan dari repositori cetak, dapat dipindahkan ke luar folder `Skripsi/` atau ditambahkan ke `.gitignore`.

**Berkas terdampak.** `analisis_project_dan_skripsi.md`

---

## #11 — Anggaran sampel benchmark tidak akurat (Bab 4.4.1 & 4.4.2)

**Masalah.** Bab 4.4.1 menyatakan setiap aset diukur dengan "tiga puluh *frame* pemanasan + seratus dua puluh *frame* terukur", dan paragraf pembuka 4.4.2 beserta caption Tabel 4.4 menyebut angka itu sebagai "rerata tiga pengulangan, masing-masing atas 120 *frame* terukur." Verifikasi terhadap `examples/bench_webgl/src/lib.rs` menunjukkan anggaran sampel bersifat ADAPTIF lewat fungsi `budget_for`: aset di bawah `HEAVY_OPS_THRESHOLD = 5.000` ops memakai `WARMUP = 30` + `SAMPLES = 120`, sedangkan aset berat di atas ambang memakai `HEAVY_WARMUP = 5` + `HEAVY_SAMPLES = 20`. Karena `paris-30k.svg` memiliki 50.620 ops, baris paling penting pada Tabel 4.4 (argumen *bottleneck* CPU) sebenarnya diukur pada 5 + 20, bukan 30 + 120. Selain itu klaim "tiga pengulangan" tidak terenkode di harness (harness berjalan sekali per aset lalu melapor min/median/mean).

**Solusi.** Bab 4.4.1 ditulis ulang untuk mendeskripsikan anggaran adaptif `budget_for` secara eksplisit (ambang 5.000 ops; ringan 30+120; berat 5+20) beserta alasan determinisme beban CPU. Paragraf pembuka 4.4.2 dan caption Tabel 4.4 dikoreksi: frasa "rerata tiga pengulangan" dihapus dan diganti pernyataan jumlah sampel per aset (120 untuk el_gato dan Ghostscript_Tiger; 20 untuk paris-30k). Angka pada Tabel 4.4 tidak diubah — hanya deskripsi metodologinya yang diselaraskan dengan kode.

**Berkas terdampak.** `bab4_implementasi_dan_hasil.md`

---

## #12 — Pembersihan kode agar selaras dengan naskah (pra-sidang)

**Masalah.** Tiga celah kode yang berpotensi memancing pertanyaan penguji saat membuka repo:
1. **Dead code ukuran ubin.** `src/lib.rs` mendeklarasikan `TILE_WIDTH = 4.0` dan `TILE_HEIGHT = 4.0`, serta `src/render/webgl.rs` menyetel `tile_height: 4u32` pada Config UBO — padahal ubin efektif yang dipakai pipeline adalah 16×8 (`TILE_W`/`TILE_H` di `blocks.rs`/`builder.rs` dan `#define TILE_WIDTH 16u`/`TILE_HEIGHT 8u` di kedua shader). Angka "4" bisa disalahartikan sebagai inkonsistensi terhadap klaim 16×8 di naskah.
2. **Blend func vs klaim premultiplied alpha.** Naskah (Bab 3.4.2/3.4.3) menyebut output "premultiplied alpha" (benar: shader mengeluarkan `rgb*a*coverage`), tetapi `webgl.rs` memakai `blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA)` yang mengasumsikan sumber non-premultiplied.
3. **Komentar usang harness.** Komentar kepala `examples/bench_webgl/src/lib.rs` menyebut aset Tiger/SVG Logo/Bismillah, padahal yang di-benchmark adalah Tiger/el_gato/paris (SVG Logo & Bismillah hanya capture-only untuk Bab 4.6).

**Solusi.**
1. Konstanta `TILE_WIDTH`/`TILE_HEIGHT` (4.0) dihapus dari `src/lib.rs` beserta import-nya yang tak terpakai di `src/tile.rs`; ditambahkan komentar yang mengarahkan ke `TILE_W`/`TILE_H` di `blocks.rs`. Field `tile_height` pada Config UBO dipertahankan (untuk menjaga layout std140 tetap selaras byte-per-byte dengan shader) namun nilainya dikoreksi `4u32 → 8u32` dengan komentar penjelas bahwa shader menghitung scanline dari `#define TILE_HEIGHT 8u`, bukan dari uniform ini. Struct legacy `common::Tile` (4-px) dan konstanta `WIDTH`/`HEIGHT`/`PIXEL_ROWS = 4` ditandai sebagai prototipe lama yang tidak dipakai jalur render aktif (dikonfirmasi oleh warning kompilator `struct Tile is never constructed`).
2. Blend func diubah menjadi `blend_func(ONE, ONE_MINUS_SRC_ALPHA)` agar konsisten dengan keluaran premultiplied shader, disertai komentar. Untuk warna solid opak (alpha=1) hasil visual identik dengan sebelumnya, sehingga aset uji tidak terpengaruh; perbaikan ini benar untuk bidang semi-transparan.
3. Komentar kepala harness diperbarui agar menyebut aset yang benar (Tiger/el_gato/paris di-time; SVG Logo/Bismillah capture-only).

**Verifikasi.** `cargo check -p arabella --lib --target wasm32-unknown-unknown --features webgl` berhasil (exit 0); tidak ada error baru, hanya warning dead-code/unused-import yang sudah ada sebelumnya.

**Berkas terdampak.** `src/lib.rs`, `src/tile.rs`, `src/render/webgl.rs`, `src/render/common.rs`, `examples/bench_webgl/src/lib.rs`

---

## Catatan tindak lanjut

Seluruh item #1–#12 telah dikerjakan dan tidak ada lagi yang menggantung. Nama pengembang Blaze (Aurimas Gasiulis) sudah dikonfirmasi, sehingga seluruh entri sitasi telah terverifikasi. Item #11 menyelaraskan deskripsi metodologi benchmark dengan anggaran sampel adaptif yang benar-benar dikodekan pada harness; item #12 membersihkan tiga celah kode (dead code ukuran ubin, blend func premultiplied, komentar usang harness) agar source code selaras dengan naskah menjelang sidang.
