# Log Kemajuan Revisi Bab 3 Metodologi

**Spec:** `revisi-bab3-metodologi`
**Tanggal selesai:** 28 Mei 2026
**Status keseluruhan:** ✅ **SELESAI** — 32/32 task completed, 8/8 correctness property PASS
**Berkas keluaran utama:** `Skripsi/bab3_metodologi.md` (60 981 byte, 381 baris)

---

## Ringkasan Eksekutif

Bab 3 Metodologi pada skripsi telah direvisi total agar setiap klaim teknis selaras dengan source code Arabella di `src/`, `Cargo.toml`, `examples/`, dan `tests/`. Revisi ini menggantikan narasi lama yang merujuk algoritma ray shooting, klasifikasi tipe ubin (TileType / EMPTY / INTERIOR / EDGE), fungsi implisit linear/kuadratik kanonik/kubik PPGA, serta klaim Rust edisi 2021 dengan deskripsi pipeline aktual: binning DDA dua tahap, akumulator signed-area per scanline 8.8 fixed-point, propagasi backdrop kiri-ke-kanan, evaluasi cakupan piksel `line_box` (integral trapezoidal), dan fill rule NonZero / EvenOdd pada fragment shader analitik tunggal.

Pekerjaan terbagi menjadi tiga fase deterministik: (1) ekstraksi fakta dari source code, (2) penulisan ulang per subbab, (3) validasi pasca-tulis terhadap delapan correctness property. Seluruh fase berjalan tanpa galat dan tanpa modifikasi pada source code Rust/GLSL.

---

## Fase 1 — Ekstraksi Fakta Source Code (9 task, paralel)

Sembilan task ekstraksi read-only menghasilkan berkas fakta verbatim dengan nomor baris persis dari source code, tersimpan di `.kiro/specs/revisi-bab3-metodologi/facts/`.

| Task | Sumber | Berkas Fakta | Cakupan |
|------|--------|--------------|---------|
| 1.1 | `Cargo.toml` | `1.1-cargo-toml.md` | `edition = "2024"`, sepuluh crate dependensi langsung dengan versi exact, blok target wasm32, feature `multithreading` opt-in |
| 1.2 | `src/blocks.rs` | `1.2-blocks.md` | Konstanta `TILE_W = 16`, `TILE_H = 8`; signature `bin_line`, `Blocks::build_block`, `record_per_scanline_crossings`; outer DDA empat arah diagonal + tiga kasus khusus; inner DDA empat arah utama; format F24Dot8 dan 8.8 fixed-point |
| 1.3 | `src/builder.rs` | `1.3-builder.md` | Signature `Builder::build_path`, `Builder::generate_tiles`; field `Builder` (termasuk `covers: RefCell<CoverStorage>`); alur propagasi backdrop kiri-ke-kanan dengan SIMD `i16x8.add` |
| 1.4 | `src/tile.rs`, `src/render/common.rs` | `1.4-tile.md` | Tata letak `#[repr(C)]` struct `Tile` 44 byte; verifikasi penjumlahan field |
| 1.5 | Shader files | `1.5-shaders.md` | Fungsi `line_box`, konstanta `WINDING_UNIT = 256.0`, formula fill rule NonZero dan EvenOdd, konfirmasi jalur kode tunggal |
| 1.6 | `src/path.rs`, `src/flatten.rs` | `1.6-flatten.md` | Signature `convert_cubics_to_quadratic_curves`, `estimate_number_of_quadratic_curves`, `flatten_quadratic`, `flatten_recursive`; `FLATNESS_THRESHOLD = 32` |
| 1.7 | `src/scene.rs`, `src/render/webgl.rs` | `1.7-scene-webgl.md` | Signature `Scene::fill`, `Scene::stroke`, `WebGlRenderer::new`, `WebGlRenderer::render`, `initialize_tile_vao`; bukti vertex divisor 44 byte/tile |
| 1.8 | `src/pico_svg.rs` | `1.8-pico-svg.md` | Dispatch `<g>` dan `<path>`; atribut `fill`, `stroke`, `stroke-width`, `transform`; konfirmasi parser bukan SVG 1.1 Core lengkap |
| 1.9 | Demo + tests | `1.9-demo-tests.md` | Resolusi DPR-aware demo native; empat metrik overlay FPS (`update_overlay`); konstanta `W: u16 = 1080`, `H: u16 = 520` di `tests/test.rs` |

