#!/usr/bin/env python3
from __future__ import annotations
import re
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
BASELINE='ccf2263104455681cc07ecceda2569c4f7ce0de9'
ALLOWED=(
 'database/schema.sql','database/seed/reference_data.sql','database/tests/invariants.sql',
 'scripts/verify_schema.py','scripts/verify_phase06.py','scripts/phase06_policy.py',
 'src-tauri/src/','src/','tests/','playwright.phase06.config.ts','package.json','package-lock.json',
 'src-tauri/Cargo.toml','src-tauri/Cargo.lock','.github/workflows/','docs/architecture/','docs/PHASE-06-REPORT.md',
)
BANNED_SUFFIXES=('.zip','.tar','.tgz','.gz','.7z','.rar','.b64','.base64','.chunk','.chunks')
BANNED_PARTS=('payload','transport-workflow','write-workflow','temporary-workflow')

def run(*args:str)->str:
 return subprocess.check_output(args,cwd=ROOT,text=True).strip()

def fail(message:str)->None:
 raise SystemExit(f'PHASE06 POLICY FAILED: {message}')

def main()->int:
 changed=[line for line in run('git','diff','--name-only',f'{BASELINE}...HEAD').splitlines() if line]
 outside=[p for p in changed if not any(p==prefix or p.startswith(prefix) for prefix in ALLOWED)]
 if outside: fail(f'ownership violation: {outside}')
 bad=[p for p in changed if p.lower().endswith(BANNED_SUFFIXES) or any(part in p.lower() for part in BANNED_PARTS) or '/helpers.' in p.lower() or '/helper.' in p.lower()]
 if bad: fail(f'helper/payload/archive/chunk artifact forbidden: {bad}')
 for path in changed:
  candidate=ROOT/path
  if candidate.is_file() and candidate.stat().st_size>2_000_000: fail(f'oversized source artifact: {path}')
  if candidate.name.startswith('.env'): fail(f'environment secret file forbidden: {path}')
 for workflow in (ROOT/'.github/workflows').glob('*.yml'):
  text=workflow.read_text(encoding='utf-8')
  if 'permissions:' in text and not re.search(r'permissions:\s*\n\s*contents:\s*read',text): fail(f'workflow permissions are not contents: read: {workflow.name}')
 source='\n'.join(p.read_text(encoding='utf-8',errors='ignore') for root in [ROOT/'src',ROOT/'src-tauri/src'] for p in root.rglob('*') if p.is_file())
 for token in ['reqwest::','hyper::Client','ureq::','XMLHttpRequest','WebSocket(','axios.']:
  if token in source: fail(f'runtime network client forbidden: {token}')
 for version in range(1,6):
  paths=list((ROOT/'database/migrations').glob(f'{version:04d}_*.sql'))
  if len(paths)!=1: fail(f'accepted migration {version:04d} missing or duplicated')
 print(f'PHASE06 POLICY PASS: {len(changed)} owned paths; frozen migrations 0001-0005; contents read workflows')
 return 0
if __name__=='__main__': raise SystemExit(main())
