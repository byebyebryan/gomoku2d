import {chromium, firefox, webkit} from "../../../gomoku-web/node_modules/@playwright/test/index.mjs";
import {mkdir,writeFile} from "node:fs/promises";
import {resolve} from "node:path";

const base=process.env.HISTORY_STUDY_URL || "http://127.0.0.1:46865/";
const output=process.env.HISTORY_STUDY_CHECK_OUTPUT || "gomoku-web/test-results/v063-history-design/checks";
await mkdir(output,{recursive:true});
const results=[];
for(const [engine,type] of Object.entries({chromium,firefox,webkit})) {
 const browser=await type.launch();
 try {
  const page=await browser.newPage({reducedMotion:"reduce"});
  const errors=[],outside=[];
  page.on("pageerror",e=>errors.push(String(e)));
  page.on("console",m=>{if(m.type()==="error")errors.push(m.text());});
  page.on("request",r=>{if(!r.url().startsWith(base)&&!r.url().startsWith("data:"))outside.push(r.url());});
  for(const [width,font] of [[1280,"100%"],[390,"100%"],[320,"100%"],[320,"200%"]]) {
   await page.setViewportSize({width,height:900});
   for(const layout of ["journal","screens","browser"]) {
    await page.goto(base+`?case=${engine}-${width}-${encodeURIComponent(font)}-${layout}#layout=${layout}&version=0.6.2&screen=Public+player&viewport=portrait`);
    await page.locator("#history .capture-grid img").first().waitFor();
    await page.evaluate(size=>document.documentElement.style.fontSize=size,font);
    await page.evaluate(async()=>{
     document.body.getBoundingClientRect();
     await document.fonts.ready;
     await new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
    });
    const overflow=await page.evaluate(()=>Math.max(document.documentElement.scrollWidth-document.documentElement.clientWidth,document.body.scrollWidth-document.body.clientWidth));
    if(overflow>1)throw Error(`${engine}/${layout}/${width}/${font}: overflow ${overflow}`);
    const duplicateIds=await page.evaluate(()=>{
     const ids=[...document.querySelectorAll("[id]")].map(e=>e.id);
     return ids.filter((value,index)=>ids.indexOf(value)!==index);
    });
    if(duplicateIds.length)throw Error("Duplicate navigation targets: "+duplicateIds.join(","));
    const actual=page.locator(layout==="journal" ? "#journal-compare-public-player img" : layout==="screens" ? ".screen-trail .capture-grid img" : "#browser-compare-public-player img");
    for(const img of await actual.all()) {
     await img.scrollIntoViewIfNeeded();
     await img.evaluate(async e=>{
      if(!e.complete)await new Promise((resolve,reject)=>{
       e.addEventListener("load",resolve,{once:true});
       e.addEventListener("error",()=>reject(Error("Capture image failed to load")),{once:true});
      });
      await e.decode();
     });
    }
    const scales=await actual.evaluateAll(images=>images.map(i=>({width:i.naturalWidth,height:i.naturalHeight,scale:i.getBoundingClientRect().width/i.naturalWidth})));
    if(scales.length!==2 || scales.some(i=>i.width!==390||i.height!==844) || Math.abs(scales[0].scale-scales[1].scale)>.001)throw Error("Source pixel scale mismatch: "+JSON.stringify(scales));
    if(engine==="chromium" && font==="100%" && [1280,390].includes(width)) {
     await page.evaluate(()=>scrollTo(0,0));
     await page.screenshot({path:`${output}/${layout}-${width}.png`});
     const target=page.locator(layout==="journal" ? "#journal-detail" : layout==="screens" ? ".screen-trail" : ".milestone-panel");
     await target.scrollIntoViewIfNeeded();
     await page.screenshot({path:`${output}/${layout}-comparison-${width}.png`});
    }
    await page.addScriptTag({path:resolve("gomoku-web/node_modules/axe-core/axe.min.js")});
    const audit=await page.evaluate(()=>axe.run("body",{runOnly:{type:"tag",values:["wcag2a","wcag2aa","wcag21aa"]}}));
    const violations=audit.violations.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}));
    results.push({engine,layout,width,font,overflow,scales,violations,incomplete:audit.incomplete.map(v=>v.id)});
    if(violations.length)throw Error(JSON.stringify(results.at(-1)));
   }
  }
  await page.evaluate(()=>document.documentElement.style.fontSize="100%");
  await page.setViewportSize({width:1280,height:900});
  await page.goto(base+"#layout=browser&version=0.6.2&screen=Public+player&viewport=portrait");
  await page.locator('[data-version-link="0.4.5"]').click();
  await page.locator("#browser-milestone-heading").filter({hasText:"Settings gets a home"}).waitFor();
  if(!(await page.locator("#browser-milestone-heading").evaluate(e=>e===document.activeElement)))throw Error("Milestone focus lost");
  await page.reload();
  await page.locator("#browser-milestone-heading").filter({hasText:"Settings gets a home"}).waitFor();
  await page.goBack();
  await page.locator("#browser-milestone-heading").filter({hasText:"A public page with a useful record"}).waitFor();
  await page.locator('[data-layout="screens"]').click();
  await page.getByRole("heading",{name:"Public player, over time",exact:true}).waitFor();
  await page.locator('[data-control="screen-replay"]').click();
  await page.getByRole("heading",{name:"Replay, over time",exact:true}).waitFor();
  if(await page.locator(".trail-item").count()<5)throw Error("Incomplete screen trail");
  if(!(await page.locator('[data-control="screen-replay"]').evaluate(e=>e===document.activeElement)))throw Error("Screen focus lost");
  await page.locator('[data-control="screen-public-player"]').click();
  await page.getByRole("heading",{name:"Public player, over time",exact:true}).waitFor();
  await page.getByRole("link",{name:"Compare previous step",exact:true}).click();
  if(!(await page.locator("#screen-heading").evaluate(e=>e===document.activeElement)))throw Error("Comparison focus lost");
  await page.setViewportSize({width:390,height:844});
  await page.locator('[data-layout="browser"]').click();
  await page.locator("#milestone-rail summary").click();
  await page.locator('[data-version-link="0.2.4"]').click();
  await page.locator("#browser-milestone-heading").filter({hasText:"The local-play baseline"}).waitFor();
  if(await page.locator("#milestone-rail").getAttribute("open")!==null)throw Error("Mobile milestone menu did not collapse");
  await page.locator('[data-control="viewport-desktop"]').click();
  await page.locator('[data-control="viewport-desktop"][aria-current="page"]').waitFor();
  const source=page.locator('#browser-compare-replay figure[data-version="0.2.4"] img');
  await source.scrollIntoViewIfNeeded();
  await source.evaluate(async e=>{
   if(!e.complete)await new Promise((resolve,reject)=>{
    e.addEventListener("load",resolve,{once:true});
    e.addEventListener("error",()=>reject(Error("Selected source failed to load")),{once:true});
   });
   await e.decode();
  });
  if(await source.evaluate(e=>e.naturalWidth)!==1280)throw Error("Viewport source did not switch");
  await page.goto(base+"?keyboard=fresh-document");
  await page.keyboard.press("Tab");
  if(!(await page.locator(".dev-skip").evaluate(e=>e===document.activeElement)))throw Error("Skip link absent");
  await page.keyboard.press("Enter");
  if(!(await page.locator("#content").evaluate(e=>e===document.activeElement)))throw Error("Skip target focus absent");
  if(errors.length||outside.length)throw Error(JSON.stringify({engine,errors,outside}));
  results.push({engine,interactions:"options, milestone/screen navigation, source switching, history, reload, keyboard and mobile menu",passed:true,errors,outside});
 } finally {await browser.close();}
}
await writeFile(`${output}/results.json`,JSON.stringify({base,results},null,2)+"\n");
console.log(JSON.stringify({responsiveCases:results.filter(r=>r.layout).length,interactionEngines:3,violations:results.reduce((n,r)=>n+(r.violations?.length||0),0),incomplete:results.reduce((n,r)=>n+(r.incomplete?.length||0),0)}));
