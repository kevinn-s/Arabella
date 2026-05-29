# BAB 1 PENDAHULUAN

## 1.1 Latar Belakang

Grafis vektor merupakan salah satu fondasi representasi visual dalam komputasi modern. Berbeda dengan grafis berbasis piksel yang merepresentasikan citra sebagai kumpulan titik berwarna pada kisi tetap, grafis vektor mendefinisikan bentuk-bentuk sembarang (arbitrary shapes) secara presisi matematis menggunakan jalur primitif (primitive paths) yang tersusun dari kurva Bézier, garis, dan poligon. Keunggulan mendasar dari pendekatan ini terletak pada sifatnya yang independen terhadap resolusi: objek grafis vektor dapat diskalakan ke ukuran berapapun tanpa kehilangan ketajaman, karena representasi matematisnya tidak bergantung pada kerapatan piksel perangkat tampilan.

Namun demikian, keunggulan tersebut tidak hadir tanpa konsekuensi komputasional. Setiap jalur primitif harus melalui proses rasterisasi — yakni konversi dari representasi matematis kontinu ke representasi piksel diskrit — sebelum dapat ditampilkan di layar. Proses ini harus dilakukan untuk setiap jalur secara individual. Seiring meningkatnya kompleksitas adegan (scene), jumlah jalur bertambah secara signifikan, yang pada gilirannya meningkatkan biaya pemrosesan secara proporsional. Dalam konteks aplikasi modern seperti antarmuka pengguna grafis, visualisasi data interaktif, dan peta digital berskala besar, keterbatasan ini menjadi hambatan nyata bagi pencapaian performa rendering yang memadai.

Kompleksitas pemrosesan inilah yang mendorong komunitas riset grafis komputer untuk mengeksplorasi pendekatan komputasi paralel sebagai solusi. Komputasi paralel memungkinkan beban kerja didistribusikan ke sejumlah unit pemrosesan yang bekerja secara bersamaan, sehingga waktu total pemrosesan dapat dikurangi secara substansial. Dalam konteks rendering grafis vektor, Unit Pemrosesan Grafis (GPU) menjadi kandidat utama untuk akselerasi karena arsitekturnya yang secara eksplisit dirancang untuk mendukung operasi paralel masif pada ribuan thread secara simultan.

Tonggak penting dalam perkembangan rendering vektor paralel berbasis GPU diletakkan oleh Ganacim dkk. [2014], yang memperkenalkan pipeline rendering grafis vektor yang paralel secara masif. Pada pendekatan tersebut, tahap pra-pemrosesan (preprocessing) dan tahap rendering keduanya diparalelkan melalui struktur data yang dikenal sebagai shortcut tree, yang memungkinkan traversal hierarki geometri secara efisien di GPU. Selanjutnya, Li dkk. [2016] mengusulkan metode rendering jalur berbasis GPU menggunakan scanline rasterizer yang diparalelkan melalui konsep fragmen batas (boundary fragments) — unit pemrosesan berukuran 2×2 piksel yang memastikan setiap thread GPU menangani beban kerja berukuran tetap. Mekanisme ini menghasilkan distribusi beban yang sangat seimbang dan menjadi salah satu kontribusi algoritmik paling berpengaruh dalam bidang ini.

Konsep fragmen batas yang diperkenalkan oleh Li dkk. kemudian menginspirasi sejumlah implementasi engine rendering vektor modern. Di antaranya adalah Forma dari Google, Pathfinder dari proyek Servo, dan Vello — ketiganya merupakan engine rendering vektor yang beroperasi di atas compute pipeline GPU dan mengadopsi gagasan paralelisasi berbasis unit geometri berukuran tetap. Perkembangan ini menandai pergeseran paradigma yang signifikan: rendering vektor berkinerja tinggi kini dipandang hampir selalu memerlukan akses ke compute shader.

Compute shaders memungkinkan komputasi tujuan umum (general-purpose computation) yang secara tradisional merupakan ranah CPU untuk dilakukan langsung di GPU [Khronos Group, 2016]. Kemampuan ini membuka peluang paralelisme yang jauh lebih luas dibandingkan pipeline rasterisasi konvensional, karena thread shader dapat membaca dan menulis ke memori GPU secara arbitrer tanpa terikat pada alur vertex-to-fragment yang kaku. Sebagian besar pendekatan rendering vektor paralel mutakhir mengandalkan kemampuan ini sebagai fondasi arsitekturalnya.

