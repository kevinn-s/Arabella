# Alur Pembelajaran Kode Program Arabella

> Panduan ini menjelaskan cara membaca dan memahami source code **Arabella** —
> sebuah *renderer* grafik vektor 2D hibrida CPU/GPU — secara bertahap, dari
> konsep paling dasar sampai bagian tersulit. Bahasanya sengaja dibuat
> sederhana, dengan analogi sehari-hari, supaya mudah diikuti meskipun kamu
> baru pertama kali melihat kodenya.
>
> 📊 **Visualisasi interaktif** dari semua algoritma penting tersedia di file
> [`visualisasi.html`](./visualisasi.html). Buka file itu di browser (cukup
> klik dua kali), lalu ikuti panduan ini sambil mencoba demo-nya.

---

## 0. Gambaran Besar: Arabella itu apa?

Bayangkan kamu punya gambar vektor (SVG) — misalnya logo, ikon, atau gambar
harimau. Gambar vektor itu bukan kumpulan piksel, melainkan kumpulan
**instruksi matematis**: "tarik garis dari sini ke sini", "buat kurva
melengkung begini". Layar komputer hanya bisa menampilkan **piksel** (kotak
warna). Jadi seseorang harus menerjemahkan instruksi matematis itu menjadi
piksel. Proses penerjemahan inilah yang disebut **rendering** (atau lebih
spesifik, **rasterisasi**).

Arabella melakukan rendering dengan cara membagi tugas antara dua "pekerja":

- **CPU** (otak serba bisa) — bertugas menyiapkan dan menata geometri:
  memecah kurva jadi garis-garis lurus, lalu mengelompokkan garis ke dalam
  petak-petak kecil (*tile*).
- **GPU** (pekerja paralel super cepat) — bertugas mewarnai piksel secara
  massal dan serentak.

Filosofinya: **CPU menyiapkan, GPU mengeksekusi.** Kerja sama ini disebut
arsitektur **hibrida**.

### Peta perjalanan satu frame

```
  File SVG
     │  (1) Parsing
     ▼
  Daftar bentuk (Fill / Stroke / Group)
     │  (2) Flattening: kurva → garis lurus
     ▼
  Kumpulan segmen garis (koordinat F24Dot8)
     │  (3) DDA Binning: garis dipotong per-tile
     ▼
  Block per (garis, tile)
     │  (4) Akumulasi winding per-scanline
     ▼
  Backdrop awal tiap tile
     │  (5) Propagasi backdrop kiri → kanan
     ▼
  Daftar Tile siap kirim ke GPU
     │  (6) Upload ke GPU (vertex buffer + texture)
     ▼
  GPU: Vertex Shader (gambar quad per tile)
     │  (7) Fragment Shader: hitung coverage tiap piksel
     ▼
  Piksel berwarna di layar 🎉
```

Tahap 1–5 jalan di **CPU**. Tahap 6–7 jalan di **GPU**. Sisa panduan ini
membahas setiap tahap satu per satu.

---

## Peta File ke Konsep

Supaya tidak tersesat, ini daftar file utama dan perannya:

| File | Peran | Tahap |
|------|-------|-------|
| `src/pico_svg.rs` | Membaca & mengurai file SVG | 1 |
| `src/scene.rs` | Pintu masuk utama (`Scene::fill`, `Scene::stroke`) | koordinator |
| `src/path.rs` | Transformasi titik + kubik→kuadratik + memanggil flatten | 2 |
| `src/flatten.rs` | Memecah kurva kuadratik jadi garis lurus | 2 |
| `src/blocks.rs` | DDA binning + akumulasi winding per-scanline | 3, 4 |
| `src/builder.rs` | Mengatur seluruh proses CPU + propagasi backdrop | 4, 5 |
| `src/tile.rs` | Definisi struktur data `Tile` | data |
| `src/render/webgl.rs` | Upload data & menggambar di GPU (WebGL) | 6 |
| `src/render/shaders/*.vert/.frag` | Program GPU (shader) | 7 |
| `examples/native_webgl/` | Demo interaktif (pan, zoom, FPS) | aplikasi |

Urutan membaca yang disarankan: **ikuti tahap 1 → 7** seperti di bawah ini.
Jangan langsung loncat ke `blocks.rs` (itu yang paling rumit).

---

## Tahap 1 — Membaca File SVG (`pico_svg.rs`)

**Tujuan:** mengubah teks XML SVG menjadi struktur data Rust yang rapi.

**Analogi:** seperti membaca resep masakan dan menuliskannya ulang dalam
bentuk daftar langkah yang terstruktur, supaya gampang dieksekusi nanti.

