import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { readReviewCatalog, webRoot, repoRoot } from "./review_catalog.mjs";
export async function publishReviews(destination=resolve(webRoot,"dist")){
  const records=await readReviewCatalog();
  for(const record of records){
    const target=resolve(destination,"reviews",record.id);await mkdir(target,{recursive:true});
    await writeFile(resolve(target,"record.json"),JSON.stringify(record,null,2)+"\n");
    for(const pair of record.comparisons)for(const media of pair.media)await copyFile(resolve(repoRoot,media.sourcePath),resolve(target,media.file));
  }
  console.log(`Published ${records.length} curated review records and their allowlisted media.`);
}
if(process.argv[1]===new URL(import.meta.url).pathname)await publishReviews(process.argv[2]);
