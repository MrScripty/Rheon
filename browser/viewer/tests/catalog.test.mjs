import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readRecordingFolder,FOLDER_LIMITS} from '../catalog.js';

const column={cases:[{name:'saved',snapshots:[{time:0,profiles:[[.25,1],[.75,2]],energy:3,bulk:4}]}]};
function file(path,raw=column){
  const bytes=new TextEncoder().encode(typeof raw==='string'?raw:JSON.stringify(raw));
  return {name:path.split('/').at(-1),webkitRelativePath:path,size:bytes.length,
    async arrayBuffer(){return bytes.buffer.slice(0);}};
}
test('folder catalog retains immutable native values and exposes unsupported/refused files',async()=>{
  const input=[file('outputs/z/records.json'),file('outputs/unknown.json',{schema:'future'}),file('outputs/a/results.json'),file('outputs/strain.tsv','stored rows')];
  let unsupportedReads=0;input[3].arrayBuffer=()=>{unsupportedReads++;throw Error('must not read');};
  const out=await readRecordingFolder(input);
  assert.equal(out.schema,'rheon-local-record-catalog-v1');assert(Object.isFrozen(out.entries));
  assert.deepEqual(out.entries.map(e=>e.path),['outputs/a/results.json','outputs/strain.tsv','outputs/unknown.json','outputs/z/records.json']);
  assert.deepEqual(out.entries.map(e=>e.status),['ready','unsupported','refused','ready']);
  assert.equal(unsupportedReads,0);assert.equal(out.entries[0].data.cases[0].frames[0].time,0);
  assert.deepEqual(out.entries[0].data.cases[0].frames[0].profile,[[.25,1],[.75,2]]);
  assert.equal(out.entries[0].data.provenance.sha256,out.entries[3].data.provenance.sha256);
  assert(Object.isFrozen(out.entries[0].data));assert.equal(JSON.stringify(column),JSON.stringify({cases:[{name:'saved',snapshots:[{time:0,profiles:[[.25,1],[.75,2]],energy:3,bulk:4}]}]}));
});
test('invalid rosters, paths and aggregate quotas refuse before any file read',async()=>{
  let reads=0;const f=file('outputs/results.json');f.arrayBuffer=()=>{reads++;throw Error('must not read');};
  for(const roster of [[],{length:FOLDER_LIMITS.files+1},[f,{...f}],[f,{...f,webkitRelativePath:'../escape.json'}],[f,{...f,webkitRelativePath:'/absolute.json'}],[f,{...f,webkitRelativePath:'outputs/other.json',size:FOLDER_LIMITS.bytes}],[f,{...f,webkitRelativePath:'outputs/other.json',size:NaN}]]) {
    await assert.rejects(readRecordingFolder(roster));
  }
  assert.equal(reads,0);
});
test('oversized and empty records are visible refusals without reading',async()=>{
  const out=await readRecordingFolder([{...file('outputs/large.json'),size:FOLDER_LIMITS.recordBytes+1},{...file('outputs/empty.json'),size:0}]);
  assert(out.entries.every(e=>e.status==='refused'));
});
test('stale or detached selection halts admission before subsequent reads',async()=>{
  let live=true,secondReads=0;const a=file('outputs/a.json'),b=file('outputs/b.json');
  const read=a.arrayBuffer;a.arrayBuffer=async()=>{const raw=await read();live=false;return raw;};
  b.arrayBuffer=()=>{secondReads++;throw Error('must not read');};
  await assert.rejects(readRecordingFolder([a,b],{isCurrent:()=>live}),/superseded/);
  assert.equal(secondReads,0);
});
test('byte length must match the declared file size',async()=>{
  const f=file('outputs/results.json');f.size=1;
  const out=await readRecordingFolder([f]);assert.equal(out.entries[0].status,'refused');
  assert.match(out.entries[0].message,/size changed/);
});
