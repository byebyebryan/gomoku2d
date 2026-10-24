import manifest from "../../assets/icons/manifest.json";
const sources=import.meta.glob<string>("../../assets/icons/*.svg",{query:"?raw",import:"default",eager:true});
export const canonicalIcons=manifest.icons.map(item=>{
  const source=sources[`../../assets/icons/${item.name}.svg`];
  if(!source)throw new Error(`Missing canonical icon ${item.name}`);
  const mask=new Set<number>();
  for(const rect of source.matchAll(/<rect\b([^>]+)\/?>/g)) {
    const attrs=Object.fromEntries([...rect[1].matchAll(/(x|y|width|height)="(\d+)"/g)].map(m=>[m[1],Number(m[2])]));
    for(let y=attrs.y;y<attrs.y+attrs.height;y++)for(let x=attrs.x;x<attrs.x+attrs.width;x++)mask.add(y*24+x);
  }
  const xs=[...mask].map(n=>n%24),ys=[...mask].map(n=>Math.floor(n/24));
  return {...item,svg:source.replace(/<title>.*?<\/title>\s*/s,""),pixels:mask.size,mask:[...mask],margins:[Math.min(...xs),Math.min(...ys),23-Math.max(...xs),23-Math.max(...ys)]};
});
export type CanonicalIcon=typeof canonicalIcons[number];
export function selectedIcons(params:URLSearchParams):string[]{
  const names=(params.get("compare")??params.get("icon")??"share").split(",");
  const known=[...new Set(names)].filter(name=>canonicalIcons.some(icon=>icon.name===name)).slice(0,4);
  return known.length?known:["share"];
}
