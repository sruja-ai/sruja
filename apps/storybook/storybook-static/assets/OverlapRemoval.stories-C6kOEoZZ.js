import{j as m}from"./iframe-CLHqt8sP.js";import{L as r,l as c,d as s,S as i,c as e,a as u}from"./c4-layout-BaP4Bnfv.js";import"./preload-helper-PPVm8Dsz.js";const C={title:"Examples/Overlap Removal",component:r};function p(){const t={id:e("Sys"),label:"Sys",kind:"SoftwareSystem",level:"context",tags:new Set},l={id:e("A"),label:"A",kind:"SoftwareSystem",level:"context",tags:new Set},n={id:e("B"),label:"B",kind:"SoftwareSystem",level:"context",tags:new Set},d={id:e("C"),label:"C",kind:"SoftwareSystem",level:"context",tags:new Set},S=[{id:"A->Sys",from:l.id,to:t.id},{id:"B->Sys",from:n.id,to:t.id},{id:"C->Sys",from:d.id,to:t.id}];return u([t,l,n,d],S)}const a={render:()=>m.jsx(r,{result:c(p(),i(e("Sys")),{...s,overlapRemoval:{...s.overlapRemoval,enabled:!1}})})},o={render:()=>m.jsx(r,{result:c(p(),i(e("Sys")),{...s,overlapRemoval:{...s.overlapRemoval,enabled:!0}})})};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  render: () => <LayoutSVG result={layout(build(), SystemContextView(createC4Id('Sys')), {
    ...CompactPreset,
    overlapRemoval: {
      ...CompactPreset.overlapRemoval,
      enabled: false
    }
  } as any)} />
}`,...a.parameters?.docs?.source}}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => <LayoutSVG result={layout(build(), SystemContextView(createC4Id('Sys')), {
    ...CompactPreset,
    overlapRemoval: {
      ...CompactPreset.overlapRemoval,
      enabled: true
    }
  } as any)} />
}`,...o.parameters?.docs?.source}}};const x=["Disabled","Enabled"];export{a as Disabled,o as Enabled,x as __namedExportsOrder,C as default};
