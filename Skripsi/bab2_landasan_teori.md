# BAB 2 TINJAUAN REFERENSI

## 2.1 Landasan Teoritis

### 2.1.1 Polinomial Bernstein

Polinomial Bernstein adalah keluarga polinomial basis yang menjadi fondasi matematis bagi representasi kurva Bézier. Untuk derajat $n$, terdapat $n + 1$ polinomial Bernstein yang didefinisikan sebagai (Farin, 2002):

$$B_{i,n}(t) = \binom{n}{i} t^i (1-t)^{n-i}, \quad i = 0, 1, \ldots, n, \quad 0 \leq t \leq 1$$

dengan $\binom{n}{i}$ menyatakan koefisien binomial. Polinomial Bernstein memiliki sejumlah sifat penting yang menjadikannya basis ideal untuk desain geometri. Pertama, sifat non-negativitas: setiap $B_{i,n}(t) \geq 0$ pada selang $[0, 1]$. Kedua, sifat partisi kesatuan (partition of unity), yaitu jumlah seluruh polinomial basis pada sembarang nilai $t$ selalu sama dengan satu:

$$\sum_{i=0}^{n} B_{i,n}(t) = 1$$

Kombinasi kedua sifat ini menjamin bahwa setiap titik pada kurva merupakan kombinasi konveks (convex combination) dari titik-titik kontrolnya, sehingga kurva selalu berada di dalam convex hull titik-titik kontrol. Sifat-sifat inilah yang membuat polinomial Bernstein menjadi dasar yang stabil secara numerik dan intuitif secara geometris bagi konstruksi kurva Bézier (Farin, 2002).

### 2.1.2 Kurva Bézier

Kurva Bézier adalah garis lengkung yang halus yang didefinisikan oleh rumus matematika dan titik-titik kontrol. Kurva Bézier menggunakan polinomial Bernstein (Subbab 2.1.1) sebagai basis. Sebuah kurva Bézier dengan derajat $n$ (order $n + 1$) direpresentasikan sebagai (Farin, 2002):

$$r(t) = \sum_{i=0}^{n} b_i B_{i,n}(t), \quad 0 \leq t \leq 1$$

Koefisiennya, $b_i$, merepresentasikan titik kontrol atau titik Bézier, sedangkan $B_{i,n}(t)$ adalah polinomial basis Bernstein yang menentukan kontribusi setiap titik kontrol terhadap bentuk kurva. Keduanya secara bersama-sama menentukan bentuk akhir kurva. Dalam praktik grafis komputer, kurva Bézier kuadratik ($n = 2$) dan kubik ($n = 3$) paling lazim dipakai karena keseimbangan antara fleksibilitas bentuk dan kesederhanaan komputasi. Kurva Bézier memiliki sifat-sifat yang diwarisi dari basis Bernstein, antara lain selalu melewati titik kontrol pertama dan terakhir, bersinggungan dengan poligon kontrol pada kedua ujungnya, serta seluruhnya termuat di dalam convex hull titik-titik kontrolnya (Farin, 2002).

### 2.1.3 Fungsi Implisit

Fungsi implisit merupakan fungsi dengan banyak variabel, di mana salah satu variabelnya merupakan fungsi dari kumpulan variabel lainnya. Berbeda dengan fungsi eksplisit yang memiliki bentuk umum, fungsi implisit menyajikan interaksi antar variabel dalam satu ruas persamaan, yang secara umum direpresentasikan sebagai:

$$F(x, y) = 0$$

### 2.1.4 Winding Number

Winding number merupakan konsep matematika untuk menyatakan berapa kali sebuah kurva tertutup melintasi titik acuan tertentu. Dalam sistem koordinat dua dimensi, nilai winding number ($\omega$) ditentukan oleh orientasi atau arah lintasan kurva:

1. **Counter-Clockwise (CCW):** Lintasan yang berlawanan dengan arah jarum jam menghasilkan nilai positif ($\omega > 0$).
2. **Clockwise (CW):** Lintasan yang searah dengan arah jarum jam menghasilkan nilai negatif ($\omega < 0$).

Dalam bidang komputer grafis, aplikasi winding number digunakan sebagai alat penentu area dalam atau interior pada objek vektor yang kompleks. Konsep winding number juga menjadi fondasi bagi berbagai algoritma dengan tujuan yang serupa.

