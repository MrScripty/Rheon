import {readRecording} from './adapters.js';

export const FOLDER_LIMITS = Object.freeze({files:128, bytes:32*1024*1024, recordBytes:8*1024*1024, pathLength:1024});
const require = (ok, message) => { if (!ok) throw Error(message); };
const current = isCurrent => require(isCurrent(), 'Folder import superseded or closed');

/** Local, in-memory admission of unchanged files. No paths are fetched, no
 * solver or export runs, and SHA labels identify bytes rather than qualification.
 * Validate the entire selection before reading any file; admit sequentially.
 */
export async function readRecordingFolder(files, {isCurrent=()=>true}={}) {
  current(isCurrent);
  require(Number.isSafeInteger(files?.length) && files.length>0 && files.length<=FOLDER_LIMITS.files,
    'Choose an output folder with 1–128 files');
  let bytes=0; const paths=new Set(); const selected=[];
  for (const file of Array.from(files)) {
    current(isCurrent);
    const path=file.webkitRelativePath || file.name;
    require(typeof path==='string' && path.length>0 && path.length<=FOLDER_LIMITS.pathLength &&
      !/[\x00-\x1f\\]/.test(path) && !path.startsWith('/') &&
      path.split('/').every(part=>part && part!=='.' && part!=='..'), 'Invalid folder-relative path');
    require(!paths.has(path), 'Duplicate folder-relative path'); paths.add(path);
    require(Number.isSafeInteger(file.size) && file.size>=0, 'Invalid folder file size');
    bytes+=file.size;
    require(bytes<=FOLDER_LIMITS.bytes, 'Output folder exceeds the 32 MiB total limit');
    selected.push({file,path});
  }
  selected.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0);
  const entries=[];
  for (const {file,path} of selected) {
    current(isCurrent);
    let entry={path,bytes:file.size};
    if (!/\.json$/i.test(path)) {
      entry={...entry,status:'unsupported',message:'Only native recording JSON is supported'};
    } else if (file.size===0 || file.size>FOLDER_LIMITS.recordBytes) {
      entry={...entry,status:'refused',message:'Recording JSON must contain 1 byte to 8 MiB'};
    } else {
      try {
        const data=await readRecording(file); current(isCurrent);
        entry={...entry,status:'ready',message:data.label,data};
      } catch(error) {
        current(isCurrent);
        entry={...entry,status:'refused',message:error.message};
      }
    }
    entries.push(Object.freeze(entry));
  }
  current(isCurrent);
  return Object.freeze({schema:'rheon-local-record-catalog-v1',bytes,entries:Object.freeze(entries)});
}
