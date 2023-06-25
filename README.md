# memgrep

Search (and optionally erase) a particular text in process memory.

# Reference

```
memgrep 0.2.0
Eldad Zack <eldad@fogrefinery.com>
Memory Grep

Searches for a particular text in all memory regions that are not virtual or not mapped to any file:
stack, heap, or anonymous pages.

When searching, only the first match is returned for each region.

When erasing, all matches are searched and erased.

USAGE:
    memgrep [OPTIONS] --pid <PID> <TEXT>

ARGS:
    <TEXT>
            Search text

OPTIONS:
    -d, --debug
            Set log level to debug

    -e, --erase
            Erase text

    -e, --erase-value <ERASE_VALUE>
            If erase is enabled, use this value to erase the search text

            [default: 32]

    -h, --hex
            Parse search text as hex string

        --help
            Print help information

    -m, --max-region-size <MAX_REGION_SIZE>
            Set maximum region size. Regions larger than this size will not be searched

            [default: 1073741824]

    -p, --pid <PID>
            PID

    -V, --version
            Print version information
```
