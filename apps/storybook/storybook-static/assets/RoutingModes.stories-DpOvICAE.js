import{j as l}from"./iframe-CLHqt8sP.js";import{L as a,l as d,S as c,c as e,I as n,a as S}from"./c4-layout-BaP4Bnfv.js";import"./preload-helper-PPVm8Dsz.js";const x={title:"Examples/Routing Modes",component:a};function m(){const r={id:e("Sys"),label:"Sys",kind:"SoftwareSystem",level:"context",tags:new Set},o={id:e("User"),label:"User",kind:"Person",level:"context",tags:new Set},i={id:e("Payments"),label:"Payments",kind:"ExternalSystem",level:"context",tags:new Set},u=[{id:"User->Sys",from:o.id,to:r.id},{id:"Sys->Payments",from:r.id,to:i.id}];return S([r,o,i],u)}const t={render:()=>l.jsx(a,{result:d(m(),c(e("Sys")),n)})},s={render:()=>l.jsx(a,{result:d(m(),c(e("Sys")),{...n,edgeRouting:{...n.edgeRouting,algorithm:"splines"}})})};t.parameters={...t.parameters,docs:{...t.parameters?.docs,source:{originalSource:`{
  render: () => <LayoutSVG result={layout(build(), SystemContextView(createC4Id('Sys')), InteractivePreset)} />
}`,...t.parameters?.docs?.source}}};s.parameters={...s.parameters,docs:{...s.parameters?.docs,source:{originalSource:`{
  render: () => <LayoutSVG result={layout(build(), SystemContextView(createC4Id('Sys')), {
    ...InteractivePreset,
    edgeRouting: {
      ...InteractivePreset.edgeRouting,
      algorithm: 'splines'
    }
  } as any)} />
}`,...s.parameters?.docs?.source}}};const I=["Orthogonal","Splines"];export{t as Orthogonal,s as Splines,I as __namedExportsOrder,x as default};
