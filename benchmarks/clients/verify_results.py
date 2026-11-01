#!/usr/bin/env python3
"""Audit recorded client evidence; this does not re-run client applications."""
import hashlib
import json
from pathlib import Path

RESULTS = Path(__file__).parent / 'results' / '2026-11-01'
EXPECTED_TOOLS = {'vexen_check', 'vexen_explain', 'vexen_outcome', 'vexen_record'}


def audit_wire(path, manifest, require_call=True):
    pending = {}
    handshake = False
    discovered = False
    success = False
    identities = []
    for line in path.read_text().splitlines():
        row = json.loads(line)
        msg = row['message']
        if row['direction'] == 'client_to_vexen':
            method = msg.get('method')
            if method in ('initialize', 'server/discover'):
                pending = {}
                params = msg.get('params', {})
                identities.append(params.get('clientInfo') or params.get('_meta', {}).get('io.modelcontextprotocol/clientInfo'))
            if 'id' in msg:
                pending[msg['id']] = msg
            continue
        request = pending.get(msg.get('id'), {})
        result = msg.get('result')
        if not isinstance(result, dict) or 'error' in msg:
            continue
        if request.get('method') in ('initialize', 'server/discover'):
            handshake = True
        elif request.get('method') == 'tools/list':
            if {t['name'] for t in result.get('tools', [])} == EXPECTED_TOOLS:
                discovered = True
        elif request.get('method') == 'tools/call':
            params = request.get('params', {})
            if params.get('name') != 'vexen_check':
                continue
            args = params.get('arguments', {})
            if args != {'repo_root': manifest['fixture_root'], 'paths': ['sample.py']}:
                continue
            envelope = result.get('structuredContent', {})
            if result.get('isError') is False and envelope.get('repo_id') == manifest['repo_id'] and envelope.get('status') == manifest['expected_status']:
                success = any(item.get('id') == manifest['record_id'] and manifest['expected_note'] in item.get('text', '') for item in envelope.get('items', [])) or success
    assert handshake, f'{path.name}: missing successful handshake'
    assert discovered, f'{path.name}: missing the four-tool discovery response'
    assert success or not require_call, f'{path.name}: missing matching tool call and actual applicable note response'
    return {'handshake': handshake, 'four_tools_discovered': discovered, 'check_returned_expected_note': success, 'client_identities': identities}


def main():
    for line in (RESULTS / 'SHA256SUMS').read_text().splitlines():
        digest, name = line.split('  ', 1)
        assert hashlib.sha256((RESULTS / name).read_bytes()).hexdigest() == digest, f'Evidence changed: {name}'
    manifest = json.loads((RESULTS / 'manifest.json').read_text())
    clients = manifest['clients']
    assert len(clients) == len({c['id'] for c in clients}) == 13
    assert not ({'claude', 'claude-code', 'codex', 'opencode'} & {c['id'] for c in clients}), 'Previously tested clients must not fill the new-client count'
    results = []
    for client in clients:
        path = RESULTS / client['evidence']
        if client['id'] == 'copilot':
            log = path.read_text()
            assert 'vexen_check (MCP: vexen)' in log
            assert manifest['fixture_root'] in log
            assert manifest['expected_note'] in log
            proof = {'actual_tool_invocation_and_returned_note_in_client_log': True}
        elif client['id'] == 'kilo':
            event = json.loads(path.read_text())
            part = event['part']
            assert event['type'] == 'tool_use' and part['tool'] == 'vexen_vexen_check'
            assert part['state']['status'] == 'completed'
            assert part['state']['input'] == {'repo_root': manifest['fixture_root'], 'paths': ['sample.py']}
            envelope = json.loads(part['state']['output'])
            assert envelope['repo_id'] == manifest['repo_id'] and envelope['status'] == manifest['expected_status']
            assert any(i['id'] == manifest['record_id'] for i in envelope['items'])
            proof = {'actual_completed_tool_call_and_matching_record_id_in_client_event': True, 'limitation': 'Kilo event keeps compact content, not the full structuredContent note'}
        else:
            proof = audit_wire(path, manifest)
        results.append({**client, 'result': 'tool-call-verified', 'proof': proof})
    additional = []
    for check in manifest['additional_checks']:
        path = RESULTS / check['evidence']
        if check['result'] == 'discovery-only':
            proof = audit_wire(path, manifest, require_call=False)
            assert not proof['check_returned_expected_note'], 'Update the report if an additional check now has a successful call'
        else:
            assert 'Login canceled' in path.read_text()
            proof = {'login_gate_observed': True, 'mcp_call_verified': False}
        additional.append({**check, 'proof': proof})
    print(json.dumps({'verified_new_clients': len(results), 'platform': manifest['platform'], 'vexen_version': manifest['vexen_version'], 'clients': results, 'additional_checks': additional}, indent=2))


if __name__ == '__main__':
    main()
