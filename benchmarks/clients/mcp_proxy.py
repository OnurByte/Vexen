import sys,subprocess,threading,json,time
from pathlib import Path
name=sys.argv[1]
log=Path('/tmp/vexen-client-smoke-20261101')/(name+'-wire.jsonl')
p=subprocess.Popen(['/home/borat/Documents/ChatGPT/Vexen/target/debug/vexen','mcp','--stdio'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=sys.stderr.buffer)
lock=threading.Lock()
def record(direction,line):
 try:value=json.loads(line)
 except Exception:return
 with lock:
  with log.open('a') as f:f.write(json.dumps({'direction':direction,'message':value})+'\n')
def incoming():
 for line in sys.stdin.buffer:
  record('client_to_vexen',line);p.stdin.write(line);p.stdin.flush()
 p.stdin.close()
threading.Thread(target=incoming,daemon=True).start()
for line in p.stdout:
 record('vexen_to_client',line);sys.stdout.buffer.write(line);sys.stdout.buffer.flush()
sys.exit(p.wait())
