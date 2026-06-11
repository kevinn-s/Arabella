# BAB 5 SIMPULAN DAN SARAN

## 5.1 Simpulan

Berdasarkan hasil perancangan arsitektur dan pengujian performa purwarupa Arabella yang telah dilaporkan pada Bab 4, diperoleh lima simpulan berikut. Kelima butir disusun berurutan sesuai dengan kelima Rumusan Masalah pada Subbab 1.2 dan keempat Tujuan Penelitian pada Subbab 1.4.

1. Arsitektur *pipeline* hibrida *non-compute* untuk rendering grafis vektor berhasil diwujudkan tanpa *compute shader*. Paralelisme dicapai melalui pembagian beban yang tegas antara CPU dan GPU: tahap *pra-pemrosesan* di CPU memikul komputasi tujuan umum berupa *flattening* kurva menjadi segmen garis, *binning DDA* lintas ubin, akumulasi *signed-area* untuk menghitung *winding number*, dan *propagasi backdrop* saat emisi ubin, sedangkan GPU hanya menjalankan *vertex* dan *fragment shader* pada *rasterization pipeline* tradisional (Subbab 4.1 dan 4.2).

2. Tahap *pra-pemrosesan* di CPU dibangun di atas struktur data berbasis ubin yang mendukung paralelisasi, dengan paralelisme tingkat instruksi melalui SIMD yang sudah aktif pada *hot path* transformasi dan *flattening*. Paralelisme tingkat data penuh melalui pustaka Rayon belum diaktifkan karena feature `multithreading` masih opsional dan tidak ditarik pada *build* baku (Subbab 4.6). Dengan demikian fondasi struktur data untuk paralelisasi telah tersedia, sementara pengaktifan paralelisasi penuh menjadi pengembangan lanjutan (Subbab 5.2.1).

3. Pembagian beban kerja antara CPU dan GPU berlangsung efektif untuk adegan berskala kecil sampai menengah (Subbab 4.4). Biaya rasterisasi pada GPU tumbuh landai dan stabil seiring bertambahnya jumlah ubin nontrivial, sedangkan biaya *pra-pemrosesan* di CPU mendominasi total waktu satu *frame* pada seluruh aset uji. Pola ini menegaskan bahwa GPU pada *pipeline* rasterisasi konvensional bukan penentu utama waktu *frame* pada skala tersebut.

4. Pada adegan berskala sangat besar, tahap *pra-pemrosesan* CPU yang dieksekusi secara *single-thread* berubah menjadi *bottleneck* dominan. Pada `paris-30k.svg`, tahap tersebut menyumbang 150,15 ms dari total 156,15 ms (sekitar 96 persen) waktu satu *frame*, sehingga laju turun menjadi 6,4 FPS (Subbab 4.4). Beban itu hampir seluruhnya berasal dari komputasi tujuan umum yang dipindahkan ke CPU, bukan dari rasterisasi GPU. Inilah *trade-off* utama dari penghilangan *compute shader*.

5. Perbandingan terhadap renderer pembanding bersifat kualitatif pada dimensi kompatibilitas platform, kompleksitas implementasi, dan karakteristik performa arsitektural (Subbab 4.4.3 dan 4.5). Terhadap renderer berbasis *compute shader* seperti Vello maupun renderer CPU seperti Skia dan Cairo, posisi Arabella dijelaskan secara relatif tanpa *benchmark* kuantitatif langsung, karena pengukuran tatap muka pada beban kerja setara belum dilakukan. Simpulan perbandingan karenanya dibatasi pada aspek arsitektural.

Kontribusi inti penelitian ini adalah pembuktian kelayakan (*feasibility*) *pipeline* rendering vektor hibrida *non-compute* pada lingkungan WebGL 2.0 melalui purwarupa Arabella, yang mampu merender berkas SVG tanpa satu pun *compute shader*. Pembuktian ini menunjukkan bahwa rendering grafis vektor yang memadai dapat diwujudkan pada lingkungan yang hanya menyediakan *rasterization pipeline* konvensional, sehingga *compute shader* tidak menjadi syarat mutlak. Kelayakan ini dibatasi pada lingkup yang benar-benar terbukti dan tidak mencakup klaim keunggulan performa kuantitatif terhadap Skia, Cairo, maupun Vello.

## 5.2 Saran

Saran berikut diturunkan dari keterbatasan dan temuan empiris pada Bab 4, khususnya *bottleneck* tahap *pra-pemrosesan* CPU (Subbab 4.4) dan daftar keterbatasan fungsional (Subbab 4.6). Saran dikelompokkan menjadi optimasi performa, perluasan fungsionalitas, dan evaluasi lanjutan.

