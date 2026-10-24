import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import type { AssetManifest, AssetSpriteAnimation, AssetSpriteZLayer } from "../reports/asset_manifest";
import { COLOR, FRAME_SIZE } from "../board/constants";
import { multiplyTintRgbaPixels } from "../board/board_sprite_resolver";

// The same RGB multiplication as the board, never a luminance approximation.
export const spriteTints:Record<string,number>={white:COLOR.STONE_WHITE,black:COLOR.STONE_BLACK,green:COLOR.WIN_MOVE,red:COLOR.FORBIDDEN,gray:COLOR.SUBTEXT,pink:COLOR.IMMINENT_THREAT,purple:COLOR.COUNTER_THREAT,teal:0x5fc7c2};
const sheets=new Map<string,Promise<HTMLImageElement>>();
function loadSheet(file:string){
  let promise=sheets.get(file);
  if(!promise){promise=new Promise<HTMLImageElement>((resolve,reject)=>{const image=new Image();image.onload=()=>resolve(image);image.onerror=()=>{sheets.delete(file);reject(new Error(`Unable to load ${file}`));};image.src=`${import.meta.env.BASE_URL}assets/${file}`;});sheets.set(file,promise);}
  return promise;
}
export function SpriteCanvas({layers,scale=5,sequence,label}:{layers:AssetSpriteZLayer[];scale?:number;sequence?:string;label:string}){
  const canvas=useRef<HTMLCanvasElement>(null),[error,setError]=useState("");
  const identity=JSON.stringify(layers);
  useEffect(()=>{
    let active=true;
    setError("");
    Promise.all(layers.map(layer=>loadSheet(layer.file))).then(images=>{
      if(!active)return;
      const target=canvas.current,context=target?.getContext("2d");if(!target||!context)return;
      context.clearRect(0,0,FRAME_SIZE,FRAME_SIZE);
      const scratch=document.createElement("canvas");scratch.width=scratch.height=FRAME_SIZE;
      const source=scratch.getContext("2d",{willReadFrequently:true});if(!source)return;
      layers.forEach((layer,index)=>{
        const frame=layer.frame??layer.start??0;
        source.clearRect(0,0,FRAME_SIZE,FRAME_SIZE);
        source.drawImage(images[index],(frame%layer.cols)*FRAME_SIZE,Math.floor(frame/layer.cols)*FRAME_SIZE,FRAME_SIZE,FRAME_SIZE,0,0,FRAME_SIZE,FRAME_SIZE);
        const pixels=source.getImageData(0,0,FRAME_SIZE,FRAME_SIZE);
        pixels.data.set(multiplyTintRgbaPixels(pixels.data,spriteTints[layer.tint??"white"]??COLOR.STONE_WHITE));
        source.putImageData(pixels,0,0);context.drawImage(scratch,0,0);
      });
      target.dataset.ready="true";
    }).catch((reason:unknown)=>{if(active)setError(reason instanceof Error?reason.message:"Sprite unavailable");});
    return()=>{active=false;};
    // Identity tracks semantic layer values, not the caller's array identity.
  },[identity]);
  return <div className="asset-sprite-cell" style={{width:FRAME_SIZE*scale,height:FRAME_SIZE*scale}}><canvas ref={canvas} width={FRAME_SIZE} height={FRAME_SIZE} role="img" aria-label={label} data-frame={layers[0]?.frame??layers[0]?.start??0}/>{sequence&&<span className="asset-sequence">{sequence}</span>}{error&&<span role="alert">{error}</span>}</div>;
}
export function SpriteInspector({manifest}:{manifest:AssetManifest}){
  const [params,setParams]=useSearchParams();
  const animation=manifest.sprites.animations.find(item=>item.name===params.get("animation"))??manifest.sprites.animations[0];
  const tint=Object.prototype.hasOwnProperty.call(spriteTints,params.get("tint")??"")?params.get("tint")!:animation.tint??"white";
  const [offset,setOffset]=useState(0),[playing,setPlaying]=useState(false),[reduced,setReduced]=useState(false),[visible,setVisible]=useState(true),[scale,setScale]=useState(10);
  const stage=useRef<HTMLDivElement>(null);
  const key=animation.name;
  useEffect(()=>{setOffset(0);setPlaying(false);},[key]);
  useEffect(()=>{const query=matchMedia("(prefers-reduced-motion: reduce)");const update=()=>{setReduced(query.matches);if(query.matches)setPlaying(false);};update();query.addEventListener("change",update);return()=>query.removeEventListener("change",update);},[]);
  useEffect(()=>{const update=()=>{if(document.hidden)setPlaying(false);};document.addEventListener("visibilitychange",update);const observer=new IntersectionObserver(entries=>setVisible(entries[0]?.isIntersecting??false));if(stage.current)observer.observe(stage.current);return()=>{observer.disconnect();document.removeEventListener("visibilitychange",update);};},[]);
  useEffect(()=>{if(!playing||reduced||!visible)return;const timer=setInterval(()=>setOffset(n=>(n+1)%(animation.end-animation.start+1)),1000/animation.fps);return()=>clearInterval(timer);},[playing,reduced,visible,animation]);
  function update(values:Record<string,string>){const next=new URLSearchParams(params);next.set("view","sprites");for(const [k,v] of Object.entries(values))next.set(k,v);setParams(next,{preventScrollReset:true});}
  function select(item:AssetSpriteAnimation){update({animation:item.name,tint:item.tint??"white"});}
  const frame=animation.start+offset%(animation.end-animation.start+1);
  return <><div className="asset-sprite-workbench"><div className="asset-sprite-stage" ref={stage}><SpriteCanvas layers={[{...animation,z:"stone",frame,tint}]} scale={scale} label={`${animation.name} frame ${frame}, ${tint}`}/></div><div><label className="asset-field">Animation<select value={key} onChange={event=>select(manifest.sprites.animations.find(a=>a.name===event.target.value)!)}>{manifest.sprites.animations.map(item=><option key={item.name}>{item.name}</option>)}</select></label><div className="dev-tools" aria-label="Sprite tint">{Object.keys(spriteTints).map(value=><button key={value} className="dev-tool" aria-pressed={tint===value} onClick={()=>update({tint:value})}>{value}</button>)}</div><div className="dev-tools"><button className="dev-tool" aria-pressed={playing} disabled={reduced} onClick={()=>setPlaying(!playing)}>{playing?"Pause animation":"Play animation"}</button><button className="dev-tool" onClick={()=>{setPlaying(false);setOffset(n=>(n+1)%(animation.end-animation.start+1));}}>Next frame</button><button className="dev-tool" onClick={()=>{setPlaying(false);setOffset(0);}}>Reset frame</button><label>Scale<select value={scale} onChange={event=>setScale(Number(event.target.value))}>{[1,2,5,10].map(value=><option key={value} value={value}>{value}×</option>)}</select></label></div><p className="dev-meta">Frame {frame} · {animation.start}–{animation.end} · {animation.fps} fps</p><p>{animation.role}</p><p className="dev-meta">16 × 16 source pixels. RGB multiplication preserves alpha. {reduced?"Reduced motion is on; use frame stepping.":"Animation pauses offscreen and stops when the page is hidden."} These previews are independent of the game renderer.</p></div></div>
  <section className="dev-section"><h2>Animation inventory</h2><p className="dev-meta">All {manifest.sprites.animations.length} source-defined ranges. Select one to inspect; the others do no animation work.</p><div className="asset-animation-list">{manifest.sprites.animations.map(item=><button className="dev-tool" key={item.name} aria-pressed={key===item.name} onClick={()=>{select(item);stage.current?.scrollIntoView({block:"center"});}}>{item.name}</button>)}</div></section>
  <section className="dev-section"><h2>Static poses</h2><div className="asset-inventory">{manifest.sprites.static_poses.map(item=><article key={item.name}><SpriteCanvas layers={[{...item,z:"stone"}]} label={item.name}/><h3>{item.name}</h3><p className="dev-meta">{item.role} · frame {item.frame}</p></article>)}</div></section>
  <section className="dev-section"><h2>Board layering</h2><p className="dev-readable dev-muted">Surfaces sit below pointer, committed stones, sequence text and hover intent. These are frozen representative source frames; the selected animation above is the only running loop.</p><div className="asset-inventory">{manifest.sprites.z_cases.map(item=><article key={item.name}><SpriteCanvas layers={item.layers} sequence={item.sequence} label={`${item.title} layer preview`}/><h3>{item.title}</h3><p className="dev-meta">{item.note}</p><ol>{item.layers.map((layer,index)=><li key={index}>{layer.name}</li>)}</ol></article>)}</div></section>
  <details className="dev-disclosure"><summary>Seven original source sheets</summary><div className="asset-inventory">{manifest.sprites.sheets.map(sheet=><article key={sheet.file}><h3>{sheet.file.split("/").pop()}</h3><a href={`${import.meta.env.BASE_URL}assets/${sheet.file}`}><img className="asset-source-sheet" src={`${import.meta.env.BASE_URL}assets/${sheet.file}`} alt={sheet.label}/></a><p className="dev-meta">{sheet.cols} × {sheet.rows} frames · {sheet.label}</p></article>)}</div></details></>;
}
