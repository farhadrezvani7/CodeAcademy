#!/usr/bin/env python3
"""Validate dictionary.json against term-slugs.txt and the content rules."""
import json, re, sys, os
here = os.path.dirname(os.path.abspath(__file__))
slugs = [l.split('|')[0] for l in open(os.path.join(here, 'term-slugs.txt')) if l.strip()]
terms = json.load(open(os.path.join(here, 'dictionary.json')))['terms']
cats = {'software-engineering', 'dart', 'flutter', 'professional-skills'}
errors = []
seen = [t['slug'] for t in terms]
if sorted(seen) != sorted(slugs): errors.append(f'slug set mismatch: missing={set(slugs)-set(seen)} extra={set(seen)-set(slugs)}')
for t in terms:
    s = t['slug']
    if t.get('category') not in cats: errors.append(f'{s}: bad category')
    for k in ('nameEn', 'nameFa', 'definition'):
        if not t.get(k, '').strip(): errors.append(f'{s}: missing {k}')
    if len(t.get('definition', '')) > 320: errors.append(f'{s}: definition too long ({len(t["definition"])}>320)')
    if t.get('example', '').count('\n') + 1 > 14: errors.append(f'{s}: example too long (>14 lines)')
    rel = t.get('related', [])
    if not 2 <= len(rel) <= 4 or s in rel or any(r not in slugs for r in rel): errors.append(f'{s}: related must be 2-4 valid other slugs')
    if re.search(r'[0-9]', re.sub(r'`[^`]*`', '', t.get('definition', '') + t.get('nameFa', ''))): errors.append(f'{s}: Latin digits in Persian prose')
print('\n'.join(errors) if errors else f'OK: {len(terms)} terms')
sys.exit(1 if errors else 0)
