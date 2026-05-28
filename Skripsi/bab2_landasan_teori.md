# BAB 2 TINJAUAN REFERENSI

## 2.1 Landasan Teoritis

### 2.1.1 Kurva Bezier

Kurva Bezier adalah garis lengkung yang halus yang didefinisikan oleh rumus matematika dan titik-titik kontrol. Kurva Bezier menggunakan polinomial Bernstein sebagai basis. Sebuah kurva bezier dengan derajat n (order n + 1) direpresentasikan sebagai:

$$r(t) = \sum_{i=0}^{n} b_i B_{i,n}(t), \quad 0 \leq t \leq 1$$

Koefisiennya, $b_i$, merepresentasikan titik kontrol atau titik bezier. Bersama dengan polinomial bernstein $B_{i,n}(t)$:

$$B_{i,n}(t) = \binom{n}{i} t^i (1-t)^{n-i}, \quad i = 0, \ldots, n$$

digunakan sebagai dasar untuk membangun kurva. Keduanya menentukan bentuk akhir kurva.

### 2.1.2 Fungsi Implisit

Fungsi implisit merupakan fungsi dengan banyak variabel, di mana salah satu variabelnya merupakan fungsi dari kumpulan variabel lainnya. Berbeda dengan fungsi eksplisit yang memiliki bentuk umum, fungsi implisit menyajikan interaksi antar variabel dalam satu ruas persamaan, yang secara umum direpresentasikan sebagai:

$$F(x, y) = 0$$

### 2.1.3 Winding Number

Winding number merupakan konsep matematika untuk menyatakan berapa kali sebuah kurva tertutup melintasi titik acuan tertentu. Dalam sistem koordinat dua dimensi, nilai winding number ($\omega$) ditentukan oleh orientasi atau arah lintasan kurva:

1. **Counter-Clockwise (CCW):** Lintasan yang berlawanan dengan arah jarum jam menghasilkan nilai positif ($\omega > 0$).
2. **Clockwise (CW):** Lintasan yang searah dengan arah jarum jam menghasilkan nilai negatif ($\omega < 0$).

Dalam bidang komputer grafis, aplikasi winding number digunakan sebagai alat penentu area dalam atau interior pada objek vektor yang kompleks. Konsep winding number juga menjadi fondasi bagi berbagai algoritma dengan tujuan yang serupa.

### 2.1.4 Point-In-Polygon

Point-in-polygon adalah algoritma dalam komputasi geometri untuk menentukan apakah sebuah titik (koordinat x dan y) berada di dalam, luar, atau pada batas poligon. Algoritma ini banyak digunakan dalam berbagai bidang, seperti grafik komputer, pemrosesan vektor, sistem informasi geografis (GIS), dan simulasi fisika.

Beberapa metode umum yang digunakan untuk menentukan posisi titik relatif terhadap poligon antara lain:

1. **Ray Casting Algorithm** — Metode ini bekerja dengan menghitung berapa kali garis (ray) dari titik yang ingin diuji ke arah tertentu, misalnya horizontal ke kanan, mengenai sisi-sisi poligon. Jika jumlah interseksi ganjil, titik berada di dalam poligon; jika genap, titik berada di luar poligon.
2. **Winding Number Algorithm** — Seperti yang dijelaskan di subbab sebelumnya, metode ini bekerja dengan mengevaluasi arah/orientasi setiap sisi poligon relatif terhadap titik. Hasil evaluasi menghasilkan angka yang disebut winding number. Jika hasilnya nol, titik berada di luar poligon. Jika bukan nol, titik berada di dalam poligon. Metode ini memiliki keunggulan dalam menangani poligon yang kompleks atau berlapis (self-intersecting polygons) dengan lebih tepat dibanding ray casting.

### 2.1.5 Komputasi Parallel

Komputasi paralel, atau yang juga dikenal sebagai pemrograman paralel, adalah proses di mana masalah komputasi besar dipecah menjadi masalah-masalah kecil yang dapat diselesaikan secara bersamaan oleh beberapa prosesor [IBM, 2022]. Konsep ini memanfaatkan kemampuan prosesor modern, baik CPU multi-core maupun GPU, untuk mempercepat perhitungan yang bersifat independen atau memiliki pola data yang dapat dibagi. Dalam konteks grafis komputer, komputasi paralel sangat berguna untuk melakukan perhitungan vertex, fragment, atau operasi matematis kompleks seperti rasterisasi, evaluasi kurva, dan simulasi fisika secara efisien.

