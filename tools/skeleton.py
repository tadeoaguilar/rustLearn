#!/usr/bin/env python3
"""Turn a solution .rs file into an exercise skeleton.

- function bodies -> todo!("<label>")
- #[cfg(test)] mod ... { } removed
- solution crate names renamed to the exercise crate name
Usage: skeleton.py <solution_src_dir> <exercise_src_dir> <solution_crate_ident> <exercise_crate_ident> [files...]
    FORCE=1 python3 tools/skeleton.py 01-rust-fundamentals/03-ownership-borrowing/solution/src \
        01-rust-fundamentals/03-ownership-borrowing/exercise/src \
        m03_ownership_borrowing_solution m03_ownership_borrowing ex03_strings.rs

Rules: every fn body becomes todo!("<Exercise N>") (label from the file name:
exNN_* -> Exercise N, bonus_* -> Bonus); `const fn` gets a bare todo!();
`fn drop(&mut self)` gets an empty body (a panic in drop can abort the test
run). Functions returning `impl Trait` are reported on stderr: they need a
hand-written placeholder after the todo!(). #[cfg(test)] modules are removed.
"""
import os
import re
import sys


def scan_to_matching_brace(s, i):
    """s[i] == '{'. Return index just past the matching '}' (skips strings/comments/chars)."""
    depth = 0
    n = len(s)
    while i < n:
        c = s[i]
        if s.startswith('//', i):
            j = s.find('\n', i)
            i = n if j == -1 else j
            continue
        if s.startswith('/*', i):
            j = s.find('*/', i + 2)
            i = n if j == -1 else j + 2
            continue
        m = re.match(r'b?r(#*)"', s[i:])
        if m and (i == 0 or not (s[i - 1].isalnum() or s[i - 1] == '_')):
            hashes = m.group(1)
            end = s.find('"' + hashes, i + m.end())
            i = end + 1 + len(hashes)
            continue
        if c == '"':
            i += 1
            while s[i] != '"':
                i += 2 if s[i] == '\\' else 1
            i += 1
            continue
        if c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", s[i:])
            if m:
                i += m.end()
                continue
            i += 1
            continue
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise ValueError('unbalanced braces')


FN_RE = re.compile(r'^([ \t]*)(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+(\w+)', re.M)
TEST_MOD_RE = re.compile(r'^[ \t]*#\[cfg\(test\)\]\s*\n[ \t]*mod\s+\w+\s*\{', re.M)


def find_sig_body_start(s, start):
    """From the start of `fn`, find the '{' that opens the body, or None if `;` first."""
    i = start
    depth_paren = 0
    depth_angle = 0
    while i < len(s):
        c = s[i]
        if c in '([':
            depth_paren += 1
        elif c in ')]':
            depth_paren -= 1
        elif c == '<':
            depth_angle += 1
        elif c == '>' and s[i - 1] != '-':
            depth_angle -= 1
        elif c == ';' and depth_paren == 0:
            return None
        elif c == '{' and depth_paren == 0:
            return i
        i += 1
    return None


def skeletonize(src, label, crate_from, crate_to):
    # drop test modules
    while True:
        m = TEST_MOD_RE.search(src)
        if not m:
            break
        brace = src.index('{', m.start())
        end = scan_to_matching_brace(src, brace)
        # also drop following newline
        if end < len(src) and src[end] == '\n':
            end += 1
        src = src[:m.start()] + src[end:]

    out = []
    pos = 0
    while True:
        m = FN_RE.search(src, pos)
        if not m:
            out.append(src[pos:])
            break
        brace = find_sig_body_start(src, m.end())
        if brace is None:
            out.append(src[pos:m.end()])
            pos = m.end()
            continue
        end = scan_to_matching_brace(src, brace)
        indent = m.group(1)
        sig = src[m.start():brace].rstrip()
        if re.search(r'->\s*impl', sig):
            print(f'  NOTE: {m.group(2)} returns impl Trait -- needs a hand-written placeholder', file=sys.stderr)
        out.append(src[pos:m.start()])
        if re.search(r'\bconst\s+fn\b', sig):
            body = 'todo!()'
        elif re.search(r'\bfn\s+drop\s*\(\s*&mut\s+self\s*\)', sig):
            # A panic inside drop() during unwinding aborts the process.
            body = f'// TODO {label}: a todo!() here could abort the test run, so this is empty.'
        else:
            body = f'todo!("{label}")'
        out.append(f'{sig} {{\n{indent}    {body}\n{indent}}}')
        pos = end
    text = ''.join(out)
    text = text.replace(crate_from, crate_to)
    text = text.replace(crate_from.replace('_', '-'), crate_to.replace('_', '-'))
    return text


def label_for(fname):
    m = re.match(r'ex(\d+)', fname)
    if m:
        return f'Exercise {int(m.group(1))}'
    if fname.startswith('bonus'):
        return 'Bonus'
    if fname.startswith('challenge'):
        return 'Challenge'
    return 'TODO'


def main():
    sol_dir, ex_dir, crate_from, crate_to, *files = sys.argv[1:]
    if not files:
        files = []
        for root, _, names in os.walk(sol_dir):
            for n in names:
                if n.endswith('.rs') and n not in ('main.rs', 'lib.rs'):
                    files.append(os.path.relpath(os.path.join(root, n), sol_dir))
    for f in files:
        src = open(os.path.join(sol_dir, f)).read()
        label = label_for(os.path.basename(f))
        text = skeletonize(src, label, crate_from, crate_to)
        dest = os.path.join(ex_dir, f)
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        if os.path.exists(dest) and os.environ.get('FORCE') != '1':
            print(f'skip (exists): {dest}')
            continue
        open(dest, 'w').write(text)
        print(f'wrote {dest}')


if __name__ == '__main__':
    main()
