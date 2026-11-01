from http.server import ThreadingHTTPServer,BaseHTTPRequestHandler
from pathlib import Path
import json,time,re
ROOT=Path('/tmp/vexen-client-smoke-20261101')
ARGS={'repo_root':str(ROOT/'repo'),'paths':['sample.py']}
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  self.send_response(200);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(json.dumps({'object':'list','data':[{'id':'smoke-model','object':'model','created':0,'owned_by':'local-test'}]}).encode())
 def do_POST(self):
  body=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
  with (ROOT/'provider-requests.jsonl').open('a') as f:f.write(json.dumps({'path':self.path,'body':body})+'\n')
  gem='generateContent' in self.path or 'streamGenerateContent' in self.path
  if gem:
   decls=[x for t in body.get('tools',[]) for x in t.get('functionDeclarations',t.get('function_declarations',[]))]
   name=next((t['name'] for t in decls if 'vexen_check' in t.get('name','')),None)
   done=any('functionResponse' in p or 'function_response' in p for c in body.get('contents',[]) for p in c.get('parts',[]))
   part={'functionCall':{'name':name,'args':ARGS}} if name and not done else {'text':'Local test driver finished; inspect MCP wire log for actual result.'}
   result={'candidates':[{'content':{'role':'model','parts':[part]},'finishReason':'STOP','index':0}],'usageMetadata':{'promptTokenCount':10,'candidatesTokenCount':10,'totalTokenCount':20},'modelVersion':'smoke-model'}
   streaming='streamGenerateContent' in self.path
   data=('data: '+json.dumps(result)+'\n\n').encode() if streaming else json.dumps(result).encode()
  else:
   tools=body.get('tools',[])
   name=next((t.get('function',t).get('name') for t in tools if 'vexen_check' in t.get('function',t).get('name','')),None)
   messages=body.get('messages',[])
   names=[t.get('function',t).get('name') for t in tools]
   calls=[c.get('function',{}).get('name') for m in messages for c in m.get('tool_calls',[])]
   dynamic_args=None
   if not name and 'tool_search' in names:
    if 'tool_search' not in calls:
     name='tool_search';dynamic_args={'queries':['vexen_check']} if '/hermes/' in self.path else {'query':'vexen_check'}
    else:
     text=json.dumps([m for m in messages if m.get('role')=='tool'])
     found=re.findall(r'[A-Za-z0-9_./:-]*vexen[A-Za-z0-9_./:-]*check',text)
     target=next((v for v in found if v.startswith('mcp__')), next((v for v in found if v!='vexen_check'), 'mcp__vexen__vexen_check' if '/hermes/' in self.path else 'vexen_check'))
     if 'tool_describe' in names and 'tool_describe' not in calls:
      name='tool_describe';dynamic_args={'names':[target]}
     elif 'tool_call' not in calls:
      name='tool_call';dynamic_args={'calls':[{'name':target,'arguments':ARGS}]} if 'tool_describe' in names else {'name':target,'arguments':ARGS}
   if not name and 'search_tool_functions' in names:
    count=calls.count('search_tool_functions')
    text=json.dumps([m for m in messages if m.get('role')=='tool'])
    found=re.findall(r'[A-Za-z0-9_]+\.vexen_check',text)
    target=found[0] if found else 'vexen.vexen_check'
    if count==0:
     name='search_tool_functions';dynamic_args={'mode':'best_match','query':'vexen_check'}
    elif count==1:
     name='search_tool_functions';dynamic_args={'mode':'details','functions':[target]}
    elif 'run_typescript' not in calls:
     name='run_typescript';dynamic_args={'code':'async function main() { return await tools.'+target+'('+json.dumps(ARGS)+'); }'}
   done=('run_typescript' in calls) if 'search_tool_functions' in names else (('tool_call' in calls) if dynamic_args is not None or 'tool_search' in names else any(m.get('role')=='tool' for m in messages))
   msg={'role':'assistant','content':None,'tool_calls':[{'id':'smoke-call','type':'function','function':{'name':name,'arguments':json.dumps(dynamic_args if dynamic_args is not None else ({**ARGS, 'security_risk':'LOW'} if '/openhands/' in self.path else ARGS))}}]} if name and not done else {'role':'assistant','content':'Local test driver finished; inspect MCP wire log for actual result.'}
   finish='tool_calls' if name and not done else 'stop'
   result={'id':'smoke-response','object':'chat.completion','created':int(time.time()),'model':'smoke-model','choices':[{'index':0,'message':msg,'finish_reason':finish}],'usage':{'prompt_tokens':10,'completion_tokens':10,'total_tokens':20}}
   streaming=body.get('stream',False)
   if streaming:
    delta=dict(msg);delta.pop('role',None)
    if 'tool_calls' in delta:delta['tool_calls'][0]['index']=0
    chunk={k:v for k,v in result.items() if k not in ('choices','usage')};chunk['object']='chat.completion.chunk'
    first={**chunk,'choices':[{'index':0,'delta':{'role':'assistant',**delta},'finish_reason':None}]}
    last={**chunk,'choices':[{'index':0,'delta':{},'finish_reason':finish}],'usage':result['usage']}
    data=('data: '+json.dumps(first)+'\n\ndata: '+json.dumps(last)+'\n\ndata: [DONE]\n\n').encode()
   else:data=json.dumps(result).encode()
  self.send_response(200);self.send_header('Content-Type','text/event-stream' if streaming else 'application/json');self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)
ThreadingHTTPServer(('127.0.0.1',18765),Handler).serve_forever()
