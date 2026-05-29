# Gambar Skripsi

Folder ini menampung berkas gambar yang dirujuk dari dokumen skripsi.

## Verifikasi visual Bab 4.3

Bab 4.3 (Verifikasi Kebenaran Output) merujuk enam berkas gambar: untuk tiap
aset uji, satu hasil rendering Arabella dan satu rendering referensi peramban.

### Cara memperoleh gambar hasil rendering Arabella

1. Jalankan benchmark dari akar repo:
   ```
   cargo run_wasm -p bench_webgl --release
   ```
2. Buka URL yang dicetak terminal di Google Chrome 113+.
3. Tunggu sampai console mencetak `[bench] image capture done.`
4. Di bawah tabel hasil, muncul tiga gambar (Tiger, el gato, paris-30k).
   **Klik tiap gambar** untuk mengunduh PNG-nya (nama otomatis:
   `arabella_Ghostscript_Tiger.png`, `arabella_el_gato.png`,
   `arabella_paris-30k.png`).
5. Pindahkan ketiga PNG itu ke folder ini, lalu ganti namanya menjadi:
   - `4.1a-tiger-arabella.png`
   - `4.2a-elgato-arabella.png`
   - `4.3a-paris-arabella.png`

### Cara memperoleh gambar rendering referensi (pembanding)

Buka berkas SVG yang sama langsung di Google Chrome (drag-drop berkas
`assets/Ghostscript_Tiger.svg`, `assets/el_gato.svg`, `assets/paris-30k.svg`
ke tab kosong), atur perbesaran agar tampilannya sebanding, lalu ambil
tangkapan layar area gambar. Simpan di folder ini dengan nama:
   - `4.1b-tiger-referensi.png`
   - `4.2b-elgato-referensi.png`
   - `4.3b-paris-referensi.png`

Alternatif renderer referensi: resvg, Inkscape, atau Firefox — pilih salah
satu dan sebutkan di Bab 4.3 renderer referensi mana yang dipakai.

### Catatan

Setelah keenam berkas tersedia di folder ini dengan nama di atas, sintaks
`![...](gambar/...png)` pada Bab 4.3 akan menampilkannya otomatis pada
penampil Markdown maupun saat dikonversi ke PDF.