### 2.1.5 Teorema Kurva Jordan (Jordan Curve Theorem)

Teorema Kurva Jordan (Jordan Curve Theorem) adalah teorema fundamental dalam topologi yang pertama kali dirumuskan oleh Camille Jordan pada tahun 1887. Teorema ini menyatakan bahwa setiap kurva Jordan — yaitu kurva tertutup sederhana pada bidang (kurva kontinu yang tidak memotong dirinya sendiri) — membagi bidang menjadi tepat dua daerah yang saling terpisah: sebuah daerah interior yang terbatas (bounded) dan dibatasi oleh kurva, serta sebuah daerah eksterior yang tak terbatas (unbounded). Setiap lintasan kontinu yang menghubungkan sebuah titik di daerah interior dengan sebuah titik di daerah eksterior pasti memotong kurva tersebut (Hales, 2007).

Meskipun pernyataannya tampak jelas secara intuitif, pembuktian formalnya ternyata sangat tidak sepele dan menjadi tonggak penting dalam perkembangan matematika yang rigorous (Hales, 2007). Dalam konteks grafis komputer, Teorema Kurva Jordan memberikan landasan teoritis bagi konsep "dalam" dan "luar" sebuah bentuk vektor tertutup. Justru karena setiap kurva tertutup sederhana menjamin adanya pembagian interior–eksterior yang terdefinisi dengan baik, algoritma penentuan keterisian seperti point-in-polygon (Subbab 2.1.6) dan winding number (Subbab 2.1.4) memiliki dasar matematis yang sahih untuk memutuskan apakah suatu piksel berada di dalam atau di luar bentuk yang akan dirender.

### 2.1.6 Point-In-Polygon

Point-in-polygon adalah algoritma dalam komputasi geometri untuk menentukan apakah sebuah titik (koordinat x dan y) berada di dalam, luar, atau pada batas poligon. Algoritma ini banyak digunakan dalam berbagai bidang, seperti grafik komputer, pemrosesan vektor, sistem informasi geografis (GIS), dan simulasi fisika.

Beberapa metode umum yang digunakan untuk menentukan posisi titik relatif terhadap poligon antara lain:

1. **Ray Casting Algorithm** — Metode ini bekerja dengan menghitung berapa kali garis (ray) dari titik yang ingin diuji ke arah tertentu, misalnya horizontal ke kanan, mengenai sisi-sisi poligon. Jika jumlah interseksi ganjil, titik berada di dalam poligon; jika genap, titik berada di luar poligon.
2. **Winding Number Algorithm** — Seperti yang dijelaskan di subbab sebelumnya, metode ini bekerja dengan mengevaluasi arah/orientasi setiap sisi poligon relatif terhadap titik. Hasil evaluasi menghasilkan angka yang disebut winding number. Jika hasilnya nol, titik berada di luar poligon. Jika bukan nol, titik berada di dalam poligon. Metode ini memiliki keunggulan dalam menangani poligon yang kompleks atau berlapis (self-intersecting polygons) dengan lebih tepat dibanding ray casting.

### 2.1.7 Komputasi Parallel

Komputasi paralel, atau yang juga dikenal sebagai pemrograman paralel, adalah proses di mana masalah komputasi besar dipecah menjadi masalah-masalah kecil yang dapat diselesaikan secara bersamaan oleh beberapa prosesor [IBM, 2022]. Konsep ini memanfaatkan kemampuan prosesor modern, baik CPU multi-core maupun GPU, untuk mempercepat perhitungan yang bersifat independen atau memiliki pola data yang dapat dibagi. Dalam konteks grafis komputer, komputasi paralel sangat berguna untuk melakukan perhitungan vertex, fragment, atau operasi matematis kompleks seperti rasterisasi, evaluasi kurva, dan simulasi fisika secara efisien.

### 2.1.8 Flattening Kurva melalui Subdivisi Adaptif

Flattening adalah proses konversi kurva parametrik halus, seperti kurva Bézier, menjadi rangkaian segmen garis lurus (polyline) yang mendekati bentuk kurva asli dalam batas toleransi galat tertentu. Proses ini diperlukan karena sebagian besar perangkat rasterisasi bekerja jauh lebih efisien pada primitif garis lurus dibanding pada kurva derajat tinggi. Tantangan utamanya adalah memilih jumlah segmen yang memadai: terlalu sedikit menghasilkan tepi yang tampak bersudut (faceting), sedangkan terlalu banyak memboroskan komputasi tanpa peningkatan kualitas visual yang berarti.