Ketergantungan pada compute shaders, meski menghasilkan performa tinggi, menghadirkan keterbatasan kompatibilitas yang tidak dapat diabaikan dalam praktik. Dalam lingkungan berbasis web, akses ke compute shaders disediakan melalui WebGPU API — sebuah standar yang relatif baru dan belum memiliki dukungan stabil yang merata di seluruh peramban, perangkat, dan sistem operasi [W3C, 2025]. Pada perangkat low-end dan kelas konsumen yang mendominasi pasar global, keterbatasan perangkat keras sering kali mengakibatkan compute shader tidak tersedia atau beroperasi jauh di bawah kapasitas optimal. Kondisi ini menciptakan kesenjangan kompatibilitas yang nyata: engine rendering vektor modern berperforma tinggi tidak dapat dijalankan secara andal pada sebagian besar segmen perangkat yang ada saat ini.

Kesenjangan inilah yang menjadi motivasi utama penelitian ini. Penulis mengajukan rancangan pipeline rendering vektor paralel yang tidak bergantung pada compute shader dalam bentuk apapun, sehingga dapat beroperasi di seluruh lingkungan grafis yang hanya menyediakan rasterization pipeline konvensional. Pipeline yang diusulkan dibagi menjadi dua tahap utama: tahap pra-pemrosesan yang dieksekusi secara paralel di CPU, dan tahap rendering yang dieksekusi di GPU melalui vertex shader dan fragment shader tradisional. Dengan memindahkan seluruh komputasi tujuan umum — termasuk kalkulasi winding number, tiling geometri, dan penyusunan struktur data spasial — ke CPU secara paralel menggunakan multithreading, tahap rendering di GPU dapat sepenuhnya mengandalkan pipeline rasterisasi konvensional. Ketergantungan pada compute pipeline dengan demikian dieliminasi, sementara sifat paralelisme tetap dipertahankan di kedua sisi pipeline.

## 1.2 Rumusan Masalah

Berdasarkan latar belakang yang telah dipaparkan, penelitian ini dirancang untuk menjawab rumusan masalah sebagai berikut:

1. Bagaimana sebuah pipeline rendering grafis vektor paralel dapat dirancang tanpa bergantung pada compute shader, namun tetap mencapai tingkat paralelisme yang tinggi melalui pembagian beban kerja antara CPU dan GPU?
2. Sejauh mana tahap pra-pemrosesan grafis vektor dapat diparalelkan secara masif pada CPU, dan struktur data seperti apa yang paling efektif untuk mendukung paralelisasi tersebut?
3. Bagaimana beban kerja kedua tahapan dapat didistribusikan secara efektif antara CPU dan GPU agar pipeline tetap efisien tanpa mengandalkan compute-oriented pipeline?
4. Bagaimana dampak penghilangan compute shader terhadap latensi rendering dan throughput sistem, khususnya pada skenario beban kerja geometri vektor yang kompleks dan berskala besar?
5. Bagaimana perbandingan pendekatan yang diusulkan dengan metode berbasis compute shader maupun metode berbasis CPU murni, ditinjau dari aspek kompatibilitas, kompleksitas implementasi, dan performa?

## 1.3 Ruang Lingkup Penelitian

Agar penelitian ini tetap terarah dan hasilnya dapat diverifikasi secara ilmiah, penulis menetapkan batasan ruang lingkup sebagai berikut.

Dari sisi format data masukan, sistem menerima berkas grafis vektor dalam format SVG (Scalable Vector Graphics). Fitur yang didukung dibatasi pada path primitives dasar, meliputi kurva Bézier kuadratik dan kubik, garis lurus, dan poligon sederhana, beserta atribut fill dan stroke dasar. Fitur kompleks seperti filter effects (blur, drop shadow), gradien bertingkat, pola (pattern), dan animasi prosedural berada di luar cakupan penelitian ini.

