# Catatan Restrukturisasi Bab 3 (Versi Alternatif / v2)

> Dokumen ini merekam seluruh perubahan berkas Markdown yang dilakukan saat menyusun versi alternatif Bab 3 beserta penyesuaian rujukan silang pada Bab 4 dan Bab 5. Disusun sebagai jejak audit (tanggal kerja: 30 Mei 2026). Versi asli Bab 3 yang mengikuti template UML klasik **tidak diubah** dan tetap tersedia pada `Skripsi/bab3_metodologi.md`.

## Ringkasan

Dibuat satu versi alternatif Bab 3 (`bab3_metodologi_v2.md`) dengan struktur yang ditata ulang mengikuti alur pipeline rendering (Metode Penelitian → Analisis → Perancangan), lalu salinan Bab 4 dan Bab 5 pada folder `alternateVersion/` disesuaikan rujukan silangnya agar konsisten dengan penomoran baru. Tidak ada klaim teknis yang diubah substansinya; perubahan hanya pada organisasi penyajian dan nomor rujukan. Prinsip ketertelusuran kode (`berkas:simbol` / `berkas:start-end`) dipertahankan penuh.

---

## Berkas yang Terlibat

| Berkas | Status |
|---|---|
| `Skripsi/bab3_metodologi.md` (versi asli) | **TIDAK diubah** — tetap utuh sebagai versi template UML klasik |
| `Skripsi/alternateVersion/bab3_metodologi_v2.md` | **Dibuat baru** |
| `Skripsi/alternateVersion/bab4_implementasi_dan_hasil.md` | **Diedit** (4 rujukan silang) |
| `Skripsi/alternateVersion/bab5_kesimpulan.md` | Diperiksa — tidak ada perubahan diperlukan |
| `Skripsi/alternateVersion/catatan_restrukturisasi_bab3_v2.md` | Dokumen catatan ini |

Catatan: berkas `bab3_metodologi_v2.md`, `bab4_implementasi_dan_hasil.md`, dan `bab5_kesimpulan.md` dipindahkan/disalin penulis ke folder `alternateVersion/`. Penyesuaian rujukan silang dilakukan pada salinan di dalam folder tersebut.

---

## 1. Pembuatan `bab3_metodologi_v2.md`

Struktur baru disusun sebagai berikut:

```
3.1 Metode Penelitian
3.2 Analisis
    3.2.1 Analisis Perbandingan dengan Aplikasi Sejenis
    3.2.2 Analisis Permasalahan
    3.2.3 Usulan Pemecahan Masalah
3.3 Perancangan
    3.3.1 Gambaran Umum Arsitektur
    3.3.2 Input Path
        3.3.2.1 Garis
        3.3.2.2 Kurva Bézier Kuadratik
        3.3.2.3 Kurva Bézier Kubik
    3.3.3 Aturan Fill
        3.3.3.1 Winding Number
        3.3.3.2 Aturan Non-Zero
        3.3.3.3 Aturan Even-Odd
    3.3.4 Pewarnaan
        3.3.4.1 Warna Solid
    3.3.5 Rasterisasi Kasar (CPU)
        3.3.5.1 Gambaran Umum
        3.3.5.2 Konversi Kurva Kubik ke Kuadratik
        3.3.5.3 Flattening Kurva Kuadratik (Midpoint Subdivision)
        3.3.5.4 Representasi Fixed-Point (F24Dot8)
        3.3.5.5 Digital Differential Analyzer (DDA)
            3.3.5.5.1 DDA Luar: Pembagian Baris
            3.3.5.5.2 DDA Dalam: Pembagian Kolom
        3.3.5.6 Akumulasi Crossing per Scanline
        3.3.5.7 Pencatatan dan Pengurutan Segmen
        3.3.5.8 Generasi Tile (Propagasi Backdrop)
        3.3.5.9 Persiapan Resource WebGL
    3.3.6 Rasterisasi Halus (GPU)
        3.3.6.1 Vertex Shader: Instancing Tile
        3.3.6.2 Pengambilan Segmen dari Tekstur
        3.3.6.3 Analytic Coverage (Box Filter)
        3.3.6.4 Penerapan Aturan Fill
        3.3.6.5 Pewarnaan dan Kompositing
3.4 Perancangan Layar
```