Pendekatan yang lazim dipakai adalah subdivisi titik tengah (midpoint subdivision) berbasis algoritma De Casteljau. Pada algoritma De Casteljau, sebuah kurva Bézier dapat dibelah pada parameter $t = 0{,}5$ menjadi dua subkurva Bézier yang, jika digabungkan, identik dengan kurva asli. Subdivisi diterapkan secara rekursif: pada setiap langkah, kedataran (flatness) subkurva diuji melalui ukuran simpangan titik kontrol terhadap tali busur (chord) yang menghubungkan titik awal dan titik akhir. Apabila simpangan tersebut sudah berada di bawah ambang toleransi, subkurva diemisikan sebagai satu segmen garis; jika belum, subkurva dibelah lagi dan kedua paruhnya diproses secara rekursif [Farin, 2002]. Sifat adaptif inilah yang menjadikan subdivisi titik tengah efisien — wilayah kurva yang nyaris lurus menghasilkan sedikit segmen, sedangkan wilayah berkelengkungan tinggi menerima subdivisi lebih rapat.

Selain subdivisi langsung pada kurva kubik, sejumlah penelitian mutakhir terlebih dahulu mengonversi kurva kubik menjadi rangkaian kurva kuadratik sebelum di-flatten. Kurva kuadratik hanya memiliki satu titik kontrol sehingga uji kedataran dan pembelahannya lebih murah secara komputasi, sementara himpunan kurva kuadratik tetap mampu mendekati kurva kubik pada toleransi yang dikehendaki. Strategi dua tahap — kubik ke kuadratik, lalu kuadratik ke segmen garis — banyak diadopsi pada renderer vektor modern karena menyederhanakan jalur evaluasi tanpa mengorbankan akurasi.

### 2.1.9 Representasi Bilangan Fixed-Point

Fixed-point adalah skema representasi bilangan pecahan menggunakan tipe data bilangan bulat (integer), di mana sejumlah bit dialokasikan untuk bagian pecahan secara tetap. Sebuah format fixed-point dinotasikan sebagai $Q_{m.n}$, dengan $m$ bit untuk bagian bilangan bulat dan $n$ bit untuk bagian pecahan, sehingga sebuah nilai riil $x$ direpresentasikan sebagai bilangan bulat $\lfloor x \cdot 2^{n} \rceil$. Sebagai contoh, format 24.8 menyimpan nilai pada bilangan bulat 32-bit dengan delapan bit pecahan, sehingga satu satuan piksel setara dengan $2^{8} = 256$ unit fixed-point, sedangkan format 8.8 menyimpan nilai pada bilangan bulat 16-bit dengan delapan bit pecahan.

Dibandingkan dengan bilangan floating-point, fixed-point menawarkan dua keunggulan penting dalam konteks rasterisasi. Pertama, operasi fixed-point bersifat deterministik: hasil penjumlahan dan perkalian tidak bergantung pada mode pembulatan floating-point yang dapat berbeda antar perangkat keras, sehingga geometri yang sama menghasilkan keluaran piksel yang identik di berbagai platform. Kedua, fixed-point menjaga presisi yang seragam pada rentang koordinat layar, berbeda dengan floating-point yang presisinya menurun seiring membesarnya magnitudo nilai. Karena itu, format fixed-point seperti 24.8 lazim dipakai sebagai representasi koordinat internal pada pustaka rasterisasi vektor untuk menjamin konsistensi dan kecepatan komputasi.

### 2.1.10 Partisi Spasial Berbasis Ubin (Tiling)

Tiling adalah teknik partisi spasial yang membagi bidang gambar (kanvas) menjadi kisi-kisi sel persegi panjang berukuran tetap yang disebut ubin (tile). Setiap primitif geometri kemudian diasosiasikan hanya dengan ubin-ubin yang benar-benar dilintasinya, sehingga komputasi rasterisasi dapat dilokalisasi per ubin alih-alih diterapkan pada keseluruhan kanvas secara global. Pendekatan ini menjadi fondasi banyak arsitektur rendering modern karena beberapa alasan: ia membatasi cakupan kerja setiap unit pemrosesan pada wilayah kecil yang dapat ditampung dalam memori cache, memungkinkan ubin yang tidak tersentuh geometri dilewati sepenuhnya, dan menjadikan setiap ubin sebagai unit kerja independen yang ideal untuk dieksekusi secara paralel.

