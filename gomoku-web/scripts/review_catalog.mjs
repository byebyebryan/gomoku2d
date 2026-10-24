import { readFile, realpath } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve, sep } from "node:path";
const { isReviewRecord } = await import(new URL("../src/developer/review_record.ts",import.meta.url).href);
export const webRoot=resolve(import.meta.dirname,"..");
export const repoRoot=resolve(webRoot,"..");
export async function readReviewCatalog(){
  const records=JSON.parse(await readFile(resolve(webRoot,"src/developer/reviews.json"),"utf8"));
  if(!Array.isArray(records)||!records.every(isReviewRecord)||new Set(records.map(record=>record.id)).size!==records.length)throw new Error("Invalid curated review catalog");
  for(const record of records){
    const seen=new Map();
    for(const pair of record.comparisons)for(const media of pair.media){
      const path=await realpath(resolve(repoRoot,media.sourcePath));
      if(!path.startsWith(repoRoot+sep))throw new Error("Review media leaves repository");
      const bytes=await readFile(path),hash=createHash("sha256").update(bytes).digest("hex");
      if(hash!==media.sha256||bytes.subarray(0,8).toString("hex")!=="89504e470d0a1a0a"||bytes.readUInt32BE(16)!==media.width||bytes.readUInt32BE(20)!==media.height)throw new Error(`Review media identity mismatch: ${media.sourcePath}`);
      if(seen.has(media.file)&&seen.get(media.file)!==media.sha256)throw new Error("Conflicting review file names");
      seen.set(media.file,media.sha256);
    }
  }
  return records;
}