### Tiga keputusan desain yang diterapkan

1. **Gradien dihapus (opsi 1).** Subbab 3.3.4 Pewarnaan hanya memuat 3.3.4.1 Warna Solid (`unpack_rgba8`, keluaran premultiplied alpha). Ditambahkan satu kalimat jujur bahwa kerangka gradien/image adalah struktur data warisan basis kode Vello pada `src/paint/` yang **belum tersambung** ke pipeline WebGL, sehingga tidak dibahas sebagai bagian perancangan yang dievaluasi. Ini menjaga aturan ketertelusuran kode dan konsisten dengan temuan di `analisis_project_dan_skripsi.md` (D.2 no.12) bahwa `Scene::encode_paint` untuk gradien masih TODO.

2. **Diagram UML dilipat ke 3.3.1 (opsi: dilipat ke struktur baru).** Use case, sequence, dan class diagram dari versi lama (3.4.1–3.4.4) diringkas menjadi sub-bagian naratif di dalam 3.3.1 Gambaran Umum Arsitektur:
   - Spesifikasi Pustaka (sebelumnya 3.3.1)
   - Pembagian Beban Kerja Hibrida
   - Interaksi Fungsional (Use Case) — UC-01, UC-02, UC-03
   - Alur Sekuensial Satu Bingkai (Sequence) — lima pesan berurutan
   - Struktur Kelas Inti (Class) — disajikan sebagai **Tabel 3.2** (sembilan struct inti + rujukan kode)

3. **Analisis User dan Perancangan Layar dipertahankan.**
   - Analisis User → menjadi paragraf pengantar Subbab 3.2 (konteks pengguna = developer).
   - Perancangan Layar → dipindah utuh ke Subbab 3.4 (resolusi DPR-aware, overlay FPS empat metrik, kanvas tes 1080×520).

### Pemetaan konten lama → struktur baru

| Struktur baru (v2) | Sumber konten versi lama |
|---|---|
| 3.1 Metode Penelitian | 3.1 Diagram Alir Kerangka Berpikir (5 fase) |
| 3.2 (pengantar) | 3.2.1 Analisis User |
| 3.2.1 Perbandingan Aplikasi Sejenis | 3.2.2 |
| 3.2.2 Analisis Permasalahan | 3.2.3 (kolom rumusan masalah) |
| 3.2.3 Usulan Pemecahan Masalah | 3.2.3 (kolom solusi) → Tabel 3.1 |
| 3.3.1 Gambaran Umum Arsitektur | 3.3.1 (spesifikasi) + 3.4.1–3.4.4 (UML) + intro 3.5 |
| 3.3.2 Input Path | 3.5(a) + verifikasi `src/path.rs` (event Line/Quadratic/Cubic) |
| 3.3.3 Aturan Fill | 3.5(f) |
| 3.3.4 Pewarnaan → Warna Solid | shader `unpack_rgba8` (ditulis ringkas) |
| 3.3.5 Rasterisasi Kasar (CPU) | 3.5(a)–(d) + 3.7 (tata letak buffer) |
| 3.3.6 Rasterisasi Halus (GPU) | 3.5(e)–(f) + `render_tile.vert`/`render_tile.frag` |
| 3.4 Perancangan Layar | 3.6 Perancangan Layar |

### Tabel pada v2

- **Tabel 3.1** — Pemetaan rumusan permasalahan dan solusi teknis (3 baris).
- **Tabel 3.2** — Sembilan struct inti pustaka Arabella dan perannya.
- **Tabel 3.3** — Tata letak byte vertex buffer instanced 44 byte/tile (`src/tile.rs:9-23`).

### Verifikasi kode saat penulisan v2

