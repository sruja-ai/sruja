import{j as l}from"./iframe-CLHqt8sP.js";import{L as d,l as u,S as m,c as t,C as g,I as c,a as S,P as C}from"./c4-layout-BaP4Bnfv.js";import"./preload-helper-PPVm8Dsz.js";const f={title:"Examples/Layout Engine",component:d};function y(){const e={id:t("Sys"),label:"Sys",kind:"SoftwareSystem",level:"context",tags:new Set},r={id:t("User"),label:"User",kind:"Person",level:"context",tags:new Set},s={id:t("Payments"),label:"Payments",kind:"ExternalSystem",level:"context",tags:new Set},n=[{id:"User->Sys",from:r.id,to:e.id},{id:"Sys->Payments",from:e.id,to:s.id}];return S([e,r,s],n)}function x(){const e={id:t("Sys"),label:"Sys",kind:"SoftwareSystem",level:"context",tags:new Set},r={id:t("API"),label:"API",kind:"Container",level:"container",parentId:e.id,tags:new Set},s={id:t("Web"),label:"Web",kind:"Container",level:"container",parentId:e.id,tags:new Set},n={id:t("DB"),label:"DB",kind:"Database",level:"container",parentId:e.id,tags:new Set},p=[{id:"Web->API",from:s.id,to:r.id},{id:"API->DB",from:r.id,to:n.id}];return S([e,r,s,n],p)}const o={render:()=>{const e=y(),r=u(e,m(t("Sys")),c);return l.jsx(d,{result:r})}},a={render:()=>{const e=x(),r=u(e,g(t("Sys")),C);return l.jsx(d,{result:r})}},i={render:()=>{const e=y();t("User"),t("Sys"),t("Sys"),t("Payments");const r=u(e,m(t("Sys")),{...c,edgeRouting:{...c.edgeRouting,algorithm:"splines"}});return l.jsx(d,{result:r})}};o.parameters={...o.parameters,docs:{...o.parameters?.docs,source:{originalSource:`{
  render: () => {
    const graph = buildSimpleContext();
    const result = layout(graph, SystemContextView(createC4Id('Sys')), InteractivePreset);
    return <LayoutSVG result={result} />;
  }
}`,...o.parameters?.docs?.source}}};a.parameters={...a.parameters,docs:{...a.parameters?.docs,source:{originalSource:`{
  render: () => {
    const graph = buildContainers();
    const result = layout(graph, ContainerView(createC4Id('Sys')), PublicationPreset);
    return <LayoutSVG result={result} />;
  }
}`,...a.parameters?.docs?.source}}};i.parameters={...i.parameters,docs:{...i.parameters?.docs,source:{originalSource:`{
  render: () => {
    const graph = buildSimpleContext();
    const edges = [{
      id: 'User->Sys',
      from: createC4Id('User'),
      to: createC4Id('Sys'),
      preferredRoute: 'splines' as const
    }, {
      id: 'Sys->Payments',
      from: createC4Id('Sys'),
      to: createC4Id('Payments'),
      preferredRoute: 'splines' as const
    }];
    const result = layout(graph, SystemContextView(createC4Id('Sys')), {
      ...InteractivePreset,
      edgeRouting: {
        ...InteractivePreset.edgeRouting,
        algorithm: 'splines'
      }
    } as any);
    return <LayoutSVG result={result} />;
  }
}`,...i.parameters?.docs?.source}}};const w=["SystemContext","ContainerViewStory","CurvedEdges"];export{a as ContainerViewStory,i as CurvedEdges,o as SystemContext,w as __namedExportsOrder,f as default};
