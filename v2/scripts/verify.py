#!/usr/bin/env python3
"""Full beta schema and retained-source/package checks (requires jsonschema 4.23)."""
from pathlib import Path
import base64
import hashlib
import io
import json
import tarfile
import jsonschema

ROOT = Path(__file__).resolve().parents[1]
def digest(data):
    return hashlib.sha256(data).hexdigest()
def read(name):
    return json.loads((ROOT/name).read_text())

provenance = read('provenance.json')
for name, expected in provenance['files'].items():
    assert digest((ROOT/name).read_bytes()) == expected, name
for name, metadata in provenance['npm'].items():
    archive = (ROOT/'vendor'/name/'package.tgz').read_bytes()
    algorithm, expected = metadata['integrity'].split('-',1)
    assert base64.b64encode(hashlib.new(algorithm,archive).digest()).decode() == expected
    with tarfile.open(fileobj=io.BytesIO(archive), mode='r:gz') as tar:
        for member in tar.getmembers():
            if member.isfile():
                relative = Path(member.name).relative_to('package')
                assert (ROOT/'vendor'/name/relative).read_bytes() == tar.extractfile(member).read()
lock = read('tests/fixtures/own/package-lock.json')['packages']
for name, metadata in provenance['npm'].items():
    assert lock['node_modules/'+name] == metadata
manifest = read('manifest/extension-v2.json')
schema = read('tests/fixtures/beta/extension-v2.schema.json')
jsonschema.Draft202012Validator.check_schema(schema)
jsonschema.Draft202012Validator(schema).validate(manifest)
assert manifest['id'] == 'com.tezbar.native.markdown-to-jira'
assert manifest['replaces'] == ['raycast.markdown-to-jira']
assert manifest['runtime'] == 'rust'
assert manifest['platforms'] == ['macOS','Windows']
original = read('tests/fixtures/own/package.json')['commands'][0]
for key in ['name','title','description','mode','platforms']:
    assert manifest['commands'][0][key] == original[key]
assert manifest['verification'] and all(x['state']=='not-run' for x in manifest['verification'])
icon = ROOT/'manifest'/manifest['icon'].lstrip('/')
assert len(icon.read_bytes()) == provenance['icon']['bytes']
assert digest(icon.read_bytes()) == provenance['icon']['sha256']
assert icon.read_bytes() == (ROOT/'tests/fixtures/own/assets/icon.png').read_bytes()
assert icon.read_bytes() == (ROOT/'tests/fixtures/upstream/extensions/markdown-to-jira/assets/icon.png').read_bytes()
summary = read('tests/fixtures/oracle-summary.json')
assert digest((ROOT/'tests/fixtures/oracle.jsonl').read_bytes()) == summary['sha256']
assert sum(1 for _ in (ROOT/'tests/fixtures/oracle.jsonl').open()) == summary['cases']
assert read('tests/fixtures/upstream/extensions/markdown-to-jira/package.json')['author'] == 'theo_brlle'
for dependency in read('licenses/rust-dependencies.json'):
    assert dependency['notices']
    for notice in dependency['notices']:
        assert digest((ROOT/notice).read_bytes()) == dependency['noticeSha256'][notice]
# Check V1 files when this is a repository checkout, but never require a parent repo.
if (ROOT.parent/'.git').exists():
    baseline = read('tests/fixtures/v1-baseline.json')
    for name, expected in baseline.items():
        if name == 'README.md':
            assert digest((ROOT.parent/name).read_bytes().removesuffix(b'\n\n[Rust Native V2 port and integration](v2/README.md).\n')) == expected
        else:
            assert digest((ROOT.parent/name).read_bytes()) == expected, name
print(f"Full beta schema, all source/npm integrity, licenses, icon, {summary['cases']} oracle cases and available V1 baseline: pass")
