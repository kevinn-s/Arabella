# BAB 2 TINJAUAN REFERENSI

## 2.1 Landasan Teori

### 2.1.1 Polinomial Bernstein

Polinomial Bernstein adalah keluarga polinomial basis yang menjadi fondasi matematis bagi representasi kurva Bézier. Untuk derajat n, terdapat n + 1 polinomial Bernstein yang didefinisikan sebagai (Farin, 2002):

```
B_{i,n}(t) = C(n,i) * t^i * (1-t)^(n-i),  i = 0, ..., n
```

dengan C(n,i) menyatakan koefisien binomial. Polinomial Bernstein memiliki sejumlah sifat penting yang menjadikannya basis ideal untuk desain geometri. Pertama, sifat non-negativitas: setiap B_{i,n}(t) >= 0 pada selang [0,1]. Kedua, sifat partisi kesatuan (partition of unity), yaitu jumlah seluruh polinomial basis pada sembarang nilai t selalu sama dengan satu:

```
sum_{i=0}^{n} B_{i,n}(t) = 1
```

Kombinasi kedua sifat ini menjamin bahwa setiap titik pada kurva merupakan kombinasi konveks (convex combination) dari titik-titik kontrolnya, sehingga kurva selalu berada di dalam convex hull titik-titik kontrol. Sifat-sifat inilah yang membuat polinomial Bernstein menjadi dasar yang stabil secara numerik dan intuitif secara geometris bagi konstruksi kurva Bézier (Farin, 2002).

### 2.1.2 Kurva Bezier

Kurva Bezier adalah garis lengkung yang halus yang didefinisikan oleh rumus matematika dan titik-titik kontrol. Kurva Bezier menggunakan polinomial Bernstein sebagai basis. Sebuah kurva bezier dengan derajat n (order n + 1) direpresentasikan sebagai (Farin, 2002):

```
r(t) = sum_{i=0}^{n} b_i * B_{i,n}(t),  0 <= t <= 1
```

Koefisiennya, b_i, merepresentasikan titik kontrol atau titik Bézier, sedangkan B_{i,n}(t) adalah polinomial basis Bernstein yang menentukan kontribusi setiap titik kontrol terhadap bentuk kurva. Keduanya secara bersama-sama menentukan bentuk akhir kurva. Dalam praktik grafis komputer, kurva Bézier kuadratik (n = 2) dan kubik (n = 3) paling lazim dipakai karena keseimbangan antara fleksibilitas bentuk dan kesederhanaan komputasi. Kurva Bézier memiliki sifat-sifat yang diwarisi dari basis Bernstein, antara lain selalu melewati titik kontrol pertama dan terakhir, bersinggungan dengan poligon kontrol pada kedua ujungnya, serta seluruhnya termuat di dalam convex hull titik-titik kontrolnya (Farin, 2002).

### 2.1.3 Winding Number

Winding number merupakan konsep matematika untuk menyatakan berapa kali sebuah kurva tertutup melintasi titik acuan tertentu (Alciatore & Miranda, 1995). Dalam sistem koordinat dua dimensi, nilai winding number (ω) ditentukan oleh orientasi atau arah lintasan kurva:

1. Counter-Clockwise (CCW): Lintasan yang berlawanan dengan arah jarum jam menghasilkan nilai positif (ω > 0).
2. Clockwise (CW): Lintasan yang searah dengan arah jarum jam menghasilkan nilai negatif (ω < 0).

Dalam bidang komputer grafis, konsep winding number banyak dimanfaatkan untuk menentukan apakah suatu titik berada di bagian dalam atau di luar suatu objek vektor. Kemampuan tersebut menjadikan winding number sebagai salah satu dasar penting dalam pengembangan berbagai algoritma pengisian area (filling), pengujian posisi titik (point-in-polygon), serta teknik pemrosesan geometri lainnya pada grafis vektor (Hormann & Agathos, 2001).

### 2.1.4 Point-In-Polygon

Point-in-polygon adalah algoritma dalam komputasi geometri untuk menentukan apakah sebuah titik (koordinat x dan y) berada di dalam, luar, atau pada batas poligon (Hormann & Agathos, 2001). Algoritma ini banyak digunakan dalam berbagai bidang, seperti grafik komputer, pemrosesan vektor, sistem informasi geografis (GIS), dan simulasi fisika. Beberapa metode umum yang digunakan untuk menentukan posisi titik relatif terhadap poligon antara lain:

**1. Ray Casting Algorithm**

Metode ray casting menentukan posisi titik dengan cara memproyeksikan sebuah garis imajiner (ray) dari titik yang diuji ke arah tertentu, misalnya ke kanan secara horizontal. Selanjutnya, jumlah perpotongan antara garis tersebut dan sisi-sisi poligon dihitung. Apabila jumlah perpotongan yang diperoleh bernilai ganjil, maka titik dikategorikan berada di dalam poligon. Sebaliknya, jika jumlah perpotongan bernilai genap, titik dianggap berada di luar poligon (Foley et al., 1996; Haines, 1994).

