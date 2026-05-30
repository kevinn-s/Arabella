# Implementation Plan: Penulisan Bab 5 Simpulan dan Saran

## Overview

Rencana implementasi ini menerjemahkan desain menjadi serangkaian tugas penulisan dan validasi konkret terhadap satu berkas keluaran tunggal `Skripsi/bab5_kesimpulan.md`. "Implementasi" di sini berarti penulisan narasi akademik Markdown berbahasa Indonesia formal, bukan penulisan kode aplikasi. Source code Arabella (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`) dan seluruh berkas skripsi lain (`bab1`–`bab4`, `abstrak`, `kata_pengantar`, `daftar_pustaka`, `analisis_project_dan_skripsi`) bersifat **read-only / source-of-truth** dan TIDAK BOLEH dimodifikasi oleh tugas mana pun (Requirement 2).

Berbeda dengan Bab 3 dan Bab 4 yang bersifat teknis dengan ketertelusuran ke source code, **Bab 5 adalah bab sintesis**: seluruh substansi faktualnya diturunkan dari bab final (Bab 1, Bab 4, Abstrak) dan ketertelusurannya mengarah ke Rujukan_Bab (`Bab N`/`Subbab N.M[.K]`), bukan ke rujukan kode. Dua kelas constraint mendominasi: **anti-fabrikasi numerik** (hanya nilai Angka_Performa_Bab4 yang boleh dikutip) dan **kejujuran capaian** (tiada klaim `Rayon` aktif, *benchmark* kuantitatif dilakukan, atau keunggulan kuantitatif terhadap Skia/Cairo/Vello).

Pendekatan eksekusi mengikuti pipeline tiga tahap pada desain: **(Tahap 1)** sintesis fakta dari bab final dan terminologi kanonik dari Bab 3, sekaligus membangun kerangka heading dan menghapus *lorem ipsum*; **(Tahap 2)** menulis narasi per Subbab_Wajib (5.1 Simpulan; 5.2 Saran beserta 5.2.1/5.2.2/5.2.3) dengan menyisipkan Rujukan_Bab pada setiap Klaim_Sintesis; **(Tahap 3)** validasi deterministik (regex + struktural + cross-reference antar-bab) yang memetakan langsung ke tiga belas Correctness Properties pada desain.

Karena seluruh tugas penulisan mengedit berkas yang sama, tugas-tugas tersebut dijadwalkan berurutan antar-wave; tugas validasi bersifat read-only sehingga dapat dijalankan paralel setelah konten selesai. Sesuai bagian "Mengapa Property-Based Testing (Iterasi Acak) Tidak Berlaku" pada desain, validasi properti diimplementasikan sebagai pencarian teks/struktural deterministik (PASS/FAIL sekali jalan), bukan *property-based testing* iteratif acak.

## Tasks

- [x] 1. Tahap 1 — Sintesis fakta antar-bab dan pembangunan kerangka dokumen
  - [x] 1.1 Sintesis fakta dari bab final dan tulis ulang `Skripsi/bab5_kesimpulan.md` sebagai kerangka heading bersih
    - **Sintesis (read-only):** baca `Skripsi/bab1_pendahuluan.md` (Subbab 1.2 → RM-1..RM-5; Subbab 1.4 → TP-1..TP-4), `Skripsi/bab4_implementasi_dan_hasil.md` (Subbab 4.1/4.2 → pembagian beban CPU/GPU; Subbab 4.4 → Angka_Performa_Bab4 dan *bottleneck* CPU; Subbab 4.4.3 → disclaimer *benchmark*; Subbab 4.5 → *trade-off* kualitatif; Subbab 4.6 → lima keterbatasan + status `Rayon`/`multithreading` opsional), `Skripsi/abstrak.md` (Kontribusi_Inti), dan `Skripsi/bab3_metodologi.md` (istilah kanonik + konstanta numerik kanonik), tanpa operasi tulis apa pun pada berkas tersebut
    - Catat Fakta_Sintesis terstruktur (Data Model 1) per Subbab_Wajib sebagai basis kalimat Tahap 2, termasuk himpunan Angka_Performa_Bab4 (`150,15`, `156,15`, `96 persen`, `6,4`, dst.) yang menjadi satu-satunya nilai numerik performa yang boleh dikutip
    - Tulis baris pertama persis `# BAB 5 SIMPULAN DAN SARAN` (ATX, satu tanda pagar + satu spasi tunggal, kapitalisasi penuh, tanpa BOM, tanpa whitespace awal/akhir, diakhiri tepat satu newline)
    - Tulis dua heading level 2 berurutan menaik: `## 5.1 Simpulan`, `## 5.2 Saran`, dan tiga subheading level 3 kontigu di bawah 5.2: `### 5.2.1 Saran Optimasi Performa`, `### 5.2.2 Saran Perluasan Fungsionalitas`, `### 5.2.3 Saran Evaluasi Lanjutan`; TIDAK ada heading bersarang di bawah 5.1; TIDAK ada heading level 2/3 lain di luar Subbab_Wajib
    - Hapus seluruh teks *lorem ipsum* lama (`lorem ipsum`, `dolor sit amet`, `consectetur adipiscing`, `tempor incididunt`, `exercitation ullamco`, `duis aute irure`) sehingga 0 *hit*; pastikan encoding UTF-8 tanpa BOM dan Markdown CommonMark valid
    - Konfirmasi tidak ada berkas Bab 5 duplikat/draf di luar `.kiro/specs/` dan TIDAK memodifikasi Source_Of_Truth maupun berkas skripsi lain (Req 2)
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 13.4_

- [x] 2. Tahap 2 — Tulis Subbab 5.1 Simpulan
  - [x] 2.1 Tulis paragraf pembuka, lima butir simpulan, dan penegasan Kontribusi_Inti pada Subbab 5.1
    - Tulis paragraf pembuka yang menyatakan simpulan disusun untuk menjawab Rumusan_Masalah dan Tujuan_Penelitian, memuat token literal `Bab 1`, `Subbab 1.2`, dan `Subbab 1.4`, serta minimal satu kalimat penghubung bermuatan token literal `Bab 4` (konektivitas naratif)
    - Tulis tepat lima butir simpulan (ordered list lima item atau lima paragraf berurutan) yang memetakan RM-1..RM-5 secara posisional satu-ke-satu dan menaik monotonik, didahului kalimat pengantar naratif:
      - butir RM-1: arsitektur *pipeline hibrida* *non-compute* terwujud dengan pembagian beban `CPU` (*flattening*, `binning DDA`, `akumulator signed-area`, `propagasi backdrop`) dan `GPU` (*vertex shader*, `fragment shader`); Rujukan_Bab ke `Bab 4`/`Subbab 4.1`/`Subbab 4.2`
      - butir RM-2: fondasi struktur data berbasis ubin + `SIMD` aktif, sementara `Rayon` `multithreading` BELUM aktif; Rujukan_Bab ke `Subbab 4.6` + tautan ke `Subbab 5.2.1`
      - butir RM-3: pembagian beban efektif untuk skala kecil–menengah, biaya `GPU` landai/stabil sedangkan `pra-pemrosesan` `CPU` mendominasi; Rujukan_Bab ke `Subbab 4.4`
      - butir RM-4: pada skala sangat besar tahap `pra-pemrosesan` CPU *single-thread* menjadi *bottleneck* dominan; jika menyertakan angka, gunakan Angka_Performa_Bab4 identik (`150,15` ms dari `156,15` ms total, sekitar `96 persen`, `6,4` FPS pada `paris-30k.svg`); Rujukan_Bab ke `Subbab 4.4`
      - butir RM-5: perbandingan terhadap Skia, Cairo, Vello bersifat kualitatif dan *benchmark* kuantitatif langsung BELUM dilakukan; Rujukan_Bab ke `Subbab 4.4`/`Subbab 4.4.3` dan `Subbab 4.5`
    - Tutup Subbab 5.1 dengan penegasan Kontribusi_Inti yang memuat frasa literal `pipeline`, `non-compute`, `WebGL 2.0`, dan `Arabella`; tempatkan penegasan ini di Subbab 5.1 (BUKAN 5.2) tanpa mengklaim keunggulan performa kuantitatif
    - Pastikan setiap Klaim_Sintesis memiliki minimal satu Rujukan_Bab pada blok yang sama; tanpa rujukan kode `` `berkas:simbol` ``; tanpa angka di luar Angka_Performa_Bab4; tanpa klaim capaian yang tidak dilaporkan Bab 4 (Req 4.9)
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 5.1, 5.2, 5.3, 5.4, 9.1, 9.2, 9.3, 9.4, 9.5, 10.1, 10.5, 11.2, 11.3, 12.1, 12.4, 12.5, 13.2, 13.5_

- [x] 3. Tahap 2 — Tulis Subbab 5.2 Saran
  - [x] 3.1 Tulis paragraf pengantar Subbab 5.2 dan isi Subbab 5.2.1 Saran Optimasi Performa
    - Tulis paragraf pengantar `## 5.2 Saran` yang menyatakan saran pengembangan lanjutan diturunkan dari keterbatasan dan temuan pada Bab 4, memuat minimal satu Rujukan_Bab ke `Subbab 4.4` atau `Subbab 4.6`
    - Tulis Subbab 5.2.1 yang mengusulkan pengaktifan paralelisme tingkat data via `Rayon` `multithreading` pada `pra-pemrosesan` CPU (*flattening* dan `binning DDA` per jalur), memuat token literal `Rayon` dan `multithreading`
    - Tautkan usulan ke *bottleneck* CPU *single-thread* dengan Rujukan_Bab ke `Subbab 4.4`, dan nyatakan beban tiap jalur saling independen sehingga dapat-diparalelkan dengan Rujukan_Bab ke `Subbab 4.6` (tempat feature `multithreading` opsional dideklarasikan)
    - Nyatakan usulan sebagai potensi optimasi yang masih perlu diverifikasi melalui pengukuran lanjutan; JANGAN menjanjikan angka peningkatan spesifik (faktor *speedup*, FPS target, persentase pengurangan, waktu target `ms`); bila menyitir angka pendukung, gunakan Angka_Performa_Bab4 identik (koma desimal + satuan)
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 9.1, 9.2, 10.1, 10.2, 12.2, 12.5, 13.2, 13.5_

  - [x] 3.2 Tulis Subbab 5.2.2 Saran Perluasan Fungsionalitas
    - Tulis pengantar naratif lalu daftar/paragraf berisi minimal lima usulan yang dapat diidentifikasi terpisah dan dipetakan satu-ke-satu ke kelima keterbatasan fungsional Subbab 4.6: (1) paint bergradien pada `fragment shader`; (2) *image paint* dan *tinting*; (3) perluasan subset SVG di luar `g`/`path`; (4) sistem *text rendering*; (5) *filter effect* (*blur*/*drop shadow*)
    - Gunakan istilah kanonik `fragment shader`; JANGAN perkenalkan nama konstanta/fungsi/struct baru yang tidak ada pada Bab 4; nyatakan seluruh usulan sebagai *future work*/pengembangan lanjutan yang belum terimplementasi
    - Sertakan minimal satu Rujukan_Bab ke `Subbab 4.6` yang menautkan usulan ke keterbatasan terdokumentasi
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 9.1, 9.2, 11.1, 13.5_

  - [x] 3.3 Tulis Subbab 5.2.3 Saran Evaluasi Lanjutan
    - Tulis usulan *benchmark* kuantitatif langsung (*head-to-head*) yang membandingkan `Arabella` terhadap Skia, Cairo, dan Vello (sebut ketiganya secara literal)
    - Tautkan usulan ke disclaimer *benchmark* kuantitatif belum dilakukan dengan Rujukan_Bab ke `Subbab 4.4.3` (atau `Subbab 4.4`); nyatakan syarat *benchmark* harus dijalankan pada perangkat keras dan berkas uji yang identik antar-renderer
    - JANGAN menyatakan prakiraan hasil (mis. Arabella akan lebih cepat/lambat); minimal satu kali di salah satu Subbab_Wajib nyatakan keterbatasan tidak menggugurkan Kontribusi_Inti dengan kata kunci `future work` atau `pengembangan lanjutan`
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 9.1, 9.2, 12.5, 13.5_

- [x] 4. Checkpoint - Dokumen lengkap, siap divalidasi
  - Pastikan kelima Subbab_Wajib (5.1, 5.2, 5.2.1–5.2.3) terisi penuh, setiap Klaim_Sintesis memiliki Rujukan_Bab, tidak ada Source_Of_Truth maupun berkas skripsi lain yang termodifikasi. Ensure all tests pass, ask the user if questions arise.

- [x] 5. Tahap 3 — Validasi deterministik (pemetaan Correctness Properties)
  - [x] 5.1 Validasi Property 1 (single-file output identity)
    - **Property 1: Single-File Output Identity**
    - Pencarian rekursif berkas yang namanya mengandung `bab5` (*case-insensitive*) atau yang baris pertamanya `# BAB 5 SIMPULAN DAN SARAN`, abaikan `.kiro/specs/`; pastikan hanya `Skripsi/bab5_kesimpulan.md` yang memenuhi dan memuat seluruh Subbab_Wajib; tiada salinan/cadangan/draf alternatif
    - **Validates: Requirements 1.1, 1.5, 1.6**

  - [x] 5.2 Validasi Property 2 (markdown validity + lorem-ipsum absence)
    - **Property 2: Markdown Validity and Lorem-Ipsum Absence**
    - Verifikasi `.md` + UTF-8 tanpa BOM + CommonMark valid; pencarian *case-insensitive* `lorem ipsum`, `dolor sit amet`, `consectetur adipiscing`, `tempor incididunt`, `exercitation ullamco`, `duis aute irure` → PASS = 0 *hit*
    - **Validates: Requirements 1.2, 1.4, 1.7**

  - [x] 5.3 Validasi Property 3 (structural heading invariant)
    - **Property 3: Structural Heading Invariant**
    - Verifikasi baris pertama berkas; kehadiran tepat satu kali setiap Subbab_Wajib pada level benar (regex `^##\s+5\.\d+\s+.*$` dan `^###\s+5\.\d+\.\d+\s+.*$`); urutan menaik monotonik 5.1→5.2 dan 5.2.1→5.2.2→5.2.3 kontigu; tiada heading lain di luar himpunan; tiada heading bersarang di bawah 5.1; kecualikan *fenced code block*
    - **Validates: Requirements 1.3, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 13.4**

  - [x] 5.4 Validasi Property 4 (positional conclusion mapping)
    - **Property 4: Positional Conclusion Mapping**
    - Verifikasi Subbab 5.1 memuat tepat lima butir; butir ke-`n` menjawab RM-`n` (satu-ke-satu, menaik monotonik); paragraf pembuka memuat `Bab 1`, `Subbab 1.2`, `Subbab 1.4`; istilah wajib + Rujukan_Bab per butir (Data Model 6); gabungan butir mencerminkan TP-1..TP-4
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.7, 4.8**

  - [x] 5.5 Validasi Property 5 (core-contribution assertion)
    - **Property 5: Core-Contribution Assertion**
    - Verifikasi penegasan Kontribusi_Inti berada di rentang Subbab 5.1 (BUKAN 5.2) dan memuat `pipeline`, `non-compute`, `WebGL 2.0`, `Arabella`, konsisten dengan Abstrak
    - **Validates: Requirements 5.1, 5.2, 5.3**

  - [x] 5.6 Validasi Property 6 (functional-expansion coverage)
    - **Property 6: Functional-Expansion Coverage**
    - Verifikasi Subbab 5.2.2 memuat ≥5 usulan terpisah yang mencakup kelima dimensi keterbatasan Subbab 4.6 (Data Model 5), Rujukan_Bab ke `Subbab 4.6`, istilah `fragment shader`, tiada identifier kode baru, dan penanda *future work*
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5, 7.6**

  - [x] 5.7 Validasi Property 7 (evaluation-suggestion completeness)
    - **Property 7: Evaluation-Suggestion Completeness**
    - Verifikasi Subbab 5.2.3 mengusulkan *benchmark* *head-to-head* menyebut `Skia`, `Cairo`, `Vello` literal, Rujukan_Bab ke `Subbab 4.4.3`/`Subbab 4.4`, dan syarat perangkat keras/berkas uji identik antar-renderer
    - **Validates: Requirements 8.1, 8.2, 8.3**

  - [x] 5.8 Validasi Property 8 (cross-chapter traceability)
    - **Property 8: Cross-Chapter Traceability**
    - Untuk setiap blok Klaim_Sintesis, verifikasi ≥1 Rujukan_Bab pada blok yang sama (regex `Bab\s+[1-5]` / `Subbab\s+\d+\.\d+(\.\d+)?`); cross-check setiap Rujukan_Bab terhadap himpunan valid Data Model 9; verifikasi tiada rujukan kode `` `berkas:simbol` `` sebagai dasar klaim
    - **Validates: Requirements 9.1, 9.2, 9.3, 9.5, 12.2**

  - [x] 5.9 Validasi Property 9 (numerical anti-fabrication)
    - **Property 9: Numerical Anti-Fabrication**
    - Pencarian regex pola numerik berunit performa `\b\d+(?:[.,]\d+)?\s*(?:fps|FPS|ms|persen|%)\b`; setiap kemunculan wajib identik dengan Angka_Performa_Bab4 (Data Model 4, termasuk koma desimal + satuan) dan disertai Rujukan_Bab ke `Subbab 4.4` pada blok yang sama; konstanta numerik kanonik (`16×8`, `Tile` `44 byte`, `1080×520`, `F24Dot8`, `8.8 fixed-point`, `WINDING_UNIT = 256`) identik dengan Bab 3/4; tiada nilai estimasi/proyeksi/target/eksternal
    - **Validates: Requirements 4.6, 6.2, 9.4, 10.1, 10.2, 10.3, 10.4, 10.5**

  - [x] 5.10 Validasi Property 10 (honest-achievement invariant)
    - **Property 10: Honest-Achievement Invariant**
    - Pencarian pola klaim terlarang afirmatif (`Rayon` aktif/diaktifkan; *benchmark* kuantitatif telah dilakukan; Arabella unggul/lebih cepat dari Skia/Cairo/Vello; prakiraan hasil) → PASS = 0 *hit*; verifikasi Subbab 5.2.1 tidak menjanjikan angka peningkatan spesifik dan memuat kualifikasi "perlu diverifikasi/pengukuran lanjutan" + argumen independensi beban (Rujukan_Bab `Subbab 4.4`, `Subbab 4.6`)
    - **Validates: Requirements 4.9, 5.4, 6.1, 6.3, 6.4, 6.5, 8.4**

  - [x] 5.11 Validasi Property 11 (canonical terminology + forbidden-term elimination)
    - **Property 11: Canonical Terminology and Forbidden-Term Elimination**
    - Verifikasi kehadiran ≥1 tiap istilah kanonik global (`pipeline hibrida`, `non-compute`/`tanpa compute shader`, `WebGL 2.0`, `Arabella`, `pra-pemrosesan`, `CPU`, `GPU`); `winding number` hadir dan `winding_number` (underscore) tidak; tiada paragraf mencampur `pra-pemrosesan`/`preprocessing`; penomoran `5.x`/`5.x.y`; pencarian Istilah_Terlarang (Data Model 7) → PASS = 0 *hit* dalam konteks terlarang
    - **Validates: Requirements 11.1, 11.2, 11.3, 11.4, 11.5, 11.6**

  - [x] 5.12 Validasi Property 12 (narrative connectivity + anti-duplication)
    - **Property 12: Narrative Connectivity and Anti-Duplication**
    - Verifikasi token `Bab 4` hadir di Subbab 5.1; Rujukan_Bab `Subbab 4.4`/`Subbab 4.6` hadir di Subbab 5.2; kemunculan `future work`/`pengembangan lanjutan` pada konteks validitas Kontribusi_Inti; *sliding window* 30 kata terhadap Bab 1/3/4 → PASS = tiada jendela identik
    - **Validates: Requirements 12.1, 12.3, 12.5**

  - [x] 5.13 Validasi Property 13 (academic-style + formatting compliance)
    - **Property 13: Academic-Style and Formatting Compliance**
    - Pencarian *word-boundary case-insensitive* di luar *fenced code block* untuk token percakapan (`bisa`, `gak`, `enggak`, `nih`, `dong`, `kok`, `kan`, `aja`, `udah`, `mau`) dan kata ganti orang pertama/kedua (`saya`, `kami`, `kita`, `Anda`, `kamu`) → PASS = 0 *hit*; verifikasi *backtick* untuk identifier dan *italic* konsisten untuk istilah teknis Inggris; tiap Subbab_Wajib ≥1 paragraf naratif ≥3 kalimat; tiap daftar didahului kalimat pengantar
    - **Validates: Requirements 13.1, 13.2, 13.3, 13.5**

  - [x] 5.14 Smoke check invariansi Source_Of_Truth dan bab lain (Req 2)
    - Bandingkan himpunan jalur relatif + konten byte Source_Of_Truth (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`) mulai vs akhir → PASS = identik
    - Bandingkan konten delapan berkas skripsi lain (`bab1`, `bab2`, `bab3`, `bab4`, `abstrak`, `kata_pengantar`, `daftar_pustaka`, `analisis_project_dan_skripsi`) mulai vs akhir → PASS = tiada perubahan; audit tiada perintah mutasi (`cargo build/test/update`, formatter) dijalankan
    - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [x] 6. Checkpoint akhir - Seluruh validasi PASS
  - Iterasi pada `Skripsi/bab5_kesimpulan.md` hingga seluruh Property 1–13 dan *smoke check* invariansi PASS, tanpa pernah memodifikasi Source_Of_Truth maupun berkas skripsi lain. Ensure all tests pass, ask the user if questions arise.

## Notes

- Tugas bertanda `*` adalah validasi deterministik (pencarian teks/struktural + cross-reference), bukan *property-based testing* iteratif acak — sesuai bagian "Mengapa Property-Based Testing Tidak Berlaku" pada desain. Tugas ini opsional untuk MVP namun sangat direkomendasikan karena memetakan langsung ke tiga belas Correctness Properties dan menjaga integritas akademik (terutama anti-fabrikasi numerik dan kejujuran capaian).
- Setiap tugas penulisan mengedit berkas keluaran tunggal yang sama (`Skripsi/bab5_kesimpulan.md`); karena itu tugas penulisan dijadwalkan berurutan antar-wave, sedangkan tugas validasi read-only dapat berjalan paralel.
- Requirement 2 (Source-of-Truth dan Bab-Lain Invariance) berlaku global: Source_Of_Truth dan delapan berkas skripsi lain dibaca strict read-only dan tidak boleh dimodifikasi, ditambah, dihapus, atau dipindah. Tidak boleh menjalankan perintah yang memutasi Source_Of_Truth.
- Bab 5 adalah bab sintesis: ketertelusuran mengarah ke Rujukan_Bab (`Bab N`/`Subbab N.M[.K]`), bukan rujukan kode `` `berkas:simbol` ``. Hanya nilai Angka_Performa_Bab4 yang boleh dikutip sebagai metrik performa.
- Konvensi penomoran, gaya bahasa, dan format mengikuti spec pendahulu `penulisan-bab4-implementasi-dan-hasil` dan `revisi-bab3-metodologi`.
- Checkpoint memastikan validasi inkremental sebelum melanjutkan ke tahap berikutnya.

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["2.1"] },
    { "id": 2, "tasks": ["3.1"] },
    { "id": 3, "tasks": ["3.2"] },
    { "id": 4, "tasks": ["3.3"] },
    { "id": 5, "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9", "5.10", "5.11", "5.12", "5.13", "5.14"] }
  ]
}
```
