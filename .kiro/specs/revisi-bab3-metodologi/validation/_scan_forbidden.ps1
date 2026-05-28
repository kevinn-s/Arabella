param(
    [string]$Path = 'e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md'
)

$ErrorActionPreference = 'Stop'
$Utf8 = New-Object System.Text.UTF8Encoding($false)
$content = [System.IO.File]::ReadAllText($Path, $Utf8)
$lines = [System.IO.File]::ReadAllLines($Path, $Utf8)
"FILE  : $Path"
"BYTES : $((Get-Item $Path).Length)"
"LINES : $($lines.Count)"
"FIRST : $($lines[0])"
""

# ---------- helpers ----------
function Scan-CI([string]$Label, [string]$Pattern) {
    $opts = [System.Text.RegularExpressions.RegexOptions]::IgnoreCase -bor `
            [System.Text.RegularExpressions.RegexOptions]::Multiline
    $rx = [regex]::new($Pattern, $opts)
    $hits = @()
    for ($i = 0; $i -lt $lines.Count; $i++) {
        foreach ($m in $rx.Matches($lines[$i])) {
            $hits += [pscustomobject]@{
                Line = $i + 1
                Match = $m.Value
                Context = $lines[$i]
            }
        }
    }
    "[CI ] {0,-50}  pattern={1,-50}  matches={2}" -f $Label, $Pattern, $hits.Count
    foreach ($h in $hits) {
        "       L{0:D4}: {1}  | {2}" -f $h.Line, $h.Match, $h.Context.Trim()
    }
}

function Scan-CS([string]$Label, [string]$Pattern) {
    $opts = [System.Text.RegularExpressions.RegexOptions]::Multiline
    $rx = [regex]::new($Pattern, $opts)
    $hits = @()
    for ($i = 0; $i -lt $lines.Count; $i++) {
        foreach ($m in $rx.Matches($lines[$i])) {
            $hits += [pscustomobject]@{
                Line = $i + 1
                Match = $m.Value
                Context = $lines[$i]
            }
        }
    }
    "[CS ] {0,-50}  pattern={1,-50}  matches={2}" -f $Label, $Pattern, $hits.Count
    foreach ($h in $hits) {
        "       L{0:D4}: {1}  | {2}" -f $h.Line, $h.Match, $h.Context.Trim()
    }
}

# ---------- Group A: case-insensitive phrases ----------
"=== Group A (case-insensitive phrase scan) ==="
Scan-CI 'A.1 ray shooting/shoot variants'         'ray\s*shoot'
Scan-CI 'A.2 ditranspilasikan ke WebGL'           'ditranspilasikan ke WebGL'
Scan-CI 'A.3 transpilasi OpenGL ES'               'transpilasi OpenGL ES'
Scan-CI 'A.4 OpenGL ES 3.0 yang ditranspilasikan' 'OpenGL ES 3\.0 yang ditranspilasikan'
Scan-CI 'A.x any transpil* token'                 'transpil'
Scan-CI 'A.y any OpenGL token'                    'OpenGL'
""

# ---------- Group B: case-sensitive whole-word tokens ----------
"=== Group B (case-sensitive whole-word token scan) ==="
Scan-CS 'B.1 TileType'                            '\bTileType\b'
Scan-CS 'B.2 winding_number'                      '\bwinding_number\b'
Scan-CS 'B.3 PPGA'                                '\bPPGA\b'
Scan-CS 'B.4 Projective Geometric Algebra'        'Projective Geometric Algebra'
Scan-CS 'B.5 EMPTY (uppercase token)'             '\bEMPTY\b'
Scan-CS 'B.6 INTERIOR (uppercase token)'          '\bINTERIOR\b'
Scan-CS 'B.7 EDGE (uppercase token)'              '\bEDGE\b'
Scan-CI 'B.8 Rust edisi 2021'                     'Rust edisi 2021'
Scan-CI 'B.9 edisi 2021'                          'edisi 2021'
Scan-CI 'B.10 edition = "2021"'                   'edition\s*=\s*"2021"'
""

# ---------- Group C: equation forms ----------
"=== Group C (equation-form scan, case-insensitive) ==="
Scan-CI 'C.1 ax+by+c=0 (any spacing)'             'a\s*x\s*\+\s*b\s*y\s*\+\s*c\s*=\s*0'
Scan-CI 'C.2 u-v² (unicode squared)'              'u\s*-\s*v\s*\u00b2'
Scan-CI 'C.3 u-v^2 (ASCII caret)'                 'u\s*-\s*v\s*\^\s*2'
Scan-CI 'C.4 f(u,v) opening'                      'f\s*\(\s*u\s*,\s*v\s*\)'
Scan-CI 'C.5 C(x,y)=0'                            'C\s*\(\s*x\s*,\s*y\s*\)\s*=\s*0'
Scan-CI 'C.6 w_0..w_3 PPGA tokens'                'w_[0123]'
Scan-CI 'C.7 superscript-3 (³)'                   '\u00b3'
Scan-CI 'C.8 superscript-2 (²)'                   '\u00b2'
""

# ---------- Sanity: canonical replacements present ----------
"=== Sanity (canonical replacements should be present) ==="
$canon = @('binning DDA', 'akumulator signed-area', 'propagasi backdrop',
          'F24Dot8', 'fragment shader', 'WebGL 2.0', 'Rust edisi 2024',
          'edition = "2024"')
foreach ($t in $canon) {
    $count = ([regex]::Matches($content, [regex]::Escape($t))).Count
    "[CANON] {0,-40} hits={1}" -f $t, $count
}