Temuan penting yang mengoreksi rancangan awal:
- Field `Builder` aktual adalah `covers: RefCell<CoverStorage>`, bukan `cover_storage` seperti pada draf rancangan task.
- Tipe field `Tile.segments` adalah `[f32; 2]` (bukan `[u32; 2]`) — di-reinterpret-bit pada GPU melalui `floatBitsToUint`.
- Rayon dideklarasikan opsional di balik feature flag `multithreading` dan tidak dipanggil pada hot path implementasi saat ini.

---

## Fase 2 — Penulisan Ulang per Subbab (10 task, sequential antar gelombang)

Sepuluh task penulisan dijadwalkan sequential karena seluruhnya menulis ke berkas yang sama (`Skripsi/bab3_metodologi.md`).

| Task | Subbab | Aksi | Hasil |
|------|--------|------|-------|
| 2.1 | 3.1 Diagram Alir Kerangka Berpikir | Patch terminologi minor | Mengganti penyebutan ray shooting di Fase 3 dengan binning DDA + akumulator signed-area + propagasi backdrop |
| 2.2 | 3.2 Analisis Kebutuhan | Patch baris ke-3 tabel 3.2.3 | Solusi Teknis baris 3 ditulis ulang (pipeline tiga tahap), Rumusan Masalah dipertahankan |
| 2.3 | 3.3.1 Spesifikasi Aplikasi | Tulis ulang penuh | Rust edisi 2024, target `wasm32-unknown-unknown` dengan WebGL 2.0, sepuluh crate dengan ejaan/versi persis Cargo.toml, Rayon opsional, subset SVG minimal |
| 2.4 | 3.4.1 Use Case Diagram | Patch UC-03 sub-proses | Sub-proses lama → "pra-pemrosesan CPU (binning DDA + akumulator signed-area + propagasi backdrop)" dan "rasterisasi GPU (vertex shader instanced quad + fragment shader analitik)" |
| 2.5 | 3.4.2 Use Case Description | Tulis ulang UC-03 Basic Flow | Enam tahap CPU (a)-(f) + dua tahap GPU (i)-(ii); tabel UC-01/UC-02 dipertahankan dengan koreksi terminologi minor (OpenGL ES Context → konteks WebGL 2.0) |
| 2.6 | 3.4.3 Sequence Diagram | Tulis ulang penuh | Lima pesan berurutan antar lima partisipan (Aplikasi Utama, Scene, Builder, WebGlRenderer, GPU); tanpa blok `alt`/`opt`/`loop` bercabang tipe ubin |
| 2.7 | 3.4.4 Class Diagram | Tulis ulang penuh | Sembilan kotak kelas UML (`Scene`, `Builder`, `CoverStorage`, `Block`, `Blocks`, `TileBounds`, `Tile`, `WebGlRenderer`, `PicoSvg`) + delapan relasi UML; struct `Tile` dengan `backdrop: [i16; 8]` |
| 2.8 | 3.5 Perancangan Algoritma | Tulis ulang penuh | Enam sub-bagian (a)-(f): flattening cubic-to-quadratic, binning DDA dua tahap, akumulator signed-area, propagasi backdrop, `line_box` integral trapezoidal, fill rule NonZero clamp absolute & EvenOdd triangle wave; catatan Rayon potensial |
| 2.9 | 3.6 Perancangan Layar | Tulis ulang penuh | Resolusi window-fill DPR-aware demo native + empat metrik overlay FPS (CPU ms, GPU ms, paint ops, zoom ratio) + resolusi pengujian wasm-bindgen-test 1080×520; klaim 1920×1080 lama dihapus |
| 2.10 | 3.7 Perancangan Database File | Tulis ulang penuh | `Vec<Tile>` 44 byte/elemen + tekstur segmen RGBA32F (satu texel = `(p0.x, p0.y, p1.x, p1.y)`) + Tabel 3.1 tata letak vertex buffer instanced 44 byte/tile dengan offset per field |

---

## Fase 3 — Validasi 8 Correctness Property (paralel, read-only)

Seluruh delapan property mendapat verdict **PASS**. Laporan lengkap tersimpan di `.kiro/specs/revisi-bab3-metodologi/validation/`.

