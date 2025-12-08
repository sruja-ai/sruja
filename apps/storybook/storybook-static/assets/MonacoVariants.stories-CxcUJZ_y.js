import{a as o,j as a}from"./iframe-CLHqt8sP.js";import{M as n}from"./MonacoEditor-BeqEQem3.js";import"./preload-helper-PPVm8Dsz.js";const c={title:"Editor/MonacoEditor Variants"},e=()=>{const[r,s]=o.useState('person User "User"');return a.jsx("div",{style:{height:300},children:a.jsx(n,{value:r,onChange:s,theme:"vs-dark"})})},t=()=>{const[r,s]=o.useState("// taller editor");return a.jsx("div",{style:{height:600},children:a.jsx(n,{value:r,onChange:s})})};e.__docgenInfo={description:"",methods:[],displayName:"DarkTheme"};t.__docgenInfo={description:"",methods:[],displayName:"Tall"};e.parameters={...e.parameters,docs:{...e.parameters?.docs,source:{originalSource:`() => {
  const [value, setValue] = useState('person User "User"');
  return <div style={{
    height: 300
  }}>
      <MonacoEditor value={value} onChange={setValue} theme="vs-dark" />
    </div>;
}`,...e.parameters?.docs?.source}}};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`() => {
  const [value, setValue] = useState('// taller editor');
  return <div style={{
    height: 600
  }}>
      <MonacoEditor value={value} onChange={setValue} />
    </div>;
}`,...t.parameters?.docs?.source}}};const u=["DarkTheme","Tall"];export{e as DarkTheme,t as Tall,u as __namedExportsOrder,c as default};
