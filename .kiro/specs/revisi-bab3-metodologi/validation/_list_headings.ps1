$f = 'e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md'
$lines = Get-Content -Path $f
for ($i = 0; $i -lt $lines.Count; $i++) {
    if ($lines[$i] -match '^#{1,4} ') {
        '{0}: {1}' -f ($i + 1), $lines[$i]
    }
}