## 2.2 Tinjauan Pustaka Terdahulu

### 2.2.1 Loop-Blinn (Resolution Independent Curve Rendering)

Loop dan Blinn (2005) mengusulkan metode rendering kurva vektor yang bersifat resolution independent dengan memanfaatkan kemampuan pemrograman GPU modern. Inti kontribusinya adalah pengamatan bahwa setiap kurva kuadratik rasional merupakan citra proyeksi dari satu kurva kanonik tunggal yang memiliki persamaan implisit sederhana $f(u,v) = u^2 - v$. Dengan memetakan titik-titik kontrol Bézier sebagai texture coordinates pada segitiga yang melingkupi kurva, GPU dapat mengevaluasi persamaan implisit tersebut di setiap piksel melalui fragment shader — tanpa memerlukan tessellation rapat yang menghasilkan artefak sampling.

Untuk kurva kubik, Loop dan Blinn memperluas pendekatan ini dengan mengklasifikasikan setiap kurva ke dalam tiga kasus kanonik — serpentine, loop, dan cusp — berdasarkan diskriminan polinomial infleksinya. Masing-masing kasus memiliki persamaan implisit homogen dalam bentuk $c(x,y,w) = k^3 - lmn$, di mana $k$, $l$, $m$, $n$ merupakan fungsional linear yang nilainya diinterpolasi oleh GPU secara otomatis per-piksel.

Seluruh preprocessing — triangulasi Delaunay berkendala, deteksi overlap antar segitiga, klasifikasi tipe kurva, dan assignment texture coordinate — dilakukan di CPU sebagai tahap pra-komputasi satu kali. Setelah data geometri diunggah ke GPU, rendering setiap frame hanya memerlukan transformasi matriks baru tanpa keterlibatan CPU lebih lanjut. Shader yang dihasilkan sangat ringkas; Loop dan Blinn sendiri menyebutkan bahwa pixel shader kubik hanya memerlukan evaluasi satu ekspresi $k^3 - lmn$ untuk setiap piksel.

Dari perspektif penelitian ini, Loop-Blinn merupakan titik acuan yang penting namun berlawanan arah. Pendekatan mereka memindahkan evaluasi kurva ke GPU melalui fragment shader, sedangkan penelitian ini memindahkan komputasi dari GPU ke CPU secara paralel. Lebih lanjut, Loop dan Blinn sendiri mengakui dalam bagian diskusi papernya bahwa triangulasi global dan penghindaran overlap bersifat inheren sekuensial dan tidak cocok untuk dieksekusi di GPU — sebuah pengakuan yang secara langsung memperkuat motivasi penelitian ini untuk menangani komputasi irregular tersebut di CPU melalui multithreading.

### 2.2.2 Metode Triangulasi Kombinasi (Resolution Independent Rendering of Deformable Vector Objects)

Kokojima dkk. (2006) mengembangkan metode Loop-Blinn dengan mengatasi dua keterbatasan praktisnya: biaya re-triangulasi pada geometri yang berubah secara dinamis, dan ketidakmampuan menangani self-intersection secara andal. Pendekatan mereka menggabungkan dua jenis geometri: line-edged triangle fans yang dibentuk dari pivot point arbitrer dan titik-titik kontur, serta curve-edged triangles yang dibentuk dari titik-titik kontrol Bézier.

Mekanisme penentuan interior menggunakan stencil buffer GPU dengan operator bitwise-inversion `GL_INVERT`. Triangle fans digambar ke stencil buffer sehingga piksel yang tertutup jumlah ganjil kali memiliki nilai stencil nonzero — sebuah implementasi langsung dari even-odd fill rule berbasis winding. Setelah itu, convex region dari curve-edged triangles dikoreksi menggunakan evaluasi fungsi implisit Loop-Blinn di pixel shader, mengubah isi stencil buffer dari representasi ganjil-genap menjadi representasi interior kurva yang akurat.

