# Abstrak

**UNIVERSITAS BINA NUSANTARA**

Software Engineering Program
Program Studi Teknik Informatika
School of Computer Science

**PERANCANGAN PIPELINE RENDERING VEKTOR PARALEL SKALA BESAR PADA LINGKUNGAN GRAFIS NON-COMPUTE**

Kevin Sukohardjo — 2602145680
Surya Saddam Saputra — 2602157554
Arlin Lutfi Widarma — 2602160920

---

## ABSTRACT

The goal of this research was to design and realize a parallel two-dimensional vector graphics rendering pipeline that does not depend on compute shaders, so that it can operate on graphics environments providing only a conventional rasterization pipeline. The research method was divided into four stages: requirements analysis, hybrid pipeline architecture design, prototype implementation, and testing with evaluation. The prototype, named Arabella, was built in the Rust programming language compiled to a WebAssembly target and executed on top of WebGL 2.0. The architecture partitions the workload into two stages: a CPU pre-processing stage that performs Bézier curve flattening, tile-based DDA binning, winding number accumulation through a per-scanline signed-area accumulator, and backdrop propagation; and a GPU rasterization stage that evaluates pixel coverage analytically through a traditional vertex shader and fragment shader. Evaluation was carried out through visual correctness validation against test SVG files and per-frame render time measurement separated into CPU and GPU costs. The result of this research is a working prototype capable of rendering SVG files through a hybrid pipeline without any compute shader, thereby demonstrating the feasibility of a non-compute hybrid vector rendering approach on a WebGL 2.0 environment.

**Keywords:** vector rendering, hybrid CPU-GPU pipeline, non-compute shader, WebGL 2.0, tile-based rasterization, winding number

---

## ABSTRAK

Tujuan penelitian ini adalah merancang dan mewujudkan pipeline rendering grafis vektor dua dimensi paralel yang tidak bergantung pada compute shader, sehingga dapat beroperasi pada lingkungan grafis yang hanya menyediakan pipeline rasterisasi konvensional. Metode penelitian dibagi menjadi empat tahap, yaitu analisis kebutuhan, perancangan arsitektur pipeline hibrida, implementasi purwarupa, serta pengujian dan evaluasi. Purwarupa bernama Arabella dibangun menggunakan bahasa pemrograman Rust yang dikompilasi ke target WebAssembly dan dieksekusi di atas WebGL 2.0. Arsitektur membagi beban kerja menjadi dua tahap: tahap pra-pemrosesan di CPU yang melakukan flattening kurva Bézier, binning DDA berbasis ubin, akumulasi winding number melalui akumulator signed-area per scanline, dan propagasi backdrop; serta tahap rasterisasi di GPU yang mengevaluasi cakupan piksel secara analitik melalui vertex shader dan fragment shader tradisional. Evaluasi dilakukan melalui validasi kebenaran output secara visual terhadap berkas SVG uji serta pengukuran waktu render per bingkai yang dipisahkan atas biaya CPU dan GPU. Hasil penelitian adalah purwarupa yang mampu merender berkas SVG melalui pipeline hibrida tanpa satu pun compute shader, sehingga membuktikan kelayakan pendekatan rendering vektor hibrida non-compute pada lingkungan WebGL 2.0.

**Kata kunci:** rendering vektor, pipeline hibrida CPU-GPU, non-compute shader, WebGL 2.0, rasterisasi berbasis ubin, winding number