| Property | Berkas Laporan | Metrik Utama | Verdict |
|----------|----------------|--------------|---------|
| 1. Absence of Forbidden Terms | `4.1-forbidden-terms.md` | 22 pola terlarang (ray shooting, transpilasi, TileType, winding_number, PPGA, EMPTY/INTERIOR/EDGE, edisi 2021, persamaan implisit) — 0 hits untuk semua | ✅ PASS |
| 2. Presence of Required Terms | `4.2-required-terms.md` | 34/34 istilah wajib hadir (sepuluh crate di 3.3.1, `line_box`/`record_per_scanline_crossings`/`TILE_W`/`TILE_H` di 3.5, `1080×520` di 3.6, `RGBA32F` di 3.7) | ✅ PASS |
| 3. Structural Heading Invariant | `4.3-heading-structure.md` | 16/16 heading wajib hadir tepat-sekali; baris 1 = `# BAB 3 METODE PENELITIAN`; urutan kanonik strictly increasing; 0 stray heading | ✅ PASS |
| 4. Canonical Terminology Consistency | `4.4-canonical-terminology.md` | 0 sinonim non-kanonik untuk 9 komponen pipeline; 0 paragraf yang mencampur `pra-pemrosesan` dan `preprocessing` | ✅ PASS |
| 5. Technical Claim Traceability | `4.5-code-traceability.md` | 169/169 referensi kode valid (16 OK_FILE, 78 OK_LINE, 68 OK_RANGE, 6 OK_SYMBOL, 1 OK_BASENAME); seluruh delapan subbab dengan klaim teknis (3.3 sampai 3.7) PASS | ✅ PASS |
| 6. Cross-Subsection Narrative Consistency | `4.6-cross-subsection-narrative.md` | 4 subbab (3.4.2, 3.4.3, 3.4.4, 3.5) konsisten pada branching style, field reference style, dan urutan tahap pipeline; 6/6 pasangan PASS | ✅ PASS |
| 7. Numerical Claim Consistency | `4.7-numerical-consistency.md` | Setiap parameter numerik kanonik konsisten lintas subbab (16×8, 44 byte, 1080×520, RGBA32F, F24Dot8, 8.8 fixed-point); satu kemunculan `1920×1080` berada dalam kalimat penyangkalan eksplisit | ✅ PASS |
| 8. Single-File Output Identity | `4.8-single-file-identity.md` | Tepat satu berkas `Skripsi/bab3_metodologi.md` di seluruh repositori; parses CommonMark tanpa exception (60 981 byte, 381 baris, 16 heading) | ✅ PASS |

---

## Statistik Berkas Keluaran

- **Berkas:** `Skripsi/bab3_metodologi.md`
- **Ukuran:** 60 981 byte
- **Baris:** 381
- **Encoding:** UTF-8
- **Heading:** 1 H1 + 7 H2 + 8 H3 = 16 heading
- **Code fence:** 0 (seimbang)
- **Referensi kode:** 169 backtick references, 100% valid traceability

Distribusi referensi kode per subbab:

| Subbab | Total Ref | Valid | Verdict |
|--------|----------:|------:|---------|
| 3.3 (paragraf pengantar) | 1 | 1 | PASS |
| 3.3.1 Spesifikasi Aplikasi | 21 | 21 | PASS |
| 3.4.2 Use Case Description | 7 | 7 | PASS |
| 3.4.3 Sequence Diagram | 16 | 16 | PASS |
| 3.4.4 Class Diagram | 45 | 45 | PASS |
| 3.5 Perancangan Algoritma | 53 | 53 | PASS |
| 3.6 Perancangan Layar | 12 | 12 | PASS |
| 3.7 Perancangan Database File | 14 | 14 | PASS |
| **Total** | **169** | **169** | **PASS** |

Subbab 3.1, 3.2.x, dan 3.4.1 tidak memuat klaim teknis yang menyebut nama berkas/fungsi/konstanta source code, konsisten dengan scope traceability yang dibatasi pada Subbab 3.3 sampai 3.7 (paragraf pengantar Subbab 3.3 baris 61).

---

## Artefak Pendukung

Direktori `.kiro/specs/revisi-bab3-metodologi/`:

