import "math"
import "hash"
import "pe"

private rule MZHeader
{
    condition:
        uint16(0) == 0x5A4D
}

rule TextStringModifiers : suite text
{
    meta:
        author = "Artem Shymko"
        description = "Text strings with the full modifier surface"
        severity = 3
        stable = true
    strings:
        $plain = "STUDIO_MARKER_PLAIN"
        $nocase = "Studio_Marker_Case" nocase
        $wide = "STUDIO_MARKER_WIDE" wide
        $both = "STUDIO_MARKER_BOTH" ascii wide
        $full = "STUDIO_WORD" fullword
        $xored = "STUDIO_MARKER_XOR" xor(0x01-0xff)
        $b64 = "STUDIO_MARKER_B64" base64
    condition:
        MZHeader and 3 of them and $plain at 4
}

rule HexPatterns : suite hex
{
    meta:
        description = "Hex strings with wildcards, jumps and alternatives"
    strings:
        $magic = { 4D 5A 90 00 }
        $wild = { DE AD ?? EF }
        $jump = { CA FE [1-8] BA BE }
        $alt = { 00 ( AA | BB CC ) 00 }
    condition:
        $magic at 0 and $wild and $jump and $alt
}

rule RegexPatterns : suite regex
{
    meta:
        description = "Regular expressions with modifiers"
    strings:
        $url = /https?:\/\/[a-z0-9.\-]+\.example\.com/ nocase
        $ver = /VERSION_[0-9]{2}\.[0-9]{2}/
    condition:
        all of them
}

rule CountsAndOffsets : suite counts
{
    meta:
        description = "Match counters, offsets, lengths and for-loops"
    strings:
        $rep = "REPEATED_TOKEN"
    condition:
        #rep == 3 and
        @rep[1] < @rep[2] and
        !rep[1] == 14 and
        $rep in (0..filesize) and
        for all i in (1..#rep) : ( @rep[i] < filesize )
}

rule ModuleFunctions : suite modules
{
    meta:
        description = "math and hash module calls over the whole file"
    condition:
        math.entropy(0, filesize) > 0.5 and
        hash.crc32(0, filesize) != 0 and
        not pe.is_pe
}

rule FilesizeAndInts : suite primitives
{
    meta:
        description = "Integer readers and filesize keyword"
    condition:
        filesize > 64 and filesize < 1MB and
        uint16(0) == 0x5A4D and
        uint8(2) == 0x90 and
        uint32be(0) == 0x4D5A9000
}
