#!/usr/bin/env python3
"""Validate lesson seed files: python3 validate_lessons.py lessons/*.json"""
import json, sys, re
slugs = {l.split('|')[0] for l in open(__file__.rsplit('/',1)[0] + '/term-slugs.txt') if l.strip()} if '/' in __file__ else {l.split('|')[0] for l in open('term-slugs.txt') if l.strip()}
catalog = {c['slug']: c for c in json.load(open((__file__.rsplit('/',1)[0] + '/' if '/' in __file__ else '') + 'catalog.json'))['courses']}
LANGS = {'dart','bash','yaml','text','json','sql','markdown'}
errors = []
seen = {}
for path in sys.argv[1:]:
    try:
        data = json.load(open(path))
    except Exception as e:
        errors.append(f'{path}: invalid JSON: {e}'); continue
    for c in data.get('courses', []):
        slug = c.get('course')
        if slug not in catalog: errors.append(f'{path}: unknown course {slug}'); continue
        if slug in seen: errors.append(f'{path}: {slug} duplicated (also in {seen[slug]})')
        seen[slug] = path
        ls = c.get('lessons', [])
        if not 3 <= len(ls) <= 4: errors.append(f'{slug}: needs 3-4 lessons, has {len(ls)}')
        for i, l in enumerate(ls):
            p = f'{slug}#{i+1}'
            for k in ['title','what','why','tryIt']:
                if not isinstance(l.get(k), str) or len(l[k].strip()) < 10 and k != 'title': errors.append(f'{p}: bad {k}')
            if not isinstance(l.get('estimatedMinutes'), int) or not 10 <= l['estimatedMinutes'] <= 60: errors.append(f'{p}: estimatedMinutes 10..60')
            for k in ['simpleExample','realExample']:
                ex = l.get(k)
                if not isinstance(ex, dict) or ex.get('language') not in LANGS or not ex.get('code') or not ex.get('explanation'): errors.append(f'{p}: bad {k}')
            for k in ['mistakes','bestPractices']:
                v = l.get(k)
                if not isinstance(v, list) or not 2 <= len(v) <= 6 or not all(isinstance(s,str) and s for s in v): errors.append(f'{p}: bad {k}')
            rt = l.get('relatedTerms')
            if not isinstance(rt, list) or not 1 <= len(rt) <= 5: errors.append(f'{p}: relatedTerms 1..5')
            else:
                for t in rt:
                    if t not in slugs: errors.append(f'{p}: unknown term slug {t}')
            if re.search(r'[0-9]', l.get('title','')): errors.append(f'{p}: use Persian digits in title')

LIMITS = {'what': 520, 'why': 420, 'tryIt': 300}
def prose(t): return re.sub(r'`[^`]*`', '', t)
for path in sys.argv[1:]:
    try: data = json.load(open(path))
    except Exception: continue
    for c in data.get('courses', []):
        for i, l in enumerate(c.get('lessons', [])):
            p = f"{c.get('course')}#{i+1}"
            for k, lim in LIMITS.items():
                if isinstance(l.get(k), str) and len(l[k]) > lim: errors.append(f'{p}: {k} too long ({len(l[k])}>{lim})')
            for k in ['mistakes', 'bestPractices']:
                v = l.get(k) or []
                if len(v) != 3: errors.append(f'{p}: {k} must have exactly 3 items')
                for s_ in v:
                    if len(s_) > 140: errors.append(f'{p}: {k} item too long ({len(s_)}>140)')
            for k, lim in [('simpleExample', 18), ('realExample', 32)]:
                ex = l.get(k) or {}
                n = ex.get('code', '').count('\n') + 1
                if n > lim: errors.append(f'{p}: {k} code too long ({n}>{lim} lines)')
                if len(ex.get('explanation', '')) > 260: errors.append(f'{p}: {k} explanation too long')
            texts = [l.get('title',''), l.get('what',''), l.get('why',''), l.get('tryIt','')] + (l.get('mistakes') or []) + (l.get('bestPractices') or []) + [l.get(k,{}).get('explanation','') for k in ('simpleExample','realExample')]
            for t in texts:
                m = re.search(r'[0-9]+', prose(t))
                if m: errors.append(f'{p}: Latin digits in Persian prose: "{m.group()}" (use ۰-۹, or put code in backticks)'); break

print('\n'.join(errors) if errors else f'OK: {len(seen)} courses, {sum(1 for _ in seen)} checked')
sys.exit(1 if errors else 0)