```
revisi-bab3-metodologi/
├── PROGRESS.md                          ← berkas ini
├── requirements.md                      ← spesifikasi 15 requirement
├── design.md                            ← desain enam tahap CPU + dua tahap GPU
├── tasks.md                             ← 32 task implementasi
├── facts/                               ← Fase 1: 9 berkas fakta verbatim
│   ├── 1.1-cargo-toml.md
│   ├── 1.2-blocks.md
│   ├── 1.3-builder.md
│   ├── 1.4-tile.md
│   ├── 1.5-shaders.md
│   ├── 1.6-flatten.md
│   ├── 1.7-scene-webgl.md
│   ├── 1.8-pico-svg.md
│   └── 1.9-demo-tests.md
└── validation/                          ← Fase 3: laporan + skrip validasi
    ├── 4.1-forbidden-terms.md
    ├── 4.2-required-terms.md
    ├── 4.3-heading-structure.md
    ├── 4.4-canonical-terminology.md
    ├── 4.4_terminology_check.py
    ├── 4.5-code-traceability.md
    ├── 4.5-extract-refs.py
    ├── 4.5-output.md
    ├── 4.5-spot-check-output.txt
    ├── 4.6-cross-subsection-narrative.md
    ├── 4.7-numerical-consistency.md
    ├── 4.8-single-file-identity.md
    ├── _parse_commonmark.py
    ├── _scan_forbidden.ps1
    ├── _verify_4_3.py
    ├── check_required_terms.ps1
    └── spot_check.py
```

---

## Reproducibility

Validasi dapat dijalankan ulang dengan perintah berikut dari akar repositori:

```powershell
# Property 1 — Forbidden terms scan
powershell -ExecutionPolicy Bypass -File `
  ".kiro\specs\revisi-bab3-metodologi\validation\_scan_forbidden.ps1"

# Property 2 — Required terms scan
powershell -ExecutionPolicy Bypass -File `
  ".kiro\specs\revisi-bab3-metodologi\validation\check_required_terms.ps1"

# Property 3 — Heading structure invariant
python ".kiro\specs\revisi-bab3-metodologi\validation\_verify_4_3.py"

# Property 4 — Canonical terminology consistency
python ".kiro\specs\revisi-bab3-metodologi\validation\4.4_terminology_check.py"

# Property 5 — Code-reference traceability
python ".kiro\specs\revisi-bab3-metodologi\validation\4.5-extract-refs.py"
python ".kiro\specs\revisi-bab3-metodologi\validation\spot_check.py"

# Property 8 — Single-file output identity (CommonMark parse)
python ".kiro\specs\revisi-bab3-metodologi\validation\_parse_commonmark.py"
```

Property 6 (cross-subsection narrative consistency) dan Property 7 (numerical claim consistency) divalidasi secara manual via `grep_search` lintas subbab; bukti pendukung tercatat verbatim pada berkas laporan masing-masing.

---

## Catatan Tipografi Opsional (Tidak Menggugurkan PASS)

Tiga catatan tipografi minor diidentifikasi pemindaian Property 4 namun tidak memengaruhi kondisi PASS:

1. Baris 292 memakai "signed area" tanpa hyphen pada satu kalimat, sementara kalimat berikutnya pada paragraf yang sama memakai "signed-area" ber-hyphen.
2. Baris 23, 55, 111 memakai title case "Fragment Shader" pada konteks tabular berpasangan dengan "Vertex Shader" (mengikuti konvensi WebGL/OpenGL).
3. Baris 347 memakai "frame buffer" dua kata, sementara baris 36, 135, 154 memakai "framebuffer" satu kata.

Ketiganya adalah peningkatan tipografi opsional, bukan koreksi yang diperintahkan oleh property atau acceptance criteria.

---

## Status Akhir

✅ **Spec `revisi-bab3-metodologi` selesai 100%.** Berkas `Skripsi/bab3_metodologi.md` siap digunakan sebagai Bab 3 versi revisi pada skripsi.

| Fase | Total Task | Selesai | Status |
|------|-----------:|--------:|--------|
| 1. Ekstraksi fakta | 9 | 9 | ✅ Selesai |
| 2. Penulisan ulang | 10 | 10 | ✅ Selesai |
| 3. Validasi property | 8 | 8 | ✅ Selesai PASS |
| Checkpoint (Task 3, 5) | 2 | 2 | ✅ Selesai |
| Parent rollup (Task 1, 2, 4) | 3 | 3 | ✅ Selesai |
| **Total** | **32** | **32** | **✅ Selesai** |
