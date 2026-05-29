# Kemajuan: Penulisan Bab 4 Implementasi dan Hasil

Status: **Selesai** — 37/37 tugas tuntas, seluruh validasi PASS.
Berkas keluaran: `Skripsi/bab4_implementasi_dan_hasil.md`

## Ringkasan

Bab 4 ditulis penuh sebagai narasi akademik Bahasa Indonesia formal, dari heading `# BAB 4 HASIL DAN PEMBAHASAN` hingga enam subbab wajib. Seluruh klaim teknis ditulis berdasarkan pembacaan langsung source code Arabella (bersifat read-only) dengan rujukan kode inline. Subbab yang menunggu pengukuran ditulis sebagai narasi metodologi lengkap dengan placeholder eksplisit, tanpa angka karangan.

## Struktur dokumen yang dihasilkan

- 4.1 Spesifikasi Lingkungan Implementasi
- 4.2 Implementasi Modul (pengantar + 4.2.1–4.2.8)
  - 4.2.1 Parser SVG — `src/pico_svg.rs`
  - 4.2.2 Scene API — `src/scene.rs`
  - 4.2.3 Path Processing dan Flattening — `src/path.rs`, `src/flatten.rs`
  - 4.2.4 Tile Binning DDA — `src/blocks.rs`
  - 4.2.5 Pembangkit Tile dan Akumulator Backdrop — `src/builder.rs`
  - 4.2.6 Renderer WebGL — `src/render/webgl.rs`, `src/tile.rs`
  - 4.2.7 Shader Vertex dan Fragment — berkas shader `.vert`/`.frag`
  - 4.2.8 Demo Interaktif — `examples/native_webgl/src/{lib,main}.rs`
- 4.3 Verifikasi Kebenaran Output (tiga aset uji + placeholder gambar)
- 4.4 Pengujian Performa (4.4.1 Metodologi, 4.4.2 Hasil + placeholder tabel, 4.4.3 Analisis Baseline)
- 4.5 Pembahasan Trade-off Arsitektur Non-Compute
- 4.6 Keterbatasan Implementasi Saat Ini

## Rekaman eksekusi tugas

| Wave | Tugas | Hasil |
|---|---|---|
| 0 | 1.1 Kerangka dokumen + hapus placeholder lama | Selesai |
| 1 | 2.1 Subbab 4.1 | Selesai |
| 2 | 2.2 Pengantar Subbab 4.2 | Selesai |
| 3–10 | 3.1–3.8 Subbab 4.2.1–4.2.8 | Selesai |
| — | 4. Checkpoint validasi 4.1–4.2 | PASS |
| 11–14 | 5.1–5.4 Subbab 4.3 dan 4.4 | Selesai |
| 15–16 | 6.1–6.2 Subbab 4.5 dan 4.6 | Selesai |
| — | 7. Checkpoint dokumen lengkap | PASS |
| 17 | 8.1–8.11 Validasi deterministik (paralel) | Semua PASS |
| — | 9. Checkpoint akhir | PASS |

## Hasil validasi Correctness Properties

| Property | Deskripsi | Hasil |
|---|---|---|
| 1 | Single-file output identity | PASS |
| 2 | Absence of lorem ipsum | PASS |
| 3 | Structural heading invariant | PASS |
| 4 | Absence of forbidden terms | PASS |
| 5 | Presence of required terms per subsection | PASS |
| 6 | Technical claim traceability (simbol terverifikasi ada di source) | PASS |
| 7 | Numerical anti-fabrication | PASS |
| 8 | Canonical terminology consistency | PASS |
| 9 | Placeholder format invariant | PASS |
| 10 | Cross-chapter narrative link (ke Bab 3) | PASS |
| — | Gaya bahasa akademik (tanpa token percakapan/kata ganti orang) | PASS |

## Perbaikan yang diterapkan

Pemeriksaan Property 10 menemukan satu ketidaksesuaian faktual pada paragraf pengantar Subbab 4.2: deskripsi arah outer/inner DDA terbalik terhadap source code dan terhadap Subbab 4.2.4.

- Sebelum: "outer DDA yang memecah segmen lintas kolom ubin, inner DDA yang menelusuri perlintasan di dalam baris ubin"
- Sesudah: "outer DDA yang memecah segmen lintas baris ubin, inner DDA yang menelusuri perlintasan antar-kolom di dalam satu baris ubin"

Diverifikasi terhadap `src/blocks.rs`: `bin_line` (outer DDA) maju lintas baris ubin, `bin_line_in_row` (inner DDA) menelusuri kolom di dalam satu baris. Urutan delapan tahap pipeline tetap utuh.

## Invariansi Source-of-Truth

Tidak ada berkas source yang dimodifikasi. `git status` mengonfirmasi hanya `Skripsi/bab4_implementasi_dan_hasil.md` dan metadata spec `.kiro` yang berubah; tidak ada perubahan pada `src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`.

## Catatan kecil (non-blocking)

Kata "paint" diitalik secara tidak konsisten pada beberapa frasa prosa (mis. "operasi paint", "jenis paint"). Bisa dinormalkan saat pembacaan akhir, tidak menggugurkan validasi mana pun.