**2. Winding Number Algorithm**

Sebagaimana telah dijelaskan pada subbab sebelumnya, metode winding number memanfaatkan orientasi setiap sisi poligon terhadap titik yang sedang diuji. Proses ini menghasilkan suatu nilai yang disebut winding number, yang merepresentasikan jumlah putaran poligon mengelilingi titik tersebut. Nilai winding number yang sama dengan nol menunjukkan bahwa titik berada di luar poligon, sedangkan nilai selain nol menandakan bahwa titik berada di dalam poligon. Dibandingkan dengan metode ray casting, pendekatan ini umumnya memberikan hasil yang lebih andal pada poligon dengan bentuk kompleks, termasuk poligon yang memiliki sisi-sisi saling berpotongan (self-intersecting polygons) (Hormann & Agathos, 2001).

### 2.1.5 Komputasi Parallel

Komputasi paralel adalah proses di mana masalah komputasi besar dipecah menjadi masalah-masalah kecil yang dapat diselesaikan secara bersamaan oleh beberapa prosesor (International Business Machines Corporation, 2022). Konsep ini memanfaatkan kemampuan prosesor modern, baik CPU multi-core maupun GPU, untuk mempercepat perhitungan yang bersifat independen atau memiliki pola data yang dapat dibagi. Dalam konteks grafis komputer, komputasi paralel sangat berguna untuk melakukan perhitungan vertex, fragment, atau operasi matematis kompleks seperti rasterisasi, evaluasi kurva, dan simulasi fisika secara efisien.

### 2.1.6 Jordan Curve Theorem

Teorema Kurva Jordan (Jordan Curve Theorem) adalah konsep dasar dalam matematika yang menjadi landasan teori untuk komputasi geometri. Teorema ini menyatakan bahwa setiap kurva tertutup yang sederhana, yakni kurva yang bersambung dan tidak memotong dirinya sendiri, akan membagi sebuah bidang datar menjadi tepat dua area yang terpisah. Kedua area tersebut adalah bagian dalam (interior) yang ruangnya dibatasi oleh kurva, serta bagian luar (eksterior) yang luasnya tidak terbatas. Akibatnya, setiap garis lurus yang ditarik dari suatu titik di dalam kurva menuju titik di luar kurva pasti akan memotong batas kurva tersebut (Hales, 2007).

Meskipun konsep pemisahan area ini terdengar sederhana, pembuktian matematisnya cukup rumit dan menjadi pencapaian penting dalam perkembangan analisis geometri modern (Hales, 2007). Dalam ranah grafis komputer, Teorema Kurva Jordan memberikan dasar teori untuk menentukan bagian "dalam" dan "luar" dari sebuah objek vektor yang tertutup. Dengan adanya kepastian pembagian ruang ini, mesin komputasi memiliki dasar matematis yang valid untuk menjalankan kalkulasi secara pasti. Hal ini secara langsung menjadi dasar bagi penerapan algoritma evaluasi ruang seperti point-in-polygon dan winding number. Algoritma tersebut digunakan oleh sistem untuk menentukan apakah suatu piksel berada di dalam bentuk vektor sehingga perlu diisi warna (dirender) atau tidak.

### 2.1.7 Graphics Pipeline

Graphics pipeline (alur grafis) pada dasarnya merupakan serangkaian langkah yang menerjemahkan geometri dua atau tiga dimensi beserta seluruh atributnya agar bisa tampil sebagai susunan piksel pada layar (Akenine-Möller et al., 2018). Bila ditelaah, alur ini bertumpu pada beberapa pilar utama. Fase pemrosesan geometri bekerja terlebih dahulu untuk memindahkan posisi vertex dari ruang model ke ruang layar. Setelah itu, tahap rasterisasi mengambil alih dengan menyeleksi piksel mana saja yang tepat jatuh di dalam sebuah primitif. Barulah di akhir, warna final untuk masing-masing piksel dikalkulasi dalam tahap pemrosesan piksel melalui komposisi dan shading.

Menariknya, arsitektur perangkat keras grafis saat ini memberikan fleksibilitas. Kita bisa memprogram sebagian dari tahapan tersebut menggunakan shader, walau beberapa bagian lainnya memang dirancang statis. Vertex shader secara khusus mengolah tiap vertex secara terpisah. Di ujung proses lain, luaran warna fragmen dari rasterizer menjadi tanggung jawab fragment shader. Hal yang paling memukau dari pipeline ini sebetulnya adalah skala paralelismenya. Bayangkan saja, jutaan fragmen dan vertex mampu diproses serentak oleh ragam unit komputasi di dalam GPU tanpa kendala.

