# Materi Pembelajaran Arabella

Folder ini berisi materi belajar untuk memahami source code renderer **Arabella**.

## Isi

| Berkas | Untuk apa | Cara pakai |
|--------|-----------|------------|
| [`alur_pembelajaran.md`](./alur_pembelajaran.md) | Panduan teks bertahap 7 tahap pipeline + peta file ke konsep + checklist belajar | Baca di editor/markdown viewer |
| [`visualisasi.html`](./visualisasi.html) | 8 visualisasi interaktif algoritma penting | Klik dua kali → buka di browser |

## Cara memulai

1. Buka [`alur_pembelajaran.md`](./alur_pembelajaran.md), baca sekali penuh untuk gambaran besar.
2. Buka [`visualisasi.html`](./visualisasi.html) di browser (Chrome/Edge/Firefox). Tidak perlu
   server, tidak perlu internet — semua jalan offline.
3. Pelajari tahap demi tahap: tiap bagian di panduan teks menunjuk ke demo yang relevan.

## Daftar visualisasi interaktif

0. **Gambaran Pipeline** — peta 7 tahap CPU→GPU, klik untuk lompat ke tiap demo.
1. **Flattening Kurva** — kurva dipecah jadi garis lurus (`flatten.rs`).
2. **Kubik → Kuadratik** — penyederhanaan kurva (`path.rs`).
3. **DDA Tile Binning** — garis dipotong ke petak (`blocks.rs`).
4. **Winding Per-Scanline** — luas bertanda per baris piksel (`blocks.rs`).
5. **Propagasi Backdrop** — winding diteruskan antar petak (`builder.rs`).
6. **Coverage GPU (`line_box`)** — anti-aliasing analitik (`render_tile.frag`).
7. **Fill Rule** — NonZero vs EvenOdd (`render_tile.frag`).

Tiap demo bisa diseret/diatur dengan slider, dan menampilkan nama file sumber di kodenya
sehingga mudah dirujuk balik ke source code.