Dalam praktik rendering vektor, ubin lazim diklasifikasikan secara konseptual berdasarkan relasinya terhadap geometri: ubin kosong yang tidak dilintasi bentuk apa pun, ubin interior yang sepenuhnya berada di dalam bentuk, dan ubin tepi (edge tile) yang dipotong oleh batas bentuk. Pemisahan ini memungkinkan ubin interior diisi warna secara langsung tanpa evaluasi geometri, sementara hanya ubin tepi yang memerlukan perhitungan cakupan yang lebih mahal. Ukuran ubin merupakan parameter desain yang menyeimbangkan dua hal yang bersaing: ubin yang lebih besar mengurangi overhead manajemen per ubin namun memperbesar potensi komputasi sia-sia (overdraw), sedangkan ubin yang lebih kecil melokalisasi kerja lebih ketat namun menambah jumlah ubin yang harus dikelola.

### 2.1.11 Algoritma Digital Differential Analyzer (DDA)

Digital Differential Analyzer (DDA) adalah algoritma inkremental untuk menentukan sel-sel diskrit yang dilintasi sebuah garis lurus pada kisi reguler. Diberikan sebuah segmen garis dengan titik awal $(x_0, y_0)$ dan titik akhir $(x_1, y_1)$, DDA menelusuri garis tersebut dengan menambahkan kenaikan (increment) konstan pada satu sumbu dan kenaikan proporsional pada sumbu lainnya, sehingga setiap perpotongan garis dengan batas sel dapat dihitung secara berurutan tanpa operasi perkalian atau pembagian berulang pada loop inti. Varian DDA berbasis bilangan bulat, yang berkerabat dengan algoritma garis Bresenham, mempertahankan akumulator galat (error accumulator) untuk memutuskan kapan penelusuran berpindah sel, sehingga seluruh komputasi dapat dilakukan dalam aritmetika integer yang cepat dan bebas galat pembulatan.

Dalam konteks partisi spasial berbasis ubin, DDA dipakai untuk proses binning, yaitu memetakan setiap segmen garis ke daftar ubin yang dilintasinya beserta titik perpotongan garis pada batas tiap ubin. Penelusuran semacam ini dapat disusun secara bertingkat: tahap pertama (outer DDA) menelusuri perpindahan garis melintasi baris-baris ubin secara vertikal, sedangkan tahap kedua (inner DDA) menelusuri perpindahan melintasi kolom-kolom ubin di dalam satu baris. Skema dua tingkat ini memecah sebuah segmen garis panjang menjadi potongan-potongan terklip per ubin secara efisien, yang kemudian menjadi masukan bagi tahap perhitungan cakupan dan winding number.

### 2.1.12 Akumulator Signed-Area untuk Winding Number

Perhitungan winding number sebagaimana diuraikan pada Subbab 2.1.4 dapat direalisasikan secara efisien melalui akumulator signed-area (signed-area accumulator), sebuah teknik yang menjadi inti banyak rasterizer analitik seperti pada FreeType (The FreeType Project, 2023) dan Skia (The Skia Project, 2023). Alih-alih menghitung perpotongan sinar secara eksplisit untuk setiap piksel, metode ini mengakumulasi kontribusi luas bertanda (signed area) dari setiap segmen garis terhadap baris piksel (scanline) yang dilintasinya. Tanda kontribusi ditentukan oleh arah vertikal segmen — segmen yang bergerak menurun menyumbang nilai berlawanan tanda dengan segmen yang bergerak menaik — sehingga akumulasi tanda tersebut secara langsung mencerminkan nilai winding number sesuai orientasi lintasan kurva.

Pada pendekatan per-scanline, setiap segmen garis didistribusikan ke baris-baris piksel yang dipotongnya, dan luas pertindihan vertikal segmen pada tiap baris ditambahkan ke akumulator baris yang bersangkutan. Karena luas bersifat kontinu, nilai cakupan yang dihasilkan secara alami memberikan antialiasing pada tepi bentuk tanpa memerlukan supersampling. Untuk efisiensi memori, kontribusi luas ini sering disimpan dalam format fixed-point seperti 8.8 (Subbab 2.1.9). Konsep pelengkapnya adalah backdrop, yaitu nilai winding awal yang diwarisi sebuah ubin dari seluruh geometri di sebelah kirinya pada baris yang sama. Dengan mempropagasikan backdrop secara inkremental dari kiri ke kanan sepanjang satu baris ubin, sebuah ubin interior yang tidak dilintasi tepi mana pun tetap dapat ditentukan keterisiannya hanya dari nilai backdrop yang merambat, tanpa perlu mengevaluasi ulang seluruh geometri.

