import { expect, test } from "@playwright/test";

test("developer direct entries keep private state and engine imports untouched",async({page})=>{
  const requests:string[]=[];
  page.on("request",request=>requests.push(request.url()));
  await page.addInitScript(()=>{
    localStorage.setItem("gomoku2d.local-profile.v5","private sentinel, do not read");
    const reads:string[]=[];Object.assign(window,{developerStorageReads:reads});
    const get=Storage.prototype.getItem;Storage.prototype.getItem=function(key){reads.push(key);return get.call(this,key);};
  });
  for(const path of ["/dev/","/visuals/?view=icons","/lab/","/dev/process/","/dev/reviews/","/dev/unknown/"]){
    await page.goto(path);
    await expect(page.locator(".dev-nav")).toBeVisible();
    await expect(page.locator("#developer-content h1")).toHaveCount(1);
    const reads=await page.evaluate(()=>(window as unknown as {developerStorageReads:string[]}).developerStorageReads);
    expect(reads.filter(key=>/gomoku|firebase|auth/i.test(key))).toEqual([]);
    expect(await page.evaluate(()=>localStorage.length)).toBe(1);
  }
  expect(requests.filter(url=>/\/assets\/(PlayerApp-|firebase-|cloud_|Board-|wasm_bridge-|bot_worker-|replay_analysis_worker-)|\.wasm(?:$|\?)/.test(url))).toEqual([]);
  expect(requests.filter(url=>/googleapis|identitytoolkit|securetoken|firestore|gapi/.test(url))).toEqual([]);
});

test("canonical icon selection, comparison, filters and history survive navigation",async({page})=>{
  await page.goto("/visuals/?view=icons&compare=share,save,publish");
  await expect(page.locator("[data-inspect]")).toHaveCount(3);
  await expect(page.locator(".asset-catalog-open")).toHaveCount(41);
  await page.getByRole("button",{name:"Compare Home",exact:true}).click();
  await expect(page.locator("[data-inspect]")).toHaveCount(4);
  await page.getByRole("button",{name:"Compare Bot",exact:true}).click();
  await expect(page.getByRole("status")).toContainText("up to four");
  await page.goBack();await expect(page.locator("[data-inspect]")).toHaveCount(3);
  await page.goForward();await page.reload();await expect(page.locator("[data-inspect]")).toHaveCount(4);
  await page.getByRole("searchbox").fill("sign");await expect(page.locator(".asset-catalog-open")).toHaveCount(2);
  await page.getByRole("searchbox").fill("no-match-here");await expect(page.getByText("No matching icons.")).toBeVisible();
  await page.goto("/visuals/?view=icons&compare=missing,share,share,../../secret");await expect(page.locator("[data-inspect]")).toHaveCount(1);
  await expect(page.locator('[data-inspect="share"]')).toBeVisible();
});

test("developer pages and inspector detail views reflow with enlarged text",async({page})=>{
  test.setTimeout(90_000);
  for(const [width,height] of [[1280,900],[390,844],[320,800],[844,390]])for(const font of ["100%","200%"]){
    await page.setViewportSize({width,height});
    for(const path of ["/dev/","/dev/process/","/dev/reviews/","/visuals/","/visuals/?view=icons&compare=share,save,publish,home","/visuals/?view=sprites","/lab/","/lab/?tab=search"]){
      await page.goto(path);await expect(page.locator("#developer-content h1")).toHaveCount(1);
      await page.evaluate(size=>document.documentElement.style.fontSize=size,font);
      await page.evaluate(()=>document.fonts.ready);
      const overflow=await page.locator(".dev-page").evaluate(e=>e.scrollWidth-e.clientWidth);
      expect(overflow,`${path} ${width} ${font}`).toBeLessThanOrEqual(1);
    }
  }
});

test("public navigation, keyboard entry and unknown-route recovery work",async({page})=>{
  await page.goto("/dev/");await expect(page.getByRole("heading",{name:"Inside Gomoku2D",exact:true})).toBeVisible();
  await page.keyboard.press("Tab");await expect(page.getByRole("link",{name:"Skip to content",exact:true})).toBeFocused();
  await page.keyboard.press("Enter");await expect(page.locator("#developer-content")).toBeFocused();
  await page.getByRole("navigation",{name:"Developer sections"}).getByRole("link",{name:"Process",exact:true}).click();
  await expect(page.locator("#developer-content")).toBeFocused();
  await page.goBack();await expect(page.getByRole("heading",{name:"Inside Gomoku2D",exact:true})).toBeVisible();
  await page.goto("/dev/not-a-page/");await expect(page.getByRole("heading",{name:"Reference not found"})).toBeVisible();
  await page.getByRole("link",{name:"Developer overview",exact:true}).click();await expect(page).toHaveURL(/\/dev\/$/);
});
