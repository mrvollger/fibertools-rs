```
ft-center 0.0.4
This command centers fiberseq data around given reference positions. This is useful for making
aggregate m6a and CpG observations, as well as visualization of SVs

USAGE:
    ft center [OPTIONS] <BAM> <BED>

ARGS:
    <BAM>
            fiberseq bam file, must be aligned and have an index

    <BED>
            Bed file on which to center fiberseq reads. Data is adjusted to the start position of
            the bed file and corrected for strand if a 4th strand column is included.

            If you include strand information in the 4th column it will orient data accordingly and
            use the end position of bed record instead of the start if on the minus strand. This
            means that profiles of motifs in both the forward and minus orientation will align to
            the same central position. Furthermore for CpG methylation if we are on the minus strand
            bases are further shifted by and extra -1 to account for the di-nucleotide nature of CpG
            methylation. This will allow profiles from the forward and reverse strands to align.

OPTIONS:
    -m, --min-ml-score <MIN_ML_SCORE>
            Minium score in the ML tag to include in the output

            [default: 20]

    -w, --wide
            Provide data in wide format, one row per read

    -h, --help
            Print help information

    -V, --version
            Print version information
```