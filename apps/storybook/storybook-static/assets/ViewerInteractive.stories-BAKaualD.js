import{a,j as t}from"./iframe-CLHqt8sP.js";import{S as n}from"./SrujaViewerView-CWue8X71.js";import"./preload-helper-PPVm8Dsz.js";import"./cytoscape.esm-BnkdMOzK.js";import"./elk.bundled-BsTYLubE.js";import"./logger-CfJbYVt4.js";const u={title:"Viewer/SrujaViewerView Interactive"},d={metadata:{name:"Sample",version:"1.0.0"},architecture:{systems:[{id:"WebApp",label:"Web Application",containers:[{id:"API",label:"API Service"},{id:"DB",label:"Database"}]}],persons:[{id:"User",label:"End User"}],relations:[{from:"User",to:"WebApp.API",verb:"Visit"},{from:"WebApp.API",to:"WebApp.DB",verb:"Query"}]}},e=()=>{const[r,s]=a.useState(null);return t.jsxs("div",{children:[t.jsx("div",{style:{height:400},children:t.jsx(n,{data:d,onSelect:i=>s(i)})}),t.jsxs("div",{style:{padding:8},children:["Selected: ",r||"none"]})]})};e.__docgenInfo={description:"",methods:[],displayName:"Selectable"};e.parameters={...e.parameters,docs:{...e.parameters?.docs,source:{originalSource:`() => {
  const [selected, setSelected] = useState<string | null>(null);
  return <div>
      <div style={{
      height: 400
    }}>
        <SrujaViewerView data={DATA} onSelect={id => setSelected(id)} />
      </div>
      <div style={{
      padding: 8
    }}>Selected: {selected || 'none'}</div>
    </div>;
}`,...e.parameters?.docs?.source}}};const b=["Selectable"];export{e as Selectable,b as __namedExportsOrder,u as default};
