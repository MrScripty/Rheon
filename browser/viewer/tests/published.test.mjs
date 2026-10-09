import {test} from 'node:test';
import assert from 'node:assert/strict';
import {webcrypto} from 'node:crypto';
import {validatePublishedCatalog,fetchBounded,readPublishedRecording} from '../published.js';
if(!globalThis.crypto)globalThis.crypto=webcrypto;
const sha=async bytes=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(x=>x.toString(16).padStart(2,'0')).join('');
async function fixture(){
 const record=new TextEncoder().encode(JSON.stringify({schema:'rheon-obstacle-flow-records-v1',shear_cases:[{id:'saved',dt:.125,centers:[.25],density:2,viscosity:1,frames:[{step:0,velocity:[1]}]}]}));
 const hash=await sha(record);const receipt=new TextEncoder().encode(JSON.stringify({schema:'rheon-obstacle-flow-qualification-v1',clean:true,source_head:'1'.repeat(40),records_sha256:hash}));
 const entry={id:'2'.repeat(64),label:'saved/records.json',state:'completed',message:'recorded',format:'rheon-obstacle-flow-records-v1',record:{sha256:hash,bytes:record.length},provenance:{source_head:'1'.repeat(40),receipt:{sha256:await sha(receipt),bytes:receipt.length}}};
 const blobs=new Map([[hash,record],[entry.provenance.receipt.sha256,receipt]]);
 return {entry,blobs,fetcher:async url=>new Response(blobs.get(url.pathname.split('/').at(-1).slice(0,-5)))};
}
test('published recording checks original receipt and preserves exact native values',async()=>{
 const f=await fixture(),data=await readPublishedRecording(new URL('https://example.test/viewer/outputs/catalog.json'),f.entry,{fetcher:f.fetcher});
 assert.deepEqual(data.cases[0].frames[0].profile,[[.25,1]]);assert.equal(data.provenance.source_head,'1'.repeat(40));assert(Object.isFrozen(data));
});
test('changed bytes and changed producer identity refuse',async()=>{
 const f=await fixture();f.blobs.set(f.entry.record.sha256,new TextEncoder().encode('{}'));
 await assert.rejects(readPublishedRecording(new URL('https://example.test/viewer/outputs/catalog.json'),f.entry,{fetcher:f.fetcher}),/bytes differ/);
 const g=await fixture();g.entry.provenance.source_head='3'.repeat(40);
 await assert.rejects(readPublishedRecording(new URL('https://example.test/viewer/outputs/catalog.json'),g.entry,{fetcher:g.fetcher}),/source identity/);
});
test('incomplete states, duplicate IDs and invalid byte references cannot be selected',async()=>{
 const f=await fixture(),catalog={schema:'rheon-producer-output-catalog-v1',entries:[f.entry]};assert(Object.isFrozen(validatePublishedCatalog(catalog)));
 for(const change of [c=>c.entries.push(c.entries[0]),c=>c.entries[0].state='incomplete',c=>c.entries[0].record.bytes=8*1024*1024+1,c=>c.schema='future',c=>c.entries[0].provenance.source_head='latest']){const c=structuredClone(catalog);change(c);assert.throws(()=>validatePublishedCatalog(c));}
});
test('stream limits and stale reads refuse without concatenating oversized data',async()=>{
 await assert.rejects(fetchBounded('unused',2,{fetcher:async()=>new Response('too large')}),/byte limit/);
 await assert.rejects(fetchBounded('unused',2,{fetcher:async()=>new Response('ok',{headers:{'content-length':'3'}})}),/byte limit/);
 let live=true;await assert.rejects(fetchBounded('unused',8,{isCurrent:()=>live,fetcher:async()=>{live=false;return new Response('ok');}}),/superseded/);
});
