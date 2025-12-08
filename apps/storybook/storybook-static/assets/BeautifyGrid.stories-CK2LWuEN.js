import{j as i}from"./iframe-CLHqt8sP.js";import{L as a,l as d,S as c,c as e,a as u,I as S}from"./c4-layout-BaP4Bnfv.js";import"./preload-helper-PPVm8Dsz.js";const f={title:"Examples/Beautify & Grid",component:a};function l(){const s={id:e("Sys"),label:"Sys",kind:"SoftwareSystem",level:"context",tags:new Set},o={id:e("A"),label:"A",kind:"SoftwareSystem",level:"context",tags:new Set},n={id:e("B"),label:"B",kind:"SoftwareSystem",level:"context",tags:new Set},m=[{id:"A->Sys",from:o.id,to:s.id},{id:"B->Sys",from:n.id,to:s.id}];return u([s,o,n],m)}const t={render:()=>i.jsx(a,{result:d(l(),{...c(e("Sys")),snapToGrid:!1},S)})},r={render:()=>i.jsx(a,{result:d(l(),{...c(e("Sys")),snapToGrid:!0,gridSize:20},S)})};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  render: () => <LayoutSVG result={layout(build(), {
    ...SystemContextView(createC4Id('Sys')),
    snapToGrid: false
  }, InteractivePreset)} />
}`,...t.parameters?.docs?.source}}};r.parameters={...r.parameters,docs:{...r.parameters?.docs,source:{originalSource:`{
  render: () => <LayoutSVG result={layout(build(), {
    ...SystemContextView(createC4Id('Sys')),
    snapToGrid: true,
    gridSize: 20
  }, InteractivePreset)} />
}`,...r.parameters?.docs?.source}}};const G=["NoSnap","SnapToGrid"];export{t as NoSnap,r as SnapToGrid,G as __namedExportsOrder,f as default};