Keunggulan utama pendekatan ini dibanding Loop-Blinn adalah eliminasi preprocessing CPU yang mahal: tidak ada triangulasi Delaunay berkendala, tidak ada subdivisi overlap, dan tidak ada ketergantungan pada urutan gambar objek karena transparency multisampling digunakan sebagai pengganti alpha blending. Hasilnya, objek vektor yang terdeformasi secara dinamis dapat dirender tanpa intervensi CPU per-frame. Kokojima dkk. melaporkan rendering lebih dari tiga ratus karakter TrueType yang berdeformasi dinamis pada sekitar 40 fps, lebih dari sepuluh kali lebih cepat dibandingkan metode Loop-Blinn pada kondisi yang sama.

Dalam konteks penelitian ini, Kokojima dkk. menunjukkan bahwa penggunaan cerdas stencil buffer — fitur standar rasterization pipeline yang tersedia di semua GPU — dapat menggantikan sebagian peran compute shader untuk operasi penentuan interior. Prinsip ini relevan dengan pendekatan penelitian ini yang membatasi GPU pada rasterization pipeline konvensional. Namun berbeda dengan Kokojima dkk. yang masih bergantung pada evaluasi Loop-Blinn di fragment shader per-piksel, penelitian ini memindahkan kalkulasi winding number sepenuhnya ke CPU, sehingga fragment shader dapat disederhanakan menjadi lookup ke struktur data yang telah disiapkan.

### 2.2.3 Skia

Skia adalah pustaka grafis 2D sumber terbuka yang dikembangkan oleh Google dan berfungsi sebagai engine rendering untuk Chrome, Android, Flutter, dan berbagai produk Google lainnya. Secara arsitektural, Skia mengadopsi model berbasis CPU dengan akselerasi GPU opsional melalui backend yang dapat dikonfigurasi.

Representasi geometri inti Skia menggunakan struktur `SkPath` yang bersifat immutable setelah dibentuk, dengan konstruksi dilakukan melalui `SkPathBuilder` menggunakan builder pattern. Setiap path terdiri dari urutan verb (`kMove`, `kLine`, `kQuad`, `kConic`, `kCubic`, `kClose`) beserta array titik yang bersesuaian. Skia mendukung kurva kuadratik, kubik, dan conic section dalam satu representasi terpadu, dengan properti seperti bounding box dan convexity dihitung secara lazy dan di-cache.

Untuk rendering, jalur vektor di Skia melalui tahap flattening (konversi kurva menjadi segmen garis dengan toleransi adaptif), diikuti oleh rasterisasi berbasis scanline di CPU. Optimasi SIMD diterapkan pada operasi-operasi kritis untuk meningkatkan throughput. Backend GPU Skia generasi terbaru — Graphite — memperkenalkan pipeline berbasis Vulkan dan Metal dengan kemampuan rendering GPU yang lebih luas, namun pipeline CPU-nya tetap menjadi jalur utama untuk kompatibilitas lintas platform.

Keterbatasan fundamental Skia dari perspektif penelitian ini adalah bahwa pipeline renderingnya per-frame bersifat sekuensial: meskipun Skia sangat dioptimalkan, proses flattening dan rasterisasi scanline tidak diparalelkan secara agresif menggunakan seluruh core CPU yang tersedia. Skia berfungsi sebagai baseline perbandingan dalam penelitian ini: ia merepresentasikan praktik terbaik renderer berbasis CPU yang mapan dan telah teruji secara produksi, sehingga menjadi tolok ukur yang tepat untuk mengukur percepatan yang dicapai oleh paralelisasi CPU yang diusulkan.

### 2.2.4 Cairo

Cairo merupakan pustaka rendering grafis vektor dua dimensi berbasis sumber terbuka yang secara arsitektural beroperasi penuh di atas Central Processing Unit (CPU). Cairo mengadopsi pendekatan rendering konvensional yang sangat bergantung pada rasterisasi poligon dasar. Dalam alur kerjanya, representasi matematis dari kurva yang kompleks (seperti kurva Bézier) akan dievaluasi dan diratakan (flattening) terlebih dahulu menjadi serangkaian pecahan garis lurus (line segments).

