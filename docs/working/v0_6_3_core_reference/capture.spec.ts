import {mkdir,writeFile} from "node:fs/promises";
import {createHash} from "node:crypto";
import {resolve} from "node:path";
import {execFileSync} from "node:child_process";
import {test,expect,cloudTestOrigins,projectId} from "./helpers.js";
import {seedLocalSavedMatch} from "../playtests/helpers/local_history.js";
const output=resolve("../docs/working/v0_6_3_core_reference");
const moves=[112,0,113,15,114,30,115,1,116];
const uid="core-reference-public-fixture";
function field(v:any):any{if(v&&typeof v==="object"&&"timestampValue" in v)return v;if(v===null)return{nullValue:null};if(typeof v==="string")return{stringValue:v};if(typeof v==="boolean")return{booleanValue:v};if(typeof v==="number")return{integerValue:String(v)};if(Array.isArray(v))return{arrayValue:{values:v.map(field)}};return{mapValue:{fields:Object.fromEntries(Object.entries(v).map(([k,x])=>[k,field(x)]))}};}
async function seed(path:string,value:any){const fields=Object.fromEntries(Object.entries(value).map(([k,v])=>[k,field(v)]));fields.updated_at={timestampValue:"2026-10-24T00:00:00Z"};const response=await fetch(`${cloudTestOrigins.firestore}/v1/projects/${projectId}/databases/(default)/documents/${path}`,{method:"PATCH",headers:{Authorization:"Bearer owner","Content-Type":"application/json"},body:JSON.stringify({fields})});expect(response.ok).toBe(true);}
test("capture the released 0.6.2 core screens with explicit fixtures",async({page,browser,browserName})=>{
 test.setTimeout(120_000);await mkdir(output,{recursive:true});
 const wire={v:1,r:"freestyle",m:Buffer.from(moves).toString("base64url"),b:{k:"h",n:"Reference Player",u:uid},w:{k:"h",n:"Reference Opponent"},at:4};
 const payload=Buffer.from(JSON.stringify(wire)).toString("base64url");
 await seed(`public_profiles/${uid}`,{schema_version:1,uid,display_name:"Reference Player",enabled:true,generation:1,revision:1,catalog_page_counts:[1,0,0,0],shared_stats:{matches:1,wins:1,losses:0,draws:0,updated_at:{timestampValue:"2026-10-24T00:00:00Z"}}});
 await seed(`public_profiles/${uid}/pages/0`,{schema_version:1,generation:1,revision:1,entries:[{fingerprint:"x".repeat(43),payload,title:"A short freestyle finish",view_ply:4}]});
 await page.goto("/");await seedLocalSavedMatch(page,{displayName:"Reference Player",id:"reference-match",moves:moves.map(cell=>({row:Math.floor(cell/15),col:cell%15})),preferredVariant:"freestyle",savedAt:"2026-10-24T00:00:00.000Z",status:"black_won",variant:"freestyle"});
 await page.emulateMedia({reducedMotion:"reduce"});
 const captures=[];
 for(const [width,height,label,font] of [[1280,900,"desktop","100%"],[390,844,"portrait","100%"],[844,390,"landscape","100%"],[320,800,"enlarged","200%"]] as const){
  await page.setViewportSize({width,height});
  for(const [name,path] of [["home","/"],["match","/match/local"],["replay",`/replay/shared/#g2r1=${payload}`],["settings","/settings"],["profile","/profile"],["public-player",`/player/?id=${uid}`]]){
   await page.goto(path);await expect(page.locator("h1")).toBeVisible();
   if(name==="match"||name==="replay"){await expect(page.getByRole("grid",{name:"Gomoku board"})).toBeVisible();await expect(page.locator("canvas")).toBeVisible();}
   if(name==="public-player"){await expect(page.getByRole("heading",{name:"Reference Player",exact:true})).toBeVisible();await expect(page.getByRole("listitem").filter({hasText:"Reference Opponent"})).toBeVisible();}
   await page.evaluate(size=>document.documentElement.style.fontSize=size,font);await page.evaluate(()=>document.fonts.ready);
   const image=await page.screenshot({path:resolve(output,`${name}-${label}.png`),fullPage:false,animations:"disabled"});
   captures.push({file:`${name}-${label}.png`,sha256:createHash("sha256").update(image).digest("hex"),width,height,font,screen:name,browser:browserName,conditions:"Fresh guest browser, deterministic local/public emulator fixtures, reduced motion, viewport capture; no real accounts"});
  }
 }
 const source=execFileSync("git",["rev-parse","HEAD"],{encoding:"utf8"}).trim();
 const applicationTree=execFileSync("git",["rev-parse","HEAD:gomoku-web/src"],{encoding:"utf8"}).trim();
 await writeFile(resolve(output,"capture.json"),JSON.stringify({source,applicationTree,version:"v0.6.2",captured_at:new Date().toISOString(),browser:browser.version(),browserName,deviceScaleFactor:1,emulator_project:projectId,port_offset:1000,fixtures:{moves,uid,display_name:"Reference Player"},captures},null,2)+"\n");
});
