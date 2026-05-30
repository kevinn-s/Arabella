# Kemajuan: Penulisan Bab 5 Simpulan dan Saran

Status: **Selesai** — 25/25 tugas tuntas, seluruh validasi PASS.
Berkas keluaran: `Skripsi/bab5_kesimpulan.md`

## Ringkasan

Bab 5 ditulis penuh sebagai narasi akademik Bahasa Indonesia formal, dari heading `# BAB 5 SIMPULAN DAN SARAN` hingga lima subbab wajib (5.1, 5.2, 5.2.1–5.2.3). Berbeda dengan Bab 4 yang bersifat teknis dengan rujukan kode, Bab 5 adalah **bab sintesis**: seluruh substansi faktual diturunkan dari bab final (Bab 1, Bab 4, Abstrak) dan ketertelusurannya mengarah ke Rujukan_Bab (`Bab N`/`Subbab N.M[.K]`), bukan rujukan kode. Dua kelas constraint mendominasi dan terjaga sepanjang penulisan: **anti-fabrikasi numerik** (hanya nilai Angka_Performa_Bab4 yang dikutip) dan **kejujuran capaian** (tiada klaim `Rayon` aktif, *benchmark* kuantitatif telah dilakukan, atau keunggulan kuantitatif terhadap Skia/Cairo/Vello).

## Struktur dokumen yang dihasilkan

- 5.1 Simpulan — paragraf pembuka (taut ke Bab 1, Subbab 1.2/1.4, Bab 4), lima butir simpulan pemetaan satu-ke-satu RM-1..RM-5, dan penegasan Kontribusi_Inti
- 5.2 Saran — paragraf pengantar (taut ke Subbab 4.4/4.6)
  - 5.2.1 Saran Optimasi Performa — usulan paralelisme data via `Rayon` `multithreading` pada `pra-pemrosesan` CPU
  - 5.2.2 Saran Perluasan Fungsionalitas — lima usulan pemetaan satu-ke-satu ke lima keterbatasan Subbab 4.6
  - 5.2.3 Saran Evaluasi Lanjutan — usulan *benchmark* *head-to-head* terhadap Skia, Cairo, Vello

## Rekaman eksekusi tugas

| Wave | Tugas | Hasil |
|---|---|---|
| 0 | 1.1 Sintesis fakta antar-bab + kerangka heading bersih | Selesai |
| 1 | 2.1 Subbab 5.1 (pembuka, lima butir, Kontribusi_Inti) | Selesai |
| 2 | 3.1 Pengantar 5.2 + Subbab 5.2.1 | Selesai |
| 3 | 3.2 Subbab 5.2.2 | Selesai |
| 4 | 3.3 Subbab 5.2.3 | Selesai |
| — | 4. Checkpoint dokumen lengkap | PASS |
| 5 | 5.1–5.14 Validasi deterministik (paralel) | Semua PASS |
| — | 6. Checkpoint akhir | PASS |

## Hasil validasi Correctness Properties

| Property | Deskripsi | Hasil |
|---|---|---|
| 1 | Single-file output identity | PASS |
| 2 | Markdown validity + lorem-ipsum absence | PASS |
| 3 | Structural heading invariant | PASS |
| 4 | Positional conclusion mapping (RM-1..RM-5) | PASS |
| 5 | Core-contribution assertion (di Subbab 5.1) | PASS |
| 6 | Functional-expansion coverage (lima dimensi 4.6) | PASS |
| 7 | Evaluation-suggestion completeness (Skia/Cairo/Vello) | PASS |
| 8 | Cross-chapter traceability (Rujukan_Bab valid) | PASS |
| 9 | Numerical anti-fabrication | PASS |
| 10 | Honest-achievement invariant | PASS |
| 11 | Canonical terminology + forbidden-term elimination | PASS |
| 12 | Narrative connectivity + anti-duplication | PASS |
| 13 | Academic-style + formatting compliance | PASS |
| — | Smoke check invariansi Source-of-Truth dan bab lain (Req 2) | PASS |

## Perbaikan yang diterapkan

Pemeriksaan Property 13 (gaya/format akademik) menemukan satu ketidakkonsistenan format: istilah `Rayon` ditulis tegak (plain) pada Subbab 5.1 namun diitalik (`*Rayon*`) pada tiga kemunculan di Subbab 5.2.1.

- Sebelum: `*Rayon*` pada Subbab 5.2.1 (tiga kemunculan)
- Sesudah: `Rayon` tegak di seluruh kemunculan

Penyelarasan mengikuti konvensi nama pustaka/proper noun yang sudah dipakai untuk `Arabella`, `Skia`, `Cairo`, dan `Vello` (ditulis tegak). Setelah perbaikan, Property 13 di-validasi ulang → PASS. Validasi Property 9 (anti-fabrikasi numerik) yang sempat batal pada eksekusi paralel juga dijalankan ulang → PASS.

## Anti-fabrikasi numerik (terverifikasi)

Hanya empat nilai Angka_Performa_Bab4 yang muncul, seluruhnya identik dengan Subbab 4.4 (pemisah desimal koma + satuan) dan disertai Rujukan_Bab `Subbab 4.4` pada blok yang sama: `150,15` ms, `156,15` ms, `96 persen`, `6,4` FPS pada `paris-30k.svg`. Tiada nilai estimasi/proyeksi/target, tiada bentuk desimal titik.

## Invariansi Source-of-Truth

Tidak ada berkas source yang dimodifikasi. Inspeksi `git status` (read-only) mengonfirmasi hanya `Skripsi/bab5_kesimpulan.md` (dan metadata spec `.kiro`) yang berubah; tidak ada perubahan pada `src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, maupun delapan berkas skripsi lain (`bab1`–`bab4`, `abstrak`, `kata_pengantar`, `daftar_pustaka`, `analisis_project_dan_skripsi`). Tiada perintah mutasi (`cargo build/test/update`, formatter) dijalankan.