Kumpulan garis lurus tersebut kemudian digunakan sebagai basis untuk metode triangulasi, di mana kalkulasi CPU akan merekonstruksi bentuk vektor asli menjadi jaring poligon yang terdiri dari bangun segitiga. Setelah jaring segitiga ini terbentuk dan area interiornya terdefinisi, tahapan selanjutnya adalah mengeksekusi proses filling di mana segitiga tersebut dirender dan diberikan warna.

Meskipun metode rendering ini mampu menghasilkan output visual yang presisi, ketergantungan absolut pada pemrosesan sekuensial CPU menimbulkan bottleneck performa. Pada skenario objek vektor skala besar, biaya komputasi untuk proses pemecahan kurva dan pembentukan jaring segitiga secara dinamis akan sangat membebani CPU, sehingga tidak seefisien metode rendering yang memanfaatkan komputasi paralel.

### 2.2.5 Slug (Vector Texture)

Slug, yang dikembangkan oleh Eric Lengyel, merupakan metode perenderan grafis vektor dan tipografi berbasis GPU yang beroperasi sepenuhnya di dalam tahap fragment shader tanpa memerlukan tessellation geometri poligon atau dukungan compute shader. Pendekatan inti Slug adalah mengemas data titik kontrol kurva ke dalam struktur data tekstur khusus yang disebut vector texture. Pada tahap rendering, GPU cukup menggambar kotak pembatas (bounding box) kasar untuk setiap objek vektor. Selanjutnya, fragment shader akan melakukan pengambilan data (texture fetch) dari vector texture untuk mengevaluasi posisi piksel relatif terhadap kurva secara analitis.

Metode ini memanfaatkan matematika aljabar proyektif untuk mengevaluasi jarak presisi dan cakupan anti-aliasing langsung pada tingkat piksel. Keunggulan utama Slug adalah independensi resolusinya yang mutlak dan kemampuannya berjalan pada pipeline rasterisasi konvensional. Namun, dalam konteks penelitian ini, Slug memiliki keterbatasan praktis berupa beban komputasi fragment shader yang sangat berat. Pada skenario di mana banyak kurva bertumpuk (overlapping), setiap piksel harus secara berulang mengevaluasi matriks kurva yang melingkupinya, sehingga berpotensi menciptakan bottleneck performa pada GPU kelas bawah (low-end).

### 2.2.6 Pathfinder (Tiling/Ubin)

Pathfinder adalah pustaka rendering grafis vektor berkinerja tinggi yang pada awalnya dikembangkan oleh Mozilla untuk mesin peramban Servo. Arsitektur Pathfinder memperkenalkan pergeseran paradigma yang berpusat pada konsep partisi spasial berbasis ubin (tiling). Alih-alih merender keseluruhan bentuk secara global yang rentan terhadap inefisiensi, Pathfinder memecah area kanvas layar menjadi kisi-kisi ubin berukuran tetap (umumnya $16 \times 16$ piksel).

Pada iterasi awalnya, Pathfinder melakukan tahap pra-pemrosesan di CPU untuk mengklasifikasikan ubin ke dalam tiga kategori: ubin kosong, ubin penuh, dan ubin yang memotong tepi kurva (edge tiles). Data klasifikasi ini kemudian dikirimkan ke GPU untuk dirasterisasi. Mekanisme pemisahan spasial ini sangat sejalan dengan arsitektur hibrida yang diusulkan dalam penelitian ini. Walaupun pada versi-versi terbarunya Pathfinder bertransisi menggunakan compute shader untuk memparalelkan proses partisi tersebut secara penuh di GPU, iterasi klasiknya membuktikan bahwa dekopel (decoupling) antara beban kerja ubin tepi yang kompleks dan ubin interior yang dirender secara instan efektif meminimalkan overdraw di GPU, sebuah prinsip yang diadopsi langsung dalam algoritma usulan penelitian ini.

### 2.2.7 RAVG (Random Access Vector Graphics)