Dari sisi arsitektur pipeline, tahap pra-pemrosesan pada CPU dirancang untuk memanfaatkan paralelisme — meliputi paralelisme tingkat instruksi melalui SIMD dan paralelisme tingkat data melalui pustaka multithreading Rust Rayon — untuk menangani proses flattening kurva, kalkulasi winding number, tiling geometri, dan penyusunan struktur data spasial. Tahap rendering pada GPU dibatasi secara ketat pada penggunaan rasterization pipeline tradisional yang terdiri dari vertex shader dan fragment shader. Penggunaan compute shader, geometry shader, tessellation shader, maupun fitur spesifik perangkat keras tidak akan diterapkan demi menjaga kompatibilitas lintas platform.

Dari sisi lingkungan pengembangan, purwarupa (prototype) dibangun menggunakan bahasa pemrograman Rust dan API grafis yang tidak mewajibkan fitur compute, yakni WebGL 2.0 yang diakses melalui kompilasi ke target WebAssembly. Pilihan ini memastikan bahwa hasil penelitian dapat direplikasi dan dijalankan pada perangkat yang tidak mendukung Vulkan maupun WebGPU.

Dari sisi pengujian dan evaluasi, pengukuran performa difokuskan pada dekomposisi waktu rendering per bingkai (frame time) menjadi biaya tahap pra-pemrosesan di CPU dan biaya tahap rasterisasi di GPU, sehingga kontribusi masing-masing tahap pada pipeline hibrida dapat dianalisis secara terpisah. Sebelum pengujian performa dilaksanakan, validasi kebenaran output (correctness validation) dilakukan terlebih dahulu dengan membandingkan hasil rendering purwarupa secara visual terhadap output renderer referensi pada sekumpulan berkas SVG uji yang telah ditentukan.

Dari sisi perbandingan, evaluasi dilakukan terhadap dua kelompok pembanding: renderer berbasis CPU murni seperti Cairo atau Skia sebagai baseline performa sekuensial, dan pendekatan berbasis compute shader seperti Vello sebagai baseline kualitatif untuk menilai trade-off antara kompatibilitas dan performa yang dihasilkan oleh pendekatan yang diusulkan.

## 1.4 Tujuan Penelitian

Tujuan utama penelitian ini adalah mendemonstrasikan kelayakan rendering vektor pada lingkungan grafis yang tidak menyediakan dukungan compute shader, dengan cara merancang dan mengevaluasi arsitektur pipeline hibrida CPU-GPU yang membagi beban kerja secara tegas antara kedua sisi pipeline. Secara lebih rinci, penelitian ini bertujuan untuk mencapai hal-hal berikut:

1. Merancang arsitektur pipeline rendering vektor hibrida yang mengeliminasi ketergantungan pada compute shader, dengan pembagian beban kerja yang jelas antara CPU sebagai unit pra-pemrosesan dan GPU sebagai unit rasterisasi yang terbatas pada vertex shader dan fragment shader konvensional.
2. Mewujudkan tahap pra-pemrosesan grafis vektor di CPU — meliputi flattening kurva, kalkulasi winding number melalui akumulator signed-area, dan penyusunan struktur data spasial berbasis ubin — sebagai komputasi tujuan umum yang memindahkan beban tersebut keluar dari GPU, sekaligus menyiapkan fondasi struktur data yang memungkinkan paralelisasi CPU pada pengembangan lanjutan.
3. Mengukur dan menganalisis performa purwarupa yang dihasilkan dengan mendekomposisi waktu rendering per bingkai (frame time) menjadi biaya tahap pra-pemrosesan di CPU dan biaya tahap rasterisasi di GPU, serta memvalidasi kebenaran output secara visual terhadap renderer referensi.
4. Mengidentifikasi dan mendokumentasikan trade-off yang muncul dari pendekatan non-compute dalam hal kompatibilitas platform, kompleksitas implementasi, dan karakteristik performa dibandingkan metode berbasis compute shader.

## 1.5 Manfaat Penelitian

Hasil penelitian ini diharapkan memberikan manfaat pada dua dimensi berikut.