Satu hal krusial yang patut dicatat, sistem rasterisasi konvensional sejak awal dioptimalkan hanya untuk menangani bentuk-bentuk simpel seperti garis atau segitiga. Imbasnya, sistem tidak bisa langsung mengeksekusi kurva vektor berderajat tinggi. Kita wajib menyisipkan tahap pra-pemrosesan di awal untuk memecah kurva rumit tersebut menjadi elemen primitif dasar. Tanpa konversi ini, pipeline grafis tidak akan mampu "mencerna" dan melakukan rendering pada kurva tersebut (Akenine-Möller et al., 2018).

### 2.1.8 WebGL 2.0

Kemampuan rendering grafis 2D serta 3D di dalam peramban yang diakselerasi langsung oleh perangkat keras (GPU) tanpa sedikit pun menuntut keberadaan plugin tambahan difasilitasi lewat WebGL 2.0. Secara harfiah, teknologi yang distandardisasi sekaligus terus dikembangkan oleh Khronos Group ini beroperasi sebagai antarmuka pemrograman aplikasi (API) level rendah dengan pijakan utama JavaScript (Khronos Group, 2022). Karena pondasi teknisnya tiada lain merupakan binding JavaScript untuk OpenGL ES 3.0, maka kerangka pemrograman pipeline rasterisasi konvensional beserta keseluruhan kapabilitas milik pendahulunya tersebut secara otomatis ikut terwariskan.

Lonjakan fitur yang diboyong oleh rilis 2.0 ini terbilang sangat esensial bila dihadapkan dengan versi 1.0. Pengelolaan format data maupun teksturnya disajikan jauh lebih kaya. Selain itu, versi ini juga mulai menyertakan dukungan langsung bagi Vertex Array Objects (VAO), tekstur 3D, Uniform Buffer Objects (UBO), sampai dengan Transform Feedback.

Kendati arsitekturnya makin matang, WebGL 2.0 masih menyimpan satu kekosongan mendasar, yakni ketiadaan compute shader. Padahal, instrumen semacam ini sudah menjadi hal lumrah pada API generasi mutakhir sekelas WebGPU ataupun Vulkan. Limitasi struktural tersebut jelas memaksa para pengembang untuk mengambil langkah antisipasi. GPU sama sekali tidak bisa dijejali beban komputasi paralel umum (yang sangat dibutuhkan saat memproses grafis vektor) secara penuh melalui jalur WebGL 2.0 ini. Sebagai jalan keluarnya, kalkulasi mentah dari komputasi tersebut mesti dititipkan ke unit pemrosesan lain seperti CPU. Sesudah hasil perhitungannya rampung dikerjakan di luar, barulah data tersebut dipasok kembali ke pipeline rasterisasi WebGL supaya wujud visualnya berhasil digambar (Khronos Group, 2022).

### 2.1.9 Rust

Secara garis besar, Rust adalah bahasa pemrograman sistem bersifat open-source yang dibangun dengan tiga fokus utama. Ketiga fokus tersebut yakni keamanan, kecepatan, serta konkurensi (Klabnik & Nichols, 2022). Keunikan paling menonjol dari bahasa ini sebenarnya terletak pada konsep ownership (kepemilikan), mekanisme peminjaman data, dan batasan lifetimes. Semuanya itu diverifikasi secara langsung oleh kompilator pada tahap kompilasi. Karena adanya proses pengecekan ketat di awal inilah, keamanan memori maupun thread bisa sepenuhnya terjamin. Hebatnya, jaminan tersebut dicapai tanpa harus bergantung sama sekali pada garbage collector. Alhasil, performa komputasi Rust sanggup menyaingi bahasa C atau C++, namun diiringi dengan tingkat keselamatan sistem yang jauh lebih mumpuni.

Pemilihan bahasa ini untuk riset rendering grafis berbasis paralelisasi CPU bukanlah tanpa alasan. Adanya perlindungan thread sejak fase kompilasi yang biasa dikenal di komunitasnya dengan istilah "fearless concurrency" sangat menguntungkan pengembang (Klabnik & Nichols, 2022). Kita bisa menyusun kode multithreaded secara leluasa tanpa pusing memikirkan ancaman data race. Yang paling penting, kinerja komputasinya tidak ikut merosot.

Dukungan ekosistemnya pun terbilang sangat praktis. Sebagai contoh, terdapat pustaka bernama Rayon (The Rust Project Developers, 2024). Pustaka ini sangat membantu peneliti untuk meringkas penulisan paralelisme data. Selain itu, Rust juga memiliki dukungan langsung untuk kompilasi ke target WebAssembly. Hal ini memungkinkan kode yang dibuat untuk bisa dijalankan di dalam peramban web. Performanya saat dijalankan di peramban bahkan nyaris mendekati kinerja aplikasi native. Keputusan untuk menjadikan Rust sebagai bahasa implementasi di penelitian ini pada akhirnya ditarik dari perpaduan keunggulan tersebut: memori yang aman, performa yang cepat, sistem konkuren yang bebas risiko, serta keluwesan kompilasi menuju WebAssembly.