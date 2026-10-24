import { useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { useDocumentTitle } from "../app/useDocumentTitle";
import { loadAssetManifest, type AssetManifest } from "../reports/asset_manifest";
import { IconInspector } from "../developer/IconInspector";
import { SpriteInspector } from "../developer/SpriteInspector";
import "../developer/assets.css";

export function AssetPreviewRoute() {
  useDocumentTitle("Design & assets");
  const [params]=useSearchParams(), selected=params.get("view"), view=selected==="icons"||selected==="sprites"?selected:"style";
  const [manifest,setManifest]=useState<AssetManifest|null>(null),[error,setError]=useState("");
  useEffect(()=>{let active=true;loadAssetManifest().then(value=>{if(active)setManifest(value);}).catch((reason:unknown)=>{if(active)setError(reason instanceof Error?reason.message:"Assets unavailable");});return()=>{active=false;};},[]);
  return <><header className="dev-heading"><p className="dev-eyebrow">Design & assets</p><h1>Pixels with a purpose</h1><p className="dev-lead">The current visual language, from controls to the smallest source pixel.</p></header><nav className="dev-views" aria-label="Asset views">{["style","icons","sprites"].map(id=><Link key={id} to={`/visuals/?view=${id}`} aria-current={view===id?"page":undefined}>{id}</Link>)}</nav>
  {error?<section><h2>Assets unavailable</h2><p>{error}</p><button className="dev-tool" onClick={()=>window.location.reload()}>Reload</button></section>:view==="icons"?<IconInspector/>:!manifest?<p aria-busy="true">Loading assets…</p>:view==="sprites"?<SpriteInspector manifest={manifest}/>:<StyleReference manifest={manifest}/>}</>;
}
function StyleReference({manifest}:{manifest:AssetManifest}) {
  const [tokens,setTokens]=useState<Record<string,string>>({});
  useEffect(()=>{const css=getComputedStyle(document.documentElement);setTokens(Object.fromEntries(manifest.guide.palette.flatMap(g=>g.tokens.map(t=>[t.name,css.getPropertyValue(t.name).trim()]))));},[manifest]);
  return <><section><h2>Colours with a job</h2><p className="dev-muted dev-readable">Current values from application CSS. Semantic roles stay consistent as the product evolves.</p><div className="asset-palette">{manifest.guide.palette.flatMap(group=>group.tokens.map(token=><div className="asset-swatch" key={token.name}><span style={{background:`var(${token.name})`}}/><div>{token.label}<code>{token.name} · {tokens[token.name]??"…"}</code><small>{token.role}</small></div></div>))}</div></section>
  <section className="dev-section"><h2>Actual app controls</h2><p className="dev-muted">The same classes, with pointer, pressed, focus and disabled states.</p><div className="asset-controls">{["Primary","Secondary","Neutral","Danger","Accent"].map(role=><button key={role} className={`uiAction uiAction${role}`}>{role}</button>)}<button className="uiAction uiActionNeutral" disabled>Disabled</button></div></section>
  <section className="dev-section"><h2>Pixel type, with room to read</h2><div className="asset-inventory">{manifest.guide.font_roles.map(font=><article key={font.file}><h3>{font.role}</h3><p style={{fontFamily:font.family}}>{font.sample}</p><p className="dev-meta">{font.note}</p><a href={`${import.meta.env.BASE_URL}assets/${font.file}`}>{font.file.split("/").pop()}</a></article>)}</div><p className="dev-meta"><a href={`${import.meta.env.BASE_URL}assets/fonts/LICENSE.txt`}>Font license</a></p></section>
  <section className="dev-section"><h2>Principles in the product</h2><div className="asset-inventory">{manifest.guide.principles.map(item=><article key={item.title}><h3>{item.title}</h3><p className="dev-muted">{item.description}</p></article>)}</div></section><details className="dev-disclosure"><summary>Source files and revision context</summary><div className="dev-readable"><p>Tokens are read from this build's CSS. Icons and sprites link to the canonical source files bundled with it. Availability in the pack does not imply that every icon has a current control assignment.</p><a href={`${import.meta.env.BASE_URL}assets/asset_manifest.json`}>Asset manifest</a></div></details></>;
}
