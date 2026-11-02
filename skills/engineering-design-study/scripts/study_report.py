#!/usr/bin/env python3
"""Validate a study evidence manifest and render a portable review and journal draft."""
import argparse
import hashlib
import html
import json
import math
from pathlib import Path
from urllib.parse import quote


def require(condition, message):
    if not condition:
        raise ValueError(message)


def number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def validate(path):
    path = Path(path).resolve()
    data = json.loads(path.read_text())
    require(isinstance(data, dict) and data.get('version') == 1, 'expected manifest version 1')
    for key in ('title', 'scope', 'source_revision', 'next_experiment'):
        require(isinstance(data.get(key), str) and data[key].strip(), f'{key}: nonempty text required')
    require(data.get('status') in ('in_progress', 'awaiting_review', 'blocked', 'complete'), 'invalid study status')
    require(isinstance(data.get('review_gate', ''), str), 'review_gate must be text')
    for key in ('trace_limitations', 'artifacts', 'experiments', 'checks', 'assumptions', 'blockers'):
        require(isinstance(data.get(key), list), f'{key}: list required')
    for key in ('trace_limitations', 'assumptions', 'blockers'):
        require(all(isinstance(x, str) for x in data[key]), f'{key}: text entries required')
    ids = set()
    for artifact in data['artifacts']:
        require(isinstance(artifact, dict), 'artifact must be an object')
        for key in ('id', 'path', 'label'):
            require(isinstance(artifact.get(key), str) and artifact[key], f'artifact {key} required')
        require(artifact['id'] not in ids, 'duplicate artifact id')
        ids.add(artifact['id'])
        relative = Path(artifact['path'])
        target = (path.parent / relative).resolve()
        require(not relative.is_absolute() and target.is_relative_to(path.parent), 'artifact escapes study directory')
        require(target.is_file(), f'missing artifact: {relative}')
        if 'sha256' in artifact:
            actual = hashlib.sha256(target.read_bytes()).hexdigest()
            require(actual == artifact['sha256'], f'hash mismatch: {relative}')
    for record in data['experiments'] + data['checks']:
        require(isinstance(record, dict), 'evidence record must be an object')
        require(isinstance(record.get('evidence'), list), 'evidence IDs required')
        require(all(isinstance(x, str) and x in ids for x in record['evidence']), 'unknown evidence artifact')
    experiments = set()
    for record in data['experiments']:
        for key in ('id', 'question', 'expected', 'observed', 'inference', 'failures', 'correction', 'next_experiment'):
            require(isinstance(record.get(key), str), f'experiment {key}: text required')
        require(record['id'] and record['id'] not in experiments, 'empty or duplicate experiment id')
        experiments.add(record['id'])
    required = []
    for check in data['checks']:
        require(isinstance(check.get('name'), str) and check['name'], 'check name required')
        require(check.get('status') in ('pass', 'fail', 'unsupported', 'pending'), 'invalid check status')
        require(isinstance(check.get('detail', ''), str), 'check detail must be text')
        require(isinstance(check.get('required_for_acceptance', True), bool), 'required_for_acceptance must be boolean')
        if check.get('required_for_acceptance', True):
            required.append(check)
        if check['status'] in ('pass', 'fail'):
            require(bool(check['evidence']), 'pass/fail needs evidence')
        numeric = any(key in check for key in ('measured', 'expected', 'tolerance'))
        if numeric:
            require(check['status'] in ('pass', 'fail'), 'unsupported/pending cannot claim measured numerical results')
            require(number(check.get('expected')) and number(check.get('measured')), 'finite expected/measured required')
            require(isinstance(check.get('unit'), str) and check['unit'], 'numerical check unit required')
            tolerance = check.get('tolerance')
            require(isinstance(tolerance, dict) and tolerance and set(tolerance) <= {'absolute', 'relative'}, 'invalid tolerance')
            require(all(number(v) and v >= 0 for v in tolerance.values()), 'tolerances must be finite and nonnegative')
            allowed = max(tolerance.get('absolute', 0), tolerance.get('relative', 0) * abs(check['expected']))
            passed = abs(check['measured'] - check['expected']) <= allowed
            require(passed == (check['status'] == 'pass'), f'numerical claim disagrees with tolerance: {check["name"]}')
    if data['status'] == 'complete':
        require(required and all(x['status'] == 'pass' for x in required) and not data['blockers'],
                'complete requires required checks passing and no blockers')
    return data


