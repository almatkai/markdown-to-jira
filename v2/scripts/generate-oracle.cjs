// Development-only oracle: executes the byte-exact OWN command and locked npm
// library. No rewritten JavaScript converter or expected-output implementation.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '..');
const own = path.join(root, '.work/own');
const ts = require(path.join(own, 'node_modules/typescript'));
const jira2md = require(path.join(own, 'node_modules/jira2md'));
for (const name of ['jira2md', 'marked']) {
  const lock = JSON.parse(fs.readFileSync(path.join(root, 'tests/fixtures/own/package-lock.json'))).packages['node_modules/'+name];
  const installed = name === 'marked' ? require.resolve(name, {paths:[path.dirname(fs.realpathSync(path.join(own,'node_modules/jira2md/index.js')))]}) : require.resolve(path.join(own,'node_modules/jira2md'));
  const entry = path.join(root, 'vendor', name, name === 'jira2md' ? 'index.js' : 'lib/marked.cjs');
  assert.deepEqual(fs.readFileSync(installed), fs.readFileSync(entry));
  console.log(`${name}: locked ${lock.version}, executable bytes verified`);
}
const source = fs.readFileSync(path.join(root, 'tests/fixtures/own/src/markdown-to-jira.ts'), 'utf8');
const js = ts.transpileModule(source, {compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,esModuleInterop:true}}).outputText;
let selected, events, fail;
const api = {
  getSelectedText: async()=>{ events.push(['selected']); if(fail==='selected') throw Error('synthetic failure'); return selected; },
  Clipboard: {
    paste:async(value)=>{events.push(['paste',value]);if(fail==='paste')throw Error('synthetic failure');},
    copy:async(value)=>{events.push(['copy',value]);if(fail==='copy')throw Error('synthetic failure');},
  },
  showHUD:async(value)=>{events.push(['hud',value]);if(fail==='hud')throw Error('synthetic failure');},
  showToast:async(value)=>{events.push(['toast',value]);},Toast:{Style:{Failure:'Failure'}},
};
const commandExports = {};
vm.runInNewContext(js,{exports:commandExports,require:(name)=>name==='@raycast/api'?api:name==='jira2md'?jira2md:(()=>{throw Error('unexpected import')})()},{filename:'actual-own-command.cjs'});
let seed=0x1a12c0de;
function random(n){seed^=seed<<13;seed^=seed>>>17;seed^=seed<<5;return (seed>>>0)%n;}
const terms=['\n','\r\n','\r','\u2028','\u2029'];
const atoms=['','plain','Қазақ 😀 日本','**bold**','*italic*','__bold__','***both***','*_mixed_*','****four****','_*abc*_','* a\n  * b','1. a\n   23. b','> quote\n>>nested','# H','####### H','H\n===','H\n---',' ~~deleted~~ ','~~edge~~','<del>x</del>','<ins>x</ins>','<sup>x</sup>','<sub>x</sub>','<ins><sub>x</sub></ins>','<del>x\ny</del>','`inline`','``','```\nx **y**\n```','```js\na\n```','```js\r\na\r\n```','[a](https://x/a(b))','![](x)','[![a](x)](y)','<https://x>','<raw html>','$& $1 $$ $` $\'','\n|A|\n|---|\n|B|\n\n','\n|A|B|\n|---|---|\n|C|D|\n\n','\n|||\n|---|\n\n','\n|A|\n|---|\n','\n| A |\n| :---: |\n| B |\n\n','**unfinished','[broken](a','<ins>unfinished','a\u0085b','a\u180eb','a\ufeffb','\t\t* item','  12.\nnext','*\nnext','[multi\nline](url\npath)','`multi\nline`','![a\nb](x\ny)'];
const cases = [...atoms,fs.readFileSync(path.join(root,'vendor/jira2md/test/test.md'),'utf8')];
for (const a of atoms) for (const t of terms) cases.push(a.replace(/\n/g,t));
for(const w of ['\t','\v','\f',' ','\xa0','\u1680','\u2000','\u200a','\u2028','\u2029','\u202f','\u205f','\u3000','\ufeff','\u0085','\u180e']) for(const a of atoms) cases.push(w+a+w,'*'+w+a+'*','1.'+w+a);
for(let n=0;n<12000;n++) {let s='';for(let i=0,len=1+random(5);i<len;i++)s+=atoms[random(atoms.length)]+terms[random(terms.length)];cases.push(s);}
const chars=['*','_','`','#','[',']','(',')','|','-','=','!','<','>','/','~','$','a',' ','\t','\n','\r','😀','\u2028','\ufeff'];
for(let n=0;n<3000;n++){let s='';for(let i=0,len=random(90);i<len;i++)s+=chars[random(chars.length)];cases.push(s);}
(async()=>{
  let lines=[]; let sourceFailures=0;
  for(const input of cases){
    let output,error;
    try {output=jira2md.to_jira(input);}catch(e){error=String(e);sourceFailures++;}
    selected=input;events=[];fail=undefined;await commandExports.default();
    if(error){assert.deepEqual(events.map(x=>x[0]),['selected','toast']);}
    else {assert.deepEqual(events.map(x=>x[0]),['selected','paste','copy','hud']);assert.equal(events[1][1],output);assert.equal(events[2][1],output);assert.equal(events[3][1],'Successfully converted Markdown to Jira markup!');}
    lines.push(JSON.stringify(error?{input,error}:{input,output}));
  }
  for(const f of ['selected','paste','copy','hud']){
    selected='**synthetic**';events=[];fail=f;await commandExports.default();
    const stages=['selected','paste','copy','hud'];assert.deepEqual(events.map(x=>x[0]),[...stages.slice(0,stages.indexOf(f)+1),'toast']);
    assert.equal(events.at(-1)[1].title,'Failed to convert Markdown to Jira markup');assert.equal(events.at(-1)[1].style,'Failure');
  }
  const corpus=lines.join('\n')+'\n';fs.writeFileSync(path.join(root,'tests/fixtures/oracle.jsonl'),corpus);
  fs.writeFileSync(path.join(root,'tests/fixtures/oracle-summary.json'),JSON.stringify({cases:cases.length,sourceFailures,sourceFailureStages:4,seed:'0x1a12c0de',sha256:crypto.createHash('sha256').update(corpus).digest('hex')},null,2)+'\n');
  console.log(`Executed actual command + pinned library: ${cases.length} cases, ${sourceFailures} library errors, all 4 failure stages`);
})().catch(e=>{console.error(e);process.exitCode=1;});