### 2.1.13 Evaluasi Cakupan Piksel Analitik

Cakupan piksel (pixel coverage) adalah fraksi luas sebuah piksel yang tertutup oleh suatu bentuk, bernilai antara nol (piksel sepenuhnya di luar) hingga satu (piksel sepenuhnya di dalam). Nilai cakupan inilah yang menjadi dasar antialiasing berkualitas tinggi: tepi bentuk yang memotong sebagian piksel direpresentasikan dengan nilai cakupan pecahan, menghasilkan transisi warna yang halus alih-alih tepi bergerigi (aliasing). Pendekatan analitik menghitung cakupan ini secara eksak melalui integrasi geometris, berbeda dengan pendekatan supersampling yang mengaproksimasinya melalui pencuplikan banyak titik per piksel sehingga jauh lebih mahal.

Salah satu cara menghitung cakupan analitik adalah melalui integral luas bertanda di bawah setiap segmen garis yang melintasi kotak piksel. Kontribusi setiap segmen terhadap luas tertutup di dalam piksel dapat dievaluasi dalam bentuk tertutup (closed form) menggunakan integrasi trapezoidal, lalu kontribusi seluruh segmen yang melintasi piksel dijumlahkan untuk memperoleh cakupan total. Keunggulan pendekatan ini adalah cakupan dihitung tepat dalam satu evaluasi per segmen tanpa pencuplikan berulang, sehingga sesuai untuk dieksekusi pada fragment shader pipeline rasterisasi konvensional. Hasil integrasi tersebut kemudian dipetakan menjadi koefisien cakupan akhir sesuai aturan pengisian (fill rule) yang berlaku, baik NonZero maupun EvenOdd, sebagaimana diuraikan pada konsep winding number di Subbab 2.1.4.

### 2.1.14 Single Instruction, Multiple Data (SIMD)

Single Instruction, Multiple Data (SIMD) adalah model komputasi paralel pada tingkat instruksi (instruction-level parallelism) di mana satu instruksi tunggal dieksekusi secara serentak terhadap beberapa elemen data sekaligus. Prosesor modern menyediakan register lebar — misalnya 128-bit — yang dapat menampung beberapa nilai sekaligus, seperti empat bilangan floating-point presisi tunggal atau delapan bilangan bulat 16-bit, lalu mengoperasikannya dalam satu siklus instruksi. Berbeda dengan paralelisme tingkat thread yang membagi pekerjaan antar unit pemrosesan terpisah (Subbab 2.1.7), SIMD mempercepat komputasi di dalam satu thread dengan memproses banyak elemen data per operasi.

Dalam konteks pra-pemrosesan grafis vektor, SIMD sangat efektif untuk operasi yang bersifat seragam dan berulang atas banyak data, seperti transformasi affine terhadap deretan titik kontrol, evaluasi polinomial kurva pada banyak parameter sekaligus, serta akumulasi nilai winding antar-ubin. Operasi fused multiply-add (FMA), yang menggabungkan perkalian dan penjumlahan dalam satu instruksi terkurung, kerap dipakai bersama SIMD untuk mempercepat evaluasi matriks transformasi sekaligus menjaga presisi numerik. Pada lingkungan WebAssembly, kapabilitas SIMD diekspos melalui set instruksi SIMD128 yang menyediakan register 128-bit, sehingga optimasi tingkat instruksi semacam ini tetap dapat dimanfaatkan pada eksekusi di dalam peramban.

### 2.1.15 Graphics Pipeline

Graphics pipeline (pipeline grafis) adalah rangkaian tahapan terurut yang mengubah deskripsi geometri tiga atau dua dimensi beserta atributnya menjadi citra raster berupa piksel pada layar (Akenine-Möller dkk., 2018). Secara konseptual, pipeline ini terbagi menjadi beberapa tahap utama: tahap pemrosesan geometri (geometry processing) yang menangani transformasi vertex dari ruang model ke ruang layar, tahap rasterisasi (rasterization) yang menentukan piksel-piksel mana yang tercakup oleh setiap primitif, dan tahap pemrosesan piksel (pixel processing) yang menghitung warna akhir setiap piksel melalui operasi shading dan komposisi.

