#!/usr/bin/env python3
"""Analyze every Dart example in lesson/dictionary seed files with the Dart analyzer.

usage: check_code.py <verify-project-dir> <file.json>...
Each snippet is written to lib/<file-stem>/ in the verify project (a Flutter
project with provider, flutter_bloc, flutter_riverpod, go_router, dio, http,
equatable, get_it, shared_preferences, flutter_secure_storage, mocktail,
bloc_test, integration_test). Only analyzer *errors* fail the check.
"""
import json, os, re, shutil, subprocess, sys

project, files = sys.argv[1], sys.argv[2:]
failed = 0
for path in files:
    stem = os.path.splitext(os.path.basename(path))[0].replace('-', '_')
    out = os.path.join(project, 'lib', stem)
    shutil.rmtree(out, ignore_errors=True); os.makedirs(out)
    data = json.load(open(path))
    names = {}
    if 'courses' in data:
        for c in data['courses']:
            for i, l in enumerate(c['lessons'], 1):
                for k in ('simpleExample', 'realExample'):
                    ex = l[k]
                    if ex['language'] == 'dart':
                        n = f"{c['course'].replace('-', '_')}_{i}_{k[:4]}.dart"
                        names[n] = f"{c['course']} lesson {i} {k}"
                        open(os.path.join(out, n), 'w').write(ex['code'])
    else:
        for t in data['terms']:
            if t.get('example', '').strip():
                n = f"term_{t['slug'].replace('-', '_')}.dart"
                names[n] = f"term {t['slug']}"
                open(os.path.join(out, n), 'w').write(t['example'])
    r = subprocess.run(['dart', 'analyze', '--format=machine', out], capture_output=True, text=True, cwd=project)
    errors = [l for l in (r.stdout + r.stderr).splitlines() if l.startswith('ERROR|')]
    for e in errors:
        parts = e.split('|')
        f = os.path.basename(parts[3]); print(f"{names.get(f, f)} (line {parts[4]}): {parts[7]}")
    failed += len(errors)
    print(f"{path}: {len(names)} dart snippets, {len(errors)} errors")
sys.exit(1 if failed else 0)