Secara teoretis, penelitian ini memberikan kontribusi wawasan ilmiah mengenai strategi pembagian beban kerja antara CPU dan GPU dalam konteks rendering grafis 2D, khususnya pada skenario di mana akses ke compute pipeline tidak tersedia. Penelitian ini juga menjadi referensi dalam literatur grafis komputer untuk pendekatan rendering hibrida yang mencapai paralelisme tinggi melalui dekomposisi beban kerja lintas unit pemrosesan, bukan melalui compute shader.

Secara praktis, penelitian ini menyediakan landasan algoritmik untuk membangun mesin rendering vektor yang portabel dan dapat beroperasi di berbagai lingkungan, termasuk lingkungan web melalui WebGL 2.0 dan perangkat seluler kelas bawah yang belum mendukung WebGPU secara stabil. Bagi industri pengembangan perangkat lunak, penelitian ini membuka potensi untuk menampilkan grafis vektor yang kompleks di peramban maupun aplikasi mobile tanpa membatasi basis pengguna hanya pada pemilik perangkat high-end yang mendukung compute pipeline modern.

## 1.6 Metodologi Penelitian

### 1.6.1 Metode Pengumpulan Data

Pengumpulan data dan informasi pendukung penelitian dilakukan melalui tiga pendekatan berikut:

1. **Studi Pustaka:** Dilakukan analisis mendalam terhadap literatur ilmiah yang relevan, mencakup metode shortcut tree (Ganacim dkk., 2014), scanline rasterization berbasis fragmen batas (Li dkk., 2016), teknik rendering kurva berbasis implicit function seperti metode Loop-Blinn, serta karya-karya terkait pipeline rendering vektor modern. Studi pustaka juga mencakup dokumentasi teknis API grafis WebGL 2.0 dan spesifikasi format SVG.
2. **Observasi Data Masukan:** Dikumpulkan sekumpulan berkas SVG representatif yang mencakup variasi kompleksitas geometri, mulai dari bentuk sederhana dengan sedikit segmen kurva hingga adegan kompleks dengan ribuan jalur. Koleksi ini digunakan baik untuk pengembangan maupun evaluasi performa.
3. **Studi Dokumentasi Perangkat Lunak:** Dilakukan analisis terhadap dokumentasi API grafis WebGL 2.0 serta kemampuan multithreading bahasa pemrograman Rust untuk memahami batasan teknis yang berlaku dalam lingkungan tanpa compute shader.

### 1.6.2 Metode Perancangan dan Pengembangan

Pengembangan sistem mengikuti alur penelitian eksperimental yang terbagi dalam empat tahapan utama berikut:

1. **Analisis Kebutuhan:** Mengidentifikasi kebutuhan fungsional dan non-fungsional sistem, terutama terkait kompatibilitas pada lingkungan grafis tanpa compute shader, batasan API yang digunakan, dan karakteristik data masukan SVG yang akan ditangani.
2. **Perancangan Arsitektur Pipeline:** Merancang pembagian beban kerja hibrida secara rinci, di mana tahap pra-pemrosesan — meliputi flattening kurva Bézier menjadi segmen garis, kalkulasi winding number, tiling geometri, dan penyusunan struktur data spasial — dirancang untuk dieksekusi di CPU. Tahap rendering berbasis evaluasi cakupan piksel analitik dirancang untuk dieksekusi di GPU melalui vertex shader dan fragment shader konvensional.
3. **Implementasi Purwarupa:** Membangun purwarupa menggunakan bahasa pemrograman Rust dengan optimasi SIMD pada hot path pra-pemrosesan CPU, serta WebGL 2.0 sebagai antarmuka rasterization pipeline GPU melalui kompilasi ke target WebAssembly. Implementasi mencakup seluruh alur kerja dari pembacaan berkas SVG hingga output piksel akhir.
4. **Pengujian dan Evaluasi:** Tahap ini dibagi menjadi dua sub-tahap: pertama, validasi kebenaran output dengan membandingkan hasil rendering purwarupa terhadap renderer referensi secara visual pada sekumpulan berkas SVG uji; kedua, pengujian performa untuk mengukur frame time yang didekomposisi menjadi biaya CPU dan biaya GPU per bingkai, disertai analisis perbandingan kualitatif terhadap renderer referensi yang telah ditetapkan.