Random-Access Vector Graphics (RAVG), yang pertama kali dipelopori oleh Nehab dan Hoppe (2008), adalah algoritma yang memungkinkan evaluasi grafis vektor pada sembarang koordinat piksel secara acak (random access) tanpa terikat pada urutan rendering pelukis tradisional (order-independent). Algoritma ini mencapainya dengan mengonversi grafis vektor kompleks ke dalam dua struktur data tekstur dua dimensi: tekstur sel yang memetakan indeks partisi spasial, dan tekstur primitif yang menyimpan atribut serta geometri kurva.

Ketika dieksekusi, fragment shader merujuk pada tekstur sel untuk mengidentifikasi kurva mana saja yang memotong koordinat suatu piksel, lalu mengambil data geometrinya untuk menghitung winding number analitis dan melakukan pewarnaan. Pendekatan RAVG berhasil mengeleminasi kebutuhan compute shader dan sangat ideal untuk skenario navigasi spasial seperti panning ekstrem. Namun, kelemahan mendasar RAVG terletak pada tingginya biaya pra-komputasi untuk membangun struktur data tekstur tersebut, serta kebutuhan bandwidth memori GPU yang intensif untuk membaca dua lapisan tekstur secara konstan. Hal inilah yang menjadi motivasi penelitian ini untuk memindahkan beban pembangunan struktur spasial ke CPU menggunakan paralelisasi masif, sehingga GPU menerima struktur data linear (vertex buffer) yang sudah matang tanpa perlu melakukan pencarian tekstur (texture lookup) yang berlebihan.

### 2.2.8 Vello

Vello adalah mesin rendering dua dimensi eksperimental berbasis bahasa pemrograman Rust yang dikembangkan oleh komunitas open-source Linebender. Berbeda secara diametral dengan pustaka grafis tradisional yang bergantung pada logika sekuensial CPU, Vello mendefinisikan dirinya melalui arsitektur GPU compute-centric. Pendekatan ini memindahkan hampir seluruh beban kerja rendering yang berat — seperti path tessellation, pengurutan spasial (sorting), dan pemotongan (clipping) — secara langsung ke GPU dengan memanfaatkan kapabilitas compute shaders.

Kinerja masif Vello dicapai melalui penerapan algoritma parallel prefix-sum (atau scan) pada grafis vektor. Rendering vektor secara tradisional sulit diparalelkan karena operasi seperti penentuan winding number atau pengelolaan stack untuk clipping memiliki kompleksitas $O(n)$ yang bersifat sekuensial. Vello mengatasi hambatan tersebut menggunakan prefix sums untuk mengubah tugas serial menjadi tugas paralel dengan kompleksitas $O(\log n)$ yang dapat dieksekusi oleh ribuan thread GPU secara bersamaan. Pipeline rendering pada Vello dieksekusi melalui serangkaian compute shader dispatches yang terstruktur dalam empat tahap krusial:

1. **Scene Ingestion:** CPU mengompresi (encoding) data adegan ke dalam format biner yang ringkas, lalu mendelegasikan GPU untuk menangani transformasi affine guna menjaga ketajaman resolusi.
2. **Mega Stage:** Tahap ini melakukan perataan (flattening) kurva Bézier menjadi segmen garis dan menangani perluasan goresan (stroke expansion) menggunakan algoritma Euler spirals untuk menjamin akurasi kurva tingkat tinggi.
3. **Coarse Rasterization (Binning):** Vello menggunakan arsitektur berbasis ubin (tiling), mempartisi layar menjadi kisi-kisi (umumnya 16×16 piksel). Sistem mendeteksi objek mana yang memotong setiap ubin dan menggunakan operasi atomik perangkat keras untuk mengelola memori segmen.
4. **Fine Rasterization:** Tahap akhir yang menghitung cakupan piksel (pixel coverage) secara analitik untuk keperluan anti-aliasing. Proses komposisi warna (blending) dikerjakan langsung di dalam compute shader, yang secara efektif menghindari penurunan performa akibat penggunaan multisampling (MSAA) yang berat.

Sebagai solusi fallback untuk perangkat tanpa dukungan GPU yang memadai, Vello juga menyediakan Vello CPU. Varian ini menggunakan representasi perantara bernama sparse strips dan optimasi instruksi SIMD (Single Instruction, Multiple Data) untuk memproses ubin berukuran 4×4 piksel secara independen.
