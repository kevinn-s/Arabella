# Requirements Document

## Introduction

Dokumen ini menetapkan persyaratan penulisan berkas `Skripsi/bab5_kesimpulan.md` (Bab 5: Simpulan dan Saran) dari nol. Berkas tersebut saat ini berisi 100% teks placeholder (lorem ipsum) pada dua subbab dummy (`## 5.1 Subbab 1`, `## 5.2 Subbab 1`, `### 5.2.1 Subbab 2`) dan harus diganti seluruhnya dengan narasi akademik berbahasa Indonesia formal yang menyimpulkan penelitian dan mengusulkan pengembangan lanjutan.

Spec ini adalah spec dokumentasi akademik. Yang ditulis adalah konten naratif Markdown, bukan kode. Spec ini mewarisi prinsip dari dua spec pendahulunya — `revisi-bab3-metodologi` dan `penulisan-bab4-implementasi-dan-hasil` — yaitu **source-of-truth-driven writing**: source code Arabella (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`) bersifat read-only dan TIDAK BOLEH dimodifikasi.

Namun Bab 5 berbeda secara mendasar dari Bab 3 dan Bab 4. Bab 3 dan Bab 4 adalah bab teknis yang setiap klaimnya ditelusuri langsung ke source code melalui rujukan berformat `` `berkas:simbol` ``. Sebaliknya, **Bab 5 adalah bab sintesis, bukan bab teknis baru**. Bab 5 tidak memperkenalkan klaim teknis, struktur data, algoritma, angka performa, atau terminologi baru. Seluruh substansi faktual Bab 5 diturunkan dari tiga sumber yang sudah selesai ditulis: (a) rumusan masalah dan tujuan penelitian pada Bab 1, (b) hasil implementasi, pengukuran performa nyata, pembahasan trade-off, dan keterbatasan pada Bab 4, serta (c) abstrak. Oleh karena itu, ketertelusuran (traceability) Bab 5 mengarah ke bab-bab tersebut (melalui rujukan antar-bab seperti `Bab 1`, `Subbab 1.2`, `Subbab 4.4`, `Subbab 4.6`), bukan ke rujukan kode.

Konsekuensi terpenting dari sifat sintesis ini adalah larangan fabrikasi numerik yang ketat: Bab 5 TIDAK BOLEH memunculkan satu pun angka performa, metrik, atau klaim faktual yang tidak hadir secara literal pada Bab 4 atau Bab 1. Setiap angka yang dikutip pada Bab 5 (misalnya waktu pra-pemrosesan CPU, laju frame, persentase bottleneck) harus identik dengan nilai yang dilaporkan pada Bab 4 dan disertai rujukan ke subbab Bab 4 yang bersangkutan.

Penulisan mempertahankan terminologi lintas-bab yang sudah konsisten dengan Bab 1 sampai Bab 4. Istilah kanonik (pipeline hibrida, non-compute, pra-pemrosesan, binning DDA, akumulator signed-area, propagasi backdrop, fragment shader, winding number) dipakai persis seperti pada bab-bab sebelumnya, sedangkan Istilah_Terlarang yang sudah dilarang oleh Spec_Bab3 dan Spec_Bab4 diteruskan dan tetap dilarang pada Bab 5 agar referensi silang antar bab tetap koheren.

## Glossary

- **Bab_5**: Berkas `Skripsi/bab5_kesimpulan.md` versi pasca-penulisan (output akhir spec ini), berjudul "BAB 5 SIMPULAN DAN SARAN".
- **Source_Of_Truth**: Source code Arabella di direktori `src/`, `examples/`, `tests/`, `assets/`, beserta `Cargo.toml`, `Cargo.lock`, dan `.cargo/config.toml` di akar repositori. Source_Of_Truth bersifat read-only untuk spec ini.
- **Bab_1_Final**: Berkas `Skripsi/bab1_pendahuluan.md` yang memuat latar belakang, Rumusan_Masalah (Subbab 1.2), ruang lingkup (Subbab 1.3), dan Tujuan_Penelitian (Subbab 1.4). Bab_5 wajib menjaga konsistensi dengan Bab_1_Final.
- **Bab_4_Final**: Berkas `Skripsi/bab4_implementasi_dan_hasil.md` yang sudah selesai ditulis, memuat spesifikasi lingkungan (Subbab 4.1), implementasi modul (Subbab 4.2), verifikasi correctness (Subbab 4.3), pengujian performa beserta angka nyata (Subbab 4.4), pembahasan trade-off (Subbab 4.5), dan keterbatasan implementasi (Subbab 4.6). Bab_4_Final adalah sumber utama seluruh klaim faktual Bab_5.
- **Abstrak**: Berkas `Skripsi/abstrak.md` yang merangkum tujuan, metode, dan hasil penelitian, termasuk penegasan kelayakan pendekatan rendering vektor hibrida non-compute pada lingkungan WebGL 2.0.
- **Bab_3_Final**: Berkas `Skripsi/bab3_metodologi.md` versi pasca-revisi (output spec `revisi-bab3-metodologi`) yang menetapkan terminologi kanonik dan kontrak referensi silang.
- **Spec_Bab3**: Direktori `.kiro/specs/revisi-bab3-metodologi/` yang mendefinisikan Istilah_Terlarang dan terminologi kanonik.
- **Spec_Bab4**: Direktori `.kiro/specs/penulisan-bab4-implementasi-dan-hasil/` yang meneruskan Istilah_Terlarang dan menetapkan klaim numerik kanonik.
- **Penulis**: Subjek aktif yang menulis Bab 5. Dalam dokumen requirements ini "Penulis" adalah sistem agen yang melaksanakan penulisan.
- **Klaim_Sintesis**: Pernyataan dalam Bab_5 yang menyimpulkan hasil penelitian atau mengusulkan pengembangan lanjutan, yang substansinya diturunkan dari Bab_1_Final, Bab_4_Final, atau Abstrak. Klaim_Sintesis TIDAK memperkenalkan fakta teknis, nilai numerik, atau terminologi baru yang tidak ada pada bab-bab tersebut.
- **Rujukan_Bab**: Token literal rujukan silang antar-bab di dalam Bab_5, ditulis sebagai `Bab N` (contohnya `Bab 1`, `Bab 4`) atau `Subbab N.M[.K]` (contohnya `Subbab 1.2`, `Subbab 4.4`, `Subbab 4.6`), yang menunjuk bab atau subbab yang benar-benar ada pada skripsi.
- **Subbab_Wajib**: Heading subbab yang harus hadir di Bab_5: `## 5.1 Simpulan`, `## 5.2 Saran`, beserta subheading `### 5.2.1 Saran Optimasi Performa`, `### 5.2.2 Saran Perluasan Fungsionalitas`, dan `### 5.2.3 Saran Evaluasi Lanjutan`.
- **Rumusan_Masalah**: Kelima butir pertanyaan penelitian pada Subbab 1.2 Bab_1_Final, dinomori RM-1 sampai RM-5 dalam dokumen ini sesuai urutannya pada Bab 1:
  - **RM-1**: Bagaimana pipeline rendering grafis vektor paralel dapat dirancang tanpa bergantung pada compute shader namun tetap mencapai paralelisme tinggi melalui pembagian beban CPU dan GPU.
  - **RM-2**: Sejauh mana tahap pra-pemrosesan dapat diparalelkan secara masif pada CPU, dan struktur data apa yang paling efektif mendukung paralelisasi tersebut.
  - **RM-3**: Bagaimana beban kerja kedua tahap didistribusikan secara efektif antara CPU dan GPU tanpa mengandalkan compute-oriented pipeline.
  - **RM-4**: Bagaimana dampak penghilangan compute shader terhadap latensi rendering dan throughput, khususnya pada beban kerja geometri vektor kompleks dan berskala besar.
  - **RM-5**: Bagaimana perbandingan pendekatan yang diusulkan dengan metode berbasis compute shader maupun CPU murni, ditinjau dari aspek kompatibilitas, kompleksitas implementasi, dan performa.
- **Tujuan_Penelitian**: Keempat butir tujuan pada Subbab 1.4 Bab_1_Final, dinomori TP-1 sampai TP-4 dalam dokumen ini: (TP-1) merancang arsitektur pipeline hibrida non-compute; (TP-2) mewujudkan pra-pemrosesan CPU dan menyiapkan fondasi paralelisasi CPU; (TP-3) mengukur dan menganalisis performa dekomposisi CPU/GPU serta memvalidasi correctness; (TP-4) mengidentifikasi dan mendokumentasikan trade-off non-compute.
- **Kontribusi_Inti**: Penegasan bahwa kelayakan (feasibility) pipeline rendering vektor hibrida non-compute pada lingkungan WebGL 2.0 telah terbukti melalui purwarupa Arabella, sebagaimana dinyatakan pada Abstrak dan disimpulkan dari hasil Bab 4.
- **Angka_Performa_Bab4**: Himpunan nilai numerik performa yang dilaporkan pada Tabel 4.4 dan narasi Subbab 4.4 Bab_4_Final, yaitu untuk `el_gato.svg` (0,69 ms CPU; 0,100 ms GPU; 0,79 ms total; 1266 FPS), `Ghostscript_Tiger.svg` (9,37 ms CPU; 0,321 ms GPU; 9,69 ms total; 103 FPS), dan `paris-30k.svg` (150,15 ms CPU; 6,00 ms GPU; 156,15 ms total; 6,4 FPS), beserta temuan bottleneck pra-pemrosesan CPU single-thread sebesar 150,15 ms dari 156,15 ms (sekitar 96 persen) pada `paris-30k.svg`.
- **Istilah_Wajib**: Himpunan istilah kanonik yang wajib muncul minimal satu kali pada subbab tertentu sebagaimana dirinci pada AC Requirement 4 sampai 8 dan Requirement 11.
- **Istilah_Terlarang**: Himpunan istilah yang tidak boleh muncul pada Bab_5. Daftar diteruskan dari Spec_Bab3 dan Spec_Bab4, dirinci pada AC Requirement 11.

## Requirements

### Requirement 1: Identitas Berkas Output Tunggal

**User Story:** Sebagai dosen pembimbing, saya ingin satu berkas tunggal `Skripsi/bab5_kesimpulan.md` hasil penulisan menggantikan placeholder lorem ipsum lama, sehingga saya dapat membaca Bab 5 final tanpa menelusuri beberapa lokasi.

#### Acceptance Criteria

1. THE Penulis SHALL menempatkan seluruh unit konten Bab_5 — heading utama level 1, seluruh heading Subbab_Wajib, dan paragraf naratif — di dalam satu berkas tunggal dengan jalur `Skripsi/bab5_kesimpulan.md` relatif terhadap akar repositori.
2. THE Penulis SHALL menyimpan berkas Bab_5 dengan ekstensi `.md`, encoding UTF-8 tanpa BOM, dan akhiran baris yang konsisten sehingga isinya dapat diurai sebagai Markdown CommonMark valid dan dicocokkan secara byte-deterministik.
3. THE Penulis SHALL meletakkan heading utama `# BAB 5 SIMPULAN DAN SARAN` pada baris pertama berkas Bab_5 dengan kapitalisasi penuh, prefiks `# ` (satu tanda pagar diikuti tepat satu spasi tunggal), tanpa BOM, tanpa karakter whitespace di awal maupun akhir baris heading, dan diakhiri tepat satu karakter newline tunggal sebelum konten berikutnya.
4. THE Penulis SHALL menghapus seluruh teks lorem ipsum yang ada pada berkas saat ini sehingga frasa `lorem ipsum`, `dolor sit amet`, `consectetur adipiscing`, `tempor incididunt`, `exercitation ullamco`, dan `duis aute irure` (case-insensitive) TIDAK muncul pada Bab_5 final; substansi naratif yang menggantikan teks placeholder tersebut diatur oleh Requirement 4 sampai Requirement 13.
5. THE Penulis SHALL TIDAK membuat berkas Bab 5 tambahan, salinan parsial, cadangan, atau draf alternatif di repositori, di mana "berkas Bab 5 tambahan" didefinisikan sebagai berkas yang memenuhi salah satu dari dua kriteria berikut: (a) baris pertama berkas mencocokkan persis `# BAB 5 SIMPULAN DAN SARAN`, atau (b) nama berkas mengandung substring `bab5` (case-insensitive); pemindaian kriteria ini SHALL mengabaikan seluruh berkas di dalam direktori `.kiro/specs/` agar berkas-berkas spec ini sendiri tidak menghasilkan false positive.
6. IF terdapat berkas lain di repositori (di luar direktori `.kiro/specs/`) yang baris pertamanya mencocokkan persis `# BAB 5 SIMPULAN DAN SARAN`, THEN THE Penulis SHALL menghapus berkas tersebut sehingga `Skripsi/bab5_kesimpulan.md` menjadi satu-satunya pemilik heading tersebut.
7. IF berkas Bab_5 memiliki ekstensi `.md` namun gagal diurai sebagai Markdown CommonMark valid oleh parser referensi, THEN THE Bab_5 SHALL dianggap tidak memenuhi syarat Requirement 1 hingga galat sintaks tersebut diperbaiki.

### Requirement 2: Source-of-Truth dan Bab-Lain Invariance

**User Story:** Sebagai pengembang yang merangkap sebagai mahasiswa, saya ingin penulisan Bab 5 tidak mengubah satu pun berkas source code maupun bab skripsi lain, sehingga purwarupa dan bab-bab yang sudah final tetap utuh.

#### Acceptance Criteria

1. THE Penulis SHALL TIDAK memodifikasi berkas apa pun di dalam Source_Of_Truth (`src/`, `examples/`, `tests/`, `assets/`, `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`), di mana "memodifikasi" berarti perubahan konten pada level byte antara titik mulai dan titik akhir eksekusi spec ini.
2. THE Penulis SHALL TIDAK menambah, menghapus, memindahkan, maupun menamai ulang berkas apa pun di dalam Source_Of_Truth; jumlah berkas dan himpunan jalur relatif berkas di dalam Source_Of_Truth SHALL identik antara titik mulai dan titik akhir eksekusi spec ini.
3. WHILE eksekusi spec ini berlangsung — yaitu periode dari mulainya tugas pertama pada `tasks.md` sampai selesainya tugas terakhir pada `tasks.md` — THE Penulis SHALL TIDAK mengubah konten berkas skripsi lain selain Bab_5, yakni `Skripsi/bab1_pendahuluan.md`, `Skripsi/bab2_landasan_teori.md`, `Skripsi/bab3_metodologi.md`, `Skripsi/bab4_implementasi_dan_hasil.md`, `Skripsi/abstrak.md`, `Skripsi/kata_pengantar.md`, `Skripsi/daftar_pustaka.md`, dan `Skripsi/analisis_project_dan_skripsi.md`.
4. WHEN Penulis perlu memverifikasi rumusan masalah, tujuan penelitian, angka performa, atau klaim faktual untuk Klaim_Sintesis, THE Penulis SHALL membaca Bab_1_Final, Bab_4_Final, Abstrak, dan Source_Of_Truth secara strict read-only tanpa operasi tulis (write), tambah (append), pembuatan berkas baru, maupun perubahan metadata pada berkas-berkas tersebut.
5. THE Penulis SHALL TIDAK menjalankan perintah yang memutasi Source_Of_Truth sebagai efek samping, termasuk namun tidak terbatas pada `cargo build`, `cargo test`, `cargo update`, atau perintah formatter yang menulis ulang berkas sumber, selama setiap perintah semacam itu menghasilkan perubahan byte pada berkas mana pun di dalam Source_Of_Truth.

### Requirement 3: Kelengkapan Struktural Subbab Wajib

**User Story:** Sebagai mahasiswa yang menulis Bab 5 sesuai panduan kampus BINUS, saya ingin struktur bab penutup hadir dengan heading yang konsisten, sehingga Bab 5 mengikuti format "BAB 5 SIMPULAN DAN SARAN" dan dapat diverifikasi secara deterministik.

#### Acceptance Criteria

1. THE Bab_5 SHALL memuat heading utama `# BAB 5 SIMPULAN DAN SARAN` tepat satu kali pada level heading 1 dengan format ATX, pada baris pertama berkas sebagaimana Requirement 1 AC 3.
2. THE Bab_5 SHALL memuat heading `## 5.1 Simpulan` tepat satu kali pada level heading 2 dengan format ATX, dengan baris heading dimulai pada kolom pertama tanpa karakter whitespace di awal baris, prefiks `## ` berupa tepat dua tanda pagar diikuti tepat satu spasi tunggal sebelum nomor subbab, tanpa urutan penutup tanda pagar (closing sequence) dan tanpa karakter tambahan apa pun setelah teks `5.1 Simpulan`, dengan kemunculan di dalam fenced code block dikecualikan dari penghitungan sebagaimana didefinisikan pada AC 7.
3. THE Bab_5 SHALL memuat heading `## 5.2 Saran` tepat satu kali pada level heading 2 dengan format ATX, beserta subheading `### 5.2.1 Saran Optimasi Performa`, `### 5.2.2 Saran Perluasan Fungsionalitas`, dan `### 5.2.3 Saran Evaluasi Lanjutan`, masing-masing tepat satu kali pada level heading 3 dengan format ATX; setiap baris heading tersebut SHALL dimulai pada kolom pertama tanpa karakter whitespace di awal baris, dengan prefiks (`## ` atau `### `) diikuti tepat satu spasi tunggal dan tanpa karakter tambahan apa pun setelah teks heading; kemunculan di dalam fenced code block dikecualikan sebagaimana AC 7.
4. THE Bab_5 SHALL menyusun heading subbab level 2 pada urutan menaik monotonik 5.1 → 5.2 tanpa nomor yang dilewati, diulang, atau dibalik, dan SHALL TIDAK memuat heading level 2 lain di luar kedua Subbab_Wajib tersebut.
5. THE Bab_5 SHALL menempatkan ketiga subheading `### 5.2.y` setelah heading induknya `## 5.2 Saran`, dengan urutan `y` menaik monotonik kontigu 1 → 2 → 3 tanpa nomor yang dilewati, diulang, atau dibalik, dan SHALL TIDAK memuat heading level 3 lain di luar ketiga subheading 5.2.1–5.2.3 tersebut di bagian mana pun Bab_5.
6. THE Bab_5 SHALL TIDAK memuat subheading level 3 maupun level yang lebih dalam (`###`, `####`, dan seterusnya) di bawah `## 5.1 Simpulan`; seluruh konten Subbab 5.1 SHALL berupa paragraf naratif dan/atau daftar (list) tanpa heading bersarang.
7. THE Bab_5 SHALL mengecualikan kemunculan teks heading di dalam fenced code block (blok yang dibatasi oleh ` ``` ` atau `~~~`) dari penghitungan kehadiran heading pada AC 1 sampai AC 6; hanya heading Markdown ATX di luar fenced code block yang dihitung sebagai heading struktural.

### Requirement 4: Simpulan Menjawab Rumusan Masalah dan Tujuan Penelitian (Subbab 5.1)

**User Story:** Sebagai dosen penguji, saya ingin Subbab Simpulan menjawab secara eksplisit setiap rumusan masalah dan tujuan penelitian yang ditetapkan pada Bab 1 berdasarkan hasil pada Bab 4, sehingga simpulan benar-benar menutup lingkaran penelitian dan bukan sekadar ringkasan bebas.

#### Acceptance Criteria

1. THE Bab_5 SHALL memuat pada Subbab 5.1 satu paragraf pembuka — didefinisikan sebagai blok teks Markdown non-kosong pertama yang muncul tepat setelah heading `## 5.1 Simpulan` dan sebelum kelima butir simpulan pada AC 2 — yang menyatakan bahwa simpulan disusun untuk menjawab Rumusan_Masalah pada Subbab 1.2 dan Tujuan_Penelitian pada Subbab 1.4, dengan menuliskan token literal `Bab 1` minimal satu kali serta token literal `Subbab 1.2` dan `Subbab 1.4` masing-masing minimal satu kali pada paragraf tersebut.
2. THE Bab_5 SHALL menyajikan pada Subbab 5.1 tepat lima butir simpulan, tidak kurang dan tidak lebih, yang ditulis sebagai satu daftar terurut (ordered list) Markdown berisi tepat lima item atau sebagai tepat lima paragraf naratif terpisah yang berurutan, di mana butir ke-n untuk n bernilai 1 sampai 5 menjawab Rumusan_Masalah RM-n secara posisional sehingga pemetaan setiap butir simpulan ke Rumusan_Masalah bersifat satu-ke-satu, menaik monotonik dari RM-1 sampai RM-5, dan dapat diverifikasi berdasarkan urutan kemunculannya.
3. THE Bab_5 SHALL menyatakan pada butir simpulan yang menjawab RM-1 bahwa arsitektur pipeline hibrida non-compute berhasil dirancang dan diwujudkan dengan pembagian beban CPU (pra-pemrosesan: flattening, binning DDA, akumulator signed-area, propagasi backdrop) dan GPU (rasterisasi melalui vertex shader dan fragment shader), dengan Rujukan_Bab ke `Bab 4` (atau `Subbab 4.1` atau `Subbab 4.2`).
4. THE Bab_5 SHALL menyatakan pada butir simpulan yang menjawab RM-2 secara jujur bahwa fondasi struktur data berbasis ubin untuk paralelisasi pra-pemrosesan CPU telah terwujud dan paralelisme tingkat instruksi melalui SIMD telah aktif, sementara paralelisme tingkat data penuh melalui Rayon multithreading belum diaktifkan pada implementasi saat ini, dengan Rujukan_Bab ke `Subbab 4.6` dan SHALL menautkan butir ini ke saran pada Subbab 5.2.1.
5. THE Bab_5 SHALL menyatakan pada butir simpulan yang menjawab RM-3 bahwa pembagian beban kerja antara CPU dan GPU berlangsung efektif untuk adegan berskala kecil sampai menengah, dengan dukungan temuan pada Subbab 4.4 bahwa biaya rasterisasi GPU tumbuh landai dan stabil sementara biaya pra-pemrosesan CPU mendominasi total waktu satu frame, disertai Rujukan_Bab ke `Subbab 4.4`.
6. THE Bab_5 SHALL menyatakan pada butir simpulan yang menjawab RM-4 bahwa pada adegan berskala sangat besar tahap pra-pemrosesan CPU single-thread menjadi bottleneck dominan, dan WHERE butir ini menyertakan nilai numerik, THE nilai tersebut SHALL diambil dari Angka_Performa_Bab4 (yaitu 150,15 ms dari 156,15 ms total, sekitar 96 persen, dan 6,4 FPS pada `paris-30k.svg`) secara identik dan disertai Rujukan_Bab ke `Subbab 4.4`.
7. THE Bab_5 SHALL menyatakan pada butir simpulan yang menjawab RM-5 secara jujur bahwa perbandingan terhadap renderer berbasis compute shader (Vello) dan renderer CPU murni (Skia, Cairo) yang dapat ditarik dari penelitian ini bersifat kualitatif pada dimensi kompatibilitas, kompleksitas implementasi, dan performa arsitektural, dan bahwa benchmark kuantitatif langsung belum dilakukan, dengan Rujukan_Bab ke `Subbab 4.4` (atau `Subbab 4.4.3`) dan `Subbab 4.5`.
8. THE Bab_5 SHALL memastikan bahwa kelima butir simpulan pada AC 2 secara kolektif menjawab dan konsisten dengan Tujuan_Penelitian TP-1 sampai TP-4, sehingga setiap butir Tujuan_Penelitian tercermin pada minimal satu butir simpulan.
9. IF suatu butir simpulan pada AC 2 hendak menyatakan capaian yang tidak dilaporkan tercapai pada Bab_4_Final, THEN THE Bab_5 SHALL TIDAK memuat pernyataan tersebut, dan secara khusus tidak ada butir simpulan yang menyatakan bahwa paralelisme tingkat data penuh melalui Rayon multithreading telah aktif, bahwa benchmark kuantitatif langsung antar-renderer telah dilakukan, maupun bahwa Arabella unggul secara performa kuantitatif terhadap Skia, Cairo, atau Vello.

### Requirement 5: Penegasan Kontribusi Inti (Subbab 5.1)

**User Story:** Sebagai pembaca, saya ingin Subbab Simpulan menegaskan kontribusi inti penelitian secara ringkas dan tegas, sehingga nilai penelitian terbaca jelas tanpa melebih-lebihkan.

#### Acceptance Criteria

1. THE Bab_5 SHALL memuat pada Subbab 5.1 minimal satu kalimat yang menegaskan Kontribusi_Inti, yaitu bahwa kelayakan pipeline rendering vektor hibrida non-compute pada lingkungan WebGL 2.0 telah terbukti melalui purwarupa Arabella.
2. THE Bab_5 SHALL menuliskan pada penegasan Kontribusi_Inti minimal frasa literal `pipeline` dan `non-compute` serta token `WebGL 2.0` dan `Arabella`, sehingga penegasan tersebut konsisten dengan rumusan pada Abstrak.
3. THE Bab_5 SHALL menempatkan penegasan Kontribusi_Inti sebagai bagian dari Subbab 5.1 (paragraf penutup Subbab 5.1 atau salah satu butir simpulan), dan SHALL TIDAK menempatkannya di dalam Subbab 5.2.
4. THE Bab_5 SHALL TIDAK menyatakan klaim kelayakan yang melampaui ruang lingkup yang terbukti, yakni SHALL TIDAK mengklaim keunggulan performa kuantitatif terhadap Skia, Cairo, atau Vello, mengingat benchmark kuantitatif langsung belum dilakukan sebagaimana dinyatakan pada Subbab 4.4.3.

### Requirement 6: Saran Optimasi Performa (Subbab 5.2.1)

**User Story:** Sebagai peneliti lanjutan, saya ingin saran optimasi performa yang diturunkan langsung dari temuan bottleneck pada Bab 4, sehingga arah pengembangan berikutnya berpijak pada bukti pengukuran, bukan spekulasi.

#### Acceptance Criteria

1. THE Bab_5 SHALL mengusulkan pada Subbab 5.2.1 pengaktifan paralelisme tingkat data melalui Rayon multithreading pada tahap pra-pemrosesan CPU — khususnya flattening dan binning DDA per jalur yang menjadi penyumbang dominan waktu CPU — dengan menuliskan minimal satu kemunculan masing-masing token literal `Rayon` dan `multithreading` pada Subbab 5.2.1.
2. THE Bab_5 SHALL menautkan usulan pada AC 1 secara eksplisit ke temuan bottleneck pra-pemrosesan CPU single-thread pada Subbab 4.4 dengan menuliskan minimal satu Rujukan_Bab ke `Subbab 4.4` pada paragraf atau butir yang sama dengan usulan tersebut, dan WHERE usulan menyertakan nilai numerik pendukung, THE nilai tersebut SHALL diambil dari Angka_Performa_Bab4 secara identik, termasuk penyajian numerik yang sama persis (tanda koma sebagai pemisah desimal, misalnya `150,15`, dan satuan yang sama seperti `ms`, `FPS`, atau `persen`).
3. THE Bab_5 SHALL menyatakan pada Subbab 5.2.1 bahwa beban pra-pemrosesan (flattening dan binning tiap jalur) bersifat saling independen sehingga dapat-diparalelkan, sebagai dasar argumentatif bahwa Rayon multithreading berpotensi mengurangi waktu pra-pemrosesan CPU pada adegan berskala sangat besar (sebagaimana tercermin pada `paris-30k.svg`), dengan menuliskan minimal satu Rujukan_Bab ke `Subbab 4.6` tempat status feature `multithreading` dideklarasikan opsional.
4. THE Bab_5 SHALL TIDAK menjanjikan angka peningkatan performa spesifik sebagai hasil pengaktifan Rayon, dengan enumerasi eksplisit kategori yang dilarang: faktor speedup (misalnya kali lipat), nilai FPS target, persentase pengurangan waktu pra-pemrosesan, dan nilai waktu target dalam `ms`, karena angka-angka tersebut belum diukur.
5. THE Bab_5 SHALL menyatakan pada Subbab 5.2.1 bahwa usulan pengaktifan Rayon merupakan potensi optimasi yang masih perlu diverifikasi melalui pengukuran lanjutan, sehingga usulan tidak disajikan sebagai peningkatan performa yang sudah terbukti.

### Requirement 7: Saran Perluasan Fungsionalitas (Subbab 5.2.2)

**User Story:** Sebagai dosen penguji, saya ingin saran perluasan fungsionalitas yang diturunkan langsung dari keterbatasan yang didaftarkan pada Bab 4, sehingga setiap usulan menutup celah fungsional yang sudah teridentifikasi secara jujur.

#### Acceptance Criteria

1. THE Bab_5 SHALL mengusulkan pada Subbab 5.2.2 minimal empat perluasan fungsionalitas yang masing-masing dipetakan satu-ke-satu ke tepat satu butir keterbatasan fungsional yang didaftarkan pada Subbab 4.6 dan secara eksplisit menyebut keterbatasan yang ditutupnya, mencakup: (a) pengaktifan paint bergradien (linear, radial, sweep) pada fragment shader; (b) dukungan image paint dan tinting; (c) perluasan subset SVG di luar elemen `g` dan `path` (misalnya `defs`, `use`, bentuk dasar, gradien, pattern); dan (d) penambahan sistem text rendering.
2. THE Bab_5 SHALL menyertakan pada Subbab 5.2.2 minimal satu usulan tambahan berupa penambahan filter effect (misalnya blur atau drop shadow), sehingga total perluasan fungsionalitas yang diusulkan mencakup kelima dimensi keterbatasan fungsional pada Subbab 4.6, yaitu: (1) paint bergradien pada fragment shader; (2) image paint dan tinting; (3) perluasan subset SVG di luar elemen `g` dan `path`; (4) sistem text rendering; dan (5) filter effect; dengan setiap dimensi diwakili minimal satu usulan yang dapat diidentifikasi secara terpisah.
3. THE Bab_5 SHALL menuliskan pada Subbab 5.2.2 minimal satu Rujukan_Bab ke `Subbab 4.6` yang menautkan usulan perluasan fungsionalitas ke keterbatasan yang sudah didokumentasikan.
4. THE Bab_5 SHALL menyusun usulan perluasan fungsionalitas pada AC 1 dan AC 2 sebagai daftar (ordered list atau unordered list) Markdown atau sebagai paragraf-paragraf, di mana setiap usulan dapat diidentifikasi secara terpisah sebagai tepat satu item daftar atau tepat satu blok teks Markdown — yakni satu atau lebih baris non-kosong berurutan yang dipisahkan dari blok lain oleh setidaknya satu baris kosong — sehingga setiap usulan dapat dipetakan satu-ke-satu ke keterbatasan yang ditutupnya.
5. THE Bab_5 SHALL menggunakan pada Subbab 5.2.2 terminologi kanonik `fragment shader` ketika merujuk shader piksel WebGL, dan SHALL TIDAK memperkenalkan nama konstanta, fungsi, atau struct baru yang tidak muncul pada Bab_4_Final ketika mendeskripsikan usulan tersebut.
6. THE Bab_5 SHALL menyatakan setiap perluasan fungsionalitas pada Subbab 5.2.2 sebagai pengembangan lanjutan (future work) yang belum diwujudkan pada implementasi Arabella saat ini, dan SHALL TIDAK menyatakan maupun menyiratkan bahwa salah satu perluasan tersebut sudah terimplementasi atau sudah aktif, konsisten dengan status keterbatasan yang didokumentasikan pada Subbab 4.6.

### Requirement 8: Saran Evaluasi Lanjutan (Subbab 5.2.3)

**User Story:** Sebagai dosen penguji, saya ingin saran evaluasi lanjutan yang menutup keterbatasan metodologis Bab 4 (ketiadaan benchmark kuantitatif langsung), sehingga arah validasi penelitian berikutnya jelas.

#### Acceptance Criteria

1. THE Bab_5 SHALL mengusulkan pada Subbab 5.2.3 pelaksanaan benchmark kuantitatif langsung (head-to-head) yang membandingkan Arabella terhadap renderer pembanding, dengan menyebut minimal Skia, Cairo, dan Vello secara literal.
2. THE Bab_5 SHALL menautkan usulan pada AC 1 ke disclaimer pada Subbab 4.4.3 yang menyatakan bahwa benchmark kuantitatif langsung belum dilakukan, dengan menuliskan Rujukan_Bab ke `Subbab 4.4.3` atau `Subbab 4.4`.
3. THE Bab_5 SHALL menyatakan pada Subbab 5.2.3 bahwa benchmark lanjutan tersebut harus dijalankan pada perangkat keras dan berkas uji yang identik antar-renderer agar hasilnya dapat dibandingkan secara adil, konsisten dengan catatan ketergantungan hasil pada konfigurasi mesin uji pada Subbab 4.4.3.
4. THE Bab_5 SHALL TIDAK menyatakan prakiraan hasil benchmark (misalnya bahwa Arabella akan lebih cepat atau lebih lambat dari renderer tertentu), karena hasil tersebut belum terukur.

### Requirement 9: Traceability Klaim Sintesis ke Bab 1 dan Bab 4

**User Story:** Sebagai dosen penguji, saya ingin setiap klaim sintesis di Bab 5 dapat saya telusuri ke bab sumbernya (Bab 1 atau Bab 4), sehingga saya yakin Bab 5 tidak memperkenalkan klaim atau angka baru yang tidak berdasar.

#### Acceptance Criteria

1. WHEN Bab_5 memuat Klaim_Sintesis yang menyimpulkan hasil penelitian atau menyitir temuan empiris, THE Penulis SHALL menyertakan minimal satu Rujukan_Bab pada paragraf atau butir yang sama dengan Klaim_Sintesis tersebut, di mana "paragraf atau butir yang sama" didefinisikan sebagai blok teks Markdown yang terdiri dari satu atau lebih baris non-kosong berurutan dan dipisahkan dari blok lain oleh setidaknya satu baris kosong, atau satu item daftar.
2. THE Penulis SHALL menulis setiap Rujukan_Bab dalam format `Bab N` atau `Subbab N.M` atau `Subbab N.M.K` (contohnya `Bab 1`, `Subbab 1.2`, `Subbab 4.4`, `Subbab 4.4.3`, `Subbab 4.6`), dengan `N`, `M`, dan `K` menunjuk bab atau subbab yang benar-benar ada pada skripsi.
3. THE Penulis SHALL memastikan setiap Rujukan_Bab menunjuk bab atau subbab yang benar-benar ada, yaitu `Bab 1` sampai `Bab 5` untuk rujukan tingkat bab, dan untuk rujukan tingkat subbab hanya nomor yang hadir pada bab yang dirujuk (untuk Bab 1: 1.1 sampai 1.6; untuk Bab 4: 4.1 sampai 4.6 beserta subheading 4.2.1–4.2.8, 4.4.1–4.4.3).
4. THE Penulis SHALL TIDAK memasukkan ke Bab_5 Klaim_Sintesis yang menyebut fakta teknis, nilai numerik performa, atau temuan empiris yang tidak hadir pada Bab_1_Final, Bab_4_Final, atau Abstrak.
5. THE Penulis SHALL TIDAK menyertakan pada Bab_5 rujukan kode berformat `` `berkas:simbol` `` atau `` `berkas:start-end` `` sebagai dasar Klaim_Sintesis, karena Bab 5 adalah bab sintesis yang ketertelusurannya mengarah ke bab lain (Rujukan_Bab), bukan ke source code; penyebutan nama identifier kode (misalnya `Builder::build_path`) hanya diperbolehkan WHERE diperlukan untuk menjaga konsistensi terminologi dan TIDAK dijadikan satu-satunya dasar ketertelusuran klaim.

### Requirement 10: Anti-Fabrikasi Numerik

**User Story:** Sebagai pembaca, saya ingin yakin bahwa setiap angka yang muncul di Bab 5 berasal dari pengukuran nyata yang sudah dilaporkan pada Bab 4, sehingga tidak ada data baru atau angka karangan yang masuk melalui bab penutup.

#### Acceptance Criteria

1. THE Bab_5 SHALL TIDAK memuat nilai FPS, CPU ms, GPU ms, total frame time, throughput, jumlah operasi paint, jumlah ubin, persentase bottleneck, atau metrik performa numerik lain kecuali nilai tersebut identik dengan nilai yang termuat dalam Angka_Performa_Bab4 dan disertai Rujukan_Bab ke `Subbab 4.4` (atau subbab Bab 4 yang melaporkannya) pada paragraf atau butir yang sama.
2. THE Bab_5 SHALL TIDAK memperkenalkan nilai numerik performa baru, nilai hasil estimasi, nilai proyeksi, nilai target, maupun nilai yang dikutip dari literatur eksternal sebagai hasil pengukuran Arabella.
3. WHERE Bab_5 menyebut parameter numerik kanonik yang berasal dari konstanta kode atau konfigurasi pengujian (misalnya dimensi ubin 16×8, ukuran rekord `Tile` 44 byte, resolusi pengujian 1080×520, format `F24Dot8`, format `8.8 fixed-point`, atau `WINDING_UNIT = 256`), THE nilai tersebut SHALL identik dengan nilai yang dilaporkan pada Bab_3_Final atau Bab_4_Final dan disertai Rujukan_Bab ke bab yang melaporkannya.
4. THE Bab_5 SHALL TIDAK memuat nilai kontradiktif untuk parameter numerik kanonik pada AC 3, dengan enumerasi eksplisit: dimensi ubin selain 16×8, ukuran rekord `Tile` selain 44 byte, resolusi pengujian selain 1080×520, format fixed-point segmen selain `F24Dot8`, dan format fixed-point akumulator winding selain `8.8 fixed-point`.
5. THE Bab_5 SHALL memastikan bahwa setiap nilai pada Angka_Performa_Bab4 yang dikutip ditulis dengan penyajian numerik yang sama persis dengan Bab_4_Final, termasuk penggunaan tanda koma sebagai pemisah desimal (misalnya `150,15` bukan `150.15`) dan satuan yang sama (`ms`, `FPS`, `persen`).

### Requirement 11: Konsistensi Terminologi Kanonik dan Eliminasi Istilah Terlarang

**User Story:** Sebagai pembaca skripsi yang membaca Bab 1 sampai Bab 5 secara berurutan, saya ingin istilah lintas-bab tetap konsisten, sehingga Bab 5 tidak merusak referensi silang yang sudah dibangun pada bab-bab sebelumnya.

#### Acceptance Criteria

1. WHEN Bab_5 merujuk konsep arsitektural yang sudah didefinisikan pada Bab_3_Final atau Bab_4_Final, THE Bab_5 SHALL menggunakan istilah kanonik berikut secara literal: `pipeline hibrida` untuk arsitektur Arabella secara keseluruhan, `non-compute` (atau frasa `tanpa compute shader`) untuk sifat pendekatan, `pra-pemrosesan` untuk fase CPU keseluruhan, `binning DDA` untuk tahap pemecahan segmen lintas ubin, `akumulator signed-area` untuk akumulator winding per scanline, `propagasi backdrop` untuk akumulasi kiri-ke-kanan saat emisi tile, dan `fragment shader` untuk shader piksel WebGL.
2. THE Bab_5 SHALL memuat minimal satu kemunculan masing-masing istilah kanonik berikut di seluruh dokumen: `pipeline hibrida`, `non-compute` (atau `tanpa compute shader`), `WebGL 2.0`, `Arabella`, `pra-pemrosesan`, `CPU`, dan `GPU`.
3. THE Bab_5 SHALL memuat minimal satu kemunculan istilah `winding number` (tanpa underscore, dengan satu spasi tunggal antar kata, case-insensitive) ketika merujuk konsep winding, dan SHALL TIDAK menjadikannya nama field skalar pada struct apa pun.
4. THE Bab_5 SHALL TIDAK mencampur varian `pra-pemrosesan` dan `preprocessing` dalam satu paragraf yang sama, di mana "paragraf yang sama" didefinisikan sebagai blok teks Markdown CommonMark berupa satu atau lebih baris non-kosong berurutan yang dipisahkan dari blok lain oleh setidaknya satu baris kosong.
5. THE Bab_5 SHALL TIDAK memuat istilah berikut yang termasuk Istilah_Terlarang yang diteruskan dari Spec_Bab3 dan Spec_Bab4:
   - `Ray Shooting`, `Ray Shoot`, `ray shooting`, `ray shoot` sebagai frasa utuh nama algoritma (case-insensitive);
   - `TileType` sebagai token kata utuh nama enum atau jenis ubin (case-sensitive);
   - `EMPTY`, `INTERIOR`, `EDGE` sebagai token kata utuh label tipe ubin atau cabang fragment shader (case-sensitive);
   - `winding_number` sebagai token kata utuh nama field skalar pada struct Tile (case-sensitive, dengan underscore);
   - `fungsi implisit linear`, `fungsi implisit kuadratik kanonik`, `fungsi implisit kubik`, `PPGA`, `Projective Geometric Algebra` sebagai frasa utuh (case-insensitive untuk frasa berbahasa Indonesia, case-sensitive untuk akronim `PPGA`);
   - frasa `OpenGL ES 3.0 yang ditranspilasikan`, `ditranspilasikan ke WebGL`, `transpilasi OpenGL ES` (case-insensitive);
   - frasa `Rust edisi 2021`, `edisi 2021`, dan token `edition = "2021"` (case-sensitive untuk literal `edition = "2021"`).
6. THE Bab_5 SHALL mempertahankan kosakata dan format penomoran subbab yang sudah dipakai di Bab 1 sampai Bab 4 (`pustaka`, `perangkat lunak`, `pra-pemrosesan`, `ubin`, `purwarupa`, format `5.x` dan `5.x.y`).

### Requirement 12: Konektivitas Naratif dan Sifat Sintesis

**User Story:** Sebagai pembaca, saya ingin Bab 5 terbaca sebagai sintesis yang koheren dari Bab 1 sampai Bab 4 dan bukan pengulangan teknis, sehingga bab penutup terasa menutup penelitian tanpa mengulang materi tanpa keperluan.

#### Acceptance Criteria

1. THE Bab_5 SHALL memuat pada Subbab 5.1 minimal satu kalimat penghubung yang merujuk hasil pada Bab 4 sebagai dasar simpulan, dengan menuliskan token literal `Bab 4` minimal satu kali.
2. THE Bab_5 SHALL memuat pada Subbab 5.2 minimal satu kalimat yang menyatakan bahwa saran pengembangan lanjutan diturunkan dari keterbatasan dan temuan pada Bab 4, dengan menuliskan minimal satu Rujukan_Bab ke `Subbab 4.4` atau `Subbab 4.6`.
3. THE Bab_5 SHALL TIDAK memuat blok verbatim sepanjang lebih dari 30 kata berurutan yang identik dengan blok teks pada Bab_1_Final, Bab_3_Final, atau Bab_4_Final, sehingga duplikasi naratif dengan bab lain dapat diuji secara mekanis.
4. THE Bab_5 SHALL TIDAK memuat deskripsi implementasi tingkat-rinci baru (uraian langkah algoritma, signature fungsi, atau tata letak struktur data) yang merupakan materi Bab 3 atau Bab 4; penyebutan tahap pipeline pada Bab_5 SHALL berupa rujukan ringkas untuk keperluan sintesis, bukan uraian teknis ulang.
5. THE Bab_5 SHALL menyatakan minimal satu kali di salah satu Subbab_Wajib bahwa keterbatasan yang ada tidak menggugurkan validitas Kontribusi_Inti, dengan memuat salah satu kata kunci `future work` atau `pengembangan lanjutan` (case-insensitive) pada konteks tersebut.

### Requirement 13: Gaya Bahasa dan Format Akademik

**User Story:** Sebagai mahasiswa yang menulis skripsi sesuai panduan kampus, saya ingin Bab 5 menggunakan gaya bahasa akademik formal Indonesia yang konsisten dengan bab-bab sebelumnya.

#### Acceptance Criteria

1. THE Bab_5 SHALL ditulis dalam bahasa Indonesia formal akademik dengan kalimat lengkap yang sekurang-kurangnya memuat unsur subjek dan predikat (berpola subjek-predikat-objek atau subjek-predikat-keterangan), dan SHALL TIDAK memuat token bahasa percakapan berikut sebagai kata utuh — dicocokkan dengan batas kata (word boundary) dan secara case-insensitive — di luar fenced code block: `bisa`, `gak`, `enggak`, `nih`, `dong`, `kok`, `kan`, `aja`, `udah`, dan `mau`; sebagai gantinya THE Bab_5 SHALL menggunakan bentuk formal baku seperti `dapat` alih-alih `bisa`.
2. THE Bab_5 SHALL membungkus setiap nama identifier kode (struct, fungsi, konstanta, feature flag, nama berkas, jalur berkas) yang disebut dengan backtick (`` ` ``), dan SHALL membungkus istilah teknis berbahasa Inggris yang dipakai sebagai frasa terminologi disiplin (misalnya *bottleneck*, *single-thread*, *future work*) dengan italic (`*frasa*` atau `_frasa_`); WHERE suatu istilah teknis berbahasa Inggris diberi format italic pada salah satu kemunculannya, THE Bab_5 SHALL memberi format italic pada seluruh kemunculan istilah yang sama secara konsisten di luar fenced code block.
3. THE Bab_5 SHALL menggunakan kalimat pasif atau bentuk impersonal akademik dan SHALL TIDAK menggunakan kata ganti orang pertama (`saya`, `kami`, `kita`) maupun kata ganti orang kedua (`Anda`, `kamu`) sebagai kata utuh — dicocokkan dengan batas kata (word boundary) dan secara case-insensitive — di dalam narasi Bab 5 di luar fenced code block.
4. THE Bab_5 SHALL menuliskan setiap heading subbab level 2 dengan format `5.x` dan setiap heading subbab level 3 dengan format `5.x.y`, di mana `x` dan `y` adalah bilangan bulat positif tanpa angka nol di depan, mengikuti pola penomoran subbab yang sama dengan heading subbab pada Bab 1 sampai Bab 4.
5. THE Bab_5 SHALL menyusun isi setiap Subbab_Wajib dengan minimal satu paragraf naratif yang terdiri atas sekurang-kurangnya tiga kalimat lengkap, dan WHERE daftar dipakai untuk butir simpulan atau butir saran, THE daftar tersebut SHALL menggunakan sintaks daftar Markdown yang valid (ordered list yang diawali `1.` atau unordered list yang diawali `- `) serta SHALL didahului oleh minimal satu kalimat pengantar naratif.