Hasil parsing adalah pohon `Item`:

- `Item::Fill` — bentuk yang diisi warna (punya `path` + `color`).
- `Item::Stroke` — garis tepi bentuk (punya `width`, `color`, `path`).
- `Item::Group` — kumpulan item dengan satu transformasi affine bersama.

Yang perlu dipahami di sini:

1. **Path** disimpan sebagai `BezPath` (dari pustaka `kurbo`) — yaitu
   rangkaian perintah `MoveTo`, `LineTo`, `QuadTo`, `CurveTo`, `ClosePath`.
2. **Transform affine** — matriks untuk menggeser, memutar, dan menskala
   bentuk. Group bisa punya transform sendiri yang diwariskan ke anak-anaknya.

> 💡 Mulai dari sini dulu karena ini bagian paling "biasa" — tidak ada
> matematika rumit, hanya membaca teks dan menyusun struktur data.

---

## Tahap 2 — Flattening: Mengubah Kurva Jadi Garis Lurus (`path.rs`, `flatten.rs`)

Ini konsep kunci pertama. GPU paling cepat menangani **garis lurus**, bukan
kurva melengkung. Jadi semua kurva harus dipecah dulu jadi banyak garis lurus
pendek yang, kalau disambung, kelihatan seperti kurva aslinya. Proses ini
disebut **flattening** (perataan).

> 📊 **Buka visualisasi:** lihat demo **"Flattening"** dan
> **"Kubik → Kuadratik"**.

### 2a. Kubik → Kuadratik (`path.rs`)

SVG sering memakai **kurva Bézier kubik** (4 titik kontrol). Arabella terlebih
dahulu mengubahnya menjadi beberapa **kurva Bézier kuadratik** (3 titik
kontrol) yang lebih sederhana. Jumlah kuadratik yang dibutuhkan diperkirakan
oleh `estimate_number_of_quadratic_curves` — makin melengkung kurvanya, makin
banyak potongan kuadratik yang dipakai.

**Analogi:** memotong satu kurva rumit jadi beberapa kurva sederhana, seperti
memecah jalan berkelok panjang jadi beberapa tikungan kecil.

### 2b. Kuadratik → Garis (`flatten.rs`)

Setiap kurva kuadratik lalu dipecah jadi garis lurus dengan metode
**recursive midpoint subdivision** ala Blaze (De Casteljau):

1. **Cek kelurusan** (`is_flat_enough`): ukur jarak L1 dari titik tengah tali
   busur (`p0`→`p2`) ke titik kontrol `p1`. Kalau jaraknya sudah lebih kecil
   dari ambang `FLATNESS_THRESHOLD`, kurva dianggap "cukup lurus".
2. **Kalau sudah lurus** → simpan sebagai satu garis `p0`→`p2`.
3. **Kalau belum** → bagi kurva jadi dua di titik tengah, lalu ulangi proses
   untuk masing-masing setengahnya (rekursi).

**Analogi:** kamu mau menggambar lengkungan pakai penggaris lurus. Kalau
lengkungan masih kelihatan bengkok, potong jadi dua dan cek lagi tiap bagian.
Ulangi sampai tiap potongan kelihatan lurus.

### Format koordinat: F24Dot8

Koordinat tidak disimpan sebagai pecahan (float) biasa, melainkan sebagai
**bilangan bulat fixed-point 24.8**: nilai `i32` di mana `256 = 1 piksel`.
Jadi `1.5` piksel disimpan sebagai `384`. Kenapa? Karena operasi bilangan
bulat lebih cepat dan hasilnya **konsisten** (tidak ada selisih pembulatan
aneh antar-platform), yang penting untuk algoritma binning berikutnya.

---

## Tahap 3 — DDA Tile Binning: Memotong Garis ke Petak (`blocks.rs`)

Ini bagian **tersulit sekaligus terpenting**. Sabar membacanya.

> 📊 **Buka visualisasi:** demo **"DDA Tile Binning"** sangat membantu di sini.

### Kenapa pakai tile (petak)?

Layar dibagi menjadi grid petak kecil berukuran **16×8 piksel** (lihat
`TILE_W = 16`, `TILE_H = 8` di `blocks.rs`). Daripada GPU memikirkan seluruh
layar sekaligus, tiap petak bisa dikerjakan **mandiri dan paralel**. Tapi
supaya itu bisa terjadi, tiap garis harus "didaftarkan" ke petak-petak mana
saja yang dilewatinya.

**Analogi:** bayangkan peta kota dibagi jadi kotak-kotak (zona). Sebuah jalan
panjang melewati beberapa zona. Kita perlu mencatat: "potongan jalan ini ada
di zona A, potongan itu di zona B", supaya tiap zona tahu jalan apa saja yang
melintasinya.

