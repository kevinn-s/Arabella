$path = 'e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md'
$bytes = (Get-Item $path).Length
$lines = (Get-Content $path -Encoding UTF8).Count
$raw = Get-Content $path -Raw -Encoding UTF8
$crlf = ([regex]::Matches($raw, "`r`n")).Count
$lf = ([regex]::Matches($raw, "`n")).Count
"bytes=$bytes lines=$lines crlf=$crlf lf=$lf"
