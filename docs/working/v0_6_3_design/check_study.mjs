import { chromium, firefox, webkit } from "../../../gomoku-web/node_modules/@playwright/test/index.mjs";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const base = process.env.STUDY_BASE_URL;
if (!base) throw new Error("Set STUDY_BASE_URL to the owned preview or published report.");
const output = resolve(process.env.STUDY_CHECK_OUTPUT || "gomoku-web/test-results/v063-design/browser");
await mkdir(output,{recursive:true});
const assert=(value,message)=>{if(!value)throw new Error(message);};
const results=[];
for(const [engine,type] of Object.entries({chromium,firefox,webkit})) {
  if(process.env.STUDY_ENGINES&&!process.env.STUDY_ENGINES.split(",").includes(engine))continue;
  const browser=await type.launch();
  const context=await browser.newContext({viewport:{width:1280,height:900},deviceScaleFactor:1});
  const page=await context.newPage();
  const errors=[],outside=[];
  page.on("pageerror",error=>errors.push(error.message));
  page.on("console",message=>{if(message.type()==="error")errors.push(message.text());});
  page.on("request",request=>{if(!request.url().startsWith(new URL(base).origin)&&!request.url().startsWith("data:"))outside.push(request.url());});
  const checks=[];
  try {
    await page.goto(base+"#icons");
    await page.locator(".catalog-card").last().waitFor();
    await page.evaluate(()=>document.fonts.ready);
    assert(await page.locator(".catalog-card").count()===41,"Complete source icon inventory");checks.push("41 canonical icons");
    await page.getByRole("button",{name:"Compare sharing actions",exact:true}).click();
    assert(await page.locator("[data-inspect]").count()===3,"Sharing comparison");
    await page.getByRole("button",{name:"Compare Home",exact:true}).click();
    assert(await page.locator("[data-inspect]").count()===4,"Four comparison slots");
    await page.getByRole("button",{name:"Compare Bot",exact:true}).click();
    assert(await page.locator("[data-inspect]").count()===4,"Comparison limit");
    assert((await page.locator("#announce").textContent()).includes("four"),"Limit feedback");checks.push("selection, comparison and limit feedback");
    await page.goBack(); await page.waitForFunction(()=>document.querySelectorAll("[data-inspect]").length===3);
    await page.goForward(); await page.waitForFunction(()=>document.querySelectorAll("[data-inspect]").length===4);
    await page.reload(); await page.locator("[data-inspect]").last().waitFor();
    assert(await page.locator("[data-inspect]").count()===4,"Reload restores selected icons");checks.push("direct state, Back/Forward and reload");
    await page.locator("#icon-search").fill("sign");
    assert(await page.locator(".catalog-card").count()===2,"Filter account pair");
    assert(await page.locator("#icon-search").evaluate(e=>e===document.activeElement),"Search retains focus");
    await page.locator("#icon-search").fill("no-such-icon");assert(await page.locator("#icon-catalog").textContent()==="No matching icons.","Empty filter");checks.push("filter, empty state, input focus");
    await page.goto(base+"#icons?icon=share&grid=off");await page.locator('[data-inspect="share"]').waitFor();
    await page.getByRole("button",{name:"Dark surface",exact:true}).click();
    assert(await page.locator(".inspector").evaluate(e=>e.classList.contains("light-surface")),"Light inspection surface");
    await page.getByRole("button",{name:"Light surface",exact:true}).click();
    assert(await page.getByRole("button",{name:"Dark surface",exact:true}).evaluate(e=>e===document.activeElement),"Control focus survives update");
    const raster=[];
    for(const [selector,scale] of [[".native-one",1],[".native-two",2],[".pixel-grid",10]]) {
      const png=await page.locator(selector).screenshot({path:resolve(output,`${engine}-native-${scale}x.png`)});
      const measured=await page.evaluate(async({src,scale})=>{
        const image=new Image();image.src=src;await image.decode();
        const canvas=document.createElement("canvas");canvas.width=image.width;canvas.height=image.height;
        const context=canvas.getContext("2d");context.drawImage(image,0,0);
        const pixels=context.getImageData(0,0,canvas.width,canvas.height).data;
        const mask=new Set(window.STUDY_DATA.icons.find(icon=>icon.name==="share").mask);
        let mismatch=Infinity,offset;
        // Element screenshots can include one extra row at fractional page positions.
        // Allow that crop translation; every foreground pixel must still match.
        for(let dy=-1;dy<=1;dy++)for(let dx=-1;dx<=1;dx++) {
          let difference=0;
          for(let y=0;y<canvas.height;y++)for(let x=0;x<canvas.width;x++) {
            const sx=Math.floor((x-dx)/scale),sy=Math.floor((y-dy)/scale);
            const expected=sx>=0&&sx<24&&sy>=0&&sy<24&&mask.has(sy*24+sx);
            if((pixels[(y*canvas.width+x)*4]>200)!==expected)difference++;
          }
          if(difference<mismatch){mismatch=difference;offset=[dx,dy];}
        }
        const colours=[...new Set(Array.from({length:pixels.length/4},(_,i)=>Array.from(pixels.slice(i*4,i*4+4)).join(",")))];
        return {scale,width:canvas.width,height:canvas.height,mismatch,offset,colours};
      },{src:"data:image/png;base64,"+png.toString("base64"),scale});
      assert(measured.mismatch===0,`Canonical raster ${scale}x: ${JSON.stringify(measured)}`);
      assert(measured.colours.every(c=>["30,30,30,255","245,245,245,255","87,87,86,255"].includes(c)),"No antialias colours");
      raster.push(measured);
    }
    checks.push("1x/2x/10x canonical raster masks, zero mismatched or antialiased pixels");
    const geometry=await page.locator(".native-one,.native-two,.pixel-grid").evaluateAll(es=>es.map(e=>({width:e.getBoundingClientRect().width,height:e.getBoundingClientRect().height})));
    assert(geometry.map(x=>x.width).join(",")==="24,48,240"&&geometry.every(x=>x.width===x.height),"Integer view dimensions");
    await page.goto(base+"#lab");await page.locator(".bot-row").last().waitFor();
    assert(await page.locator(".bot-row").count()===8,"Actual lab rows");
    assert((await page.locator(".bot-score").first().textContent()).includes("69.1%"),"Source score, not invented data");
    await page.locator(".bot-summary").first().click();assert(await page.locator(".bot-row").first().getAttribute("open")!==null,"Bot details expand");
    await page.getByRole("link",{name:"Search",exact:true}).click();assert(await page.locator(".search-row").count()===8,"Search views");
    await page.getByRole("link",{name:"Analysis",exact:true}).click();await page.locator(".proof-board").waitFor();
    assert((await page.locator(".proof-board").getAttribute("aria-label")).includes("14 earlier stones"),"Before-move prefix");
    await page.getByRole("link",{name:"Before 28",exact:true}).click();assert((await page.locator(".proof-board").getAttribute("aria-label")).includes("27 earlier stones"),"Recorded frame selection");checks.push("8 real Lab rows, expansion, stages and recorded proof prefix");
    await page.goto(base+"#reviews");await page.locator(".capture-pane img").last().waitFor();await page.locator(".capture-pane img").evaluateAll(es=>Promise.all(es.map(e=>e.decode())));
    assert(await page.locator(".capture-pane img").evaluateAll(es=>es.every(e=>e.complete&&e.naturalWidth===390)),"Review images loaded");
    assert((await page.locator("main").textContent()).includes("not the final 0.6.2 baseline"),"Historical snapshot label");
    await page.getByRole("button",{name:"Show full captures",exact:true}).click();assert(await page.locator(".full-captures").count()===1,"Full captures");
    await page.getByRole("link",{name:"Public player",exact:true}).click();await page.locator('img[src="review/public-after.png"]').waitFor();checks.push("dated review media, full-size and case navigation");
    await page.goto(base+"#icons?view=sprites");await page.locator("#sprite-canvas[data-frame]").waitFor();
    await page.getByRole("button",{name:"black",exact:true}).click();await page.waitForFunction(()=>document.querySelector("#sprite-canvas")?.dataset.tint==="black");
    const colors=await page.locator("#sprite-canvas").evaluate(c=>{const x=c.getContext("2d").getImageData(0,0,16,16).data;return [...x].filter((_,i)=>i%4!==3);});assert(Math.max(...colors)<=64,"Black sprite retains player colour");
    await page.getByRole("button",{name:"Next frame",exact:true}).click();assert(await page.locator("#sprite-canvas").getAttribute("data-frame")==="1","Frame step");
    await page.emulateMedia({reducedMotion:"reduce"});await page.getByRole("button",{name:"Play animation",exact:true}).click();assert(await page.locator("#sprite-play").getAttribute("aria-pressed")==="false","Reduced motion stays paused");await page.emulateMedia({reducedMotion:"no-preference"});checks.push("source sprite frame, black tint and reduced motion");
    const layouts=[];
    for(const [width,height] of [[1280,900],[390,844],[320,800],[844,390]]) {
      await page.setViewportSize({width,height});
      for(const enlarged of [false,true]) {
        for(const section of ["overview","icons","lab","reviews","process"]) {
          await page.goto(base+"#"+section);await page.locator(`[data-section="${section}"][aria-current="page"]`).waitFor();await page.evaluate(()=>document.fonts.ready);
          await page.evaluate(big=>document.documentElement.style.fontSize=big?"200%":"",enlarged);
          const m=await page.evaluate(()=>({viewport:innerWidth,width:document.documentElement.scrollWidth,headings:[...document.querySelectorAll("h1,h2")].every(e=>e.getBoundingClientRect().right<=innerWidth+1)}));
          assert(m.width<=width+1&&m.headings,`${section} ${width} ${enlarged?"200%":"100%"} reflow: ${JSON.stringify(m)}`);
          layouts.push({section,width,height,enlarged,overflow:m.width-width});
          if(!enlarged&&(width===1280||width===390))await page.screenshot({path:resolve(output,`${engine}-${section}-${width}.png`),fullPage:false,animations:"disabled"});
        }
      }
    }
    for(const width of [390,320])for(const enlarged of [false,true])for(const route of ["icons?compare=share,save,publish,home","icons?view=style","icons?view=sprites","lab?tab=search","lab?tab=analysis","reviews?case=public&full=1"]) {
      await page.setViewportSize({width,height:844});
      await page.goto(base+"#"+route);
      await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
      await page.evaluate(()=>document.fonts.ready);
      await page.evaluate(big=>document.documentElement.style.fontSize=big?"200%":"",enlarged);
      const m=await page.evaluate(()=>({viewport:innerWidth,width:document.documentElement.scrollWidth}));
      assert(m.width<=width+1,`${route} ${width} ${enlarged?"200%":"100%"} reflow: ${JSON.stringify(m)}`);
      layouts.push({section:route,width,height:844,enlarged,overflow:m.width-width});
    }
    checks.push(`${layouts.length} responsive/enlarged-text combinations`);
    await page.setViewportSize({width:1280,height:900});await page.goto(base+"#overview");await page.reload();await page.evaluate(()=>document.fonts.ready);
    await page.keyboard.press("Tab");assert(await page.getByRole("link",{name:"Skip to content",exact:true}).evaluate(e=>e===document.activeElement),"Keyboard skip link");await page.keyboard.press("Enter");assert(await page.locator("main").evaluate(e=>e===document.activeElement),"Skip destination");checks.push("keyboard skip destination");
    assert(!errors.length,"Browser errors: "+errors.join(" | "));assert(!outside.length,"Unexpected network: "+outside.join(","));
    results.push({engine,passed:true,checks,layouts,raster,errors,outside});
    console.log(JSON.stringify({engine,passed:true,checks:checks.length,layouts:layouts.length}));
  } catch(error) {
    console.log(JSON.stringify(await page.evaluate(()=>({overflow:[...document.querySelectorAll("body *")].map(e=>({tag:e.tagName,cls:String(e.className),text:e.textContent.slice(0,60),left:e.getBoundingClientRect().left,right:e.getBoundingClientRect().right,scroll:e.scrollWidth,client:e.clientWidth})).filter(e=>e.right>innerWidth+1||e.left<0||e.scroll>e.client+1)}))));
    results.push({engine,passed:false,checks,error:String(error),errors,outside});console.log(JSON.stringify({engine,passed:false,error:String(error)}));
    await page.screenshot({path:resolve(output,`${engine}-failure.png`),fullPage:true});
  } finally { await browser.close(); }
}
await writeFile(resolve(output,"results.json"),JSON.stringify({base,results},null,2)+"\n");
if(results.some(x=>!x.passed))process.exitCode=1;