### 5.2.1 Saran Optimasi Performa

Optimasi yang paling mendesak adalah pengaktifan paralelisme tingkat data pada tahap *pra-pemrosesan* CPU, terutama *flattening* kurva dan *binning DDA* per jalur yang menjadi penyumbang dominan waktu CPU. Pengukuran pada `paris-30k.svg` menunjukkan tahap *single-thread* ini menyumbang sekitar 96 persen waktu *frame* (150,15 ms dari 156,15 ms) sehingga laju turun menjadi 6,4 FPS. Karena komputasi *flattening* dan *binning* tiap jalur saling independen, beban tersebut dapat-diparalelkan tanpa ketergantungan data antar-jalur melalui pustaka Rayon, yang sudah tersedia sebagai feature `multithreading` opsional (Subbab 4.6) dan tinggal diaktifkan serta diintegrasikan ke *hot path*.

Usulan ini merupakan potensi optimasi, bukan peningkatan yang sudah terbukti. Besar perbaikannya bergantung pada jumlah inti CPU, biaya penjadwalan utas, dan pola distribusi beban, sehingga hanya dapat ditetapkan melalui pengukuran lanjutan pada perangkat keras dan beban kerja yang setara.

### 5.2.2 Saran Perluasan Fungsionalitas

Perluasan fungsionalitas berikut dipetakan satu-ke-satu terhadap kelima keterbatasan fungsional pada Subbab 4.6 dan seluruhnya merupakan *pengembangan lanjutan* yang belum diwujudkan pada implementasi saat ini.

1. Pengaktifan paint bergradien (linear, radial, dan sweep) pada *fragment shader*, menutup keterbatasan jalur rendering yang masih terbatas pada *paint* solid.
2. Penambahan dukungan *image paint* beserta *tinting*, yang menuntut perwujudan jalur pengunggahan tekstur paint dari adegan menuju GPU.
3. Perluasan subset SVG di luar elemen `g` dan `path`, mencakup `defs`, `use`, bentuk dasar (`rect`, `circle`, dan sejenisnya), gradien, serta `pattern`, agar dokumen yang memanfaatkan elemen tersebut dapat dirender secara utuh.
4. Penambahan sistem *text rendering* yang mencakup pemuatan *font*, *shaping* glyph, dan rasterisasi teks sebagai bagian dari adegan.
5. Penambahan *filter effect* seperti *blur* dan *drop shadow* melalui operasi penyaringan citra pasca-rasterisasi.

Kelima usulan ini menyediakan peta jalan penyempurnaan fungsionalitas yang berpijak pada keterbatasan terdokumentasi, di atas fondasi arsitektur *pipeline* hibrida *non-compute* yang telah terbukti berfungsi.

### 5.2.3 Saran Evaluasi Lanjutan

Evaluasi lanjutan yang utama adalah pelaksanaan *benchmark* kuantitatif langsung (*head-to-head*) antara Arabella dan renderer pembanding, yakni Skia, Cairo, dan Vello. Perbandingan pada penelitian ini masih kualitatif (Subbab 4.4.3), sedangkan data pada Subbab 4.4 hanya mengukur performa internal Arabella berupa dekomposisi waktu CPU dan GPU per *frame*, bukan perbandingan tatap muka. Posisi performa relatif antar-renderer karenanya belum dapat ditetapkan.

Karena seluruh angka performa Arabella terikat pada konfigurasi mesin uji tertentu (Subbab 4.4.3), *benchmark* lanjutan hanya sahih apabila dijalankan pada kondisi yang dikendalikan secara ketat dan seragam:

- Perangkat keras yang identik antar-renderer (prosesor, kapasitas memori, dan pemroses grafis yang sama), agar selisih waktu tidak bersumber dari perbedaan kapasitas perangkat.
- Berkas uji yang identik, yakni himpunan dokumen SVG yang sama dari adegan berskala kecil sampai sangat besar, sehingga setiap renderer memikul beban kerja yang setara.
- Prosedur pengukuran yang seragam, mencakup resolusi *viewport*, metode pencatatan waktu per *frame*, dan jumlah iterasi yang sama.

Ketiadaan *benchmark* kuantitatif langsung tidak menggugurkan kontribusi inti penelitian ini, melainkan menandai arah validasi berikutnya. Dengan kerangka evaluasi yang dikendalikan secara ketat, penelitian lanjutan dapat menempatkan performa Arabella secara terukur terhadap Skia, Cairo, dan Vello.