Pada perangkat keras grafis modern, sebagian tahap pipeline ini bersifat dapat diprogram (programmable) melalui shader, sementara sebagian lainnya bersifat tetap (fixed-function). Vertex shader memproses setiap vertex secara independen, sedangkan fragment shader (disebut juga pixel shader) menghitung warna setiap fragmen yang dihasilkan rasterizer. Karakteristik penting dari pipeline grafis konvensional adalah sifat paralelnya yang masif: jutaan vertex dan fragmen dapat diproses secara bersamaan oleh banyak unit eksekusi GPU. Namun, pipeline rasterisasi konvensional dirancang untuk primitif sederhana seperti segitiga dan garis, sehingga rendering kurva vektor derajat tinggi memerlukan tahap pra-pemrosesan untuk mengubah kurva menjadi primitif yang dapat dicerna pipeline tersebut (Akenine-Möller dkk., 2018).

### 2.1.16 WebGL 2.0

WebGL 2.0 adalah antarmuka pemrograman aplikasi (API) grafis tingkat rendah berbasis JavaScript yang memungkinkan rendering grafis 2D dan 3D dengan akselerasi perangkat keras (GPU) langsung di dalam peramban web tanpa memerlukan plugin tambahan (Khronos Group, 2022). WebGL 2.0 dikembangkan dan distandarisasi oleh Khronos Group, dan secara teknis merupakan binding JavaScript dari OpenGL ES 3.0, sehingga mewarisi kapabilitas dan model pemrograman pipeline rasterisasi konvensional dari OpenGL ES.

Dibandingkan dengan WebGL 1.0, versi 2.0 memperkenalkan sejumlah fitur penting seperti dukungan terhadap Vertex Array Objects (VAO), Transform Feedback, Uniform Buffer Objects (UBO), texture 3D, serta penanganan tekstur dan format data yang lebih kaya. Meski demikian, WebGL 2.0 tidak menyediakan compute shader sebagaimana yang tersedia pada API modern seperti Vulkan atau WebGPU. Keterbatasan ini menjadi pertimbangan arsitektural yang relevan: rendering grafis vektor yang membutuhkan komputasi paralel umum (general-purpose) tidak dapat sepenuhnya dialihkan ke GPU melalui WebGL 2.0, sehingga komputasi semacam itu perlu ditangani melalui jalur lain, misalnya di CPU, sebelum hasilnya diserahkan ke pipeline rasterisasi WebGL untuk digambar (Khronos Group, 2022).

### 2.1.17 Rust

Rust adalah bahasa pemrograman sistem (systems programming language) sumber terbuka yang menekankan tiga tujuan utama secara simultan: keamanan (safety), kecepatan (speed), dan konkurensi (concurrency) (Klabnik & Nichols, 2022). Keunggulan paling khas Rust terletak pada model kepemilikan (ownership) beserta sistem peminjaman (borrowing) dan masa hidup (lifetimes) yang diverifikasi oleh kompilator pada waktu kompilasi. Mekanisme ini menjamin keamanan memori (memory safety) dan keamanan thread (thread safety) tanpa memerlukan garbage collector, sehingga Rust mampu menghasilkan kinerja yang setara dengan C dan C++ namun dengan jaminan keselamatan yang jauh lebih kuat.

Bagi penelitian rendering grafis berbasis paralelisasi CPU, Rust menawarkan kombinasi yang sangat sesuai. Jaminan thread safety pada waktu kompilasi — yang sering disebut "fearless concurrency" — memungkinkan penulisan kode multithreaded yang bebas dari data race tanpa mengorbankan kinerja (Klabnik & Nichols, 2022). Ekosistem Rust juga menyediakan pustaka seperti Rayon untuk paralelisme data yang ringkas (The Rust Project Developers, 2024), serta dukungan kompilasi ke target WebAssembly yang memungkinkan kode Rust dijalankan di dalam peramban dengan kinerja mendekati native. Kombinasi keamanan memori, kinerja tinggi, konkurensi yang aman, dan kemampuan kompilasi ke WebAssembly inilah yang menjadikan Rust pilihan tepat sebagai bahasa implementasi dalam penelitian ini.

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
