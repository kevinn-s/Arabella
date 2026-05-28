$file = "e:\Users\Documents\A - TUGAS BINUS\Thesis\Code\Skripsi\bab3_metodologi.md"
$lines = Get-Content -LiteralPath $file -Encoding UTF8

# Section boundaries (start..end inclusive, 1-indexed line numbers)
$sections = @(
    @{ Name = "3.1";        Start = 3;   End = 29 },
    @{ Name = "3.2.1";      Start = 32;  End = 37 },
    @{ Name = "3.2.2";      Start = 38;  End = 47 },
    @{ Name = "3.2.3";      Start = 48;  End = 57 },
    @{ Name = "3.3-intro";  Start = 58;  End = 61 },
    @{ Name = "3.3.1";      Start = 62;  End = 86 },
    @{ Name = "3.4-intro";  Start = 87;  End = 88 },
    @{ Name = "3.4.1";      Start = 89;  End = 96 },
    @{ Name = "3.4.2";      Start = 97;  End = 136 },
    @{ Name = "3.4.3";      Start = 137; End = 155 },
    @{ Name = "3.4.4";      Start = 156; End = 275 },
    @{ Name = "3.5";        Start = 276; End = 317 },
    @{ Name = "3.6";        Start = 318; End = 350 },
    @{ Name = "3.7";        Start = 351; End = 381 }
)

function Get-SectionName($lineNum) {
    foreach ($s in $sections) {
        if ($lineNum -ge $s.Start -and $lineNum -le $s.End) { return $s.Name }
    }
    return "?"
}

$terms = @(
    @{ Label = "WebGL 2.0";                       Pattern = 'WebGL 2\.0';                  CS = $true  },
    @{ Label = "Rust edisi 2024";                 Pattern = 'Rust edisi 2024';             CS = $true  },
    @{ Label = "F24Dot8";                         Pattern = 'F24Dot8';                     CS = $true  },
    @{ Label = "24.8 fixed-point";                Pattern = '24\.8 fixed-point';           CS = $true  },
    @{ Label = "8.8 fixed-point";                 Pattern = '8\.8 fixed-point';            CS = $false },
    @{ Label = "DDA";                             Pattern = '\bDDA\b';                     CS = $true  },
    @{ Label = "outer DDA";                       Pattern = 'outer DDA';                   CS = $false },
    @{ Label = "inner DDA";                       Pattern = 'inner DDA';                   CS = $false },
    @{ Label = "signed-area";                     Pattern = 'signed-area';                 CS = $false },
    @{ Label = "backdrop";                        Pattern = 'backdrop';                    CS = $false },
    @{ Label = "propagasi backdrop";              Pattern = 'propagasi backdrop';          CS = $false },
    @{ Label = "flattening";                      Pattern = 'flattening';                  CS = $false },
    @{ Label = "midpoint subdivision";            Pattern = 'midpoint subdivision';        CS = $false },
    @{ Label = "cubic-to-quadratic";              Pattern = 'cubic-to-quadratic';          CS = $false },
    @{ Label = "line_box";                        Pattern = 'line_box';                    CS = $true  },
    @{ Label = "trapezoidal";                     Pattern = 'trapezoidal';                 CS = $false },
    @{ Label = "fearless_simd";                   Pattern = 'fearless_simd';               CS = $true  },
    @{ Label = "lyon_path";                       Pattern = 'lyon_path';                   CS = $true  },
    @{ Label = "lyon_geom";                       Pattern = 'lyon_geom';                   CS = $true  },
    @{ Label = "kurbo";                           Pattern = '\bkurbo\b';                   CS = $true  },
    @{ Label = "peniko";                          Pattern = '\bpeniko\b';                  CS = $true  },
    @{ Label = "roxmltree";                       Pattern = '\broxmltree\b';               CS = $true  },
    @{ Label = "bytemuck";                        Pattern = '\bbytemuck\b';                CS = $true  },
    @{ Label = "thiserror";                       Pattern = '\bthiserror\b';               CS = $true  },
    @{ Label = "hashbrown";                       Pattern = '\bhashbrown\b';               CS = $true  },
    @{ Label = "smallvec";                        Pattern = '\bsmallvec\b';                CS = $true  },
    @{ Label = "NonZero";                         Pattern = 'NonZero';                     CS = $true  },
    @{ Label = "EvenOdd";                         Pattern = 'EvenOdd';                     CS = $true  },
    @{ Label = "16x8";                            Pattern = '16\s*[\u00d7x\\]\s*8';        CS = $true  },
    @{ Label = "Rayon";                           Pattern = '\bRayon\b';                   CS = $false },
    @{ Label = "record_per_scanline_crossings";   Pattern = 'record_per_scanline_crossings'; CS = $true  },
    @{ Label = "TILE_W";                          Pattern = 'TILE_W';                      CS = $true  },
    @{ Label = "TILE_H";                          Pattern = 'TILE_H';                      CS = $true  },
    @{ Label = "1080x520";                        Pattern = '1080\s*[\u00d7x\\]\s*520';    CS = $true  },
    @{ Label = "RGBA32F";                         Pattern = 'RGBA32F';                     CS = $true  }
)

$rows = @()

foreach ($t in $terms) {
    $hits = New-Object System.Collections.ArrayList
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $matched = if ($t.CS) { $line -cmatch $t.Pattern } else { $line -match $t.Pattern }
        if ($matched) {
            [void]$hits.Add(($i + 1))
        }
    }
    $count = $hits.Count
    $secNames = New-Object System.Collections.Generic.HashSet[string]
    foreach ($h in $hits) { [void]$secNames.Add((Get-SectionName $h)) }
    $secList = ($secNames | Sort-Object) -join ","
    $lineList = ($hits | Sort-Object) -join ","
    $rows += [pscustomobject]@{
        Term = $t.Label
        Count = $count
        Sections = $secList
        Lines = $lineList
    }
}

$rows | Format-Table -AutoSize | Out-String -Width 4096
