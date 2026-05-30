# Catatan Revisi Konsistensi Skripsi

> Dokumen ini mencatat masalah konsistensi antar-bab yang ditemukan saat review skripsi Arabella, beserta solusi yang sudah diterapkan. Disusun sebagai jejak audit revisi (tanggal review: 30 Mei 2026).

## Ringkasan

Sepuluh perbaikan diterapkan: enam terkait konsistensi antar-bab (#1–#6), satu personalisasi Kata Pengantar (#7), dan tiga perbaikan akurasi/kerapian tambahan (#8 jumlah dependensi, #9 sitasi Blaze/FreeType, #10 penandaan dokumen kerja usang). Seluruh perbaikan menyangkut keselarasan antar-dokumen dan akurasi rujukan, bukan kebenaran teknis inti — substansi dan kejujuran ilmiah skripsi sudah baik sebelum revisi.

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

## Catatan tindak lanjut

Seluruh item #1–#10 telah dikerjakan dan tidak ada lagi yang menggantung. Nama pengembang Blaze (Aurimas Gasiulis) sudah dikonfirmasi, sehingga seluruh entri sitasi telah terverifikasi.