### Apa itu DDA?

**DDA = Digital Differential Analyzer.** Ini algoritma klasik untuk menentukan
petak-petak mana yang dilewati sebuah garis, dengan hanya memakai penjumlahan
(tanpa perkalian/pembagian berulang yang lambat). Arabella memakai DDA dua
tingkat:

- **Outer DDA** (`bin_line`) — memotong garis berdasarkan **baris** petak
  (sumbu Y). "Garis ini melewati baris petak 3, 4, dan 5."
- **Inner DDA** (`bin_line_in_row`) — di dalam satu baris, memotong garis
  berdasarkan **kolom** petak (sumbu X). "Di baris 3, garis lewat kolom 2 dan
  3."

Karena garis bisa bergerak ke 8 arah (atas/bawah × kiri/kanan), ada 8 fungsi
khusus (`outer_dda_down_right`, `inner_dda_left_up`, dst). Semuanya melakukan
hal yang sama, hanya beda arah.

### Hasilnya: `Block`

Setiap pasangan (garis, petak) menghasilkan satu `Block` yang berisi titik
ujung garis **dalam koordinat lokal petak** (sudah dipotong agar pas di dalam
petak itu). Block inilah yang nanti dikirim ke GPU.

> 💡 Tips membaca: pahami dulu satu fungsi DDA saja (misalnya
> `inner_dda_right_down`), gambar di kertas, baru lihat yang lain. Strukturnya
> identik, cuma beda tanda + dan −.

---

## Tahap 4 — Akumulator Winding Per-Scanline (`blocks.rs`)

> 📊 **Buka visualisasi:** demo **"Winding Per-Scanline"**.

Untuk tahu piksel mana yang "di dalam" bentuk dan mana yang "di luar",
renderer memakai konsep **winding number** (angka lilitan). Idenya:

- Setiap garis tepi punya **arah** (naik atau turun).
- Untuk satu titik, tarik garis khayal ke kiri. Hitung berapa kali garis tepi
  menyilang, dengan tanda: garis **naik** = `+1`, garis **turun** = `−1`.
- Kalau totalnya bukan nol → titik itu **di dalam** bentuk.

Fungsi `record_per_scanline_crossings` menghitung kontribusi tiap garis per
**scanline** (baris piksel) di dalam petak. Tapi bukan sekadar "+1/−1",
melainkan **signed area** (luas bertanda) dalam format fixed-point 8.8 (di
mana `256 = satu unit winding penuh`). Kenapa luas, bukan sekadar hitungan
silang? Supaya hasilnya **halus** dan tidak menimbulkan garis-garis nyasar
(*streak*) saat GPU melakukan anti-aliasing.

**Analogi:** alih-alih cuma mencatat "ada garis lewat di baris ini", kita
mencatat "garis ini menutupi 70% tinggi baris ini, arah turun" → nilainya
`−0.7 × 256`.

---

## Tahap 5 — Propagasi Backdrop Kiri ke Kanan (`builder.rs`)

> 📊 **Buka visualisasi:** demo **"Propagasi Backdrop"**.

Di `generate_tiles`, untuk setiap baris petak, ada satu **akumulator** yang
berjalan dari **kiri ke kanan**. Nilainya adalah jumlah semua winding dari
petak-petak di sebelah kirinya. Nilai inilah yang disebut **backdrop** —
"kondisi awal" winding ketika GPU mulai memproses sebuah petak.

Kenapa penting? Karena bagian **dalam** sebuah bentuk yang besar bisa terdiri
dari banyak petak yang **tidak punya garis sama sekali** (petak interior).
Petak-petak itu tetap harus diisi warna penuh. Backdrop dari tetangga kiri
memberi tahu petak: "winding sebelum kamu sudah `+1`, jadi kamu ada di dalam
bentuk — isi penuh."

**Analogi:** seperti menghitung saldo rekening. Tiap petak adalah satu hari.
Backdrop = saldo awal hari ini (hasil akumulasi kemarin-kemarin). Crossings di
petak ini = transaksi hari ini. Saldo diteruskan ke hari (petak) berikutnya.

Di tahap ini juga, struktur `Tile` final dibentuk dan disimpan — lengkap
dengan backdrop per-scanline, daftar segmen garisnya, warna (payload), dan
fill rule.

---

## Tahap 6 — Upload ke GPU (`render/webgl.rs`)

> 📊 **Buka visualisasi:** demo **"Pipeline GPU"**.

Sekarang data CPU dikirim ke GPU lewat WebGL 2.0:

