# Answers · 31 CLI Tools

## Exercise 1: Why stderr?

Because stdout is *data*. `minigrep foo *.txt | wc -l` should count matching
lines, not matching lines plus "missing.txt: No such file". Errors on stderr
still reach the terminal when the user is watching, but pipes and
redirections carry only results. The exit code is the third channel: a
script asks "did it match?" with `if grep -q ...` without parsing
anything -- which is why 1 (no match) and 2 (trouble) must differ.

## Exercise 2: Why stream?

Log files are often gigabytes. Reading the whole file before printing the
last ten lines would take memory proportional to the file; a ring buffer of
N lines takes memory proportional to N. `filter` and `stats` stream too:
each line is parsed, used and dropped. (Real `tail` goes further and reads
the file *backwards* from the end.)

## Exercise 3: Why the same directory?

`rename` is atomic only within one filesystem: the directory entry is
switched from the old file to the new one in a single step, so a reader (or
a crash) sees either the complete old store or the complete new one. A
temporary file in `/tmp` may be on a different filesystem (often a RAM
disk), and then "rename" degrades into copy-and-delete -- exactly the
half-written state we wanted to avoid. (For durability across power loss,
you'd also `fsync` the file and then the directory.)