Klaim teknis pada subbab baru ditelusuri langsung ke source code:
- `src/path.rs` — `fill_impl`, penanganan `PathEvent::{Line, Quadratic, Cubic, End}`, `emit_line`, `f32_to_f24dot8`.
- `src/flatten.rs` — `flatten_quadratic`, `flatten_recursive`, `is_flat_enough`, `FLATNESS_THRESHOLD`.
- `src/blocks.rs` — outer/inner DDA, `record_per_scanline_crossings`, konstanta `TILE_W`/`TILE_H`/`TILE_*_F24DOT8`.
- `src/builder.rs` — `build_path`, `generate_tiles`, propagasi backdrop, `CoverStorage`.
- `src/tile.rs` — tata letak `#[repr(C)]` 44 byte.
- `src/render/webgl.rs` — `initialize_tile_vao`, `upload_data_to_rgba32f_texture`, `draw_arrays_instanced`.
- `src/render/shaders/render_tile.vert` — instancing quad, NDC mapping.
- `src/render/shaders/render_tile.frag` — `line_box`, `unpack_rgba8`, fill rule NonZero/EvenOdd, `WINDING_UNIT`.
- `src/paint/paint.rs` — `PremulColor::from_alpha_color`.

---

## 2. Penyesuaian rujukan silang di `alternateVersion/bab4_implementasi_dan_hasil.md`

Empat rujukan ke subbab lama `Subbab 3.4.4` (class diagram) diubah menjadi `Subbab 3.3.1` (struktur kelas inti pada Bab 3, Tabel 3.2), karena pada v2 class diagram dilipat menjadi Tabel 3.2 di dalam 3.3.1. Frasa "kotak kelas pada diagram" disesuaikan menjadi "kelas" agar natural terhadap penyajian tabel.

| Lokasi | Lama | Baru |
|---|---|---|
| Pengantar 4.2 Implementasi Modul | `Subbab 3.4.4` (class diagram) | `Subbab 3.3.1` (struktur kelas inti, Tabel 3.2) |
| 4.2.4 Tile Binning DDA | `Subbab 3.4.4` (class diagram) | `Subbab 3.3.1` (struktur kelas inti, Tabel 3.2) |
| 4.2.5 Pembangkit Tile dan Akumulator Backdrop | `Subbab 3.4.4` (class diagram) | `Subbab 3.3.1` (struktur kelas inti, Tabel 3.2) |
| 4.2.6 Renderer WebGL | `Subbab 3.4.4` (class diagram) | `Subbab 3.3.1` (struktur kelas inti, Tabel 3.2) |

### Rujukan yang sengaja TIDAK diubah (tetap valid)

- "Render Frame (UC-03) pada Bab 3" — UC-03 dipertahankan di v2 (3.3.1, bagian Interaksi Fungsional).
- Rujukan internal Bab 4 (4.1, 4.3, 4.4, 4.6) — penomoran Bab 4 tidak berubah.
- Rujukan ke Bab 2 (`Subbab 2.2.3`, `2.2.4`, `2.2.8`) pada 4.4.3 dan 4.5 — di luar lingkup restrukturisasi Bab 3.

---

## 3. Pemeriksaan `alternateVersion/bab5_kesimpulan.md`

Diperiksa untuk rujukan Chapter 3. **Tidak ditemukan** rujukan silang ke subbab Bab 3, sehingga tidak ada perubahan yang diperlukan.

---

## Verifikasi akhir

- Pencarian pola `3.4.x`, `3.5`, `3.6`, `3.7` pada kedua berkas Bab 4/5 → tidak ada sisa.
- Seluruh rujukan Bab 3 kini menunjuk `Subbab 3.3.1`.
- `getDiagnostics` pada `bab3_metodologi_v2.md`, `bab4_implementasi_dan_hasil.md`, dan `bab5_kesimpulan.md` → bersih, tanpa diagnostik.

## Catatan tindak lanjut (belum dikerjakan)

- Bila versi alternatif menjadi naskah final, periksa juga rujukan ke Bab 2 (`2.2.3`, `2.2.4`, `2.2.8`) seandainya penomoran Bab 2 ikut berubah.
- Pastikan daftar isi / table of contents (bila ada berkas terpisah) diperbarui agar selaras dengan struktur v2.