- **Vertex buffer** — daftar `Tile` (tiap tile jadi satu *instance* quad).
- **Texture RGBA32F** — daftar segmen garis (tiap garis = 1 texel berisi
  `p0.x, p0.y, p1.x, p1.y`).
- **Uniform buffer** — konfigurasi (lebar/tinggi layar, dll).

Lalu dipanggil `draw_arrays_instanced` yang menggambar semua tile sekaligus.
Teknik **instancing** ini membuat ribuan petak digambar dalam satu perintah.

---

## Tahap 7 — Shader GPU: Mewarnai Piksel (`render/shaders/`)

> 📊 **Buka visualisasi:** demo **"Coverage GPU (line_box)"** dan **"Fill
> Rule"**.

Ada dua program kecil yang jalan di GPU:

### Vertex Shader (`render_tile.vert`)

Tugasnya menempatkan empat sudut quad tiap petak di posisi yang benar di layar
(konversi koordinat piksel → NDC), lalu meneruskan data petak (backdrop,
daftar segmen, warna) ke fragment shader.

### Fragment Shader (`render_tile.frag`)

Ini berjalan **sekali untuk tiap piksel**. Langkahnya:

1. Ambil **backdrop** scanline tempat piksel ini berada (winding awal).
2. Untuk tiap segmen garis di petak, hitung kontribusinya pakai fungsi
   **`line_box`** — yaitu **integral trapesium**: berapa luas kotak piksel
   (ukuran 1×1) yang ada di sebelah kanan garis. Hasilnya angka halus antara
   −1 sampai +1, bukan cuma 0/1. Inilah sumber **anti-aliasing** (tepi yang
   mulus, tidak bergerigi).
3. Jumlahkan semua kontribusi → dapat **winding final** piksel.
4. Terapkan **fill rule**:
   - **NonZero**: di dalam jika winding ≠ 0. `coverage = clamp(|winding|, 0, 1)`.
   - **EvenOdd**: pola selang-seling (ganjil = di dalam, genap = di luar).
5. Kalikan warna dengan coverage → warna akhir piksel.

**Analogi `line_box`:** bayangkan tiap piksel adalah kotak kecil. Sebuah garis
melintasinya. Kita tidak bertanya "apakah pusat kotak di dalam atau di luar?"
(itu menghasilkan tepi bergerigi), melainkan "berapa **persen** kotak ini yang
tertutup?" → tepi pun jadi gradasi halus.

---

## Ringkasan Algoritma Penting (untuk Bab Skripsi)

| # | Algoritma | File | Inti idenya |
|---|-----------|------|-------------|
| 1 | Recursive midpoint subdivision (flatten) | `flatten.rs` | Pecah kurva jadi garis sampai cukup lurus |
| 2 | Cubic → Quadratic | `path.rs` | Sederhanakan kurva kubik jadi kuadratik |
| 3 | Transformasi affine SIMD | `path.rs` | Transformasi banyak titik sekaligus (f32x4/f32x8) |
| 4 | DDA tile binning (outer + inner, 8 arah) | `blocks.rs` | Daftarkan tiap garis ke petak yang dilewatinya |
| 5 | Signed-area winding accumulator 8.8 | `blocks.rs` | Catat luas bertanda per-scanline |
| 6 | Propagasi backdrop kiri→kanan | `builder.rs` | Teruskan winding antar petak dalam satu baris |
| 7 | Integral trapesium `line_box` | `render_tile.frag` | Hitung coverage halus per-piksel (anti-alias) |
| 8 | Fill rule NonZero / EvenOdd | `render_tile.frag` | Ubah winding jadi "di dalam / di luar" |

---

## Saran Urutan Belajar (Checklist)

- [ ] Baca panduan ini sekali penuh tanpa membuka kode (dapat gambaran besar).
- [ ] Buka `visualisasi.html`, mainkan tiap demo sambil baca tahap terkait.
- [ ] Baca `scene.rs` → pahami alur `fill()` memanggil `build_path` lalu
      `generate_tiles`.
- [ ] Baca `flatten.rs` (paling pendek & paling mudah dipahami dari kode).
- [ ] Baca `path.rs` (transform + cubic→quad + emit line).
- [ ] Baca `builder.rs` bagian `build_path` dan `generate_tiles`.
- [ ] Baca `blocks.rs` — fokus satu fungsi DDA dulu, gambar di kertas.
- [ ] Baca `render_tile.frag` — pahami `line_box` dan fill rule.
- [ ] Baca `webgl.rs` terakhir (banyak boilerplate WebGL).

Selamat belajar! Kalau bingung di satu tahap, kembali ke analogi dan demo
visualnya dulu sebelum menyelam ke kode.
