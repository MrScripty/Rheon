import {adaptRecording} from './adapters.js';
const CATALOG_LIMIT=256*1024, RECORD_LIMIT=8*1024*1024, RECEIPT_LIMIT=2*1024*1024;
const SHA=/^[0-9a-f]{64}$/, HEAD=/^[0-9a-f]{40}$/;
const check=(ok,message)=>{if(!ok)throw Error(message);};
const current=live=>check(live(),'Output discovery superseded or closed');
const freeze=value=>{if(value&&typeof value==='object'){Object.values(value).forEach(freeze);Object.freeze(value);}return value;};
const decode=raw=>JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(raw));
const digest=async raw=>[...new Uint8Array(await crypto.subtle.digest('SHA-256',raw))].map(x=>x.toString(16).padStart(2,'0')).join('');
function reference(value,limit){check(value&&SHA.test(value.sha256)&&Number.isSafeInteger(value.bytes)&&value.bytes>0&&value.bytes<=limit,'Invalid output byte reference');}
export function validatePublishedCatalog(value){
 check(value?.schema==='rheon-producer-output-catalog-v1'&&Array.isArray(value.entries)&&value.entries.length<=128,'Unsupported or oversized producer catalog');
 const ids=new Set();let records=0;const receipts=new Map();
 for(const e of value.entries){
  check(e&&SHA.test(e.id)&&!ids.has(e.id)&&typeof e.label==='string'&&e.label.length>0&&e.label.length<=400&&typeof e.message==='string'&&e.message.length<=400,'Invalid producer entry');ids.add(e.id);
  check(['completed','incomplete','failed','unsupported'].includes(e.state),'Unknown producer completion state');
  if(e.provenance){
   reference(e.provenance.receipt,RECEIPT_LIMIT);receipts.set(e.provenance.receipt.sha256,e.provenance.receipt.bytes);
   if(e.provenance.source_head!==undefined)check(HEAD.test(e.provenance.source_head),'Invalid producer source identity');
   if(e.provenance.pipeline_receipt){reference(e.provenance.pipeline_receipt,RECEIPT_LIMIT);receipts.set(e.provenance.pipeline_receipt.sha256,e.provenance.pipeline_receipt.bytes);}
  }
  if(e.record){
   check(e.state==='completed'&&HEAD.test(e.provenance?.source_head)&&['rheon-obstacle-flow-records-v1','rheon-rigid-motion-json-v1'].includes(e.format),'Unavailable or unsupported completed recording');
   reference(e.record,RECORD_LIMIT);records+=e.record.bytes;
  }
 }
 check(records<=32*1024*1024&&[...receipts.values()].reduce((a,b)=>a+b,0)<=32*1024*1024,'Producer catalog aggregate byte limit exceeded');
 return freeze(structuredClone(value));
}
export async function fetchBounded(url,limit,{isCurrent=()=>true,fetcher=fetch}={}){
 current(isCurrent);
 const response=await fetcher(url,{cache:'no-store',redirect:'error'});
 try{
  current(isCurrent);check(response.ok,'Producer output request failed ('+response.status+')');
  const length=response.headers.get('content-length');
  check(length===null||(/^\d+$/.test(length)&&Number(length)<=limit),'Producer response exceeds its byte limit');
  check(response.body,'Producer response body unavailable');
 }catch(error){try{await response.body?.cancel();}catch{}throw error;}
 const reader=response.body.getReader();const chunks=[];let count=0;
 try{
  for(;;){current(isCurrent);const {done,value}=await reader.read();current(isCurrent);if(done)break;count+=value.byteLength;check(count<=limit,'Producer response exceeds its byte limit');chunks.push(value);}
 }catch(error){try{await reader.cancel();}catch{}throw error;}finally{reader.releaseLock();}
 const raw=new Uint8Array(count);let offset=0;for(const chunk of chunks){raw.set(chunk,offset);offset+=chunk.byteLength;}return raw;
}
export async function readPublishedCatalog(url,options){return validatePublishedCatalog(decode(await fetchBounded(url,CATALOG_LIMIT,options)));}
const blobURL=(base,ref)=>new URL('./blobs/'+ref.sha256+'.json',base);
async function readBlob(base,ref,limit,options){
 const raw=await fetchBounded(blobURL(base,ref),limit,options);
 check(raw.byteLength===ref.bytes&&await digest(raw)===ref.sha256,'Producer output bytes differ from catalog snapshot');current(options.isCurrent);return decode(raw);
}
export async function readPublishedRecording(base,entry,{isCurrent=()=>true,fetcher=fetch}={}){
 check(entry?.state==='completed'&&entry.record&&entry.provenance,'Choose a completed producer recording');
 const options={isCurrent,fetcher},receipt=await readBlob(base,entry.provenance.receipt,RECEIPT_LIMIT,options);
 check(receipt?.source_head===entry.provenance.source_head,'Producer source identity differs from original receipt');
 if(entry.format==='rheon-obstacle-flow-records-v1'){
  check(receipt.schema==='rheon-obstacle-flow-qualification-v1'&&receipt.clean===true&&!receipt.failure&&receipt.qualified!==false&&receipt.records_sha256===entry.record.sha256,'Obstacle output completion differs from receipt');
 }else{
  const filename=entry.label.split('/').at(-1);
  check(entry.format==='rheon-rigid-motion-json-v1'&&receipt.qualified===true&&!receipt.failure&&Array.isArray(receipt.fixtures)&&receipt.fixtures.some(f=>f.name+'.stdout.json'===filename&&f.stdout_sha256===entry.record.sha256)&&receipt.evidence_sha256?.[filename]===entry.record.sha256,'Rigid output completion differs from receipt');
 }
 if(entry.provenance.pipeline_receipt){
  const parent=await readBlob(base,entry.provenance.pipeline_receipt,RECEIPT_LIMIT,options);
  const path=entry.label.split('/').slice(1,-1).join('/')+'/qualification.json';
  check(parent.qualified===true&&parent.source_clean===true&&!parent.failure&&parent.source_head===receipt.source_head&&parent.evidence_sha256?.[path]===entry.provenance.receipt.sha256,'Producer pipeline completion differs from receipt');
 }
 const raw=await readBlob(base,entry.record,RECORD_LIMIT,options);current(isCurrent);
 return adaptRecording(raw,{name:entry.label,sha256:entry.record.sha256,source:'Producer recorded output',source_head:entry.provenance.source_head,receipt_sha256:entry.provenance.receipt.sha256});
}