def documents(data):
    esc = lambda value: html.escape(str(value), quote=True)
    artifact_map = {a['id']: a for a in data['artifacts']}

    def links(ids):
        return ' · '.join(f'<a href="{quote(artifact_map[i]["path"], safe="/")}">{esc(artifact_map[i]["label"])}</a>' for i in ids)

    def paragraph(label, text):
        return f'<p><b>{esc(label)}:</b> {esc(text)}</p>'

    def listing(title, values):
        return f'<h2>{title}</h2><ul>' + ''.join(f'<li>{esc(v)}</li>' for v in values) + '</ul>'

    css = 'body{max-width:1050px;margin:40px auto;padding:0 24px;font:17px/1.6 system-ui;color:#193449;background:#f5f7fa}h1,h2{line-height:1.25}article{background:white;padding:20px;margin:20px 0;border-left:5px solid #b4822b}table{border-collapse:collapse;width:100%}th,td{text-align:left;padding:10px;border-bottom:1px solid #ccd6dd}td:nth-child(2){white-space:nowrap}a{color:#17628a}img{max-width:100%;max-height:650px}pre{white-space:pre-wrap}p,td{overflow-wrap:anywhere}'
    parts = [f'<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>{esc(data["title"])}</title><style>{css}</style><main><h1>{esc(data["title"])}</h1>']
    md = [f'# {data["title"]} — journal draft', '', 'Generated from the manifest; merge into the authored journal, preserving prior entries.', '']
    for key in ('status', 'scope', 'source_revision', 'review_gate', 'next_experiment'):
        value = data.get(key, '')
        parts.append(paragraph(key.replace('_', ' ').capitalize(), value))
        md += [f'**{key}:** {value}', '']
    for title, key in [('Assumptions', 'assumptions'), ('Blockers', 'blockers'), ('Trace limitations', 'trace_limitations')]:
        parts.append(listing(title, data[key]))
        md += [f'## {title}', ''] + [f'- {v}' for v in data[key]] + ['']
    parts.append('<h2>Checks</h2><table><tr><th>Check</th><th>Status</th><th>Result / evidence</th></tr>')
    md += ['## Checks', '']
    for check in data['checks']:
        detail = check.get('detail', '')
        if 'measured' in check:
            detail += f' Expected {check["expected"]}; measured {check["measured"]} {check["unit"]}; tolerance {check["tolerance"]}.'
        parts.append(f'<tr><td>{esc(check["name"])}</td><td>{esc(check["status"])}</td><td>{esc(detail)}<br>{links(check["evidence"])}</td></tr>')
        md += [f'- {check["name"]}: **{check["status"]}**. {detail} Evidence: {", ".join(check["evidence"])}']
    parts.append('</table><h2>Experiments</h2>')
    for experiment in data['experiments']:
        parts.append(f'<article><h3>{esc(experiment["id"])}</h3>')
        md += ['', f'## Experiment {experiment["id"]}', '']
        for key in ('question', 'expected', 'observed', 'inference', 'failures', 'correction', 'next_experiment'):
            parts.append(paragraph(key.replace('_', ' ').capitalize(), experiment[key]))
            md += [f'**{key}:** {experiment[key]}', '']
        parts.append(links(experiment['evidence']) + '</article>')
        md += ['Evidence: ' + ', '.join(experiment['evidence']), '']
    parts.append('<h2>Retained artifacts</h2>')
    md += ['## Retained artifacts', '']
    for artifact in data['artifacts']:
        parts.append('<p>' + links([artifact['id']]) + '</p>')
        md.append(f'- [{artifact["label"]}](<{quote(artifact["path"], safe="/")}>) — {artifact["id"]}')
        if Path(artifact['path']).suffix.lower() in ('.png', '.jpg', '.jpeg', '.webp'):
            parts.append(f'<img src="{quote(artifact["path"], safe="/")}" alt="{esc(artifact["label"])}">')
    parts.append('</main></html>')
    return '\n'.join(parts), '\n'.join(md) + '\n'


def render(path, outputs, replace=False):
    path = Path(path).resolve()
    data = validate(path)
    targets = [Path(p).resolve() for p in outputs]
    require(len(set(targets)) == 2, 'output paths must differ')
    protected = {path} | {(path.parent / a['path']).resolve() for a in data['artifacts']}
    for target in targets:
        require(target.parent == path.parent, 'outputs must be beside the manifest for portable links')
        require(target not in protected, 'cannot overwrite manifest or evidence artifact')
        require(replace or not target.exists(), f'output exists: {target}; use --replace only for generated files')
    for target, content in zip(targets, documents(data)):
        with target.open('w' if replace else 'x') as stream:
            stream.write(content)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('check', 'render'))
    parser.add_argument('manifest', type=Path)
    parser.add_argument('--html', type=Path)
    parser.add_argument('--journal', type=Path)
    parser.add_argument('--replace', action='store_true')
    args = parser.parse_args()
    try:
        if args.command == 'check':
            data = validate(args.manifest)
            print(f'Valid evidence manifest: {data["title"]} ({data["status"]}); content claims still require review.')
        else:
            require(args.html and args.journal, 'render requires --html and --journal')
            render(args.manifest, [args.html, args.journal], args.replace)
            print(f'Rendered {args.html} and {args.journal}')
    except (ValueError, OSError, TypeError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
